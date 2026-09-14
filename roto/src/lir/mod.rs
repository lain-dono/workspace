//! Low-level Intermediate representation (LIR)
//!
//! The IR is the representation between the AST and cranelift. Evaluating
//! it does not need to be particularly fast yet, but the evaluation is safe
//! in the sense that it in the case anything unexpected happens (e.g the
//! wrong type being given) it will panic instead of performing undefined
//! behavior. By evaluating the IR, we can run tests to test this
//! compilation step.
//!
//! The IR has the following characteristics:
//!
//!  - The names of all variables are global.
//!  - Blocks are also identified by readable labels.
//!  - Values are a tagged enum and types are checked at runtime.
//!  - Expressions are simple (as opposed to complex).
//!  - Control flow is represented with basic blocks.
//!
//! The instructions in the IR are inspired by the instructions defined by
//! [cranelift].
//!
//! [cranelift]: https://docs.rs/cranelift-frontend/latest/cranelift_frontend/

mod clone_drop;
mod context;
mod emit;
mod eval;
mod lower;
mod value;

pub use self::context::LowerCtx;
pub use self::emit::{Block, FlowBuilder};
pub use self::eval::{Memory, eval};
pub use self::lower::Lowerer;
pub use self::value::{Type, Value};

use crate::{
    ast,
    label::LabelRef,
    mir,
    runtime::{self, layout::Layout},
    types::{self, ResolvedName, ScopeRef},
};

#[derive(Clone, Debug)]
pub enum Operand {
    Place(crate::var::Var),
    Value(Value),
}

impl From<crate::var::Var> for Operand {
    fn from(value: crate::var::Var) -> Self {
        Self::Place(value)
    }
}

impl From<mir::TypedVar> for Operand {
    fn from(mir::TypedVar(value, _): mir::TypedVar) -> Self {
        Self::Place(value)
    }
}

impl From<TypedVar> for Operand {
    fn from(TypedVar(value, _): TypedVar) -> Self {
        Self::Place(value)
    }
}

impl From<Value> for Operand {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

#[derive(Clone, Debug)]
pub struct TypedVar(pub crate::var::Var, pub Type);

impl TypedVar {
    #[must_use]
    pub fn explicit(scope: ScopeRef, ident: impl Into<ast::ident::Ident>, ty: Type) -> Self {
        Self(crate::var::Var::Ident(scope, ident.into()), ty)
    }

    #[must_use]
    pub fn ret(scope: ScopeRef, ty: Type) -> Self {
        Self(crate::var::Var::Return(scope), ty)
    }
}

/// A location is a lot like a [`mir::Place`], but it makes the distinction
/// that a reference type (which is placed in a stack slot) always gets the
/// `Pointer` variant. If the `mir::Place` has any projections, the location
/// is also a `Pointer`, because it then represents some offset in a stack
/// slot.
///
/// So we get the following mapping:
///
/// - a variable of a value type becomes `Location::Var`.
/// - a variable of a reference type becomes `Location::Pointer` with offset 0
/// - a variable of any type with some projection becomes a `Location::Pointer` with some offset.
#[derive(Debug)]
pub enum Location {
    Var(crate::var::Var),
    Ptr(Pointer),
}

impl Location {
    pub fn ptr(base: impl Into<Operand>, offset: u32) -> Self {
        Self::Ptr(Pointer::new(base, offset))
    }
}

#[derive(Debug)]
#[must_use]
pub struct Pointer {
    pub base: Operand,
    pub offset: u32,
}

impl Pointer {
    pub fn new(base: impl Into<Operand>, offset: u32) -> Self {
        let base = base.into();
        Self { base, offset }
    }
}

#[derive(Clone, Debug)]
pub enum Instruction {
    /// Jump to a block
    Jump(LabelRef),

    /// Switch on the integer value of the `examinee`
    Switch {
        examinee: Operand,
        branches: Vec<(usize, LabelRef)>,
        fallback: LabelRef,
    },

    Branch {
        cond: Operand,
        accept: LabelRef,
        reject: LabelRef,
    },

    /// Assign the value `from` to `to`.
    Assign { to: TypedVar, from: Operand },

    Unary {
        op: UnOp,
        to: TypedVar,
        val: Operand,
    },

    /// Get the address of a constant
    ConstAddr { to: TypedVar, name: ResolvedName },

