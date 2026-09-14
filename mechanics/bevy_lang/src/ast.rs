pub type Ident = String;
pub type Block = Vec<Stmt>;

pub enum Token {}

pub enum Decl {}

pub enum Stmt {
    Expr(Expr),
    Assign(Ident, Token, Vec<Expr>),
    Labeled(Ident, Box<Self>),
    Block(Block),
    Return(Vec<Expr>),
    Break(Option<Ident>),
    Continue(Option<Ident>),
    If(Expr, Block, Option<Block>),
    For(Option<Box<Self>>, Block),
}

pub enum Expr {
    Ident(Ident),
    Literal(super::literal::Literal),
    Unary(Token, Box<Self>),
    Binary(Box<Self>, Token, Box<Self>),
    Call(Box<Self>, Vec<Self>),
    Paren(Box<Self>),
    Index(Box<Self>, Box<Self>),
    Access(Box<Self>, Ident),
}
