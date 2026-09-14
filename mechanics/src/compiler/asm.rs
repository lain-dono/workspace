use super::ir::{Action, BinaryOp, Label, Module, Primitive, SystemDecl, Type, Var};
use crate::stat::Stat;
use bevy::ecs::component::ComponentId;
use bevy::ecs::query::QueryIter;
use bevy::ecs::system::{
    FilteredResourcesMutParamBuilder, LocalBuilder, Query, QueryParamBuilder, System,
    SystemParamBuilder,
};
use bevy::ecs::world;
use bevy_lang::arena::{FastHashMap, Handle, Unique};
use bevy_lang::vm::{self, Context, Machine, ScriptResult, drop_command};
use core::mem::{offset_of, transmute};

pub type QueryRef<'w, 's, 'e> = Query<'w, 's, world::FilteredEntityRef<'e>, ()>;
pub type QueryMut<'w, 's, 'e> = Query<'w, 's, world::FilteredEntityMut<'e>, ()>;

pub type IterRef<'w, 's, 'e> = QueryIter<'w, 's, world::FilteredEntityRef<'e>, ()>;
pub type IterMut<'w, 's, 'e> = QueryIter<'w, 's, world::FilteredEntityMut<'e>, ()>;

pub type EntityRef<'a> = world::FilteredEntityRef<'a>;
pub type EntityMut<'a> = world::FilteredEntityMut<'a>;

pub type ResRef<'w, 's> = world::FilteredResources<'w, 's>;
pub type ResMut<'w, 's> = world::FilteredResourcesMut<'w, 's>;

pub type ImmIterRef = vm::Imm<IterRef<'static, 'static, 'static>>;
pub type ImmIterMut = vm::Imm<IterMut<'static, 'static, 'static>>;
pub type ImmCmd = vm::Imm<vm::Cmd>;
pub type ImmUntyped = vm::Imm<()>;

pub type ImmEntityRef = vm::Imm<EntityRef<'static>>;
pub type ImmEntityMut = vm::Imm<EntityMut<'static>>;

#[derive(Clone)]
pub enum ImmIter {
    Ref(ImmIterRef),
    Mut(ImmIterMut),
}

