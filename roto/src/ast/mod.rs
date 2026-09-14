//! Abstract Syntax Tree (AST) for Roto
//!
//! A [`SyntaxTree`] is the output of the Roto parser. It contains a
//! representation of the Roto script as Rust types for further processing.
//!
//! Parser for Roto scripts
//!
//! The parser is a fairly standard recursive descent parser.
//!
//! There is currently no way that the parser can recover from invalid syntax.
//! Therefore, we can only report one parse error.

mod error;
mod expr;
mod filter_map;
pub mod ident;
mod meta;
mod parser;
mod token;

pub use self::error::{ParseError, ParseErrorKind};
pub use self::meta::{Meta, MetaId, Span, Spans};
pub use self::parser::Parser;
pub use self::token::{Keyword, Lexer, Token};

pub type Ident = Meta<self::ident::Ident>;

#[derive(Clone, Debug)]
pub struct SyntaxTree {
    pub declarations: Vec<Decl>,
}

#[derive(Clone, Debug)]
pub enum Decl {
    Import(Vec<Meta<Path>>),
    Struct(StructDecl),
    Enum(EnumDecl),

    Fmap(FilterMap),
    Func(FuncDecl),
    Test(Test),
}

#[derive(Clone, Debug)]
pub struct Params(pub Vec<(Ident, Meta<TypeExpr>)>);

/// The value of a typed record
#[derive(Clone, Debug)]
pub struct StructDecl {
    pub ident: Ident,
    pub params: Vec<Ident>,
    pub fields: NamedFields,
}

#[derive(Clone, Debug)]
pub struct EnumDecl {
    pub ident: Ident,
    pub params: Vec<Ident>,
    pub variants: Meta<Vec<Variant>>,
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub ident: Ident,
    pub fields: Vec<Meta<TypeExpr>>,
}

#[derive(Clone, Debug)]
pub struct NamedFields {
    pub fields: Meta<Vec<(Ident, Meta<TypeExpr>)>>,
}

#[derive(Clone, Debug)]
pub enum TypeExpr {
    Never,
    Unit,
    Option(Box<Meta<TypeExpr>>),
    Path(Meta<Path>, Option<Meta<Vec<Meta<TypeExpr>>>>),
    Struct(NamedFields),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterType {
    FilterMap,
    Filter,
}

#[derive(Clone, Debug)]
pub struct FilterMap {
    pub filter_type: FilterType,
    pub ident: Ident,
    pub params: Meta<Params>,
    pub body: Meta<Block>,
}

/// A function declaration, including the [`Block`] forming its definition
#[derive(Clone, Debug)]
pub struct FuncDecl {
    pub ident: Ident,
    pub params: Meta<Params>,
    pub result: Option<Meta<TypeExpr>>,
    pub body: Meta<Block>,
}

#[derive(Clone, Debug)]
pub struct Test {
    pub ident: Ident,
    pub body: Meta<Block>,
}

/// A block of multiple statements
#[derive(Clone, Debug)]
pub struct Block {
    pub imports: Vec<Meta<Path>>,
    pub body: Vec<Meta<Stmt>>,
    pub last: Option<Box<Meta<Expr>>>,
}

/// A statement in a block
#[derive(Clone, Debug)]
pub enum Stmt {
    Let(Ident, Option<Meta<TypeExpr>>, Meta<Expr>),
    Expr(Meta<Expr>),
}

#[derive(Clone, Debug)]
pub struct Path {
    pub idents: Vec<Ident>,
}

/// A Roto expression
#[derive(Clone, Debug)]
pub enum Expr {
    /// Return from the current function or filtermap
    ///
    /// Optionally takes an expression for the value being returned.
    Return(ReturnKind, Option<Box<Meta<Expr>>>),

    /// A literal expression
    Literal(Meta<Literal>),
    /// f-string
    FString(Vec<Meta<FStringPart>>),

    /// A function call expression
    FunctionCall(Box<Meta<Expr>>, Meta<Vec<Meta<Expr>>>),

    /// A field access expression
    Access(Box<Meta<Expr>>, Ident),

    /// A variable use
    Path(Meta<Path>),

    /// A record that doesn't have a type mentioned in the assignment of it
    ///
    /// For example: `{ value_1: 100, value_2: "bla" }`. This can also be a
    /// sub-record of a record that does have an explicit type.
    Record(Meta<Record>),

    /// An expression of a record that does have a type
    ///
    /// For example: `MyType { value_1: 100, value_2: "bla" }`, where `MyType`
    /// is a user-defined Record Type.
    TypedRecord(Meta<Path>, Meta<Record>),

    /// An expression that yields a list of values, e.g. `[100, 200, 300]`
    List(Vec<Meta<Expr>>),

    /// An assignment expression
    // TODO: Arbitrary place expressions should be allowed at some point, but
    //       for now that's not supported.
    Assign(Meta<Path>, Box<Meta<Expr>>),

    /// A binary operator expression
    ///
    /// Takes a left operand, the operator and the right operand
    Binary(Box<Meta<Expr>>, BinOp, Box<Meta<Expr>>),

    /// A unary not expression
    Not(Box<Meta<Expr>>),
    Negate(Box<Meta<Expr>>),
    /// Question mark operator
    QuestionMark(Box<Meta<Expr>>),

    /// An if or if-else expression
    Select(Box<Meta<Expr>>, Meta<Block>, Option<Meta<Block>>),
    /// A match expression,
    Match(Box<Meta<Match>>),
    /// A while expression
    Loop(Box<Meta<Expr>>, Meta<Block>),
}

#[derive(Clone, Debug)]
pub enum FStringPart {
    String(String),
    Expr(Meta<Expr>),
}

#[derive(Clone, Debug)]
pub enum ReturnKind {
    Return,
    Accept,
    Reject,
}

impl ReturnKind {
    #[must_use]
    pub fn str(&self) -> &'static str {
        match self {
            ReturnKind::Return => "return",
            ReturnKind::Accept => "accept",
            ReturnKind::Reject => "reject",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Record {
    pub fields: Vec<(Ident, Meta<Expr>)>,
}

#[derive(Clone, Debug)]
pub struct Match {
    pub expr: Meta<Expr>,
    pub arms: Vec<MatchArm>,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Meta<Pattern>,
    pub guard: Option<Meta<Expr>>,
    pub body: Meta<Block>,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Underscore,
    EnumVariant {
        variant: Ident,
        fields: Option<Meta<Vec<Ident>>>,
    },
}

#[derive(Clone, Debug)]
pub enum Literal {
    Unit,
    Bool(bool),
    Float(f64),
    String(String),
    Integer(i64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    /// Logical and (`&&`)
    And,
    /// Logical or (`||`)
    Or,

    /// Equals (`==`)
    Eq,
    /// Not equals (`!=`)
    Ne,
    /// Less than (`<`)
    Lt,
    /// Less than or equal (`<=`)
    Le,
    /// Greater than (`>`)
    Gt,
    /// Greater than or equal (`>=`)
    Ge,

    /// In
    In,
    /// Not in
    NotIn,

    /// Addition (`+`)
    Add,
    /// Subtraction (`-`)
    Sub,
    /// Multiplication (`*`)
    Mul,
    /// Division (`/`)
    Div,
}

impl core::fmt::Display for BinOp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<=",
            Self::Le => "<",
            Self::Gt => ">=",
            Self::Ge => ">",
            Self::In => "in",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::NotIn => "not in",
        })
    }
}
