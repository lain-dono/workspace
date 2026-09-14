use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream, Parser};
use std::{collections::HashMap, env, fs};

mod token;
mod value;

pub use self::token::*;
pub use self::value::Value;

pub fn main() {
    let src = fs::read_to_string(env::args().nth(1).expect("Expected file argument"))
        .expect("Failed to read file");

    run(&src);
}

pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
enum BinaryOp {
    ADD, // +
    SUB, // -
    MUL, // *
    QUO, // /
    REM, // %

    EQ, // ==
    NE, // !=

    LT, // <
    LE, // <=
    GT, // >
    GE, // >=

    LAND, // &&
    LOR,  // ||

          //  AND,  // &
          //  OR,   // |
          //  XOR,  // ^
          //  SHL,  // <<
          //  SHR,  // >>
          //  NAND, // &^
}

// An expression node in the AST. Children are spanned so we can generate useful runtime errors.
#[derive(Debug)]
enum Expr {
    Error,
    Value(Value),
    List(Vec<Spanned<Self>>),
    Local(String),
    Let(String, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Then(Box<Spanned<Self>>, Box<Spanned<Self>>),
    Binary(Box<Spanned<Self>>, BinaryOp, Box<Spanned<Self>>),
    Call(Box<Spanned<Self>>, Vec<Spanned<Self>>),
    If(Box<Spanned<Self>>, Box<Spanned<Self>>, Box<Spanned<Self>>),
    Print(Box<Spanned<Self>>),
}

// A function node in the AST.
#[derive(Debug)]
struct Func {
    args: Vec<String>,
    body: Spanned<Expr>,
}

fn strategy(
    a: (Token, Token),
    b: (Token, Token),
    c: (Token, Token),
) -> chumsky::recovery::NestedDelimiters<Token, fn(Span) -> Spanned<Expr>, 2> {
    nested_delimiters(a.0, a.1, [b, c], |span| (Expr::Error, span))
}

fn expr_parser() -> impl Parser<Token, Spanned<Expr>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let raw_expr = recursive(|raw_expr| {
            let val = select! {
                Token::NULL => Expr::Value(Value::Null),
                Token::Bool(x) => Expr::Value(Value::Bool(x)),
                Token::Num(n) => Expr::Value(Value::Num(n.parse().unwrap())),
                Token::Str(s) => Expr::Value(Value::Str(s)),
            }
            .labelled("value");

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

            // 'Atoms' are expressions that contain no ambiguity
            let atom = val
                .or(ident.map(Expr::Local))
                .or(let_)
                .or(list)
                // In Nano Rust, `print` is just a keyword, just like Python 2, for simplicity
                .or(just(Token::Print)
                    .ignore_then(
                        expr.clone()
                            .delimited_by(just(Token::LPAREN), just(Token::RPAREN)),
                    )
                    .map(|expr| Expr::Print(Box::new(expr))))
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
                };
            }

            // product, sum, comparison, logic
            let op = parse_op!(call => [MUL, QUO, REM]);
            let op = parse_op!(op => [ADD, SUB]);
            let op = parse_op!(op => [EQ, NE, LT, LE, GT, GE]);
            let op = parse_op!(op => [LAND]);
            let op = parse_op!(op => [LOR]);

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

fn funcs_parser() -> impl Parser<Token, HashMap<String, Func>, Error = Simple<Token>> + Clone {
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
                .delimited_by(just(Token::LBRACE), just(Token::LBRACE))
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

struct Error {
    span: Span,
    msg: String,
}

struct Interpreter<'a> {
    funcs: &'a HashMap<String, Func>,
    stack: Vec<(String, Value)>,
}

impl<'a> Interpreter<'a> {
    fn new(funcs: &'a HashMap<String, Func>) -> Self {
        Self {
            funcs,
            stack: Vec::new(),
        }
    }

    fn num(&mut self, e: &Spanned<Expr>) -> Result<f64, Error> {
        let value = self.expr(e)?;
        match value {
            Value::Num(x) => Ok(x),
            _ => Err(Error {
                span: e.1.clone(),
                msg: format!("'{}' is not a number", value),
            }),
        }
    }

    fn bool(&mut self, e: &Spanned<Expr>) -> Result<bool, Error> {
        let value = self.expr(e)?;
        match value {
            Value::Bool(x) => Ok(x),
            _ => Err(Error {
                span: e.1.clone(),
                msg: format!("'{}' is not a boolean", value),
            }),
        }
    }

