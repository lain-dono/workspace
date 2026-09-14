use super::{BinaryOp, Span, Spanned, Token, UnaryOp, Value};
use chumsky::{prelude::*, recovery::NestedDelimiters};
use std::collections::HashMap;

// An expression node in the AST.
// Children are spanned so we can generate useful runtime errors.
#[derive(Debug)]
pub enum Expr {
    Error,
    Value(Value),
    List(Vec<Spanned<Self>>),
    Local(String),
    Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Then(Box<Spanned<Self>>, Box<Spanned<Self>>),

    Unary(UnaryOp, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),

    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Print(Box<Spanned<Self>>),
}

// A function node in the AST.
#[derive(Debug)]
pub struct Func {
    pub args: Vec<String>,
    pub body: Spanned<Expr>,
}

type Strategy = NestedDelimiters<Token, fn(Span) -> (Expr, Span), 2>;
fn strategy(a: (Token, Token), b: (Token, Token), c: (Token, Token)) -> Strategy {
    nested_delimiters(a.0, a.1, [b, c], |span| (Expr::Error, span))
}

pub fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let raw_expr = recursive(|raw_expr| {
            let literal = select! {
                Token::NULL => Expr::Value(Value::Null),
                Token::Bool(x) => Expr::Value(Value::Bool(x)),
                Token::Num(n) => Expr::Value(Value::Num(n.parse().unwrap())),
                Token::Str(s) => Expr::Value(Value::Str(s)),
            }
            .labelled("literal value");

            let ident = select! { Token::Ident(ident) => ident }.labelled("identifier");

            // A list of expressions
            let items = expr
                .clone()
                .separated_by(just(Token::COMMA))
                .allow_trailing();

            // A let expression
            let let_ = just(Token::LET)
                .ignore_then(ident)
                .then_ignore(just(Token::ASSIGN))
                .then(raw_expr)
                .then_ignore(just(Token::SEMICOLON))
                .then(expr.clone())
                .map(|((name, val), body)| Expr::Let(name, Box::new(val), Box::new(body)));

            let list = items
                .clone()
                .delimited_by(just(Token::LBRACK), just(Token::RBRACK))
                .map(Expr::List);

            // In Nano Rust, `print` is just a keyword, just like Python 2, for simplicity
            let print = just(Token::Print)
                .ignore_then(
                    expr.clone()
                        .delimited_by(just(Token::LPAREN), just(Token::RPAREN)),
                )
                .map(Box::new)
                .map(Expr::Print);

            // 'Atoms' are expressions that contain no ambiguity
            let atom = choice((literal, ident.map(Expr::Local), let_, list, print))
                .map_with_span(|expr, span| (expr, span))
                // Atoms can also just be normal expressions, but surrounded with parentheses
                .or(expr
                    .clone()
                    .delimited_by(just(Token::LPAREN), just(Token::RPAREN)))
                // Attempt to recover anything that looks like a parenthesised expression but contains errors
                .recover_with(strategy(
                    (Token::LPAREN, Token::RPAREN),
                    (Token::LBRACK, Token::RBRACK),
                    (Token::LBRACE, Token::RBRACE),
                ))
                // Attempt to recover anything that looks like a list but contains errors
                .recover_with(strategy(
                    (Token::LBRACK, Token::RBRACK),
                    (Token::LPAREN, Token::RPAREN),
                    (Token::LBRACE, Token::RBRACE),
                ));

            // Function calls have very high precedence so we prioritise them
            let call = atom
                .then(
                    items
                        .delimited_by(just(Token::LPAREN), just(Token::RPAREN))
                        .map_with_span(|args, span: Span| (args, span))
                        .repeated(),
                )
                .foldl(|f, args| {
                    let span = f.1.start..args.1.end;
                    (Expr::Call(Box::new(f), args.0), span)
                });

            macro_rules! parse_op {
                ($prev:ident => [ $op0:ident $(, $($op:ident),+ )? ]) => {
                    $prev
                        .clone()
                        .then(
                            just(Token::$op0).to(BinaryOp::$op0)
                            $($(
                                .or(just(Token::$op).to(BinaryOp::$op))
                            )+)?
                            .then($prev)
                            .repeated()
                        )
                        .foldl(|a, (op, b)| {
                            let span = a.1.start..b.1.end;
                            (Expr::Binary(Box::new(a), op, Box::new(b)), span)
                        })
                        .boxed()
                };
            }

            // product, sum, comparison, logic
            let op = parse_op!(call => [MUL, QUO, REM]);
            let op = parse_op!(op => [ADD, SUB]);
            let op = parse_op!(op => [EQ, NE, LT, LE, GT, GE]);
            let op = parse_op!(op => [LAND]);
            let op = parse_op!(op => [LIOR]);

            op
        });

        // Blocks are expressions but delimited with braces
        let block = expr
            .clone()
            .delimited_by(just(Token::LBRACE), just(Token::RBRACE))
            // Attempt to recover anything that looks like a block but contains errors
            .recover_with(strategy(
                (Token::LBRACE, Token::RBRACE),
                (Token::LPAREN, Token::RPAREN),
                (Token::LBRACK, Token::RBRACK),
            ));

        let if_ = recursive(|if_| {
            just(Token::IF)
                .ignore_then(expr.clone())
                .then(block.clone())
                .then(
                    just(Token::ELSE)
                        .ignore_then(block.clone().or(if_))
                        .or_not(),
                )
                .map_with_span(|((cond, a), b), span: Span| {
                    (
                        Expr::If(
                            Box::new(cond),
                            Box::new(a),
                            Box::new(match b {
                                Some(b) => b,
                                // If an `if` expression has no trailing `else` block, we magic up one that just produces null
                                None => (Expr::Value(Value::Null), span.clone()),
                            }),
                        ),
                        span,
                    )
                })
        });

        // Both blocks and `if` are 'block expressions' and can appear in the place of statements
        let block_expr = block.or(if_).labelled("block");

        let block_chain = block_expr
            .clone()
            .then(block_expr.clone().repeated())
            .foldl(|a, b| {
                let span = a.1.start..b.1.end;
                (Expr::Then(Box::new(a), Box::new(b)), span)
            });

        block_chain
            // Expressions, chained by semicolons, are statements
            .or(raw_expr.clone())
            .then(just(Token::SEMICOLON).ignore_then(expr.or_not()).repeated())
            .foldl(|a, b| {
                let span = a.1.clone(); // TODO: Not correct
                (
                    Expr::Then(
                        Box::new(a),
                        Box::new(match b {
                            Some(b) => b,
                            None => (Expr::Value(Value::Null), span.clone()),
                        }),
                    ),
                    span,
                )
            })
    })
}

