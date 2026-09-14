use crate::{
    arena::Handle,
    symbols::ScopeTable,
    vm::{
        Cmd, Command, CommandCast, Context, Imm, Machine, ScriptAccess, ScriptResult, cast_command,
        drop_command,
    },
};
use bevy_reflect::PartialReflect;

struct Variable {
    imm: Imm<()>,
    drop: Command,
}

#[derive(Default)]
pub struct Semantic {
    machine: Machine,
    table: ScopeTable<String, Variable>,
}

impl Semantic {
    pub fn root<U>(&mut self, callback: impl FnOnce(Block<'_>) -> U) -> U {
        let Self { machine, table } = self;
        Block { machine, table }.block(callback)
    }

    pub fn build(self) -> Machine {
        self.machine
    }
}

macro_rules! impl_bin {
    ($( $fn:ident $op_t:ident :: $op_fn:ident )+) => {
        $(
            pub fn $fn<Lhs, Rhs, Output>(&mut self, lhs: Imm<Lhs>, rhs: Imm<Rhs>, output: Imm<Output>)
                -> (Cmd, Imm<(Imm<Lhs>, Imm<Rhs>, Imm<Output>)>)
                where Lhs: Copy + std::ops::$op_t<Rhs, Output = Output>, Rhs: Copy,
            {
                self.emit((lhs, rhs, output), |&mut (lhs, rhs, output), mut ctx| {
                    let lhs = ctx.get_ref(lhs);
                    let rhs = ctx.get_ref(rhs);
                    ctx.write(output, std::ops::$op_t::$op_fn(*lhs, *rhs));
                    Ok(())
                })
            }
        )+
    };
}

macro_rules! impl_cmp {
    ($( $fn:ident $branch:ident $op_t:ident :: $op_fn:ident )+) => {
        $(
            pub fn $fn<Lhs, Rhs>(&mut self, lhs: Imm<Lhs>, rhs: Imm<Rhs>, output: Imm<bool>)
                -> (Cmd, Imm<(Imm<Lhs>, Imm<Rhs>, Imm<bool>)>)
                where Lhs: Copy + std::cmp::$op_t<Rhs>, Rhs: Copy,
            {
                self.emit((lhs, rhs, output), |&mut (lhs, rhs, output), mut ctx| {
                    let lhs = ctx.get_ref(lhs);
                    let rhs = ctx.get_ref(rhs);
                    ctx.write(output, std::cmp::$op_t::$op_fn(lhs, rhs));
                    Ok(())
                })
            }

            pub fn $branch<Lhs, Rhs>(&mut self, lhs: Imm<Lhs>, rhs: Imm<Rhs>, addr: Cmd)
                -> (Cmd, Imm<(Imm<Lhs>, Imm<Rhs>, Cmd)>)
                where Lhs: Copy + std::cmp::$op_t<Rhs>, Rhs: Copy,
            {
                self.emit((lhs, rhs, addr), |&mut (lhs, rhs, addr), mut ctx| {
                    let lhs = ctx.get_ref(lhs);
                    let rhs = ctx.get_ref(rhs);
                    ctx.cond_jump(std::cmp::$op_t::$op_fn(lhs, rhs), addr);
                    Ok(())
                })
            }
        )+
    };
}

pub struct Block<'a> {
    machine: &'a mut Machine,
    table: &'a mut ScopeTable<String, Variable>,
}

impl Block<'_> {
    pub fn reborrow(&mut self) -> Block<'_> {
        Block {
            machine: self.machine,
            table: self.table,
        }
    }

    pub fn variable<T>(&mut self, name: impl Into<String>) -> Imm<T> {
        let imm = unsafe { self.machine.place::<T>(|_, _| Ok(())).1 };
        let var = Variable {
            imm: imm.untyped(),
            drop: unsafe { cast_command(drop_command::<T>) },
        };
        self.table.add(name, var);
        imm
    }

    pub fn variable_init<T>(&mut self, name: impl Into<String>, value: T) -> Imm<T> {
        let imm = self.machine.init_var(value).1;
        let var = Variable {
            imm: imm.untyped(),
            drop: unsafe { cast_command(drop_command::<T>) },
        };
        self.table.add(name, var);
        imm
    }

    pub fn debug_variable<T: std::fmt::Debug>(&mut self, var: Imm<T>) {
        self.machine.debug_var(var);
    }

    pub fn emit<T>(&mut self, imm: T, command: CommandCast<T>) -> (Cmd, Imm<T>) {
        self.machine.emit(imm, command)
    }

    impl_bin!(
        add Add     :: add
        sub Sub     :: sub
        div Div     :: div
        mul Mul     :: mul
        rem Rem     :: rem
        shl Shl     :: shl
        shr Shr     :: shr
        and BitAnd  :: bitand
        ior BitOr   :: bitor
        eor BitXor  :: bitxor
    );

    impl_cmp!(
        eq branch_eq PartialEq  :: eq
        ne branch_ne PartialEq  :: ne
        le branch_le PartialOrd :: le
        lt branch_lt PartialOrd :: lt
        ge branch_ge PartialOrd :: ge
        gt branch_gt PartialOrd :: gt
    );

    pub fn reflect_access<Src: PartialReflect, Dst: Copy + 'static>(
        &mut self,
        src: Imm<Src>,
        access: ScriptAccess,
        dst: Imm<Dst>,
    ) {
        self.machine.reflect_access(src, access, dst);
    }

    pub fn block<U>(&mut self, block: impl FnOnce(Block<'_>) -> U) -> U {
        self.table.push();

        let result = block(Block {
            machine: self.machine,
            table: self.table,
        });

        for (_, Variable { imm, drop }) in self.table.drain() {
            self.machine.push_command(imm, drop);
        }
        self.table.pop();

        result
    }

    pub fn foreach<I: Iterator, U>(
        &mut self,
        item: impl Into<String>,
        iter: Imm<I>,
        body: impl FnOnce(Block<'_>, Imm<I::Item>) -> U,
    ) -> U {
        struct Data<I: Iterator> {
            exit: Cmd,
            iter: Imm<I>,
            item: Imm<I::Item>,
        }

        fn command<I: Iterator>(data: &mut Data<I>, mut ctx: Context<'_>) -> ScriptResult {
            if let Some(next) = ctx.get_mut(data.iter).next() {
                ctx.write(data.item, next);
            } else {
                ctx.jump(data.exit);
            }
            Ok(())
        }

        unsafe {
            let (cmd, imm) = self.machine.place(command::<I>);

            let (item, result) = self.block(|mut block| {
                let item = block.variable::<I::Item>(item);
                (item, body(block, item))
            });

            let (end, _) = self.machine.branch(cmd);
            let exit = self.machine.cmd_next(end);
            self.machine.write(imm, Data::<I> { exit, iter, item });

            result
        }
    }
}

pub enum Item {
    Block(Vec<Self>),
    Debug(String),

    Binary(BinaryOp, Handle<Self>, Handle<Self>),
    Compare(CompareOp, Handle<Self>, Handle<Self>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,

    Div,
    Mul,
    Rem,

    Shl,
    Shr,

    And,
    Ior,
    Eor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
}
