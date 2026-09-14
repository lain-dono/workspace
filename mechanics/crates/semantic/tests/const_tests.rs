use crate::utils::SemanticTest;
use semantic::constants::{Constant, ConstantExpression, ConstantValue};
use semantic::context::SemanticStackContext;
use semantic::error::StateErrorKind;
use semantic::expression::ExpressionOperations;
use semantic::names::ConstantName;
use semantic::types::{Literal, Type};

mod utils;

#[test]
fn const_declaration() {
    let mut t = SemanticTest::new();
    let const_name = ConstantName::new("cnt1");
    let const_statement = Constant {
        name: const_name.clone(),
        ty: Type::I8,
        value: ConstantExpression {
            value: ConstantValue::Literal(Literal::I8(10)),
            operation: None,
        },
    };
    t.state.constant(&const_statement);
    assert!(t.state.global.constants.contains_key(&const_name));
    assert!(t.is_empty_error());

    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::Constant {
            const_decl: const_statement.clone()
        }
    );

    t.state.constant(&const_statement);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ConstantAlreadyExist),
        "Errors: {:?}",
        t.state.errors[0]
    );
    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 1);
}

#[test]
fn const_declaration_with_operations() {
    let mut t = SemanticTest::new();
    let const_name2 = ConstantName::new("cnt2");

    let cnt_expr_prev2 = ConstantExpression {
        value: ConstantValue::Literal(Literal::F32(1.1)),
        operation: None,
    };

    let cnt_expr_prev = ConstantExpression {
        value: ConstantValue::Constant(const_name2.clone()),
        operation: Some((ExpressionOperations::Plus, Box::new(cnt_expr_prev2))),
    };

    // constant1
    let const_name1 = ConstantName::new("cnt1");
    let const_statement = Constant {
        name: const_name1.clone(),
        ty: Type::I8,
        value: ConstantExpression {
            value: ConstantValue::Literal(Literal::I8(10)),
            operation: Some((ExpressionOperations::Plus, Box::new(cnt_expr_prev))),
        },
    };
    t.state.constant(&const_statement);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ConstantNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
    assert!(!t.state.global.constants.contains_key(&const_name1));
    t.clean_errors();

    // constant2
    let const_statement2 = Constant {
        name: const_name2.clone(),
        ty: Type::I32,
        value: ConstantExpression {
            value: ConstantValue::Literal(Literal::I8(10)),
            operation: None,
        },
    };
    t.state.constant(&const_statement2);
    assert!(t.state.global.constants.contains_key(&const_name2));

    t.state.constant(&const_statement);
    assert!(t.state.global.constants.contains_key(&const_name1));
    assert!(t.is_empty_error());

    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 2);
    assert_eq!(
        state[0],
        SemanticStackContext::Constant {
            const_decl: const_statement2
        }
    );
    assert_eq!(
        state[1],
        SemanticStackContext::Constant {
            const_decl: const_statement
        }
    );
}