pub fn funcs_parser() -> impl Parser<Token, HashMap<String, Func>, Error = Simple<Token>> + Clone {
    let ident = filter_map(|span, tok| match tok {
        Token::Ident(ident) => Ok(ident),
        _ => Err(Simple::expected_input_found(span, Vec::new(), Some(tok))),
    });

    // Argument lists are just identifiers separated by commas, surrounded by parentheses
    let args = ident
        .separated_by(just(Token::COMMA))
        .allow_trailing()
        .delimited_by(just(Token::LPAREN), just(Token::RPAREN))
        .labelled("function args");

    let func = just(Token::FN)
        .ignore_then(
            ident
                .map_with_span(|name, span| (name, span))
                .labelled("function name"),
        )
        .then(args)
        .then(
            expr_parser()
                .delimited_by(just(Token::LBRACE), just(Token::RBRACE))
                // Attempt to recover anything that looks like a function body but contains errors
                .recover_with(strategy(
                    (Token::LBRACE, Token::RBRACE),
                    (Token::LPAREN, Token::RPAREN),
                    (Token::LBRACK, Token::RBRACK),
                )),
        )
        .map(|((name, args), body)| (name, Func { args, body }))
        .labelled("function");

    func.repeated()
        .try_map(|fs, _| {
            let mut funcs = HashMap::new();
            for ((name, name_span), f) in fs {
                if funcs.insert(name.clone(), f).is_some() {
                    return Err(Simple::custom(
                        name_span,
                        format!("Function '{}' already exists", name),
                    ));
                }
            }
            Ok(funcs)
        })
        .then_ignore(end())
}
