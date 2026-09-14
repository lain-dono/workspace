mod access;
mod machine;
mod memory;

use bevy_reflect::PartialReflect;

pub use self::access::{ScriptAccess, ScriptAccessError};
pub use self::machine::{Cmd, Command, CommandCast, Context, Imm, Machine};
pub use self::memory::{Memory, MemoryBuilder};

pub type ScriptResult = Result<(), ScriptError>;

#[derive(thiserror::Error, Debug)]
pub enum ScriptError {
    #[error("access {0}")]
    Access(#[from] ScriptAccessError),

    #[error("not found {0:?}")]
    NotFound(ScriptAccess),

    #[error("bad types")]
    TypeMismatch,

    #[error("stack empty")]
    StackEmpty,
}

impl Machine {
    pub fn reflect_access<Src: PartialReflect, Dst: Copy + 'static>(
        &mut self,
        src: Imm<Src>,
        access: ScriptAccess,
        dst: Imm<Dst>,
    ) {
        let imm = (src, access, dst);

        self.emit(imm, |&mut (src, ref access, dst), mut ctx| {
            let src = ctx.get_ref(src);
            let Some(next) = access.access(src)? else {
                return Err(ScriptError::NotFound(access.clone()));
            };
            let Some(next) = next.try_downcast_ref::<Dst>() else {
                return Err(ScriptError::TypeMismatch);
            };
            ctx.write(dst, *next);
            Ok(())
        });
    }

    pub fn branch(&mut self, addr: Cmd) -> (Cmd, Imm<Cmd>) {
        self.emit(addr, |&mut addr, mut ctx| {
            ctx.cond_jump(true, addr);
            Ok(())
        })
    }

    pub fn branch_if_true(&mut self, var: Imm<bool>, addr: Cmd) -> (Cmd, Imm<(Imm<bool>, Cmd)>) {
        self.emit((var, addr), |&mut (var, addr), mut ctx| {
            ctx.cond_jump(*ctx.get_ref(var), addr);
            Ok(())
        })
    }

    pub fn branch_if_false(&mut self, var: Imm<bool>, addr: Cmd) -> (Cmd, Imm<(Imm<bool>, Cmd)>) {
        self.emit((var, addr), |&mut (var, addr), mut ctx| {
            ctx.cond_jump(!*ctx.get_ref(var), addr);
            Ok(())
        })
    }

    pub fn debug_var<T: std::fmt::Debug>(&mut self, var: Imm<T>) -> (Cmd, Imm<Imm<T>>) {
        self.emit(var, |&mut var, ctx| {
            let var = ctx.get_ref(var);
            println!("{var:?}");
            Ok(())
        })
    }

    /*
    pub fn print(&mut self, format: &str) -> usize {
        assert!(format.len() <= 255);
        unsafe {
            let imm_layout = std::alloc::Layout::array::<u8>(format.len() + 1).unwrap();
            let (cmd, imm) = self.emit_raw(imm_layout, self::debug::op_debug);
            self.write_small_str(imm, format);
            cmd
        }
    }

    unsafe fn write_small_str(&mut self, offset: usize, s: &str) {
        let src = s.as_bytes();
        let len = u8::try_from(src.len()).unwrap();
        unsafe {
            *self.memory.add(offset).as_mut() = len;
            let dst = self.memory.add(offset + 1).as_ptr();
            dst.copy_from_nonoverlapping(src.as_ptr(), src.len());
        }
    }

    pub fn loop_range(&mut self, range: std::ops::Range<usize>, body: impl FnOnce(&mut Self)) {
        self.push::<usize>(range.start);

        let (start, _) = self.dup();
        self.push::<usize>(range.end);
        let (branch, _) = self.branch_ge::<usize>(usize::MAX);

        body(self);

        self.push::<usize>(1);
        self.add::<usize>();

        let (end, _) = self.branch(start);
        unsafe {
            let target = self.get_next(end);
            self.set_imm::<usize>(branch, target);
        }
    }
    */

    pub fn init_var<T>(&mut self, value: T) -> (Cmd, Imm<T>) {
        self.emit(value, |_, _| Ok(()))
    }

    pub fn place_var<T>(&mut self) -> (Cmd, Imm<T>) {
        unsafe { self.place::<T>(|_, _| Ok(())) }
    }
}

pub unsafe fn drop_command<T>(&mut imm: &mut Imm<T>, mut ctx: Context<'_>) -> ScriptResult {
    ctx.drop_in_place(imm);
    Ok(())
}

pub unsafe fn cast_command<T>(command: CommandCast<T>) -> Command {
    unsafe { *((&raw const command).cast::<Command>()) }
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

impl Machine {
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
}
