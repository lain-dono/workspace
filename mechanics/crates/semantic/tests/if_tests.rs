use crate::utils::SemanticTest;
use semantic::block_state::BlockState;
use semantic::context::SemanticStackContext;
use semantic::error::StateErrorKind;
use semantic::expression::{ExprResult, ExprValue, Expression};
use semantic::function::{FunctionCall, FunctionDecl, FunctionHeader};
use semantic::handle::Handle;
use semantic::names::{FunctionName, LabelName, TypeName, ValueName};
use semantic::stmt::{
    BodyStatement, Condition, ExpressionCondition, ExpressionLogicCondition, IfBodyStatement,
    IfBodyStatements, IfCondition, IfLoopBodyStatement, IfStatement, Logic, LoopBodyStatement,
    Stmt,
};
use semantic::types::{Literal, StructTypes, Type, Value};
use std::collections::HashMap;

mod utils;

#[test]
fn check_if_and_else_if_statement_duplicate() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };

    let if_else_stmt = IfStatement {
        condition: IfCondition::Single(if_expr.clone()),
        body: IfBodyStatements::If(vec![]),
        else_statement: None,
        else_if_statement: None,
    };
    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::If(vec![]),
        else_statement: Some(IfBodyStatements::If(vec![])),
        else_if_statement: Some(Box::new(if_else_stmt)),
    };
    t.state.if_condition(&if_stmt, &block_state, None, None);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::IfElseDuplicated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn if_condition_calculation_simple() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    // Simple without operations
    let condition1 = IfCondition::Single(Expression {
        value: ExprValue::Literal(Literal::I8(3)),
        operation: None,
    });
    let label_if_begin: LabelName = String::from("if_begin").into();
    let label_if_else: LabelName = String::from("if_else").into();
    let label_if_end: LabelName = String::from("if_end").into();
    t.state.if_condition_calculation(
        &condition1,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        false,
    );

    // Simple with else without operations
    let condition2 = IfCondition::Single(Expression {
        value: ExprValue::Literal(Literal::I8(3)),
        operation: None,
    });
    t.state.if_condition_calculation(
        &condition2,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        true,
    );

    // Simple with wrong expression without operations
    let condition3 = IfCondition::Single(Expression {
        value: ExprValue::Variable(ValueName::new("x")),
        operation: None,
    });
    t.state.if_condition_calculation(
        &condition3,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        true,
    );

    let ctx = block_state.borrow().stack().clone().get();
    assert_eq!(ctx.len(), 2);
    assert_eq!(
        ctx[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::I8(3)),
            label_if_begin: label_if_begin.clone(),
            label_if_end,
        }
    );
    assert_eq!(
        ctx[1],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::I8(3)),
            label_if_begin,
            label_if_end: label_if_else,
        }
    );
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn if_condition_calculation_logic() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    let left_expr = Expression {
        value: ExprValue::Literal(Literal::I8(3)),
        operation: None,
    };
    let right_expr = Expression {
        value: ExprValue::Literal(Literal::I8(6)),
        operation: None,
    };
    let label_if_begin: LabelName = String::from("if_begin").into();
    let label_if_else: LabelName = String::from("if_else").into();
    let label_if_end: LabelName = String::from("if_end").into();

    // Logic: left == right
    let condition1 = IfCondition::Logic(ExpressionLogicCondition {
        left: ExpressionCondition {
            left: left_expr.clone(),
            condition: Condition::Eq,
            right: right_expr.clone(),
        },
        right: None,
    });
    t.state.if_condition_calculation(
        &condition1,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        false,
    );

    // Logic else: left == right
    let condition2 = IfCondition::Logic(ExpressionLogicCondition {
        left: ExpressionCondition {
            left: left_expr.clone(),
            condition: Condition::Eq,
            right: right_expr.clone(),
        },
        right: None,
    });
    t.state.if_condition_calculation(
        &condition2,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        true,
    );

    // Logic condition: left == right
    let condition3 = IfCondition::Logic(ExpressionLogicCondition {
        left: ExpressionCondition {
            left: left_expr.clone(),
            condition: Condition::Eq,
            right: right_expr.clone(),
        },
        right: Some((
            Logic::Or,
            Box::new(ExpressionLogicCondition {
                left: ExpressionCondition {
                    left: left_expr.clone(),
                    condition: Condition::Eq,
                    right: right_expr.clone(),
                },
                right: None,
            }),
        )),
    });
    t.state.if_condition_calculation(
        &condition3,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        false,
    );

    let ctx = block_state.borrow().stack().clone().get();
    assert_eq!(
        ctx[0],
        SemanticStackContext::ConditionExpression {
            left_result: ExprResult::Literal(Literal::I8(3)),
            right_result: ExprResult::Literal(Literal::I8(6)),
            condition: Condition::Eq,
            register: 1,
        }
    );
    assert_eq!(
        ctx[1],
        SemanticStackContext::IfConditionLogic {
            label_if_begin: label_if_begin.clone(),
            label_if_end: label_if_end.clone(),
            result_register: 1,
        }
    );
    assert_eq!(
        ctx[2],
        SemanticStackContext::ConditionExpression {
            left_result: ExprResult::Literal(Literal::I8(3)),
            right_result: ExprResult::Literal(Literal::I8(6)),
            condition: Condition::Eq,
            register: 2,
        }
    );
    assert_eq!(
        ctx[3],
        SemanticStackContext::IfConditionLogic {
            label_if_begin: label_if_begin.clone(),
            label_if_end: label_if_else,
            result_register: 2,
        }
    );
    assert_eq!(
        ctx[4],
        SemanticStackContext::ConditionExpression {
            left_result: ExprResult::Literal(Literal::I8(3)),
            right_result: ExprResult::Literal(Literal::I8(6)),
            condition: Condition::Eq,
            register: 3,
        }
    );
    assert_eq!(
        ctx[5],
        SemanticStackContext::ConditionExpression {
            left_result: ExprResult::Literal(Literal::I8(3)),
            right_result: ExprResult::Literal(Literal::I8(6)),
            condition: Condition::Eq,
            register: 4,
        }
    );
    assert_eq!(
        ctx[6],
        SemanticStackContext::LogicCondition {
            logic_condition: Logic::Or,
            left_register_result: 3,
            right_register_result: 4,
            register: 5,
        }
    );
    assert_eq!(
        ctx[7],
        SemanticStackContext::IfConditionLogic {
            label_if_begin,
            label_if_end,
            result_register: 5,
        }
    );
    assert_eq!(ctx.len(), 8);
    assert!(t.is_empty_error());
}

