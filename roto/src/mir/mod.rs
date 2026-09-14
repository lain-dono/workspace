//! Mid-level intermediate representation (MIR)
//!
//! This is the first intermediate representation. It is based on basic blocks
//! and simple expressions. We do not distinguish between types that are stored
//! in stack slots and types that are cranelift values; all types are treated
//! equally in this representation.

mod dead_code;
mod emit;
mod lower;
mod match_expr;
mod value;

use crate::{
    ast,
    label::{LabelRef, LabelStore},
    module::ModuleTree,
    runtime::Runtime,
    types::{EnumVariant, ScopeRef, Signature, Type, TypeInfo},
};

pub use self::emit::{Block, FlowBuilder};
pub use self::lower::Lowerer;
pub use self::value::{Place, Projection, Value};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypedVar(pub crate::var::Var, pub Type);

#[derive(Clone, Debug)]
pub struct TypedValue(pub Value, pub Type);

#[derive(Clone, Debug)]
pub struct Variables {
    /// Scope that belongs to this function
    pub scope: ScopeRef,
    /// Index of the largest temporary variable in this function
    pub tmp_idx: usize,
    pub variables: Vec<TypedVar>,
}

#[derive(Clone, Debug)]
pub struct Mir {
    pub functions: Vec<Function>,
}

impl Mir {
    pub fn lower(
        tree: &ModuleTree,
        rt: &Runtime,
        types: &mut TypeInfo,
        labels: &mut LabelStore,
    ) -> Self {
        let mut mir = Lowerer::tree(rt, types, tree, labels);
        mir.eliminate_dead_code();
        mir
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    /// Name of this function
    pub name: ast::ident::Ident,

    /// Parameters to this function
    pub parameters: Vec<crate::var::Var>,

    /// The function signature
    pub signature: Signature,

    /// Basic blocks of this function
    ///
    /// The first block is the entrypoint.
    pub blocks: Vec<Block>,

    pub variables: Variables,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    Jump(LabelRef),
    Switch {
        examinee: TypedVar,
        branches: Vec<(usize, LabelRef)>,
        fallback: Option<LabelRef>,
    },
    Branch {
        cond: crate::var::Var,
        accept: LabelRef,
        reject: LabelRef,
    },
    Assign(Place, Type, Value),
    SetDiscriminant {
        to: crate::var::Var,
        ty: Type,
        variant: EnumVariant,
    },
    Return(crate::var::Var),
    Drop(Place, Type),
}

#[derive(Clone, Debug)]
pub enum UnOp {
    Not,
    Neg,
}

impl core::fmt::Display for UnOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Not => "!",
            Self::Neg => "-",
        })
    }
}