    /// Create string
    InitString { to: TypedVar, string: String },

    /// Call a function.
    Call {
        to: Option<TypedVar>,
        func: ast::ident::Ident,
        args: Vec<Operand>,
        out_ptr: Option<TypedVar>,
    },

    /// Call a runtime function (i.e. a Rust function)
    CallRuntime {
        func: runtime::RuntimeFunctionRef,
        args: Vec<Operand>,
    },

    /// Return from the current function
    Return(Option<Operand>),

    /// Perform an comparison and store the result in `to`
    Cmp {
        to: TypedVar,
        cmp: Compare,
        lhs: Operand,
        rhs: Operand,
    },

    Op {
        to: TypedVar,
        op: BinOp,
        lhs: Operand,
        rhs: Operand,
    },

    Offset {
        to: TypedVar,
        from: Operand,
        offset: u32,
    },

    /// Write literal bytes to a variable
    Initialize {
        to: TypedVar,
        bytes: Vec<u8>,
        layout: Layout,
    },

    /// Write to a stack slot
    Write { to: Operand, val: Operand },

    /// Read from a stack slot
    Read { to: TypedVar, from: Operand },

    /// Copy a stack slot
    Copy {
        from: Operand,
        to: Operand,
        size: usize,
    },

    /// Clone a value with a Rust clone function
    Clone {
        from: Operand,
        to: Operand,
        /// Pointer to the clone implementation of the type
        clone: unsafe extern "C" fn(*const (), *mut ()),
    },

    /// Drop a value
    ///
    /// - For primitives and copy types, this is a noop.
    /// - For more complex types it matches Rust's Drop.
    Drop {
        var: Operand,
        /// Pointer to the drop implementation of the type
        drop: Option<unsafe extern "C" fn(*mut ())>,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum UnOp {
    Eqz,
    Clz,
    Ctz,
    Pop,
    BNot,
    INeg,
    FNeg,
}

#[derive(Clone, Copy, Debug)]
pub enum BinOp {
    IAdd,
    ISub,
    IMul,
    SRem,
    URem,
    SDiv,
    UDiv,

    FAdd,
    FSub,
    FMul,
    FDiv,
}

#[derive(Clone, Copy, Debug)]
pub enum Compare {
    IEq,
    INe,

    SLt,
    SLe,
    SGt,
    SGe,

    ULt,
    ULe,
    UGt,
    UGe,

    FEq,
    FNe,

    FLt,
    FLe,
    FGt,
    FGe,
}

impl core::fmt::Display for Compare {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::IEq => "ieq",
            Self::INe => "ine",

            Self::SLt => "slt",
            Self::SLe => "sle",
            Self::SGt => "sgt",
            Self::SGe => "sge",

            Self::ULt => "ult",
            Self::ULe => "ule",
            Self::UGt => "ugt",
            Self::UGe => "uge",

            Self::FEq => "feq",
            Self::FNe => "fne",
            Self::FLt => "flt",
            Self::FLe => "fle",
            Self::FGt => "fgt",
            Self::FGe => "fge",
        })
    }
}

#[derive(Debug)]
pub enum ValueOrSlot {
    Value(Type),
    Slot(Layout),
}

impl From<Type> for ValueOrSlot {
    fn from(value: Type) -> Self {
        Self::Value(value)
    }
}

impl From<Layout> for ValueOrSlot {
    fn from(value: Layout) -> Self {
        Self::Slot(value)
    }
}

#[derive(Debug)]
pub struct Function {
    /// Identifier of the function
    pub name: ast::ident::Ident,

    /// Scope of the function
    pub scope: ScopeRef,

    pub signature: types::Signature,

    /// Signature of the function
    pub ir_signature: Signature,

    /// Entry block of the function
    pub entry_block: LabelRef,

    /// Variables used by this function
    pub variables: Vec<(crate::var::Var, ValueOrSlot)>,

    /// Blocks belonging to this function
    pub blocks: Vec<Block>,

    /// Whether the function is public i.e. can be accessed from Rust
    pub public: bool,
}

#[derive(Clone, Debug)]
pub struct Signature {
    pub parameters: Vec<(ast::ident::Ident, Type)>,

    /// Whether this function takes a pointer for the return value
    /// passed as an argument
    pub return_ptr: bool,
    pub return_type: Option<Type>,
}

pub struct Lir {
    pub functions: Vec<Function>,
}