#[test]
fn if_condition_when_left_expr_return_error() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    // Left expression return error - function not found
    let left_expr = Expression {
        value: ExprValue::Call(FunctionCall {
            name: FunctionName::new("fn1"),
            args: vec![],
        }),
        operation: None,
    };
    let right_expr = Expression {
        value: ExprValue::Literal(Literal::I8(6)),
        operation: None,
    };
    let label_if_begin: LabelName = String::from("if_begin").into();
    let label_if_else: LabelName = String::from("if_else").into();
    let label_if_end: LabelName = String::from("if_end").into();

    let condition1 = IfCondition::Logic(ExpressionLogicCondition {
        left: ExpressionCondition {
            left: left_expr.clone(),
            condition: Condition::Eq,
            right: right_expr.clone(),
        },
        right: None,
    });
    t.state.if_condition_calculation(
        &condition1,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        false,
    );
    assert!(t.check_errors_len(2), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error_index(0, StateErrorKind::FunctionNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
    assert!(
        t.check_error_index(1, StateErrorKind::ConditionIsEmpty),
        "Errors: {:?}",
        t.state.errors[1]
    );
}

#[test]
fn if_condition_left_expr_and_right_expr_different_type() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    // Left expression return error - function not found
    let left_expr = Expression {
        value: ExprValue::Literal(Literal::I16(10)),
        operation: None,
    };
    let right_expr = Expression {
        value: ExprValue::Literal(Literal::I8(6)),
        operation: None,
    };
    let label_if_begin: LabelName = String::from("if_begin").into();
    let label_if_else: LabelName = String::from("if_else").into();
    let label_if_end: LabelName = String::from("if_end").into();

    let condition1 = IfCondition::Logic(ExpressionLogicCondition {
        left: ExpressionCondition {
            left: left_expr.clone(),
            condition: Condition::Eq,
            right: right_expr.clone(),
        },
        right: None,
    });
    t.state.if_condition_calculation(
        &condition1,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        false,
    );
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ConditionExpressionWrongType),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn if_condition_primitive_type_only_check() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();

    let type_decl = StructTypes {
        name: TypeName::new("type1"),
        attributes: HashMap::default(),
        methods: HashMap::default(),
    };
    t.state.types(&type_decl.clone());

    let fn_name = FunctionName::new("fn1");
    let fn_statement = FunctionDecl::new(fn_name.clone(), vec![], Type::Struct(type_decl), vec![]);
    t.state.function_declaration(&fn_statement);

    // Left expression return error - function not found
    let left_expr = Expression {
        value: ExprValue::Call(FunctionCall {
            name: fn_name.clone(),
            args: vec![],
        }),
        operation: None,
    };
    let right_expr = Expression {
        value: ExprValue::Call(FunctionCall {
            name: fn_name,
            args: vec![],
        }),
        operation: None,
    };
    let label_if_begin: LabelName = String::from("if_begin").into();
    let label_if_else: LabelName = String::from("if_else").into();
    let label_if_end: LabelName = String::from("if_end").into();

    let condition1 = IfCondition::Logic(ExpressionLogicCondition {
        left: ExpressionCondition {
            left: left_expr.clone(),
            condition: Condition::Eq,
            right: right_expr.clone(),
        },
        right: None,
    });
    t.state.if_condition_calculation(
        &condition1,
        &block_state,
        &label_if_begin,
        &label_if_else,
        &label_if_end,
        false,
    );
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ConditionExpressionNotSupported),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn else_if_statement() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };
    let if_else_expr = Expression {
        value: ExprValue::Literal(Literal::U64(2)),
        operation: None,
    };

    let if_else_stmt = IfStatement {
        condition: IfCondition::Single(if_else_expr),
        body: IfBodyStatements::If(vec![IfBodyStatement::Return(Expression {
            value: ExprValue::Literal(Literal::U64(30)),
            operation: None,
        })]),
        else_statement: Some(IfBodyStatements::Loop(vec![IfLoopBodyStatement::Return(
            Expression {
                value: ExprValue::Literal(Literal::U64(10)),
                operation: None,
            },
        )])),
        else_if_statement: None,
    };
    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::If(vec![IfBodyStatement::Return(Expression {
            value: ExprValue::Literal(Literal::U64(12)),
            operation: None,
        })]),
        else_statement: None,
        else_if_statement: Some(Box::new(if_else_stmt)),
    };
    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();
    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );
    println!("{:#?}", t.state.errors);
    assert!(t.is_empty_error());

    let main_ctx = block_state.borrow().stack().clone().get();
    assert_eq!(main_ctx.len(), 10);
    assert!(block_state.borrow().parent.is_none());
    assert_eq!(block_state.borrow().children.len(), 3);

    let ch_ctx1 = block_state.borrow().children[0].clone();
    assert!(ch_ctx1.borrow().parent.is_some());
    assert!(ch_ctx1.borrow().children.is_empty());

    let ch_ctx2 = block_state.borrow().children[1].clone();
    assert!(ch_ctx2.borrow().parent.is_some());
    assert!(ch_ctx2.borrow().children.is_empty());

    let ch_ctx3 = block_state.borrow().children[2].clone();
    assert!(ch_ctx3.borrow().parent.is_some());
    assert!(ch_ctx3.borrow().children.is_empty());

    let ctx1 = ch_ctx1.borrow().stack().clone().get();
    assert_eq!(ctx1.len(), 5);
    assert_eq!(
        ctx1[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::U64(1)),
            label_if_begin: String::from("if_begin").into(),
            label_if_end: String::from("if_else").into(),
        }
    );
    assert_eq!(ctx1[0], main_ctx[0]);
    assert_eq!(
        ctx1[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin").into()
        }
    );
    assert_eq!(ctx1[1], main_ctx[1]);
    assert_eq!(
        ctx1[2],
        SemanticStackContext::JumpFunctionReturn {
            expr_result: ExprResult::Literal(Literal::U64(12)),
        }
    );
    assert_eq!(ctx1[2], main_ctx[2]);
    assert_eq!(
        ctx1[3],
        SemanticStackContext::SetLabel {
            label: String::from("if_else").into()
        }
    );
    assert_eq!(ctx1[3], main_ctx[3]);
    assert_eq!(
        ctx1[4],
        SemanticStackContext::SetLabel {
            label: String::from("if_end").into()
        }
    );
    assert_eq!(ctx1[4], main_ctx[9]);

    let ctx2 = ch_ctx2.borrow().stack().clone().get();
    assert_eq!(ctx2.len(), 4);
    assert_eq!(
        ctx2[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::U64(2)),
            label_if_begin: String::from("if_begin.0").into(),
            label_if_end: String::from("if_else.0").into(),
        }
    );
    assert_eq!(ctx2[0], main_ctx[4]);
    assert_eq!(
        ctx2[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin.0").into()
        }
    );
    assert_eq!(ctx2[1], main_ctx[5]);
    assert_eq!(
        ctx2[2],
        SemanticStackContext::JumpFunctionReturn {
            expr_result: ExprResult::Literal(Literal::U64(30)),
        }
    );
    assert_eq!(ctx2[2], main_ctx[6]);
    assert_eq!(
        ctx2[3],
        SemanticStackContext::SetLabel {
            label: String::from("if_else.0").into()
        }
    );
    assert_eq!(ctx2[3], main_ctx[7]);

    let ctx3 = ch_ctx3.borrow().stack().clone().get();
    assert_eq!(ctx3.len(), 1);
    assert_eq!(
        ctx3[0],
        SemanticStackContext::JumpFunctionReturn {
            expr_result: ExprResult::Literal(Literal::U64(10)),
        }
    );
    assert_eq!(ctx3[0], main_ctx[8]);
}

