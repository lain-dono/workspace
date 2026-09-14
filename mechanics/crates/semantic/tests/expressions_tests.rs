use crate::utils::{CustomExpressionInstruction, SemanticTest};
use semantic::block_state::BlockState;
use semantic::constants::{Constant, ConstantExpression, ConstantValue};
use semantic::context::{
    ExtendedExpression, ExtendedSemanticContext, SemanticContextInstruction, SemanticStackContext,
};
use semantic::error::StateErrorKind;
use semantic::expression::{ExprResult, ExprValue, Expression, ExpressionOperations};
use semantic::function::{FunctionCall, FunctionDecl, FunctionHeader};
use semantic::handle::Handle;
use semantic::names::{ConstantName, FunctionName, ValueName};
use semantic::semantic::State;
use semantic::types::{Literal, StructAttributeType, StructTypes, Type, Value};
use std::collections::HashMap;
use std::marker::PhantomData;

mod utils;

fn set_result_type<E>(
    operation: ExpressionOperations,
    reg_left: bool,
    left: u64,
    reg_right: bool,
    right: u64,
    register: u64,
) -> SemanticStackContext<CustomExpressionInstruction, E> {
    let left_value = if reg_left {
        ExprResult::Register(Type::U16, left)
    } else {
        ExprResult::Literal(Literal::U16(left as u16))
    };
    let right_value = if reg_right {
        ExprResult::Register(Type::U16, right)
    } else {
        ExprResult::Literal(Literal::U16(right as u16))
    };
    SemanticStackContext::ExpressionOperation {
        operation,
        left_value,
        right_value,
        register,
    }
}

#[test]
fn expression_value_name_not_found() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let src = String::from("x");
    let value_name = ValueName::new(src);
    let expr = Expression {
        value: ExprValue::Variable(value_name),
        operation: None,
    };
    let res = t.state.expression(&expr, &block_state);
    assert!(res.is_none());
    assert!(
        t.check_error(StateErrorKind::ValueNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
    let state = block_state.borrow().stack().clone().get();
    assert!(state.is_empty());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
}

#[test]
fn expression_value_name_exists() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let value_name = ValueName::new("x");
    let expr = Expression {
        value: ExprValue::Variable(value_name.clone()),
        operation: None,
    };
    let ty = Type::I8;
    let value = Value {
        inner_name: "x".into(),
        inner_type: ty.clone(),
        mutable: false,
        alloca: false,
        malloc: false,
    };

    block_state.insert_value(value_name, value.clone());

    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(ty, 1));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::ExpressionValue {
            expression: value,
            register: 1,
        }
    );
    assert!(t.is_empty_error());
}

#[test]
fn expression_const_exists() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let src = String::from("x");
    let const_name = ValueName::new(src);
    let expr = Expression {
        value: ExprValue::Variable(const_name.clone()),
        operation: None,
    };
    let ty = Type::I8;
    let name: ConstantName = const_name.into();
    let value = Constant {
        name: name.clone(),
        ty: ty.clone(),
        value: ConstantExpression {
            value: ConstantValue::Literal(Literal::I8(12)),
            operation: None,
        },
    };
    t.state.global.constants.insert(name, value.clone());
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(ty, 1));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::ExpressionConst {
            expression: value,
            register: 1,
        }
    );
    assert!(t.is_empty_error());
}

#[test]
fn expression_primitive_value() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Literal(Literal::I32(10)),
        operation: None,
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Literal(Literal::I32(10)));
    let state = block_state.borrow().stack().clone().get();
    assert!(state.is_empty());
    assert!(t.is_empty_error());
}

