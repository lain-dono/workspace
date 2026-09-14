use crate::utils::SemanticTest;
use semantic::block_state::BlockState;
use semantic::error::StateErrorKind;
use semantic::expression::{ExprValue, Expression};
use semantic::function::{FunctionCall, FunctionDecl};
use semantic::handle::Handle;
use semantic::names::FunctionName;
use semantic::types::{Literal, Type};

mod utils;

#[test]
fn func_call_not_declared_func() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let fn_name = FunctionName::new("fn1");
    let param1 = Expression {
        value: ExprValue::Literal(Literal::Ptr),
        operation: None,
    };
    let fn_call = FunctionCall {
        name: fn_name.clone(),
        args: vec![param1.clone()],
    };
    let res = t.state.function_call(&fn_call, &block_state);
    assert!(res.is_none());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::FunctionNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn func_call_wrong_type() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let fn_name = FunctionName::new("fn1");
    let fn_statement = FunctionDecl::new(
        fn_name.clone(),
        vec![(String::from("x"), Type::Bool)],
        Type::I16,
        vec![],
    );
    t.state.function_declaration(&fn_statement);
    assert!(t.is_empty_error());

    let param1 = Expression {
        value: ExprValue::Literal(Literal::F64(1.2)),
        operation: None,
    };
    let fn_call = FunctionCall {
        name: fn_name.clone(),
        args: vec![param1.clone()],
    };
    let res = t.state.function_call(&fn_call, &block_state).unwrap();
    assert_eq!(res, Type::I16);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::FunctionParameterTypeWrong),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn func_call_declared_func() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let fn_name = FunctionName::new("fn1");
    let fn_statement = FunctionDecl::new(
        fn_name.clone(),
        vec![(String::from("x"), Type::Bool)],
        Type::I32,
        vec![],
    );
    t.state.function_declaration(&fn_statement);
    assert!(t.is_empty_error());

    let param1 = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let fn_call = FunctionCall {
        name: fn_name.clone(),
        args: vec![param1.clone()],
    };
    let res = t.state.function_call(&fn_call, &block_state).unwrap();
    assert_eq!(res, Type::I32);
    assert!(t.is_empty_error());
}