    fn expr(&mut self, expr: &Spanned<Expr>) -> Result<Value, Error> {
        Ok(match &expr.0 {
            Expr::Error => unreachable!(), // Error expressions only get created by parser errors, so cannot exist in a valid AST

            Expr::Value(val) => val.clone(),

            Expr::List(items) => Value::List(
                items
                    .iter()
                    .map(|item| self.expr(item))
                    .collect::<Result<_, _>>()?,
            ),

            Expr::Local(name) => self
                .stack
                .iter()
                .rev()
                .find(|(l, _)| l == name)
                .map(|(_, v)| v.clone())
                .or_else(|| {
                    Some(Value::Func(name.clone())).filter(|_| self.funcs.contains_key(name))
                })
                .ok_or_else(|| Error {
                    span: expr.1.clone(),
                    msg: format!("No such variable '{}' in scope", name),
                })?,

            Expr::Let(local, val, body) => {
                let val = self.expr(val)?;
                self.stack.push((local.clone(), val));
                let res = self.expr(body)?;
                self.stack.pop();
                res
            }

            Expr::Then(a, b) => {
                self.expr(a)?;
                self.expr(b)?
            }

            Expr::Binary(a, BinaryOp::ADD, b) => Value::Num(self.num(a)? + self.num(b)?),
            Expr::Binary(a, BinaryOp::SUB, b) => Value::Num(self.num(a)? - self.num(b)?),
            Expr::Binary(a, BinaryOp::MUL, b) => Value::Num(self.num(a)? * self.num(b)?),
            Expr::Binary(a, BinaryOp::QUO, b) => Value::Num(self.num(a)? / self.num(b)?),
            Expr::Binary(a, BinaryOp::REM, b) => Value::Num(self.num(a)? % self.num(b)?),

            Expr::Binary(a, BinaryOp::EQ, b) => Value::Bool(self.expr(a)? == self.expr(b)?),
            Expr::Binary(a, BinaryOp::LT, b) => Value::Bool(self.expr(a)? < self.expr(b)?),
            Expr::Binary(a, BinaryOp::GT, b) => Value::Bool(self.expr(a)? > self.expr(b)?),

            Expr::Binary(a, BinaryOp::NE, b) => Value::Bool(self.expr(a)? != self.expr(b)?),
            Expr::Binary(a, BinaryOp::LE, b) => Value::Bool(self.expr(a)? <= self.expr(b)?),
            Expr::Binary(a, BinaryOp::GE, b) => Value::Bool(self.expr(a)? >= self.expr(b)?),

            Expr::Binary(a, BinaryOp::LAND, b) => Value::Bool(self.bool(a)? && self.bool(b)?),
            Expr::Binary(a, BinaryOp::LOR, b) => Value::Bool(self.bool(a)? || self.bool(b)?),

            Expr::Call(func, args) => {
                let f = self.expr(func)?;
                match f {
                    Value::Func(name) => {
                        let f = &self.funcs[&name];
                        let stack = if f.args.len() != args.len() {
                            return Err(Error {
                            span: expr.1.clone(),
                            msg: format!("'{}' called with wrong number of arguments (expected {}, found {})", name, f.args.len(), args.len()),
                        });
                        } else {
                            f.args
                                .iter()
                                .zip(args.iter())
                                .map(|(name, arg)| Ok((name.clone(), self.expr(arg)?)))
                                .collect::<Result<_, _>>()?
                        };
                        let mut context = Self {
                            funcs: self.funcs,
                            stack,
                        };
                        context.expr(&f.body)?
                    }
                    f => {
                        return Err(Error {
                            span: func.1.clone(),
                            msg: format!("'{:?}' is not callable", f),
                        })
                    }
                }
            }

            Expr::If(cond, a, b) => {
                let c = self.expr(cond)?;
                match c {
                    Value::Bool(true) => self.expr(a)?,
                    Value::Bool(false) => self.expr(b)?,
                    c => {
                        return Err(Error {
                            span: cond.1.clone(),
                            msg: format!("Conditions must be booleans, found '{:?}'", c),
                        })
                    }
                }
            }

            Expr::Print(a) => {
                let val = self.expr(a)?;
                println!("{}", val);
                val
            }
        })
    }
}

fn run(stream: &str) {
    let (tokens, mut errs) = lexer().parse_recovery(stream);

    let parse_errs = if let Some(tokens) = tokens {
        let mut level = 0;
        for (tok, _span) in &tokens {
            match tok {
                Token::RPAREN | Token::RBRACE | Token::RBRACK => level -= 1,
                _ => (),
            }

            for _ in 0..level {
                print!("  ");
            }

            println!("{} {:?}", tok, tok);

            match tok {
                Token::LPAREN | Token::LBRACE | Token::LBRACK => level += 1,
                _ => (),
            }
        }
        //dbg!(&tokens);

        let len = stream.chars().count();
        let (ast, parse_errs) =
            funcs_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        //dbg!(ast);

        if let Some(funcs) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            if let Some(main) = funcs.get("main") {
                assert_eq!(main.args.len(), 0);
                match Interpreter::new(&funcs).expr(&main.body) {
                    Ok(val) => println!("Return value: {}", val),
                    Err(e) => errs.push(Simple::custom(e.span, e.msg)),
                }
            } else {
                panic!("No main function!");
            }
        }

        parse_errs
    } else {
        Vec::new()
    };

    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_label(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_color(Color::Yellow),
                    )
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Must be closed before this {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}, expected {}",
                        if e.found().is_some() {
                            "Unexpected token in input"
                        } else {
                            "Unexpected end of input"
                        },
                        if e.expected().len() == 0 {
                            "something else".to_string()
                        } else {
                            e.expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ))
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Unexpected token {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
            };

            report.finish().print(Source::from(&stream)).unwrap();
        });
}
