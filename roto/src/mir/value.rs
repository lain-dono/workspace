use crate::{
    ast,
    mir::{TypedVar, UnOp},
    runtime::RuntimeFunctionRef,
    types::{ResolvedName, Type},
    var::Var,
};

#[derive(Clone, Debug)]
#[must_use]
pub enum Value {
    Const(ast::Literal, Type),
    Constant(ResolvedName, Type),
    Clone(Place),
    Discriminant(TypedVar),

    Move(TypedVar),
    Unary(UnOp, TypedVar),
    BinOp {
        op: ast::BinOp,
        ty: Type,
        lhs: TypedVar,
        rhs: TypedVar,
    },
    Call {
        func: ResolvedName,
        args: Vec<Var>,
    },
    CallRuntime {
        func: RuntimeFunctionRef,
        args: Vec<Var>,
    },
}

#[derive(Clone, Debug)]
pub enum Projection {
    VariantField(ast::ident::Ident, usize),
    Field(ast::ident::Ident),
}

#[derive(Clone, Debug)]
pub struct Place {
    pub var: TypedVar,
    pub proj: Vec<Projection>,
}

impl From<TypedVar> for Place {
    fn from(var: TypedVar) -> Self {
        Self { var, proj: vec![] }
    }
}

impl Place {
    #[must_use]
    pub const fn new(var: Var, ty: Type) -> Self {
        Self {
            var: TypedVar(var, ty),
            proj: vec![],
        }
    }

    #[must_use]
    pub fn with_variant(mut self, name: impl Into<ast::ident::Ident>, index: usize) -> Self {
        self.proj.push(Projection::VariantField(name.into(), index));
        self
    }

    #[must_use]
    pub fn with_field(mut self, name: impl Into<ast::ident::Ident>) -> Self {
        self.proj.push(Projection::Field(name.into()));
        self
    }
}
