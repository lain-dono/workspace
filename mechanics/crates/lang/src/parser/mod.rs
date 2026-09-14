use chumsky::{input::ValueInput, prelude::*};

mod arena;
mod decl;
mod expr;
mod stmt;
mod token;
mod ty;

pub use self::arena::{Arena, Handle, UniqueArena};
pub use self::decl::{Decl, FuncDecl, FuncParam, StructField};
pub use self::expr::{BinaryOp, Expr, UnaryOp};
pub use self::stmt::{Block, Reject, Stmt};
pub use self::token::Token;
pub use self::ty::{Scalar, StructMember, Ty, Type};

/*  notes:
    [return, break, continue] must be last statement in block
    [break, continue] must be in loop or labeled block
    last [return] in function body can be omited
    [return] can be [break]
*/

#[derive(Debug, PartialEq)]
pub struct Module<'src> {
    pub decls: Vec<Decl<'src>>,
}

impl<'src> Module<'src> {
    pub fn parser<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>()
    -> impl Parser<'src, I, Self, Extra<'src, S>> {
        Decl::parser()
            .repeated()
            .collect::<Vec<_>>()
            .then_ignore(just(Token::Semi).or_not())
            .then_ignore(end())
            .map(|decls| Self { decls })
    }
}

#[test]
fn module_parse() {
    let input = "
type Foo struct {
    a i32
    b isize
}

fn foo(self Foo) i32 {}

type Bar struct { a i32 }

fn bar(self Bar) i32 {}

fn add(a i32, b i32) i32 { a + b }

fn fun() {
    let a = a + b else continue
    a += b
    break

    {
        let a = a + b else continue
        a += b
        break
    }
}
";
    let (input, eoi) = Token::scan(input).unwrap();
    dbg!(input.as_slice().iter().map(|(t, _)| *t).collect::<Vec<_>>());
    let input = input.map(eoi, |(t, s)| (t, s));

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let parser = Module::parser();
    let result = parser.parse_with_state(input, &mut arena).unwrap();

    assert_eq!(
        &arena.exprs.memory[..],
        [
            Expr::Literal("a"),
            Expr::Literal("b"),
            Expr::Binary(Handle::from_usize(0), BinaryOp::Add, Handle::from_usize(1)), // 2
            Expr::Literal("a"),
            Expr::Literal("b"),
            Expr::Binary(Handle::from_usize(3), BinaryOp::Add, Handle::from_usize(4)), // 5
            Expr::Literal("a"),
            Expr::Literal("b"),
            Expr::Assign(
                Handle::from_usize(6),
                Some(BinaryOp::Add),
                Handle::from_usize(7)
            ), // 8
            Expr::Literal("a"),
            Expr::Literal("b"),
            Expr::Binary(Handle::from_usize(9), BinaryOp::Add, Handle::from_usize(10)), // 11
            Expr::Literal("a"),
            Expr::Literal("b"),
            Expr::Assign(
                Handle::from_usize(12),
                Some(BinaryOp::Add),
                Handle::from_usize(13)
            ), // 14
        ]
    );
    let decls = vec![
        Decl::Ty {
            name: "Foo",
            fields: vec![StructField::new("a", "i32"), StructField::new("b", "isize")],
        },
        Decl::Func(FuncDecl {
            name: "foo",
            params: vec![FuncParam::new("self", "Foo")],
            result: Some("i32"),
            body: vec![],
        }),
        Decl::Ty {
            name: "Bar",
            fields: vec![StructField::new("a", "i32")],
        },
        Decl::Func(FuncDecl {
            name: "bar",
            params: vec![FuncParam::new("self", "Bar")],
            result: Some("i32"),
            body: vec![],
        }),
        Decl::Func(FuncDecl {
            name: "add",
            params: vec![FuncParam::new("a", "i32"), FuncParam::new("b", "i32")],
            result: Some("i32"),
            body: vec![Stmt::Expr(Handle::from_usize(2))],
        }),
        Decl::Func(FuncDecl {
            name: "fun",
            params: vec![],
            result: None,
            body: vec![
                Stmt::Let("a", Handle::from_usize(5), Some(Reject::Continue(None))),
                Stmt::Expr(Handle::from_usize(8)),
                Stmt::Break(None),
                Stmt::Block(vec![
                    Stmt::Let("a", Handle::from_usize(11), Some(Reject::Continue(None))),
                    Stmt::Expr(Handle::from_usize(14)),
                    Stmt::Break(None),
                ]),
            ],
        }),
    ];

    assert_eq!(result, Module { decls });
}

fn simplify_parser<'src, I, O, S: 'static>(
    parser: impl Parser<'src, I, O, Extra<'src, S>> + Clone,
) -> impl Parser<'src, I, O, Extra<'src, S>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = S>,
{
    parser
}

#[derive(Default, Clone)]
pub struct Storage<'src> {
    pub types: UniqueArena<Type<'src>>,
    pub exprs: Arena<Expr<'src>>,
}

pub type Extra<'src, S = SimpleSpan> =
    extra::Full<Rich<'src, Token<'src>, S>, extra::SimpleState<Storage<'src>>, ()>;
