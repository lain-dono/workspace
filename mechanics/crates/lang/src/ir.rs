use crate::arena::{Arena, FastIndexMap, Handle, Unique};
use crate::ty::Literal;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add, // '+'
    Sub, // '-'
    Div, // '/'
    Mul, // '*'
    Rem, // '%'
    And, // '&'
    Eor, // '^'
    Ior, // '|'
    Shl, // '<<'
    Shr, // '>>'

    Eq, // '=='
    Ne, // '!='
    Gt, // '>'
    Ge, // '>='
    Lt, // '<'
    Le, // '<='

    LogicAnd,
    LogicIor,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => f.write_str("+"),
            Self::Sub => f.write_str("-"),
            Self::Div => f.write_str("/"),
            Self::Mul => f.write_str("*"),
            Self::Rem => f.write_str("%"),
            Self::And => f.write_str("&"),
            Self::Eor => f.write_str("^"),
            Self::Ior => f.write_str("|"),
            Self::Shl => f.write_str("<<"),
            Self::Shr => f.write_str(">>"),

            Self::Eq => f.write_str("=="),
            Self::Ne => f.write_str("!="),
            Self::Gt => f.write_str(">"),
            Self::Ge => f.write_str(">="),
            Self::Lt => f.write_str("<"),
            Self::Le => f.write_str("<="),

            Self::LogicAnd => f.write_str("&&"),
            Self::LogicIor => f.write_str("||"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Deref,
    Refer,
    Negate,
    Not,
}

pub struct LocalVariable {
    pub name: Option<String>,
    pub ty: Handle<crate::ty::Type>,
    pub init: Option<Handle<Expr>>,
}

pub struct Constant {
    pub name: Option<String>,
    pub ty: Handle<crate::ty::Type>,
    pub init: Handle<Expr>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Constant(Handle<Constant>),
    Access(Handle<Self>, Handle<Self>),
    AccessIndex(Handle<Self>, u32),
    FuncParam(u32),
    Variable(Handle<LocalVariable>),
    CallResult(Handle<Func>),
    Load(Handle<Self>),

    Unary(UnaryOp, Handle<Self>),
    Binary(BinaryOp, Handle<Self>, Handle<Self>),

    Select(Handle<Self>, Handle<Self>, Handle<Self>),
}

pub struct Func {
    pub name: Option<String>,
    pub params: Vec<FuncParam>,
    pub result: Option<Handle<crate::ty::Type>>,

    pub vars: Arena<LocalVariable>,
    pub expr: Arena<Expr>,
    pub named_expressions: FastIndexMap<Handle<Expr>, String>,

    pub body: Block,
}

#[derive(Default)]
pub struct Block {}

pub struct FuncParam {
    pub name: Option<String>,
    pub ty: Handle<crate::ty::Type>,
}

#[derive(Default)]
pub struct Module {
    pub types: Unique<crate::ty::Type>,
    pub constants: Arena<Constant>,
    pub global: Arena<Expr>,
    pub funcs: Arena<Func>,
}
