use super::asm;
use bevy::ecs::{
    component::{ComponentId, Components},
    world::{FilteredEntityMut, FilteredEntityRef},
};
use bevy_lang::arena::{Arena, FastHashMap, Handle, Unique};
use core::alloc::{Layout, LayoutError};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Primitive {
    I32,
    I64,
    Isize,

    U32,
    U64,
    Usize,

    F32,
    F64,
}

impl Primitive {
    fn layout(self) -> Layout {
        match self {
            Primitive::I32 => Layout::new::<i32>(),
            Primitive::I64 => Layout::new::<i64>(),
            Primitive::Isize => Layout::new::<isize>(),

            Primitive::U32 => Layout::new::<u32>(),
            Primitive::U64 => Layout::new::<u64>(),
            Primitive::Usize => Layout::new::<usize>(),

            Primitive::F32 => Layout::new::<f32>(),
            Primitive::F64 => Layout::new::<f64>(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Primitive(Primitive),

    Ref(Handle<Type>),
    Mut(Handle<Type>),

    Struct(Vec<Handle<Type>>),

    QueryRef(Vec<Handle<Type>>),
    QueryMut(Vec<Handle<Type>>),

    FilteredEntityRef,
    FilteredEntityMut,

    Component(ComponentId),
    Resource(ComponentId),
}

impl Type {
    pub fn primitive(&self) -> Option<Primitive> {
        match *self {
            Self::Primitive(primitive) => Some(primitive),
            _ => None,
        }
    }

    fn is_loaded(&self, types: &Arena<Type>, components: &Components) -> bool {
        match *self {
            Self::Primitive(_) => true,
            Self::Ref(handle) | Self::Mut(handle) => types[handle].is_loaded(types, components),
            Self::Struct(ref handles)
            | Self::QueryRef(ref handles)
            | Self::QueryMut(ref handles) => handles
                .iter()
                .all(|&handle| types[handle].is_loaded(types, components)),

            Self::Component(id) | Self::Resource(id) => components.is_id_valid(id),

            Self::FilteredEntityRef => true,
            Self::FilteredEntityMut => true,
        }
    }

    fn layout(&self, types: &Arena<Type>, components: &Components) -> Result<Layout, LayoutError> {
        Ok(match *self {
            Type::Primitive(primitive) => primitive.layout(),
            Type::Ref(_) => Layout::new::<*const u8>(),
            Type::Mut(_) => Layout::new::<*const u8>(),

            Type::QueryRef(_) => Layout::new::<asm::IterRef>(),
            Type::QueryMut(_) => Layout::new::<asm::IterMut>(),

            Type::Struct(ref handles) => {
                let mut layout = Layout::new::<()>();
                for &handle in handles {
                    let next = types[handle].layout(types, components)?;
                    (layout, _) = layout.extend(next)?;
                }
                layout.pad_to_align()
            }

            Type::Component(id) | Type::Resource(id) => components.get_info(id).unwrap().layout(),

            Type::FilteredEntityRef => Layout::new::<FilteredEntityRef>(),
            Type::FilteredEntityMut => Layout::new::<FilteredEntityMut>(),
        })
    }
}

#[derive(Default)]
pub struct Module {
    pub types: Unique<Type>,
    pub systems: Arena<SystemDecl>,

    pub components: FastHashMap<Handle<Type>, ComponentId>,
    pub resources: FastHashMap<Handle<Type>, ComponentId>,
}

impl Module {
    pub fn ty(&mut self, ty: Type) -> Handle<Type> {
        self.types.push(ty)
    }

    pub fn resource(&mut self, component_id: ComponentId, ty: Type) -> Handle<Type> {
        let handle = self.ty(ty);
        self.resources.insert(handle, component_id);
        handle
    }

    pub fn component(&mut self, component_id: ComponentId, ty: Type) -> Handle<Type> {
        let handle = self.types.push(ty);
        self.components.insert(handle, component_id);
        handle
    }

    pub fn system(&mut self, system: SystemDecl) -> Handle<SystemDecl> {
        self.systems.push(system)
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

    pub fn system_builder(&mut self) -> SystemBuilder {
        SystemBuilder {
            module: self,
            args: Vec::new(),
            label: Arena::new(),
            body: Vec::new(),
            variables: Arena::new(),
        }
    }
}

#[derive(Debug)]
pub struct SystemDecl {
    pub name: String,
    pub args: Vec<SystemArgument>,
    pub body: Vec<Action>,
    // input/output?
    pub variables: Arena<Var>,
}

#[derive(Debug, Clone)]
pub struct SystemArgument {
    pub name: String,
    pub var: Handle<Var>,
}

pub struct Label;

pub struct SystemBuilder<'a> {
    pub module: &'a mut Module,
    pub args: Vec<SystemArgument>,

    pub label: Arena<Label>,
    pub body: Vec<Action>,
    pub variables: Arena<Var>,
}

impl SystemBuilder<'_> {
    pub fn label(&mut self) -> Handle<Label> {
        self.label.push(Label)
    }

    pub fn var(&mut self, ty: Handle<Type>) -> Handle<Var> {
        self.variables.push(Var { ty })
    }

    pub fn var_ty(&mut self, ty: Type) -> (Handle<Var>, Handle<Type>) {
        let ty = self.module.ty(ty);
        let var = self.variables.push(Var { ty });
        (var, ty)
    }

    pub fn next_ref(&mut self, label: Handle<Label>, item: Handle<Var>, iter: Handle<Var>) {
        self.body.push(Action::NextRef { label, item, iter });
    }

    pub fn jump(&mut self, label: Handle<Label>) {
        self.body.push(Action::Jump(label));
    }

    pub fn mark(&mut self, label: Handle<Label>) {
        self.body.push(Action::Mark(label));
    }

    pub fn ref_component_var(&mut self, src: Handle<Var>, ty: Handle<Type>) -> Handle<Var> {
        let (dst, _) = self.var_ty(Type::Ref(ty));
        self.ref_component(src, dst);
        dst
    }

    pub fn ref_component(&mut self, src: Handle<Var>, dst: Handle<Var>) {
        let src_ty = self.variables[src].ty;
        let src_ty = &self.module.types[src_ty];
        assert!(matches!(src_ty, Type::FilteredEntityRef));

        let dst_ty = self.variables[dst].ty;
        let dst_ty = &self.module.types[dst_ty];
        assert!(matches!(dst_ty, Type::Ref(_)));

        self.body.push(Action::RefComponent { src, dst });
    }

    pub fn drop(&mut self, var: Handle<Var>) {
        self.body.push(Action::Drop(var));
    }

    pub fn debug(&mut self, var: Handle<Var>) {
        self.body.push(Action::Debug(var));
    }

    pub fn arg_resource(
        &mut self,
        name: impl Into<String>,
        component_id: ComponentId,
        ty: Type,
    ) -> Handle<Var> {
        let ty = self.module.resource(component_id, ty);
        self.arg(name, ty)
    }

    pub fn arg_resource_opaque(
        &mut self,
        name: impl Into<String>,
        component_id: ComponentId,
    ) -> Handle<Var> {
        self.arg_resource(name, component_id, Type::Resource(component_id))
    }

    pub fn arg_query_ref(
        &mut self,
        name: impl Into<String>,
        components: impl Into<Vec<Handle<Type>>>,
    ) -> Handle<Var> {
        let ty = Type::QueryRef(components.into());
        let ty = self.module.types.push(ty);
        self.arg(name, ty)
    }

    pub fn arg_query_mut(
        &mut self,
        name: impl Into<String>,
        components: impl Into<Vec<Handle<Type>>>,
    ) -> Handle<Var> {
        let ty = Type::QueryRef(components.into());
        let ty = self.module.types.push(ty);
        self.arg(name, ty)
    }

    pub fn arg(&mut self, name: impl Into<String>, ty: Handle<Type>) -> Handle<Var> {
        let name = name.into();
        let var = self.variables.push(Var { ty });
        self.args.push(SystemArgument { name, var });
        var
    }

    pub fn build(self, name: impl Into<String>) -> SystemDecl {
        SystemDecl {
            name: name.into(),
            args: self.args,
            body: self.body,
            variables: self.variables,
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Mark(Handle<Label>),
    NextRef {
        label: Handle<Label>,
        item: Handle<Var>,
        iter: Handle<Var>,
    },
    Jump(Handle<Label>),
    RefComponent {
        src: Handle<Var>,
        dst: Handle<Var>,
    },
    MutComponent {
        src: Handle<Var>,
        dst: Handle<Var>,
    },
    Drop(Handle<Var>),
    Debug(Handle<Var>),

    Binary(Handle<Var>, BinaryOp, Handle<Var>),
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

#[derive(Debug)]
pub struct Var {
    pub ty: Handle<Type>,
}