#[test]
fn expression_struct_value_not_found() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Struct {
            name: ValueName::new("val"),
            attr: ValueName::new("attr1"),
        },
        operation: None,
    };
    let res = t.state.expression(&expr, &block_state);
    assert!(res.is_none());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn expression_struct_value_wrong_struct_type() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Struct {
            name: ValueName::new("x"),
            attr: ValueName::new("attr1"),
        },
        operation: None,
    };
    let val = Value {
        inner_name: "x".into(),
        inner_type: Type::Bool,
        mutable: false,
        alloca: false,
        malloc: false,
    };
    block_state.insert_value("x", val.clone());
    let res = t.state.expression(&expr, &block_state);
    assert!(res.is_none());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueNotStruct),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn expression_struct_value_type_not_found() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Struct {
            name: ValueName::new("x"),
            attr: ValueName::new("attr1"),
        },
        operation: None,
    };
    let s_attr = StructAttributeType {
        name: ValueName::new("attr1"),
        ty: Type::Bool,
        index: 0,
    };
    let val = Value {
        inner_name: "x".into(),
        inner_type: Type::Struct(StructTypes {
            name: "St".into(),
            attributes: {
                let mut map = HashMap::default();
                map.insert(s_attr.name.clone(), s_attr);
                map
            },
            methods: HashMap::default(),
        }),
        mutable: false,
        alloca: false,
        malloc: false,
    };
    block_state.insert_value("x", val.clone());
    let res = t.state.expression(&expr, &block_state);
    assert!(res.is_none());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::TypeNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn expression_struct_value_wrong_expression_type() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Struct {
            name: ValueName::new("x"),
            attr: ValueName::new("attr1"),
        },
        operation: None,
    };
    let s_attr = StructAttributeType {
        name: ValueName::new("attr2"),
        ty: Type::Bool,
        index: 0,
    };
    let val = Value {
        inner_name: "x".into(),
        inner_type: Type::Struct(StructTypes {
            name: "St".into(),
            attributes: {
                let mut map = HashMap::default();
                map.insert(s_attr.name.clone(), s_attr);
                map
            },
            methods: HashMap::default(),
        }),
        mutable: false,
        alloca: false,
        malloc: false,
    };
    block_state.insert_value("x", val.clone());

    let s_attr = StructAttributeType {
        name: ValueName::new("attr1"),
        ty: Type::Bool,
        index: 0,
    };
    t.state.types(&StructTypes {
        name: "St".into(),
        attributes: {
            let mut map = HashMap::default();
            map.insert(s_attr.name.clone(), s_attr);
            map
        },
        methods: HashMap::default(),
    });
    assert!(t.is_empty_error());
    let res = t.state.expression(&expr, &block_state);
    assert!(res.is_none());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::WrongExpressionType),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn expression_struct_value_wrong_struct_attribute() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expr = Expression {
        value: ExprValue::Struct {
            name: ValueName::new("x"),
            attr: ValueName::new("attr2"),
        },
        operation: None,
    };
    let s_attr = StructAttributeType {
        name: ValueName::new("attr1"),
        ty: Type::Bool,
        index: 0,
    };
    let val = Value {
        inner_name: "x".into(),
        inner_type: Type::Struct(StructTypes {
            name: "St".into(),
            attributes: {
                let mut map = HashMap::default();
                map.insert(s_attr.name.clone(), s_attr);
                map
            },
            methods: HashMap::default(),
        }),
        mutable: false,
        alloca: false,
        malloc: false,
    };
    block_state.insert_value("x", val.clone());

    let s_attr = StructAttributeType {
        name: ValueName::new("attr1"),
        ty: Type::Bool,
        index: 0,
    };
    t.state.types(&StructTypes {
        name: "St".into(),
        attributes: {
            let mut map = HashMap::default();
            map.insert(s_attr.name.clone(), s_attr);
            map
        },
        methods: HashMap::default(),
    });
    assert!(t.is_empty_error());
    let res = t.state.expression(&expr, &block_state);
    assert!(res.is_none());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::ValueNotStructField),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn expression_struct_value() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let expression_st_value = ExprValue::Struct {
        name: ValueName::new("x"),
        attr: ValueName::new("attr1"),
    };
    let expr = Expression {
        value: expression_st_value,
        operation: None,
    };
    let s_attr = StructAttributeType {
        name: ValueName::new("attr1"),
        ty: Type::Bool,
        index: 0,
    };
    let value = Value {
        inner_name: "x".into(),
        inner_type: Type::Struct(StructTypes {
            name: "St".into(),
            attributes: {
                let mut map = HashMap::default();
                map.insert(s_attr.name.clone(), s_attr);
                map
            },
            methods: HashMap::default(),
        }),
        mutable: false,
        alloca: false,
        malloc: false,
    };
    block_state.insert_value("x", value.clone());

    let s_attr = StructAttributeType {
        name: ValueName::new("attr1"),
        ty: Type::Bool,
        index: 0,
    };
    t.state.types(&StructTypes {
        name: "St".into(),
        attributes: {
            let mut map = HashMap::default();
            map.insert(s_attr.name.clone(), s_attr);
            map
        },
        methods: HashMap::default(),
    });
    assert!(t.is_empty_error());
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert!(t.is_empty_error());
    assert_eq!(res, ExprResult::Register(Type::Bool, 2));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::ExpressionStructValue {
            expression: value,
            index: 0,
            register: 1,
        }
    );
}

