use crate::utils::SemanticTest;
use semantic::block_state::BlockState;
use semantic::context::SemanticStackContext;
use semantic::error::StateErrorKind;
use semantic::expression::{ExprResult, ExprValue, Expression};
use semantic::function::{FunctionCall, FunctionDecl, FunctionHeader};
use semantic::handle::Handle;
use semantic::names::{FunctionName, ValueName};
use semantic::stmt::{
    BodyStatement, IfBodyStatements, IfCondition, IfLoopBodyStatement, IfStatement,
    LoopBodyStatement, Stmt,
};
use semantic::types::{Literal, Type, Value};

mod utils;

#[test]
fn loop_statements() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    let fn2 = FunctionDecl::new(
        FunctionName::new("fn2"),
        vec![],
        Type::U16,
        vec![BodyStatement::Expr(Expression {
            value: ExprValue::Literal(Literal::U16(23)),
            operation: None,
        })],
    );
    t.state.function_declaration(&fn2);

    let loop_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let loop_body_binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(true)),
            operation: None,
        }),
    };
    let loop_body_fn_call = Stmt::FunctionCall(FunctionCall {
        name: FunctionName::new("fn2"),
        args: vec![],
    });
    let loop_body_if = Stmt::If(IfStatement {
        condition: IfCondition::Single(Expression {
            value: ExprValue::Literal(Literal::Bool(true)),
            operation: None,
        }),
        body: IfBodyStatements::Loop(vec![IfLoopBodyStatement::Stmt(Stmt::FunctionCall(
            FunctionCall {
                name: FunctionName::new("fn2"),
                args: vec![],
            },
        ))]),
        else_statement: None,
        else_if_statement: None,
    });
    let loop_body_loop = Stmt::Loop(vec![LoopBodyStatement::Stmt(Stmt::FunctionCall(
        FunctionCall {
            name: FunctionName::new("fn2"),
            args: vec![],
        },
    ))]);

    let loop_stmt = [
        LoopBodyStatement::Stmt(loop_body_let_binding),
        LoopBodyStatement::Stmt(loop_body_binding),
        LoopBodyStatement::Stmt(loop_body_fn_call),
        LoopBodyStatement::Stmt(loop_body_if),
        LoopBodyStatement::Stmt(loop_body_loop),
    ];
    t.state.loop_statement(&loop_stmt, &block_state);

    assert!(t.is_empty_error());
    let main_ctx = block_state.borrow().stack().get();
    assert_eq!(main_ctx.len(), 17);
    assert!(block_state.borrow().parent.is_none());
    assert!(block_state.borrow().parent.is_none());
    assert_eq!(block_state.borrow().children.len(), 1);

    let ctx = block_state.borrow().children[0].clone();
    assert!(ctx.borrow().parent.is_some());
    assert_eq!(ctx.borrow().children.len(), 2);

    let ch_ctx1 = ctx.borrow().children[0].clone();
    assert!(ch_ctx1.borrow().parent.is_some());
    assert!(ch_ctx1.borrow().children.is_empty());

    let ctx1 = ch_ctx1.borrow().stack().clone().get();
    assert_eq!(ctx1.len(), 5);
    assert_eq!(
        ctx1[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
            label_if_begin: String::from("if_begin").into(),
            label_if_end: String::from("if_end").into(),
        }
    );
    assert_eq!(
        ctx1[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin").into()
        }
    );
    assert_eq!(
        ctx1[2],
        SemanticStackContext::Call {
            call: FunctionHeader {
                name: String::from("fn2").into(),
                result: Type::U16,
                args: vec![],
            },
            params: vec![],
            register: 2,
        }
    );
    assert_eq!(
        ctx1[3],
        SemanticStackContext::JumpTo {
            label: String::from("if_end").into()
        }
    );
    assert_eq!(
        ctx1[4],
        SemanticStackContext::SetLabel {
            label: String::from("if_end").into()
        }
    );

    let ch_ctx2 = ctx.borrow().children[1].clone();
    assert!(ch_ctx2.borrow().parent.is_some());
    assert!(ch_ctx2.borrow().children.is_empty());

    let ctx2 = ch_ctx2.borrow().stack().clone().get();
    assert_eq!(ctx2.len(), 5);
    assert_eq!(
        ctx2[0],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin.0").into()
        }
    );
    assert_eq!(
        ctx2[1],
        SemanticStackContext::SetLabel {
            label: String::from("loop_begin.0").into()
        }
    );
    assert_eq!(
        ctx2[2],
        SemanticStackContext::Call {
            call: FunctionHeader {
                name: String::from("fn2").into(),
                result: Type::U16,
                args: vec![],
            },
            params: vec![],
            register: 3,
        }
    );
    assert_eq!(
        ctx2[3],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin.0").into()
        }
    );
    assert_eq!(
        ctx2[4],
        SemanticStackContext::SetLabel {
            label: String::from("loop_end.0").into()
        }
    );

    let stm_ctx = ctx.borrow().stack().clone().get();
    assert_eq!(stm_ctx.len(), 17);
    assert_eq!(stm_ctx, main_ctx);
    assert_eq!(
        stm_ctx[0],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        stm_ctx[1],
        SemanticStackContext::SetLabel {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        stm_ctx[2],
        SemanticStackContext::LetBinding {
            let_decl: Value {
                inner_name: "x.0".into(),
                inner_type: Type::Bool,
                mutable: true,
                alloca: false,
                malloc: false,
            },
            expr_result: ExprResult::Literal(Literal::Bool(false)),
        }
    );
    assert_eq!(
        stm_ctx[3],
        SemanticStackContext::Binding {
            val: Value {
                inner_name: "x.0".into(),
                inner_type: Type::Bool,
                mutable: true,
                alloca: false,
                malloc: false,
            },
            expr_result: ExprResult::Literal(Literal::Bool(true)),
        }
    );
    assert_eq!(
        stm_ctx[4],
        SemanticStackContext::Call {
            call: FunctionHeader {
                name: String::from("fn2").into(),
                result: Type::U16,
                args: vec![],
            },
            params: vec![],
            register: 1,
        }
    );
    assert_eq!(stm_ctx[5], ctx1[0]);
    assert_eq!(stm_ctx[6], ctx1[1]);
    assert_eq!(stm_ctx[7], ctx1[2]);
    assert_eq!(stm_ctx[8], ctx1[3]);
    assert_eq!(stm_ctx[9], ctx1[4]);
    assert_eq!(stm_ctx[10], ctx2[0]);
    assert_eq!(stm_ctx[11], ctx2[1]);
    assert_eq!(stm_ctx[12], ctx2[2]);
    assert_eq!(stm_ctx[13], ctx2[3]);
    assert_eq!(stm_ctx[14], ctx2[4]);
    assert_eq!(
        stm_ctx[15],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        stm_ctx[16],
        SemanticStackContext::SetLabel {
            label: String::from("loop_end").into()
        }
    );
}

