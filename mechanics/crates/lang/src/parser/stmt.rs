use super::{Extra, arena::Handle, expr::Expr, token::Token};
use chumsky::input::MapExtra;
use chumsky::{input::ValueInput, prelude::*};

pub type Block<'src> = Vec<Stmt<'src>>;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt<'src> {
    Invalid,

    Block(Block<'src>),
    Expr(Handle<Expr<'src>>),
    Let(&'src str, Handle<Expr<'src>>, Option<Reject<'src>>),
    Break(Option<&'src str>),
    Continue(Option<&'src str>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Reject<'src> {
    Block(Block<'src>),
    Break(Option<&'src str>),
    Continue(Option<&'src str>),
}

impl<'src> Stmt<'src> {
    pub fn parser<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>()
    -> impl Parser<'src, I, Self, Extra<'src, S>> + Clone {
        use Token::*;

        let atom = select! {
            Token::Ident(ident) => Expr::Literal(ident),
            Token::Number(num) => Expr::Number(num),
        };
        let atom = atom.map_with(|expr, e: &mut MapExtra<'src, '_, I, Extra<'src, S>>| {
            e.state().exprs.append(expr)
        });

        recursive(|element| {
            let block = Self::block(element.clone());
            let label = select! { Token::Ident(ident) => ident };
            let label = just(Colon).ignore_then(label).or_not();

            let expr = Expr::parser(atom);
            let expr_stmt = expr.clone().map(Self::Expr);

            let let_pattern = select! { Ident(ident) => ident };
            let let_else = just(Else).ignore_then(choice((
                block.clone().map(Reject::Block),
                just(Continue).ignore_then(label).map(Reject::Continue),
                just(Break).ignore_then(label).map(Reject::Break),
            )));

            let let_stmt = group((
                just(Let),
                let_pattern,
                just(Assign),
                expr,
                let_else.or_not(),
            ));

            let fallback = |_| Self::Invalid;
            let paren = (Lparen, Rparen);
            let brace = (Lbrace, Rbrace);
            let brack = (Lbrack, Rbrack);

            choice((
                let_stmt.map(|(_, lhs, _, rhs, reject)| Self::Let(lhs, rhs, reject)),
                expr_stmt,
                just(Continue).ignore_then(label).map(Self::Continue),
                just(Break).ignore_then(label).map(Self::Break),
                block.map(Self::Block),
            ))
            .recover_with(via_parser(nested_delimiters(
                Lbrace,
                Rbrace,
                [brack, paren],
                fallback,
            )))
            .recover_with(via_parser(nested_delimiters(
                Lbrack,
                Rbrack,
                [brace, paren],
                fallback,
            )))
            .recover_with(via_parser(nested_delimiters(
                Lparen,
                Rparen,
                [brace, brack],
                fallback,
            )))
            .recover_with(skip_then_retry_until(
                any().ignored(),
                one_of([Semi, Rbrack, Rparen, Rbrace]).ignored(),
            ))
        })
    }

    pub fn block<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>(
        element: impl Parser<'src, I, Self, Extra<'src, S>> + Clone,
    ) -> impl Parser<'src, I, Block<'src>, Extra<'src, S>> + Clone {
        use Token::*;

        element
            .separated_by(just(Semi).recover_with(skip_then_retry_until(
                any().ignored(),
                // one_of([Semi, Rbrace]).ignored(),
                just(Rbrace).ignored(),
            )))
            //.allow_leading()
            .allow_trailing()
            .collect()
            .delimited_by(
                just(Lbrace),
                just(Rbrace)
                    .ignored()
                    .recover_with(via_parser(end()))
                    .recover_with(skip_then_retry_until(any().ignored(), end())),
            )
    }
}

#[test]
fn parse_stmt() {
    use super::{BinaryOp, Storage};

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("{ { break }\n{ continue } }").unwrap();
    dbg!(input.as_slice().iter().map(|(t, _)| *t).collect::<Vec<_>>());
    let input = input.map(eoi, |(t, s)| (t, s));
    let result = Stmt::parser().parse_with_state(input, &mut arena).unwrap();
    assert_eq!(
        result,
        Stmt::Block(vec![
            Stmt::Block(vec![Stmt::Break(None)]),
            Stmt::Block(vec![Stmt::Continue(None)]),
        ])
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("{ a += b }").unwrap();
    dbg!(input.as_slice().iter().map(|(t, _)| *t).collect::<Vec<_>>());
    let input = input.map(eoi, |(t, s)| (t, s));
    let result = Stmt::parser().parse_with_state(input, &mut arena).unwrap();

    assert_eq!(result, Stmt::Block(vec![Stmt::Expr(Handle::from_usize(2))]));
    assert_eq!(
        &arena.exprs.memory[..],
        [
            Expr::Literal("a"),
            Expr::Literal("b"),
            Expr::Assign(
                Handle::from_usize(0),
                Some(BinaryOp::Add),
                Handle::from_usize(1)
            ),
        ]
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("{ let a = foo; let b = bar; break\ncontinue }").unwrap();
    dbg!(input.as_slice().iter().map(|(t, _)| *t).collect::<Vec<_>>());
    let input = input.map(eoi, |(t, s)| (t, s));
    let result = Stmt::parser().parse_with_state(input, &mut arena).unwrap();

    assert_eq!(
        result,
        Stmt::Block(vec![
            Stmt::Let("a", Handle::from_usize(0), None),
            Stmt::Let("b", Handle::from_usize(1), None),
            Stmt::Break(None),
            Stmt::Continue(None)
        ])
    );

    assert_eq!(
        &arena.exprs.memory[..],
        [Expr::Literal("foo"), Expr::Literal("bar")]
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("{ let a = b + c\n a += b }").unwrap();
    dbg!(input.as_slice().iter().map(|(t, _)| *t).collect::<Vec<_>>());
    let input = input.map(eoi, |(t, s)| (t, s));
    let result = Stmt::parser().parse_with_state(input, &mut arena).unwrap();

    assert_eq!(
        result,
        Stmt::Block(vec![
            Stmt::Let("a", Handle::from_usize(2), None),
            Stmt::Expr(Handle::from_usize(5)),
        ])
    );

    assert_eq!(
        &arena.exprs.memory[..],
        [
            Expr::Literal("b"),
            Expr::Literal("c"),
            Expr::Binary(Handle::from_usize(0), BinaryOp::Add, Handle::from_usize(1)),
            Expr::Literal("a"),
            Expr::Literal("b"),
            Expr::Assign(
                Handle::from_usize(3),
                Some(BinaryOp::Add),
                Handle::from_usize(4)
            ),
        ]
    );
}
