use crate::utils::{CustomExpression, CustomExpressionInstruction, SemanticTest};
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
fn binding_wrong_expression() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::<CustomExpression<CustomExpressionInstruction>>::Variable(
            ValueName::new("x"),
        ),
        operation: None,
    };
    let binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(expr),
    };
    t.state.stmt(&binding, &block_state, None, None);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn binding_value_not_exist() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::<CustomExpression<CustomExpressionInstruction>>::Literal(Literal::I16(
            23,
        )),
        operation: None,
    };
    let binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(expr),
    };
    t.state.stmt(&binding, &block_state, None, None);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn binding_value_not_mutable() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::<CustomExpression<CustomExpressionInstruction>>::Literal(Literal::U64(
            30,
        )),
        operation: None,
    };
    let let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: false,
        ty: Some(Type::U64),
        value: Box::new(expr.clone()),
    };
    t.state.stmt(&let_binding, &block_state, None, None);
    assert!(t.is_empty_error());
    let inner_name: InnerValueName = "x.0".into();
    let val = Value {
        inner_name: inner_name.clone(),
        inner_type: Type::U64,
        mutable: false,
        alloca: false,
        malloc: false,
    };

    let state = block_state.borrow().stack().get();
    assert_eq!(state.len(), 1);
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
    let binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(expr),
    };
    t.state.stmt(&binding, &block_state, None, None);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueIsNotMutable),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn binding_value_found() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::<CustomExpression<CustomExpressionInstruction>>::Literal(Literal::U64(
            30,
        )),
        operation: None,
    };
    let let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: Some(Type::U64),
        value: Box::new(expr.clone()),
    };
    t.state.stmt(&let_binding, &block_state, None, None);
    assert!(t.is_empty_error());
    let inner_name: InnerValueName = "x.0".into();
    let val = Value {
        inner_name: inner_name.clone(),
        inner_type: Type::U64,
        mutable: true,
        alloca: false,
        malloc: false,
    };

    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);
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
    let new_expr = Expression {
        value: ExprValue::<CustomExpression<CustomExpressionInstruction>>::Literal(Literal::U64(
            100,
        )),
        operation: None,
    };
    let binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(new_expr),
    };
    t.state.stmt(&binding, &block_state, None, None);
    assert!(t.is_empty_error());
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 2);
    assert_eq!(
        state[0],
        SemanticStackContext::LetBinding {
            let_decl: val.clone(),
            expr_result: ExprResult::Literal(Literal::U64(30)),
        }
    );
    assert_eq!(
        state[1],
        SemanticStackContext::Binding {
            val: val.clone(),
            expr_result: ExprResult::Literal(Literal::U64(100)),
        }
    );
}