#[test]
fn expression_func_call() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let fn_name = FunctionName::new("fn1");
    let fn_call = FunctionCall {
        name: fn_name.clone(),
        args: vec![],
    };
    let ast_fn_call = ExprValue::Call(fn_call);
    let expr = Expression {
        value: ast_fn_call,
        operation: None,
    };
    let res = t.state.expression(&expr, &block_state);
    assert!(res.is_none());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::FunctionNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );
    t.clean_errors();

    // Declare function
    let fn_statement = FunctionDecl::new(fn_name.clone(), vec![], Type::Ptr, vec![]);
    t.state.function_declaration(&fn_statement);
    assert!(t.is_empty_error());

    let res = t.state.expression(&expr, &block_state).unwrap();
    assert!(t.is_empty_error());
    assert_eq!(res, ExprResult::Register(Type::Ptr, 2),);
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::Call {
            call: FunctionHeader {
                name: fn_name,
                result: Type::Ptr,
                args: vec![],
            },
            params: vec![],
            register: 1,
        }
    );
}

#[test]
fn expression_sub_expression() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let sub_expr = Expression {
        value: ExprValue::Literal(Literal::U32(10)),
        operation: None,
    };
    let expr = Expression {
        value: ExprValue::Expr(Box::new(sub_expr)),
        operation: None,
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert!(t.is_empty_error());
    assert_eq!(res, ExprResult::Literal(Literal::U32(10)));
    let state = block_state.borrow().stack().clone().get();
    assert!(state.is_empty());
}

#[test]
fn expression_operation() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let next_expr = Expression {
        value: ExprValue::Literal(Literal::Char('b')),
        operation: None,
    };
    let expr = Expression {
        value: ExprValue::Literal(Literal::Char('a')),
        operation: Some((ExpressionOperations::Plus, Box::new(next_expr))),
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert!(t.is_empty_error());
    assert_eq!(res, ExprResult::Register(Type::Char, 1),);
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::ExpressionOperation {
            operation: ExpressionOperations::Plus,
            left_value: ExprResult::Literal(Literal::Char('a')),
            right_value: ExprResult::Literal(Literal::Char('b')),
            register: 1,
        }
    );
}

#[test]
fn expression_operation_wrong_type() {
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let next_expr = Expression {
        value: ExprValue::Literal(Literal::I64(10)),
        operation: None,
    };
    let expr = Expression {
        value: ExprValue::Literal(Literal::U64(20)),
        operation: Some((ExpressionOperations::Plus, Box::new(next_expr))),
    };
    let res = t.state.expression(&expr, &block_state);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::WrongExpressionType),
        "Errors: {:?}",
        t.state.errors[0]
    );
    assert!(res.is_none());
    let state = block_state.borrow().stack().clone().get();
    assert!(state.is_empty());
}

