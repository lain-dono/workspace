use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use std::collections::HashMap;

pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

mod ast;
mod interpreter;
mod token;

pub use self::ast::*;
pub use self::interpreter::*;
pub use self::token::Token::*;
pub use self::token::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    List(Vec<Value>),
    Func(String),
}

impl Value {
    fn number(self, span: &Span) -> Result<f64, Error> {
        match self {
            Value::Num(x) => Ok(x),
            _ => Err(Error {
                span: span.clone(),
                msg: format!("'{}' is not a number", self),
            }),
        }
    }

    fn boolean(self, span: &Span) -> Result<bool, Error> {
        match self {
            Value::Bool(x) => Ok(x),
            _ => Err(Error {
                span: span.clone(),
                msg: format!("'{}' is not a boolean", self),
            }),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Num(a), Self::Num(b)) => Some(a.total_cmp(b)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::Bool(x) => write!(f, "{}", x),
            Self::Num(x) => write!(f, "{}", x),
            Self::Str(x) => write!(f, "{}", x),
            Self::List(xs) => write!(
                f,
                "[{}]",
                xs.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Func(name) => write!(f, "<function: {}>", name),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum UnaryOp {
    NEG,
    NOT,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NEG => write!(f, "-"),
            Self::NOT => write!(f, "!"),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub enum BinaryOp {
    ADD, // +
    SUB, // -
    MUL, // *
    QUO, // /
    REM, // %

    EQ, // ==
    NE, // !=

    LE, // <=
    LT, // <
    GE, // >=
    GT, // >

    LAND, // &&
    LIOR, // ||

          //  BAND,  // &
          //  BIOR,   // |
          //  BEOR,  // ^
          //  SHFL,  // <<
          //  SHFR,  // >>
          //  NAND, // &^
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ADD => write!(f, "+"),
            Self::SUB => write!(f, "-"),
            Self::MUL => write!(f, "*"),
            Self::QUO => write!(f, "/"),
            Self::REM => write!(f, "%"),
            Self::EQ => write!(f, "=="),
            Self::NE => write!(f, "!="),
            Self::LT => write!(f, "<"),
            Self::LE => write!(f, "<="),
            Self::GT => write!(f, ">"),
            Self::GE => write!(f, ">="),
            Self::LAND => write!(f, "&&"),
            Self::LIOR => write!(f, "||"),
        }
    }
}

pub struct Error {
    pub span: Span,
    pub msg: String,
}

fn print_ast(ast: &HashMap<String, Func>) {
    for (name, fun) in ast {
        println!("fn {}({:?}) {{", name, fun.args);
        print_expr(Some(1), &fun.body);
        println!("}}");
    }
}

fn print_expr(level: Option<usize>, expr: &Spanned<Expr>) {
    fn print_level(level: Option<usize>) {
        if let Some(level) = level {
            for _ in 0..level {
                print!("  ");
            }
        }
    }

    print_level(level);

    match &expr.0 {
        Expr::Error => unreachable!(),
        Expr::Value(val) => {
            print!("{:?}", val);
            return;
        }
        Expr::List(items) => {
            println!("[");
            for expr in items {
                print_expr(level.map(|v| v + 1), expr)
            }
            print!("]")
        }
        Expr::Local(name) => print!("{}", name),
        Expr::Let(local, val, body) => {
            print!("let {}", local);
            print!(" = ");
            print_expr(None, val);
            println!(";");
            print_expr(level, body);
            return;
        }
        Expr::Then(a, b) => {
            //println!("_start");
            print_expr(level, a);
            print_expr(level, b);

            //print_level(level);
            //println!("_end");
        }
        Expr::Unary(op, b) => {
            print!(" {}", op);
            print_expr(None, b);
        }
        Expr::Binary(a, op, b) => {
            print_expr(None, a);
            print!(" {} ", op);
            print_expr(None, b);
        }
        Expr::Call(func, args) => {
            print_expr(None, func);
            print!("(");
            for arg in args {
                print_expr(None, arg);
            }
            print!(")");
        }
        Expr::If(cond, a, b) => {
            print!("if ");
            print_expr(None, cond);

            println!(" {{");

            print_expr(level.map(|v| v + 1), a);

            print_level(level);
            println!("}} else {{");

            print_expr(level.map(|v| v + 1), b);

            print_level(level);
            println!("}}");
            return;
        }
        Expr::Print(arg) => {
            print!("print(");
            print_expr(None, arg);
            println!(");");
            return;
        }
    }

    if level.is_some() {
        println!()
    }
}

pub fn run(stream: &str) {
    let (tokens, mut errs) = lexer().parse_recovery(stream);

    let parse_errs = if let Some(tokens) = tokens {
        //dbg!(tokens);
        if false {
            let mut level = 0;
            for (tok, _span) in &tokens {
                match tok {
                    Token::RPAREN | Token::RBRACE | Token::RBRACK => level -= 1,
                    _ => (),
                }

                for _ in 0..level {
                    print!("  ");
                }

                println!("{}\t\t{:?}", tok, tok);

                match tok {
                    Token::LPAREN | Token::LBRACE | Token::LBRACK => level += 1,
                    _ => (),
                }
            }
        }

        let len = stream.chars().count();
        let (ast, parse_errs) =
            funcs_parser().parse_recovery(Stream::from_iter(len..len + 1, tokens.into_iter()));

        if let Some(ast) = &ast {
            //dbg!(ast);
            //print_ast(ast)
        }

        if let Some(funcs) = ast.filter(|_| errs.len() + parse_errs.len() == 0) {
            if let Some(main) = funcs.get("main") {
                assert_eq!(main.args.len(), 0);
                match Interpreter::new(&funcs).eval(&main.body) {
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