pub enum ArgIter<'w, 's, 'e> {
    Ref(IterRef<'w, 's, 'e>),
    Mut(IterMut<'w, 's, 'e>),
}

pub fn build_system(
    world: &mut world::World,
    module: &Module,
    system: Handle<SystemDecl>,
) -> impl System<In = (), Out = ()> + use<> {
    let context = SystemContext::new(module, system);
    let mut system = SystemAssembler::default();
    let iter = system.build(context).collect::<Vec<_>>();
    let SystemAssembler { machine, .. } = system;

    let rmut = FilteredResourcesMutParamBuilder::new(|builder| {
        for arg in &context.system.args {
            if let Some(id) = module.resource_id(context.system.variables[arg.var].ty) {
                builder.add_read_by_id(id);
            }
        }
    });

    let query = QueryParamBuilder::new(|builder| {
        for arg in &context.system.args {
            let var_ty = context.system.variables[arg.var].ty;
            if let Type::QueryRef(types) = &module.types[var_ty] {
                for id in types.iter().filter_map(|&ty| module.component_id(ty)) {
                    builder.ref_id(id);
                }
            }
        }
    });

    let state = (LocalBuilder(machine), rmut, query).build_state(world);
    state.build_system(move |mut machine, rmut, query| {
        let imm_iter = iter.iter().cloned();
        write_iters(&mut machine, imm_iter, [ArgIter::Ref(query.into_iter())]);
        machine.run().unwrap();
    })
}

fn write_iters<'w, 's, 'e: 's>(
    machine: &mut Machine,
    iter: impl Iterator<Item = ImmIter>,
    args: impl IntoIterator<Item = ArgIter<'w, 's, 'e>>,
) {
    for (imm, query) in iter.zip(args.into_iter()) {
        match (imm, query) {
            (ImmIter::Ref(imm), ArgIter::Ref(iter)) => unsafe {
                machine.write(imm, transmute::<IterRef, IterRef>(iter))
            },
            (ImmIter::Mut(imm), ArgIter::Mut(iter)) => unsafe {
                machine.write(imm, transmute::<IterMut, IterMut>(iter))
            },
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct SystemContext<'a> {
    pub system: &'a SystemDecl,
    pub types: &'a Unique<Type>,

    pub components: &'a FastHashMap<Handle<Type>, ComponentId>,
    pub resources: &'a FastHashMap<Handle<Type>, ComponentId>,
}

impl<'a> SystemContext<'a> {
    pub fn new(module: &'a Module, system: Handle<SystemDecl>) -> Self {
        Self {
            system: &module.systems[system],
            types: &module.types,
            components: &module.components,
            resources: &module.resources,
        }
    }

    pub fn resource_id(&self, ty: Handle<Type>) -> Option<ComponentId> {
        match self.types[ty] {
            Type::Ref(ty) | Type::Mut(ty) => self.resource_id(ty),
            Type::Struct(_) | Type::Resource(_) => self.resources.get(&ty).copied(),
            _ => None,
        }
    }

    pub fn component_id(&self, ty: Handle<Type>) -> Option<ComponentId> {
        match self.types[ty] {
            Type::Ref(ty) | Type::Mut(ty) => self.component_id(ty),
            Type::Struct(_) | Type::Component(_) => self.components.get(&ty).copied(),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct SystemAssembler {
    pub machine: Machine,
    pub labels: FastHashMap<Handle<Label>, vm::Cmd>,
    pub mapping: FastHashMap<usize, ImmCmd>,
    pub variables: FastHashMap<Handle<Var>, ImmUntyped>,
}

impl SystemAssembler {
    fn build(&mut self, context: SystemContext) -> impl Iterator<Item = ImmIter> {
        for arg in &context.system.args {
            let arg_ty = context.system.variables[arg.var].ty;
            if let Type::QueryRef(_) = context.types[arg_ty] {
                unsafe {
                    let (_, imm) = self.machine.place_var::<IterRef>();
                    self.variables.insert(arg.var, imm.cast());
                }
            }
        }

        for (index, action) in context.system.body.iter().enumerate() {
            match *action {
                Action::NextRef { item, iter, .. } => self.next_ref(index, item, iter),
                Action::Mark(label) => self.mark(label),
                Action::Jump(_) => self.jump(index),
                Action::RefComponent { src, dst } => self.ref_component(context, src, dst),
                Action::MutComponent { src, dst } => self.mut_component(context, src, dst),
                Action::Drop(var) => self.drop(context, var),
                Action::Debug(var) => {
                    self.machine.emit(self.variables[&var], |imm, ctx| unsafe {
                        let imm = imm.cast::<&Stat>();
                        println!("debug {:?}", ctx.get_ref(imm));
                        Ok(())
                    });
                }
                Action::Binary(lhs, op, rhs) => {
                    let lhs_ty = context.types[context.system.variables[lhs].ty].primitive();
                    let Some(lhs_ty) = lhs_ty else { continue };

                    let rhs_ty = context.types[context.system.variables[rhs].ty].primitive();
                    let Some(rhs_ty) = rhs_ty else { continue };

                    let lhs_imm = self.variables[&lhs];
                    let rhs_imm = self.variables[&rhs];
                    let output = lhs_imm;

                    macro_rules! binary {
                        (
                            $(
                                $tt:ident :: $t:ty => [ $($bop:ident :: $fn:ident),+ $(,)? ]
                            ),+
                            $(,)?
                        ) => {
                            match (lhs_ty, rhs_ty) {
                                $( (Primitive::$tt, Primitive::$tt) => binary!($t => [ $( $bop :: $fn ),+ ]), )+
                                _ => continue,
                            }
                        };

                        ($t:ty => [ $($bop:ident :: $fn:ident),+ $(,)? ]) => {
                            unsafe {
                                #[allow(unreachable_patterns)]
                                let _ = match op {
                                    $( BinaryOp::$bop => self.machine.$fn::<$t, $t, $t>(lhs_imm.cast(), rhs_imm.cast(), output.cast()), )+
                                    _ => continue,
                                };
                            }
                        };
                    }

                    binary!(
                        I32::i32 => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                            Shl::shl, Shr::shr, And::and, Ior::ior, Eor::eor,
                        ],
                        I64::i64 => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                            Shl::shl, Shr::shr, And::and, Ior::ior, Eor::eor,
                        ],
                        Isize::isize => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                            Shl::shl, Shr::shr, And::and, Ior::ior, Eor::eor,
                        ],

                        U32::u32 => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                            Shl::shl, Shr::shr, And::and, Ior::ior, Eor::eor,
                        ],
                        U64::u64 => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                            Shl::shl, Shr::shr, And::and, Ior::ior, Eor::eor,
                        ],
                        Usize::usize => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                            Shl::shl, Shr::shr, And::and, Ior::ior, Eor::eor,
                        ],

                        F32::f32 => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                        ],
                        F64::f64 => [
                            Add::add, Sub::sub, Div::div, Mul::mul, Rem::rem,
                        ],
                    )
                }
            };
        }

        for (index, action) in context.system.body.iter().enumerate() {
            if let Action::NextRef { label, .. } | Action::Jump(label) = *action {
                unsafe {
                    let imm = self.mapping[&index].cast();
                    let label = self.labels[&label];
                    self.machine.write(imm, label)
                }
            }
        }

        let iter = context.system.args.iter();
        iter.filter_map(|arg| unsafe {
            match context.types[context.system.variables[arg.var].ty] {
                Type::QueryRef(_) => Some(ImmIter::Ref(self.variables[&arg.var].cast())),
                _ => None,
            }
        })
    }

    fn binary<T>(&mut self, lhs: Handle<Var>, op: BinaryOp, rhs: Handle<Var>)
    where
        T: Copy
            + std::ops::Add<T, Output = T>
            + std::ops::Sub<T, Output = T>
            + std::ops::Div<T, Output = T>
            + std::ops::Mul<T, Output = T>
            + std::ops::Rem<T, Output = T>, // + std::ops::Shl<T, Output = T>
                                            // + std::ops::Shr<T, Output = T>
                                            // + std::ops::BitAnd<T, Output = T>
                                            // + std::ops::BitOr<T, Output = T>
                                            // + std::ops::BitXor<T, Output = T>,
    {
        let lhs = unsafe { self.variables[&lhs].cast::<T>() };
        let rhs = unsafe { self.variables[&rhs].cast::<T>() };

        let output = lhs;

        let _ = match op {
            BinaryOp::Add => self.machine.add::<T, T, T>(lhs, rhs, output),
            BinaryOp::Sub => self.machine.sub::<T, T, T>(lhs, rhs, output),

            BinaryOp::Div => self.machine.div::<T, T, T>(lhs, rhs, output),
            BinaryOp::Mul => self.machine.mul::<T, T, T>(lhs, rhs, output),
            BinaryOp::Rem => self.machine.rem::<T, T, T>(lhs, rhs, output),
            // BinaryOp::Shl => self.machine.shl::<T, T, T>(lhs, rhs, output),
            // BinaryOp::Shr => self.machine.shr::<T, T, T>(lhs, rhs, output),
            // BinaryOp::And => self.machine.and::<T, T, T>(lhs, rhs, output),
            // BinaryOp::Ior => self.machine.ior::<T, T, T>(lhs, rhs, output),
            // BinaryOp::Eor => self.machine.eor::<T, T, T>(lhs, rhs, output),
            _ => todo!(),
        };
    }

    fn next_ref(&mut self, index: usize, item: Handle<Var>, iter: Handle<Var>) {
        #[repr(C)]
        struct NextRef {
            exit: vm::Cmd,
            iter: ImmIterRef,
            item: EntityRef<'static>,
        }

        fn command(data: &mut NextRef, mut ctx: Context<'_>) -> ScriptResult {
            if let Some(next) = ctx.get_mut(data.iter).next() {
                unsafe { core::ptr::write(&mut data.item, next) };
            } else {
                ctx.jump(data.exit);
            }
            Ok(())
        }

        let machine = &mut self.machine;

        unsafe {
            let (_, imm) = machine.place(command);
            self.mapping.insert(index, imm.cast());

            let iter_imm = imm.field::<ImmIterRef>(offset_of!(NextRef, iter));

            machine.write(iter_imm, self.variables[&iter].cast());

            self.variables
                .insert(item, imm.field(offset_of!(NextRef, item)));
            self.mapping
                .insert(index, imm.field(offset_of!(NextRef, exit)));
        }
    }

    fn ref_component(&mut self, context: SystemContext, src: Handle<Var>, dst: Handle<Var>) {
        struct Data {
            place: *const u8,
            src: ImmEntityRef,
            id: ComponentId,
        }

        fn command(data: &mut Data, ctx: Context<'_>) -> ScriptResult {
            let entity = ctx.get_ref(data.src);
            data.place = unsafe { entity.get_by_id(data.id).unwrap().deref() };
            Ok(())
        }

        let imm = Data {
            place: core::ptr::null(),
            src: unsafe { self.variables[&src].cast() },
            id: context
                .component_id(context.system.variables[dst].ty)
                .unwrap(),
        };

        let (_, imm) = self.machine.emit(imm, command);

        self.variables
            .insert(dst, unsafe { imm.field(offset_of!(Data, place)) });
    }

    fn mut_component(&mut self, context: SystemContext, src: Handle<Var>, dst: Handle<Var>) {
        struct Data {
            place: *mut u8,
            src: ImmEntityMut,
            id: ComponentId,
        }

        fn command(data: &mut Data, mut ctx: Context<'_>) -> ScriptResult {
            let entity = ctx.get_mut(data.src);
            data.place = unsafe { entity.get_mut_by_id(data.id).unwrap().as_mut().deref_mut() };
            Ok(())
        }

        let imm = Data {
            place: core::ptr::null_mut(),
            src: unsafe { self.variables[&src].cast() },
            id: context
                .component_id(context.system.variables[dst].ty)
                .unwrap(),
        };

        let (_, imm) = self.machine.emit(imm, command);

        self.variables
            .insert(dst, unsafe { imm.field(offset_of!(Data, place)) });
    }

    fn mark(&mut self, label: Handle<Label>) {
        let (cmd, _) = self.machine.emit((), |_, _| Ok(()));
        self.labels.insert(label, cmd);
    }

    fn jump(&mut self, index: usize) {
        let addr = vm::Cmd::PLACEHOLDER;
        self.mapping.insert(index, self.machine.branch(addr).1);
    }

    fn drop(&mut self, context: SystemContext, var: Handle<Var>) {
        if let Type::FilteredEntityRef = context.types[context.system.variables[var].ty] {
            let imm = unsafe { self.variables[&var].cast() };
            self.machine.emit(imm, drop_command::<EntityRef>);
        } else {
            unreachable!("cant drop")
        }
    }
}

pub unsafe fn query_iter_ref(machine: &mut Machine, imm: ImmIterRef, iter: IterRef) {
    unsafe { machine.write(imm, transmute::<IterRef, IterRef>(iter)) }
}

pub unsafe fn query_iter_mut(machine: &mut Machine, imm: ImmIterMut, iter: IterMut) {
    unsafe { machine.write(imm, transmute::<IterMut, IterMut>(iter)) }
}
