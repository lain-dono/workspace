use super::{Extra, Handle, Token};
use chumsky::{
    input::{MapExtra, ValueInput},
    pratt::{Operator, infix, left, prefix, right},
    prelude::*,
};
use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinaryOp {
    Div,
    Mul,
    Rem,

    Add,
    Sub,

    And,
    Eor,
    Ior,

    Shl,
    Shr,
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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOp {
    Deref,
    Refer,
    Neg,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr<'src> {
    Error,

    Literal(&'src str),
    Number(&'src str),

    Unary(UnaryOp, Handle<Self>),

    Binary(Handle<Self>, BinaryOp, Handle<Self>),

    Assign(Handle<Self>, Option<BinaryOp>, Handle<Self>),
}

impl<'src> Expr<'src> {
    pub fn parser<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>(
        atom: impl Parser<'src, I, Handle<Self>, Extra<'src, S>> + Clone,
    ) -> impl Parser<'src, I, Handle<Self>, Extra<'src, S>> + Clone {
        atom.pratt((
            // - * ! &
            prefix::<_, _, _, _, _, Extra<'src, S>>(
                18,
                choice((
                    just(Token::Sub),
                    just(Token::Mul),
                    just(Token::Not),
                    just(Token::And),
                )),
                move |op, atom, e: &mut MapExtra<'src, '_, I, Extra<'src, S>>| {
                    e.state().exprs.append(Expr::Unary(
                        match op {
                            Token::Sub => UnaryOp::Neg,
                            Token::Mul => UnaryOp::Deref,
                            Token::Not => UnaryOp::Not,
                            Token::And => UnaryOp::Refer,
                            _ => unreachable!(),
                        },
                        atom,
                    ))
                },
            ),
            // / * %
            Self::token_infix_left(17, Token::Div, BinaryOp::Div),
            Self::token_infix_left(17, Token::Mul, BinaryOp::Mul),
            Self::token_infix_left(17, Token::Rem, BinaryOp::Rem),
            // + -
            Self::token_infix_left(16, Token::Add, BinaryOp::Add),
            Self::token_infix_left(16, Token::Sub, BinaryOp::Sub),
            // & ^ |
            Self::token_infix_left(15, Token::And, BinaryOp::And),
            Self::token_infix_left(14, Token::Eor, BinaryOp::Eor),
            Self::token_infix_left(13, Token::Ior, BinaryOp::Ior),
            // todo cmp
            infix::<_, _, _, _, _, Extra<'src, S>>(
                right(12),
                choice((
                    just(Token::Assign),
                    // /= *= %=
                    just(Token::DivAssign),
                    just(Token::MulAssign),
                    just(Token::RemAssign),
                    // += -=
                    just(Token::AddAssign),
                    just(Token::SubAssign),
                    // &= ^= |=
                    just(Token::AndAssign),
                    just(Token::EorAssign),
                    just(Token::IorAssign),
                )),
                move |lhs, op, rhs, e| {
                    let op = match op {
                        Token::Assign => None,
                        // /= *= %=
                        Token::DivAssign => Some(BinaryOp::Div),
                        Token::MulAssign => Some(BinaryOp::Mul),
                        Token::RemAssign => Some(BinaryOp::Rem),
                        // += -=
                        Token::AddAssign => Some(BinaryOp::Add),
                        Token::SubAssign => Some(BinaryOp::Sub),
                        // &= ^= |=
                        Token::AndAssign => Some(BinaryOp::And),
                        Token::EorAssign => Some(BinaryOp::Eor),
                        Token::IorAssign => Some(BinaryOp::Ior),

                        _ => unreachable!(),
                    };
                    e.state().exprs.append(Self::Assign(lhs, op, rhs))
                },
            ),
        ))
    }

    fn token_infix_left<I: ValueInput<'src, Token = Token<'src>, Span = S>, S: 'static>(
        binding_power: u16,
        token: Token<'src>,
        op: BinaryOp,
    ) -> impl Operator<'src, I, Handle<Expr<'src>>, Extra<'src, S>> + Clone {
        infix::<_, _, _, _, _, Extra<'src, S>>(
            left(binding_power),
            just(token),
            move |l, _, r, e| e.state().exprs.append(Expr::Binary(l, op, r)),
        )
    }
}

#[test]
fn parse() {
    use super::Storage;

    type Span = SimpleSpan;

    fn respan<'src, 'a>((t, s): &'a (Token<'src>, Span)) -> (&'a Token<'src>, &'a Span) {
        (t, s)
    }

    let atom = {
        let num = select! { Token::Number(num) => num };
        let num = num.map_with(|num, e: &mut MapExtra<'_, '_, _, Extra<'_, SimpleSpan>>| {
            e.state().exprs.append(Expr::Number(num))
        });

        let ident = select! { Token::Ident(ident) => ident };
        let ident = ident.map_with(
            |ident, e: &mut MapExtra<'_, '_, _, Extra<'_, SimpleSpan>>| {
                e.state().exprs.append(Expr::Literal(ident))
            },
        );

        choice((num, ident))
    };

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("*1 + -2 - -3*7").unwrap();
    let input = input.map(eoi, respan);
    let _result = Expr::parser(atom).parse_with_state(input, &mut arena);
    assert_eq!(
        &arena.exprs.memory[..],
        [
            Expr::Number("1"),
            Expr::Unary(UnaryOp::Deref, Handle::from_usize(0)),
            Expr::Number("2"),
            Expr::Unary(UnaryOp::Neg, Handle::from_usize(2)),
            Expr::Binary(Handle::from_usize(1), BinaryOp::Add, Handle::from_usize(3)),
            Expr::Number("3"),
            Expr::Unary(UnaryOp::Neg, Handle::from_usize(5)),
            Expr::Number("7"),
            Expr::Binary(Handle::from_usize(6), BinaryOp::Mul, Handle::from_usize(7)),
            Expr::Binary(Handle::from_usize(4), BinaryOp::Sub, Handle::from_usize(8)),
        ]
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("1 + 2 * 3").unwrap();
    let input = input.map(eoi, respan);
    let _result = Expr::parser(atom).parse_with_state(input, &mut arena);
    assert_eq!(
        &arena.exprs.memory[..],
        [
            Expr::Number("1"),
            Expr::Number("2"),
            Expr::Number("3"),
            Expr::Binary(Handle::from_usize(1), BinaryOp::Mul, Handle::from_usize(2)),
            Expr::Binary(Handle::from_usize(0), BinaryOp::Add, Handle::from_usize(3)),
        ]
    );

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let (input, eoi) = Token::scan("a += 2 * 3").unwrap();
    let input = input.map(eoi, respan);
    let _result = Expr::parser(atom).parse_with_state(input, &mut arena);
    assert_eq!(
        &arena.exprs.memory[..],
        [
            Expr::Literal("a"),
            Expr::Number("2"),
            Expr::Number("3"),
            Expr::Binary(Handle::from_usize(1), BinaryOp::Mul, Handle::from_usize(2)),
            Expr::Assign(
                Handle::from_usize(0),
                Some(BinaryOp::Add),
                Handle::from_usize(3)
            ),
        ]
    );
}
