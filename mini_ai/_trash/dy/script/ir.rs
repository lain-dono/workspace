use super::{Arena, FastIndexMap, Handle, UniqueArena};

pub struct Module {
    pub types: UniqueArena<Type>,
    pub constants: Arena<Constant>,
    pub variables: Arena<Variable>,
    pub expressions: Arena<Expression>,
    pub functions: Arena<Function>,
    // pub diagnostic_filters: Arena<DiagnosticFilterNode>,
    // pub diagnostic_filter_leaf: Option<Handle<DiagnosticFilterNode>>,
}

pub struct Function {
    pub name: Option<String>,
    pub arguments: Vec<FunctionArgument>,
    pub result: Option<FunctionResult>,
    pub variables: Arena<Variable>,
    pub expressions: Arena<Expression>,
    pub named_expressions: FastIndexMap<Handle<Expression>, String>,
    // pub body: Block,
    // pub diagnostic_filter_leaf: Option<Handle<DiagnosticFilterNode>>,
}

pub struct FunctionArgument {
    pub name: Option<String>,
    pub ty: Handle<Type>,
}

pub struct FunctionResult {
    pub ty: Handle<Type>,
}

pub enum Literal {
    F64(f64),
    F32(f32),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    Bool(bool),
    // AbstractInt(i64),
    // AbstractFloat(f64),
}

pub struct Constant {
    pub name: Option<String>,
    pub ty: Handle<Type>,
    pub init: Handle<Expression>,
}

pub struct Variable {
    pub name: Option<String>,
    pub ty: Handle<Type>,
    pub init: Option<Handle<Expression>>,
}

pub struct Type {
    pub name: Option<String>,
    pub inner: TypeInner,
}

pub enum TypeInner {
    Scalar(Scalar),
    Pointer(Handle<Type>),
    // ValuePointer {
    //     size: Option<VectorSize>,
    //     scalar: Scalar,
    // },
    // Array {
    //     base: Handle<Type>,
    //     size: ArraySize,
    //     stride: u32,
    // },
    // Struct {
    //     members: Vec<StructMember>,
    //     span: u32,
    // },
}

pub struct Scalar {
    pub kind: ScalarKind,
    pub width: u8,
}

#[repr(u8)]
pub enum ScalarKind {
    Sint = 0,
    Uint = 1,
    Float = 2,
    Bool = 3,
    // AbstractInt = 4,
    // AbstractFloat = 5,
}

pub enum UnaryOperator {
    Negate,
    LogicalNot,
    BitwiseNot,
}

pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    ExclusiveOr,
    InclusiveOr,
    LogicalAnd,
    LogicalOr,
    ShiftLeft,
    ShiftRight,
}

pub enum Expression {
    Literal(Literal),
    /*
    Constant(Handle<Constant>),
    Override(Handle<Override>),
    ZeroValue(Handle<Type>),
    Compose {
        ty: Handle<Type>,
        components: Vec<Handle<Expression>>,
    },
    */
    Access {
        base: Handle<Expression>,
        index: Handle<Expression>,
    },
    AccessIndex {
        base: Handle<Expression>,
        index: u32,
    },
    FunctionArgument(u32),
    // GlobalVariable(Handle<GlobalVariable>),
    LocalVariable(Handle<Variable>),
    Load {
        pointer: Handle<Expression>,
    },
    Unary {
        op: UnaryOperator,
        expr: Handle<Expression>,
    },
    Binary {
        op: BinaryOperator,
        left: Handle<Expression>,
        right: Handle<Expression>,
    },
    /*
    As {
        expr: Handle<Expression>,
        kind: ScalarKind,
        convert: Option<Bytes>,
    },
    */
    CallResult(Handle<Function>),
    /*
    ArrayLength(Handle<Expression>),
    */
}
