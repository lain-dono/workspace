use crate::parser::{BinaryOp, Decl, Expr, Module, Stmt, Storage, Token, UnaryOp};
use crate::semantic::Semantic;
use crate::vm::Machine;
use chumsky::prelude::*;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CompileError {
    #[error("err")]
    EntryPointNotFound,
}

pub fn create_machine(input: &str) -> Result<Machine, CompileError> {
    let (input, eoi) = Token::scan(input).unwrap();
    dbg!(input.as_slice().iter().map(|(t, _)| *t).collect::<Vec<_>>());
    let input = input.map(eoi, |(t, s)| (t, s));

    let mut arena = extra::SimpleState(Storage::<'_>::default());
    let parser = Module::parser();
    let ast_module = parser.parse_with_state(input, &mut arena).unwrap();

    let mut semantic = Semantic::default();

    let main = ast_module.decls.iter().find_map(|decl| match decl {
        Decl::Func(f) if f.name == "main" => Some(f),
        _ => None,
    });
    let entry = main.ok_or(CompileError::EntryPointNotFound)?;

    semantic.root(|block| {
        for stmt in &entry.body {
            match *stmt {
                Stmt::Invalid => todo!(),
                Stmt::Block(ref stmts) => todo!(),
                Stmt::Expr(expr) => match arena.exprs[expr] {
                    Expr::Error => todo!(),
                    Expr::Literal(_) => todo!(),
                    Expr::Number(_) => todo!(),

                    Expr::Unary(op, expr) => match op {
                        UnaryOp::Neg => todo!(),
                        UnaryOp::Deref => todo!(),
                        UnaryOp::Not => todo!(),
                        UnaryOp::Refer => todo!(),
                    },

                    Expr::Binary(lhs, op, rhs) => match op {
                        BinaryOp::Div => todo!(),
                        BinaryOp::Mul => todo!(),
                        BinaryOp::Rem => todo!(),

                        BinaryOp::Add => todo!(),
                        BinaryOp::Sub => todo!(),

                        BinaryOp::And => todo!(),
                        BinaryOp::Eor => todo!(),
                        BinaryOp::Ior => todo!(),

                        BinaryOp::Shl => todo!(),
                        BinaryOp::Shr => todo!(),
                    },
                    Expr::Assign(lhs, op, rhs) => match op {
                        None => todo!(),

                        Some(BinaryOp::Div) => todo!(),
                        Some(BinaryOp::Mul) => todo!(),
                        Some(BinaryOp::Rem) => todo!(),

                        Some(BinaryOp::Add) => todo!(),
                        Some(BinaryOp::Sub) => todo!(),

                        Some(BinaryOp::And) => todo!(),
                        Some(BinaryOp::Eor) => todo!(),
                        Some(BinaryOp::Ior) => todo!(),

                        Some(BinaryOp::Shl) => todo!(),
                        Some(BinaryOp::Shr) => todo!(),
                    },
                },
                Stmt::Let(lhs, rhs, ref reject) => todo!(),
                Stmt::Break(_) => todo!(),
                Stmt::Continue(_) => todo!(),
            }
        }
    });

    Ok(semantic.build())
}

#[test]
fn simple() {
    let input = "
fn main(a i32) i32 { a + 5 }
";

    let machine = create_machine(input).unwrap();
}
