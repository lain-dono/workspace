use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{prelude::*, stream::Stream};
use std::collections::HashMap;

pub mod ast;
pub mod literal;
pub mod program;
pub mod run;

/*
fn main() {

    self::run::run(src.as_str());
}
*/

fn main() {
    let path = std::env::args().nth(1).expect("Expected file argument");
    let stream = std::fs::read_to_string(path).expect("Failed to read file");

    self::run::run(&stream);

    /*
    match parser().parse(src) {
        Ok(ast) => match eval(&ast, &mut Vec::new(), &mut Vec::new()) {
            Ok(output) => println!("{}", output),
            Err(eval_err) => println!("Evaluation error: {}", eval_err),
        },
        Err(parse_errs) => parse_errs
            .into_iter()
            .for_each(|e| println!("Parse error: {}", e)),
    }
    */
}

#[derive(Debug)]
enum Expr {
    Num(f64),
    Var(String),

    Neg(Box<Self>),

    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),

    Call(String, Vec<Self>),
    Let {
        name: String,
        rhs: Box<Self>,
        then: Box<Self>,
    },
    Fn {
        name: String,
        args: Vec<String>,
        body: Box<Self>,
        then: Box<Self>,
    },
}

fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    use Expr::*;

    let ident = text::ident().padded();

    let expr = recursive(|expr| {
        let int = text::int(10)
            .map(|s: String| Expr::Num(s.parse().unwrap()))
            .padded();

        let call = ident.then(
            expr.clone()
                .separated_by(just(','))
                .allow_trailing()
                .delimited_by(just('('), just(')')),
        );
        let call = call.map(|(f, args)| Expr::Call(f, args));

        let atom = int
            .or(expr.delimited_by(just('('), just(')')))
            .or(call)
            .or(ident.map(Expr::Var));

        let unary = just('-')
            .padded()
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Neg(Box::new(rhs)));

        type BinaryFold = fn(Box<Expr>, Box<Expr>) -> Expr;
        let op = |c: char, e: BinaryFold| just(c).padded().to(e);

        fn lefty<'a, E: chumsky::Error<char>>(
            prev: impl Parser<char, Expr, Error = E> + Clone + 'a,
            op: impl Parser<char, BinaryFold, Error = E> + Clone + 'a,
        ) -> BoxedParser<'a, char, Expr, E> {
            prev.clone()
                .then(op.then(prev).repeated())
                .foldl(|lhs, (op, rhs): (BinaryFold, _)| op(Box::new(lhs), Box::new(rhs)))
                .boxed()
        }

        let mul = lefty(unary, choice((op('*', Mul), op('/', Div))));
        let sum = lefty(mul, choice((op('+', Add), op('-', Sub))));

        sum.boxed()
    });

    let decl = recursive(|decl| {
        let r#let = text::keyword("let")
            .ignore_then(ident)
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl.clone())
            .map(|((name, rhs), then)| Expr::Let {
                name,
                rhs: Box::new(rhs),
                then: Box::new(then),
            });

        let r#fn = text::keyword("fn")
            .ignore_then(ident)
            .then(ident.repeated())
            .then_ignore(just('='))
            .then(expr.clone())
            .then_ignore(just(';'))
            .then(decl)
            .map(|(((name, args), body), then)| Expr::Fn {
                name,
                args,
                body: Box::new(body),
                then: Box::new(then),
            });

        choice((r#let, r#fn, expr)).padded()
    });

    decl.then_ignore(end())
}

fn eval<'a>(
    expr: &'a Expr,
    vars: &mut Vec<(&'a String, f64)>,
    funcs: &mut Vec<(&'a String, &'a [String], &'a Expr)>,
) -> Result<f64, String> {
    match expr {
        Expr::Num(x) => Ok(*x),
        Expr::Neg(a) => Ok(-eval(a, vars, funcs)?),
        Expr::Add(a, b) => Ok(eval(a, vars, funcs)? + eval(b, vars, funcs)?),
        Expr::Sub(a, b) => Ok(eval(a, vars, funcs)? - eval(b, vars, funcs)?),
        Expr::Mul(a, b) => Ok(eval(a, vars, funcs)? * eval(b, vars, funcs)?),
        Expr::Div(a, b) => Ok(eval(a, vars, funcs)? / eval(b, vars, funcs)?),
        Expr::Var(name) => {
            if let Some((_, val)) = vars.iter().rev().find(|(var, _)| *var == name) {
                Ok(*val)
            } else {
                Err(format!("Cannot find variable `{}` in scope", name))
            }
        }
        Expr::Let { name, rhs, then } => {
            let rhs = eval(rhs, vars, funcs)?;
            vars.push((name, rhs));
            let output = eval(then, vars, funcs);
            vars.pop();
            output
        }
        Expr::Call(name, args) => {
            if let Some((_, arg_names, body)) =
                funcs.iter().rev().find(|(var, _, _)| *var == name).copied()
            {
                if arg_names.len() == args.len() {
                    let mut args = args
                        .iter()
                        .map(|arg| eval(arg, vars, funcs))
                        .zip(arg_names.iter())
                        .map(|(val, name)| Ok((name, val?)))
                        .collect::<Result<_, String>>()?;
                    vars.append(&mut args);
                    let output = eval(body, vars, funcs);
                    vars.truncate(vars.len() - args.len());
                    output
                } else {
                    Err(format!(
                        "Wrong number of arguments for function `{}`: expected {}, found {}",
                        name,
                        arg_names.len(),
                        args.len(),
                    ))
                }
            } else {
                Err(format!("Cannot find function `{}` in scope", name))
            }
        }
        Expr::Fn {
            name,
            args,
            body,
            then,
        } => {
            funcs.push((name, args, body));
            let output = eval(then, vars, funcs);
            funcs.pop();
            output
        }
    }
}