#[test]
fn expression_multiple_operation1() {
    // Expression: (1+2)*3-4-5*6
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let prev_next_expr = Expression {
        value: ExprValue::Literal(Literal::U16(2)),
        operation: None,
    };
    // Expr: 1 + 2
    let prev_expr = Expression {
        value: ExprValue::Literal(Literal::U16(1)),
        operation: Some((ExpressionOperations::Plus, Box::new(prev_next_expr))),
    };
    let next_expr4 = Expression {
        value: ExprValue::Literal(Literal::U16(6)),
        operation: None,
    };
    // Expr: 5 * 6
    let next_expr3 = Expression {
        value: ExprValue::Literal(Literal::U16(5)),
        operation: Some((ExpressionOperations::Multiply, Box::new(next_expr4))),
    };
    // Expr: 4 - 5 * 6
    let next_expr2 = Expression {
        value: ExprValue::Literal(Literal::U16(4)),
        operation: Some((ExpressionOperations::Minus, Box::new(next_expr3))),
    };
    // Expr: 3 - 4 - 5 * 6
    let next_expr1 = Expression {
        value: ExprValue::Literal(Literal::U16(3)),
        operation: Some((ExpressionOperations::Minus, Box::new(next_expr2))),
    };
    // Expr (1 + 2) * 3 - 4 - 5 * 6
    let expr = Expression {
        value: ExprValue::Expr(Box::new(prev_expr)),
        operation: Some((ExpressionOperations::Multiply, Box::new(next_expr1))),
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(Type::U16, 5));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 5);
    assert_eq!(
        state[0],
        set_result_type(ExpressionOperations::Plus, false, 1, false, 2, 1)
    );
    assert_eq!(
        state[1],
        set_result_type(ExpressionOperations::Multiply, true, 1, false, 3, 2)
    );
    assert_eq!(
        state[2],
        set_result_type(ExpressionOperations::Minus, true, 2, false, 4, 3)
    );
    assert_eq!(
        state[3],
        set_result_type(ExpressionOperations::Multiply, false, 5, false, 6, 4)
    );
    assert_eq!(
        state[4],
        set_result_type(ExpressionOperations::Minus, true, 3, true, 4, 5)
    );
    assert!(t.is_empty_error());
}

#[test]
fn expression_multiple_operation2() {
    // Expression: (100+2)*(3-4-5*6)
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let prev_next_expr = Expression {
        value: ExprValue::Literal(Literal::U16(2)),
        operation: None,
    };
    // Expr: 100 + 2
    let prev_expr = Expression {
        value: ExprValue::Literal(Literal::U16(100)),
        operation: Some((ExpressionOperations::Plus, Box::new(prev_next_expr))),
    };
    let next_expr4 = Expression {
        value: ExprValue::Literal(Literal::U16(6)),
        operation: None,
    };
    // Expr: 5 * 6
    let next_expr3 = Expression {
        value: ExprValue::Literal(Literal::U16(5)),
        operation: Some((ExpressionOperations::Multiply, Box::new(next_expr4))),
    };
    // Expr: 4 - 5 * 6
    let next_expr2 = Expression {
        value: ExprValue::Literal(Literal::U16(4)),
        operation: Some((ExpressionOperations::Minus, Box::new(next_expr3))),
    };
    // Expr: 3 - 4 - 5 * 6
    let next_expr1 = Expression {
        value: ExprValue::Literal(Literal::U16(3)),
        operation: Some((ExpressionOperations::Minus, Box::new(next_expr2))),
    };
    // Expr set brackets: (3 - 4 - 5 * 6)
    let next_expr = Expression {
        value: ExprValue::Expr(Box::new(next_expr1)),
        operation: None,
    };
    // Expr test Into transformation
    let ast_expr = ExprValue::Expr(Box::new(prev_expr.clone()));
    assert_eq!(ast_expr.to_string(), "100");
    // Expr (100 + 2) * (3 - 4 - 5 * 6)
    let expr = Expression {
        value: ExprValue::Expr(Box::new(prev_expr)),
        operation: Some((ExpressionOperations::Multiply, Box::new(next_expr))),
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(Type::U16, 5));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 5);
    assert_eq!(
        state[0],
        set_result_type(ExpressionOperations::Plus, false, 100, false, 2, 1)
    );
    assert_eq!(
        state[1],
        set_result_type(ExpressionOperations::Minus, false, 3, false, 4, 2)
    );
    assert_eq!(
        state[2],
        set_result_type(ExpressionOperations::Multiply, false, 5, false, 6, 3)
    );
    assert_eq!(
        state[3],
        set_result_type(ExpressionOperations::Minus, true, 2, true, 3, 4)
    );
    assert_eq!(
        state[4],
        set_result_type(ExpressionOperations::Multiply, true, 1, true, 4, 5)
    );
    assert!(t.is_empty_error());
}