#[test]
fn if_body_statements() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };

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

    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let if_body_binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(true)),
            operation: None,
        }),
    };
    let if_body_fn_call = Stmt::FunctionCall(FunctionCall {
        name: FunctionName::new("fn2"),
        args: vec![],
    });
    let if_body_if = Stmt::If(IfStatement {
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
    let if_body_loop = Stmt::Loop(vec![LoopBodyStatement::Stmt(Stmt::FunctionCall(
        FunctionCall {
            name: FunctionName::new("fn2"),
            args: vec![],
        },
    ))]);
    let if_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };

    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::If(vec![
            IfBodyStatement::Stmt(if_body_let_binding),
            IfBodyStatement::Stmt(if_body_binding),
            IfBodyStatement::Stmt(if_body_fn_call),
            IfBodyStatement::Stmt(if_body_if),
            IfBodyStatement::Stmt(if_body_loop),
            IfBodyStatement::Return(if_body_return),
        ]),
        else_statement: None,
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );
    assert!(t.is_empty_error());

    let main_ctx = block_state.borrow().stack().get();
    assert_eq!(main_ctx.len(), 16);
    assert!(block_state.borrow().parent.is_none());
    assert_eq!(block_state.borrow().children.len(), 1);

    let ctx = block_state.borrow().children[0].clone();
    assert!(ctx.borrow().parent.is_some());
    assert_eq!(ctx.borrow().children.len(), 2);

    let ch_ctx1 = ctx.borrow().children[0].clone();
    assert!(ch_ctx1.borrow().parent.is_some());
    assert!(ch_ctx1.borrow().children.is_empty());

    let ctx1 = ch_ctx1.borrow().stack().clone().get();
    assert_eq!(ctx1.len(), 4);
    assert_eq!(
        ctx1[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
            label_if_begin: String::from("if_begin.0").into(),
            label_if_end: String::from("if_end").into(),
        }
    );
    assert_eq!(
        ctx1[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin.0").into()
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

    let ch_ctx2 = ctx.borrow().children[1].clone();
    assert!(ch_ctx2.borrow().parent.is_some());
    assert!(ch_ctx2.borrow().children.is_empty());

    let ctx2 = ch_ctx2.borrow().stack().clone().get();
    assert_eq!(ctx2.len(), 5);
    assert_eq!(
        ctx2[0],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );

    assert_eq!(
        ctx2[1],
        SemanticStackContext::SetLabel {
            label: String::from("loop_begin").into()
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
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        ctx2[4],
        SemanticStackContext::SetLabel {
            label: String::from("loop_end").into()
        }
    );

    let stm_ctx = ctx.borrow().stack().clone().get();
    assert_eq!(stm_ctx.len(), 16);
    assert_eq!(stm_ctx, main_ctx);
    assert_eq!(
        stm_ctx[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::U64(1)),
            label_if_begin: String::from("if_begin").into(),
            label_if_end: String::from("if_end").into(),
        }
    );
    assert_eq!(
        stm_ctx[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin").into()
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
    assert_eq!(stm_ctx[9], ctx2[0]);
    assert_eq!(stm_ctx[10], ctx2[1]);
    assert_eq!(stm_ctx[11], ctx2[2]);
    assert_eq!(stm_ctx[12], ctx2[3]);
    assert_eq!(stm_ctx[13], ctx2[4]);
    assert_eq!(
        stm_ctx[14],
        SemanticStackContext::JumpFunctionReturn {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
        }
    );
    assert_eq!(
        stm_ctx[15],
        SemanticStackContext::SetLabel {
            label: String::from("if_end").into()
        }
    );
}

#[test]
fn if_loop_body_statements() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };

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
    let fn3 = FunctionDecl::new(
        FunctionName::new("fn3"),
        vec![],
        Type::I16,
        vec![BodyStatement::Expr(Expression {
            value: ExprValue::Literal(Literal::I16(32)),
            operation: None,
        })],
    );
    t.state.function_declaration(&fn3);

    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let if_body_binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(true)),
            operation: None,
        }),
    };
    let if_body_fn_call = Stmt::FunctionCall(FunctionCall {
        name: FunctionName::new("fn2"),
        args: vec![],
    });
    let if_body_if = Stmt::If(IfStatement {
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
    let if_body_loop = Stmt::Loop(vec![LoopBodyStatement::Stmt(Stmt::FunctionCall(
        FunctionCall {
            name: FunctionName::new("fn3"),
            args: vec![],
        },
    ))]);
    let if_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };

    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::Loop(vec![
            IfLoopBodyStatement::Stmt(if_body_let_binding),
            IfLoopBodyStatement::Stmt(if_body_binding),
            IfLoopBodyStatement::Stmt(if_body_fn_call),
            IfLoopBodyStatement::Stmt(if_body_if),
            IfLoopBodyStatement::Stmt(if_body_loop),
            IfLoopBodyStatement::Return(if_body_return),
        ]),
        else_statement: None,
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );
    assert!(t.is_empty_error());

    let main_ctx = block_state.borrow().stack().clone().get();
    assert_eq!(main_ctx.len(), 16);
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
    assert_eq!(ctx1.len(), 4);
    assert_eq!(
        ctx1[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
            label_if_begin: String::from("if_begin.0").into(),
            label_if_end: String::from("if_end").into(),
        }
    );
    assert_eq!(
        ctx1[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin.0").into()
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

    let ch_ctx2 = ctx.borrow().children[1].clone();
    assert!(ch_ctx2.borrow().parent.is_some());
    assert!(ch_ctx2.borrow().children.is_empty());

    let ctx2 = ch_ctx2.borrow().stack().clone().get();
    assert_eq!(ctx2.len(), 5);
    assert_eq!(
        ctx2[0],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        ctx2[1],
        SemanticStackContext::SetLabel {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        ctx2[2],
        SemanticStackContext::Call {
            call: FunctionHeader {
                name: String::from("fn3").into(),
                result: Type::I16,
                args: vec![],
            },
            params: vec![],
            register: 3,
        }
    );
    assert_eq!(
        ctx2[3],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        ctx2[4],
        SemanticStackContext::SetLabel {
            label: String::from("loop_end").into()
        }
    );

    let stm_ctx = ctx.borrow().stack().clone().get();
    assert_eq!(stm_ctx.len(), 16);
    assert_eq!(stm_ctx, main_ctx);
    assert_eq!(
        stm_ctx[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::U64(1)),
            label_if_begin: String::from("if_begin").into(),
            label_if_end: String::from("if_end").into(),
        }
    );
    assert_eq!(
        stm_ctx[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin").into()
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
    assert_eq!(stm_ctx[9], ctx2[0]);
    assert_eq!(stm_ctx[10], ctx2[1]);
    assert_eq!(stm_ctx[11], ctx2[2]);
    assert_eq!(stm_ctx[12], ctx2[3]);
    assert_eq!(stm_ctx[13], ctx2[4]);
    assert_eq!(
        stm_ctx[14],
        SemanticStackContext::JumpFunctionReturn {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
        }
    );
    assert_eq!(
        stm_ctx[15],
        SemanticStackContext::SetLabel {
            label: String::from("if_end").into()
        }
    );
}

#[test]
fn if_loop_body_instructions_after_return() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };

    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let if_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::Loop(vec![
            IfLoopBodyStatement::Return(if_body_return),
            IfLoopBodyStatement::Stmt(if_body_let_binding),
        ]),
        else_statement: None,
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );

    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterReturnDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn else_if_loop_body_instructions_after_return() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };

    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let if_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::Loop(vec![IfLoopBodyStatement::Return(if_body_return.clone())]),
        else_statement: Some(IfBodyStatements::Loop(vec![
            IfLoopBodyStatement::Return(if_body_return.clone()),
            IfLoopBodyStatement::Stmt(if_body_let_binding),
        ])),
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );

    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterReturnDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn if_body_instructions_after_return() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };

    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let if_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::If(vec![
            IfBodyStatement::Return(if_body_return),
            IfBodyStatement::Stmt(if_body_let_binding),
        ]),
        else_statement: None,
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );

    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterReturnDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn if_else_body_instructions_after_return() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };

    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let if_body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::If(vec![IfBodyStatement::Return(if_body_return.clone())]),
        else_statement: Some(IfBodyStatements::If(vec![
            IfBodyStatement::Return(if_body_return),
            IfBodyStatement::Stmt(if_body_let_binding),
        ])),
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );

    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterReturnDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn if_loop_body_instructions_after_break() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };
    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };

    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::Loop(vec![
            IfLoopBodyStatement::Break,
            IfLoopBodyStatement::Stmt(if_body_let_binding),
        ]),
        else_statement: None,
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterBreakDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn if_loop_body_instructions_after_continue() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::U64(1)),
        operation: None,
    };
    let if_body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };

    let label_loop_begin: LabelName = String::from("loop_begin").into();
    let label_loop_end: LabelName = String::from("loop_end").into();

    let if_stmt = IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::Loop(vec![
            IfLoopBodyStatement::Continue,
            IfLoopBodyStatement::Stmt(if_body_let_binding),
        ]),
        else_statement: None,
        else_if_statement: None,
    };

    t.state.if_condition(
        &if_stmt,
        &block_state,
        None,
        Some((&label_loop_begin, &label_loop_end)),
    );
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ForbiddenCodeAfterContinueDeprecated),
        "Errors: {:?}",
        t.state.errors[0]
    );
}