#[test]
fn loop_statements_instructions_after_return() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    let loop_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let loop_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };

    let loop_stmt = [
        LoopBodyStatement::Return(loop_body_return),
        LoopBodyStatement::Stmt(loop_body_let_binding),
    ];
    t.state.loop_statement(&loop_stmt, &block_state);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterReturnDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn loop_statements_instructions_after_break() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    let loop_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let loop_stmt = [
        LoopBodyStatement::Break,
        LoopBodyStatement::Stmt(loop_body_let_binding),
    ];
    t.state.loop_statement(&loop_stmt, &block_state);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterBreakDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn loop_statements_instructions_after_continue() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    let loop_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let loop_stmt = [
        LoopBodyStatement::Continue,
        LoopBodyStatement::Stmt(loop_body_let_binding),
    ];
    t.state.loop_statement(&loop_stmt, &block_state);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterContinueDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn loop_statements_with_return_invocation() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    let loop_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let loop_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };

    let loop_stmt = [
        LoopBodyStatement::Stmt(loop_body_let_binding),
        LoopBodyStatement::Return(loop_body_return),
    ];
    t.state.loop_statement(&loop_stmt, &block_state);

    let main_ctx = block_state.borrow().stack().get();
    assert_eq!(main_ctx.len(), 4);
    assert!(block_state.borrow().parent.is_none());
    assert!(block_state.borrow().parent.is_none());
    assert_eq!(block_state.borrow().children.len(), 1);

    let ctx = block_state.borrow().children[0].clone();
    assert!(ctx.borrow().parent.is_some());
    assert!(ctx.borrow().children.is_empty());

    let stm_ctx = ctx.borrow().stack().clone().get();
    assert_eq!(stm_ctx.len(), 4);
    assert_eq!(stm_ctx, main_ctx);
    assert_eq!(
        stm_ctx[0],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        stm_ctx[1],
        SemanticStackContext::SetLabel {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        stm_ctx[2],
        SemanticStackContext::LetBinding {
            let_decl: Value {
                inner_name: "x.0".into(),
                inner_type: Type::Bool,
                mutable: true,
                alloca: false,
                malloc: false,
            },
            expr_result: ExprResult::Literal(Literal::Bool(false)),
        }
    );
    assert_eq!(
        stm_ctx[3],
        SemanticStackContext::JumpFunctionReturn {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
        }
    );

    assert!(t.is_empty_error());
}