#[test]
fn expression_multiple_operation_simple1() {
    // Expression: 100-5*6
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let prev_next_expr = Expression {
        value: ExprValue::Literal(Literal::U16(6)),
        operation: None,
    };
    let prev_expr = Expression {
        value: ExprValue::Literal(Literal::U16(5)),
        operation: Some((ExpressionOperations::Multiply, Box::new(prev_next_expr))),
    };
    let expr = Expression {
        value: ExprValue::Literal(Literal::U16(100)),
        operation: Some((ExpressionOperations::Minus, Box::new(prev_expr))),
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(Type::U16, 2));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 2);
    assert_eq!(
        state[0],
        set_result_type(ExpressionOperations::Multiply, false, 5, false, 6, 1)
    );
    assert_eq!(
        state[1],
        set_result_type(ExpressionOperations::Minus, false, 100, true, 1, 2)
    );
    assert!(t.is_empty_error());
}

#[test]
fn expression_multiple_operation_simple2() {
    // Expression: 20*5-40
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let prev_next_expr = Expression {
        value: ExprValue::Literal(Literal::U16(40)),
        operation: None,
    };
    let prev_expr = Expression {
        value: ExprValue::Literal(Literal::U16(5)),
        operation: Some((ExpressionOperations::Minus, Box::new(prev_next_expr))),
    };
    let expr = Expression {
        value: ExprValue::Literal(Literal::U16(20)),
        operation: Some((ExpressionOperations::Multiply, Box::new(prev_expr))),
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(Type::U16, 2));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 2);
    assert_eq!(
        state[0],
        set_result_type(ExpressionOperations::Multiply, false, 20, false, 5, 1)
    );
    assert_eq!(
        state[1],
        set_result_type(ExpressionOperations::Minus, true, 1, false, 40, 2)
    );
    assert!(t.is_empty_error());
}

#[test]
fn expression_multiple_operation_simple3() {
    // Expression: 20*4-40-5
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let prev_next_expr2 = Expression {
        value: ExprValue::Literal(Literal::U16(5)),
        operation: None,
    };
    let prev_next_expr = Expression {
        value: ExprValue::Literal(Literal::U16(40)),
        operation: Some((ExpressionOperations::Minus, Box::new(prev_next_expr2))),
    };
    let prev_expr = Expression {
        value: ExprValue::Literal(Literal::U16(4)),
        operation: Some((ExpressionOperations::Minus, Box::new(prev_next_expr))),
    };
    let expr = Expression {
        value: ExprValue::Literal(Literal::U16(20)),
        operation: Some((ExpressionOperations::Multiply, Box::new(prev_expr))),
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(Type::U16, 3));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 3);
    assert_eq!(
        state[0],
        set_result_type(ExpressionOperations::Multiply, false, 20, false, 4, 1)
    );
    assert_eq!(
        state[1],
        set_result_type(ExpressionOperations::Minus, true, 1, false, 40, 2)
    );
    assert_eq!(
        state[2],
        set_result_type(ExpressionOperations::Minus, true, 2, false, 5, 3)
    );
    assert!(t.is_empty_error());
}

#[test]
fn expression_multiple_operation_simple4() {
    // Expression: 100-5*6-15
    let block_state = Handle::new(BlockState::new(None));
    let mut t = SemanticTest::new();
    let prev_next_expr2 = Expression {
        value: ExprValue::Literal(Literal::U16(15)),
        operation: None,
    };

    let prev_next_expr = Expression {
        value: ExprValue::Literal(Literal::U16(6)),
        operation: Some((ExpressionOperations::Minus, Box::new(prev_next_expr2))),
    };
    let prev_expr = Expression {
        value: ExprValue::Literal(Literal::U16(5)),
        operation: Some((ExpressionOperations::Multiply, Box::new(prev_next_expr))),
    };
    let expr = Expression {
        value: ExprValue::Literal(Literal::U16(100)),
        operation: Some((ExpressionOperations::Minus, Box::new(prev_expr))),
    };
    let res = t.state.expression(&expr, &block_state).unwrap();
    assert_eq!(res, ExprResult::Register(Type::U16, 3));
    let state = block_state.borrow().stack().clone().get();
    assert_eq!(state.len(), 3);
    assert_eq!(
        state[0],
        set_result_type(ExpressionOperations::Multiply, false, 5, false, 6, 1)
    );
    assert_eq!(
        state[1],
        set_result_type(ExpressionOperations::Minus, false, 100, true, 1, 2)
    );
    assert_eq!(
        state[2],
        set_result_type(ExpressionOperations::Minus, true, 2, false, 15, 3)
    );
    assert!(t.is_empty_error());
}

