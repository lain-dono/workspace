mod arena;

pub use self::arena::{Arena, Handle, Index, UniqueArena};

#[derive(Clone)]
pub struct Program {
    pub types: UniqueArena<Type>,
    pub functions: Arena<Function>,
}

#[derive(Clone)]
pub struct Function {
    pub name: Option<String>,
    pub arguments: Vec<(Option<String>, Handle<Type>)>,
    pub result: Handle<Type>,

    pub body: Vec<Statement>,
}

#[derive(Clone)]
pub enum Statement {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Unit,
    Sint(Sint),
    Uint(Uint),
    Float(Float),
    Char,
    String,
    Struct(Vec<StructMember>),
    Reflect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sint {
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uint {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Float {
    F32,
    F64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructMember {
    pub name: Option<String>,
    pub ty: Handle<Type>,
}
