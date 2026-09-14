use crate::utils::SemanticTest;
use semantic::block_state::BlockState;
use semantic::context::SemanticStackContext;
use semantic::error::StateErrorKind;
use semantic::expression::{ExprResult, ExprValue, Expression};
use semantic::handle::Handle;
use semantic::names::{InnerValueName, ValueName};
use semantic::stmt::Stmt;
use semantic::types::{Literal, Type, Value};

mod utils;

#[test]
fn let_binding_wrong_expression() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Variable(ValueName::new("x")),
        operation: None,
    };
    let let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: Some(Type::U64),
        value: Box::new(expr),
    };
    t.state.stmt(&let_binding, &block_state, None, None);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn let_binding_wrong_type() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: Some(Type::U64),
        value: Box::new(expr),
    };
    t.state.stmt(&let_binding, &block_state, None, None);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::WrongLetType),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn let_binding_value_not_found() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Literal(Literal::U64(30)),
        operation: None,
    };
    let let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: Some(Type::U64),
        value: Box::new(expr),
    };
    t.state.stmt(&let_binding, &block_state, None, None);
    assert!(t.is_empty_error());
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);
    let inner_name: InnerValueName = "x.0".into();
    let val = Value {
        inner_name: inner_name.clone(),
        inner_type: Type::U64,
        mutable: true,
        alloca: false,
        malloc: false,
    };
    assert_eq!(
        state[0],
        SemanticStackContext::LetBinding {
            let_decl: val.clone(),
            expr_result: ExprResult::Literal(Literal::U64(30)),
        }
    );
    assert!(
        block_state
            .borrow()
            .inner
            .inner_values_name
            .contains(&inner_name)
    );
    assert_eq!(
        block_state.borrow().values.get(&("x".into())).unwrap(),
        &val
    );
}

#[test]
fn let_binding_value_found() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Literal(Literal::U64(30)),
        operation: None,
    };
    let inner_name: InnerValueName = "x.0".into();
    let val = Value {
        inner_name: inner_name.clone(),
        inner_type: Type::U64,
        mutable: true,
        alloca: false,
        malloc: false,
    };
    block_state.insert_value("x", val.clone());
    let let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: Some(Type::U64),
        value: Box::new(expr),
    };
    t.state.stmt(&let_binding, &block_state, None, None);
    assert!(t.is_empty_error());
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);

    let val2 = Value {
        inner_name: "x.1".into(),
        ..val.clone()
    };
    assert_eq!(
        state[0],
        SemanticStackContext::LetBinding {
            let_decl: val2.clone(),
            expr_result: ExprResult::Literal(Literal::U64(30)),
        }
    );
    assert!(
        block_state
            .borrow()
            .inner
            .inner_values_name
            .contains(&val2.inner_name)
    );
    assert_eq!(
        block_state.borrow().values.get(&("x".into())).unwrap(),
        &val2
    );
}
