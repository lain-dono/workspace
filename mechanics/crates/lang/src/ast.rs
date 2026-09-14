use crate::Span;
use crate::arena::{Arena, FastIndexSet, Handle};
use crate::ir::{BinaryOp, UnaryOp};
use crate::symbols::ScopeTable;
use crate::ty;
use std::hash::Hash;

pub type Spanned<T> = (T, Span);

pub struct ExprContext<'input, 'tmp, 'out> {
    pub exprs: &'out mut Arena<Expr<'input>>,
    pub types: &'out mut Arena<Type<'input>>,
    pub table: &'tmp mut ScopeTable<&'input str, Handle<Local>>,
    pub local: &'out mut Arena<Local>,
    pub found: &'out mut FastIndexSet<Dependency<'input>>,
}

#[derive(Debug, Default)]
pub struct TranslationUnit<'a> {
    pub decls: Arena<(GlobalDecl<'a>, FastIndexSet<Dependency<'a>>)>,
    pub exprs: Arena<Expr<'a>>,
    pub types: Arena<Type<'a>>,
    pub comments: Vec<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct Ident<'a> {
    pub name: &'a str,
    pub span: Span,
}

#[derive(Debug)]
pub enum IdentExpr<'a> {
    Unresolved(&'a str),
    Local(Handle<Local>),
}

#[derive(Debug)]
pub struct Dependency<'a> {
    pub ident: &'a str,
    pub usage: Span,
}

impl Hash for Dependency<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
    }
}

impl PartialEq for Dependency<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Eq for Dependency<'_> {}

#[derive(Debug)]
pub enum GlobalDecl<'a> {
    Func(Func<'a>),
    Const(Const<'a>),
    Struct(Struct<'a>),
    Type(TypeAlias<'a>),
}

#[derive(Debug)]
pub struct Func<'a> {
    pub name: Ident<'a>,
    pub params: Vec<FuncParam<'a>>,
    pub result: Option<Handle<Type<'a>>>,
    pub body: Block<'a>,
    pub comments: Vec<&'a str>,
}

#[derive(Debug)]
pub struct FuncParam<'a> {
    pub name: Ident<'a>,
    pub ty: Handle<Type<'a>>,
    pub handle: Handle<Local>,
}

#[derive(Debug)]
pub struct StructMember<'a> {
    pub name: Ident<'a>,
    pub ty: Handle<Type<'a>>,
    pub comments: Vec<&'a str>,
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub name: Ident<'a>,
    pub members: Vec<StructMember<'a>>,
    pub comments: Vec<&'a str>,
}

#[derive(Debug)]
pub struct TypeAlias<'a> {
    pub name: Ident<'a>,
    pub ty: Handle<Type<'a>>,
}

#[derive(Debug)]
pub struct Const<'a> {
    pub name: Ident<'a>,
    pub ty: Option<Handle<Type<'a>>>,
    pub init: Handle<Expr<'a>>,
    pub comments: Vec<&'a str>,
}

/// The size of an [`Array`] or [`BindingArray`].
///
/// [`Array`]: Type::Array
/// [`BindingArray`]: Type::BindingArray
#[derive(Debug, Copy, Clone)]
pub enum ArraySize<'a> {
    /// The length as a constant expression.
    Constant(Handle<Expr<'a>>),
    Dynamic,
}

#[derive(Debug)]
pub enum Type<'a> {
    Scalar(ty::Scalar),
    Pointer(Handle<Type<'a>>),
    Array(Handle<Type<'a>>, ArraySize<'a>),
    User(Ident<'a>),
}

pub type Block<'a> = Vec<Spanned<Stmt<'a>>>;

#[derive(Debug)]
pub enum Stmt<'a> {
    Block(Block<'a>),
    Let(Binding<'a>),
    Assign(Handle<Expr<'a>>, Option<BinaryOp>, Handle<Expr<'a>>),

    If(Handle<Expr<'a>>, Block<'a>, Block<'a>),
    Loop(Handle<Expr<'a>>, Block<'a>),

    Break(Option<Ident<'a>>, Option<Handle<Expr<'a>>>),
    Continue(Option<Ident<'a>>),
    Return(Option<Handle<Expr<'a>>>),
}

#[derive(Debug)]
pub enum Reject<'a> {
    Block(Block<'a>),
    Break(Option<Ident<'a>>, Option<Handle<Expr<'a>>>),
    Continue(Option<Ident<'a>>),
    Return(Option<Handle<Expr<'a>>>),
}

#[derive(Debug)]
pub enum Expr<'a> {
    Literal(crate::ty::Literal),
    Ident(IdentExpr<'a>),
    Init(ConstructorType<'a>, Span, Vec<Handle<Self>>),
    Unary(UnaryOp, Handle<Self>),
    Binary(BinaryOp, Handle<Self>, Handle<Self>),
    Call(Ident<'a>, Vec<Handle<Self>>),
    Index(Handle<Self>, Handle<Self>),
    Member(Handle<Self>, Ident<'a>),
}

#[derive(Debug)]
pub enum ConstructorType<'a> {
    Scalar(ty::Scalar),
    Array(Handle<Type<'a>>, ArraySize<'a>),
    Lower(Handle<ty::Type>),
}

#[derive(Debug)]
pub struct Binding<'a> {
    pub name: Ident<'a>,
    pub ty: Option<Handle<Type<'a>>>,
    pub init: Handle<Expr<'a>>,
    pub handle: Handle<Local>,
}

#[derive(Debug)]
pub struct Local;