#[test]
fn custom_expression() {
    #[derive(Clone, Debug, PartialEq)]
    pub enum AstCustomExpression {
        GoIn(u32, u32),
        GoOut(u32),
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct CustomExpression<I: SemanticContextInstruction> {
        ast: AstCustomExpression,
        _marker: PhantomData<I>,
    }

    impl<I: SemanticContextInstruction> CustomExpression<I> {
        fn new(ast: AstCustomExpression) -> Self {
            Self {
                ast,
                _marker: PhantomData,
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    #[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
    pub enum CustomExpressionInstruction {
        GoIn { index: u32, value: u32 },
        GoOut { result: u32 },
    }

    impl SemanticContextInstruction for CustomExpressionInstruction {}

    impl ExtendedExpression<CustomExpressionInstruction>
        for CustomExpression<CustomExpressionInstruction>
    {
        fn expression(
            &self,
            _state: &mut State<Self, CustomExpressionInstruction>,
            block_state: &Handle<BlockState<CustomExpressionInstruction, Self>>,
        ) -> ExprResult {
            let reg = block_state.inc_register();
            let instr = match self.ast {
                AstCustomExpression::GoIn(x, y) => {
                    CustomExpressionInstruction::GoIn { index: x, value: y }
                }
                AstCustomExpression::GoOut(x) => CustomExpressionInstruction::GoOut { result: x },
            };
            block_state.borrow_mut().extended_expression(&instr);
            ExprResult::Register(Type::U32, reg)
        }
    }

    impl std::fmt::Display for CustomExpression<CustomExpressionInstruction> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            Ok(())
        }
    }

    let block_state = Handle::new(BlockState::<CustomExpressionInstruction, _>::new(None));
    let mut state = State::<
        CustomExpression<CustomExpressionInstruction>,
        CustomExpressionInstruction,
    >::default();

    let next_expr = Expression::<CustomExpression<CustomExpressionInstruction>> {
        value: ExprValue::Extended(Box::new(CustomExpression::new(AstCustomExpression::GoIn(
            10, 20,
        )))),
        operation: None,
    };
    let expr = Expression {
        value: ExprValue::Extended(Box::new(CustomExpression::new(AstCustomExpression::GoOut(
            30,
        )))),
        operation: Some((ExpressionOperations::Plus, Box::new(next_expr))),
    };

    let res = state.expression(&expr, &block_state).unwrap();
    assert!(state.errors.is_empty());

    assert_eq!(res, ExprResult::Register(Type::U32, 3));
    let bs = block_state.borrow().stack().clone().get();
    assert_eq!(bs.len(), 3);

    assert_eq!(
        bs[0],
        SemanticStackContext::ExtendedExpression(Box::new(CustomExpressionInstruction::GoOut {
            result: 30
        }))
    );
    assert_eq!(
        bs[1],
        SemanticStackContext::ExtendedExpression(Box::new(CustomExpressionInstruction::GoIn {
            index: 10,
            value: 20,
        }))
    );
    assert_eq!(
        bs[2],
        SemanticStackContext::ExpressionOperation {
            operation: ExpressionOperations::Plus,
            left_value: ExprResult::Register(Type::U32, 1),
            right_value: ExprResult::Register(Type::U32, 2),
            register: 3,
        }
    );

    // #[cfg(feature = "codec")]
    // {
    //     let json = serde_json::to_string(&bs).unwrap();
    //     let bs_decoded: Vec<SemanticStackContext<CustomExpressionInstruction, _>> =
    //         serde_json::from_str(&json).unwrap();
    //     assert_eq!(bs, bs_decoded);
    // }
}
