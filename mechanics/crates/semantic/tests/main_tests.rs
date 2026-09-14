use crate::utils::{CustomExpression, CustomExpressionInstruction, SemanticTest};
use semantic::constants::{Constant, ConstantExpression, ConstantValue};
use semantic::context::SemanticStackContext;
use semantic::error::StateErrorKind;
use semantic::expression::{ExprResult, ExprValue, Expression, ExpressionOperations};
use semantic::function::{FunctionCall, FunctionDecl, FunctionHeader};
use semantic::names::{ConstantName, FunctionName, ImportName, TypeName, ValueName};
use semantic::stmt::{
    BodyStatement, IfBodyStatement, IfBodyStatements, IfCondition, IfStatement, LoopBodyStatement,
    Stmt,
};
use semantic::types::{Literal, StructTypes, Type, Value};
use semantic::{ImportPath, Main, MainStatement};

mod utils;

#[test]
fn main_run() {
    let mut t = SemanticTest::new();
    let imports: ImportPath = vec![ImportName::new("import1")];
    let import_stm = MainStatement::Import(imports);

    let constant1 = Constant {
        name: ConstantName::new("const1"),
        ty: Type::None,
        value: ConstantExpression {
            value: ConstantValue::Constant(ConstantName::new("const2")),
            operation: None,
        },
    };
    let constant_stm = MainStatement::Constant(constant1.clone());

    let ty = StructTypes {
        name: TypeName::new("StructType"),
        attributes: Default::default(),
        methods: Default::default(),
    };
    let ty_stm = MainStatement::Types(ty.clone());

    let body_let_binding = Stmt::LetBinding {
        name: ValueName::new("x"),
        mutable: true,
        ty: None,
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(false)),
            operation: None,
        }),
    };
    let body_binding = Stmt::Binding {
        name: ValueName::new("x"),
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::Bool(true)),
            operation: None,
        }),
    };
    let body_fn_call = Stmt::FunctionCall(FunctionCall {
        name: FunctionName::new("fn2"),
        args: vec![],
    });
    let body_if = Stmt::If(IfStatement {
        condition: IfCondition::Single(Expression {
            value: ExprValue::Literal(Literal::Bool(true)),
            operation: None,
        }),
        body: IfBodyStatements::If(vec![IfBodyStatement::Stmt(Stmt::FunctionCall(
            FunctionCall {
                name: FunctionName::new("fn2"),
                args: vec![],
            },
        ))]),
        else_statement: None,
        else_if_statement: None,
    });
    let body_loop = Stmt::Loop(vec![LoopBodyStatement::Stmt(Stmt::FunctionCall(
        FunctionCall {
            name: FunctionName::new("fn2"),
            args: vec![],
        },
    ))]);
    let body_return = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let fn1 = FunctionDecl::new(
        FunctionName::new("fn1"),
        vec![],
        Type::Bool,
        vec![
            BodyStatement::Stmt(body_let_binding),
            BodyStatement::Stmt(body_binding),
            BodyStatement::Stmt(body_fn_call),
            BodyStatement::Stmt(body_if),
            BodyStatement::Stmt(body_loop),
            BodyStatement::Return(body_return.clone()),
        ],
    );
    let fn_stm = MainStatement::Function(fn1.clone());

    let body_expr_return = BodyStatement::Expr(Expression {
        value: ExprValue::Literal(Literal::U16(23)),
        operation: None,
    });
    let fn2 = FunctionDecl::new(
        FunctionName::new("fn2"),
        vec![],
        Type::U16,
        vec![body_expr_return],
    );
    let fn2_stm = MainStatement::Function(fn2.clone());
    let main_stm: Main<CustomExpression<CustomExpressionInstruction>> =
        vec![import_stm, constant_stm, ty_stm, fn_stm, fn2_stm];
    // For grcov
    let _ = format!("{main_stm:#?}");
    t.state.run(&main_stm);
    assert!(t.is_empty_error());

    assert_eq!(
        t.state.global.constants.get(&constant1.name).unwrap(),
        &constant1
    );
    assert_eq!(
        t.state.global.types.get(&ty.name).unwrap(),
        &Type::Struct(ty.clone())
    );
    let fn_state = t.state.global.functions.get(&fn1.name).unwrap();
    assert_eq!(fn_state.name, fn1.name);
    assert_eq!(fn_state.result, fn1.result.clone());
    assert!(fn_state.args.is_empty());

    // Function body context
    assert_eq!(t.state.blocks.len(), 2);
    let ctx1 = t.state.blocks[0].borrow();
    assert_eq!(ctx1.children.len(), 2);
    assert!(ctx1.parent.is_none());
    let ctx2 = t.state.blocks[1].borrow();
    assert!(ctx2.children.is_empty());
    assert!(ctx2.parent.is_none());

    let ch_ctx1 = ctx1.children[0].clone();
    assert!(ch_ctx1.borrow().parent.is_some());
    assert!(ch_ctx1.borrow().children.is_empty());

    let ch_ctx2 = ctx1.children[1].clone();
    assert!(ch_ctx2.borrow().parent.is_some());
    assert!(ch_ctx2.borrow().children.is_empty());

    // Semantic stack context for the block fn2
    let st_ctx2 = ctx2.stack().clone().get();
    assert_eq!(st_ctx2.len(), 1);
    assert_eq!(
        st_ctx2[0],
        SemanticStackContext::ExpressionFunctionReturn {
            expr_result: ExprResult::Literal(Literal::U16(23)),
        }
    );

    let st_ch_ctx1 = ch_ctx1.borrow().stack().clone().get();
    assert_eq!(st_ch_ctx1.len(), 5);
    assert_eq!(
        st_ch_ctx1[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
            label_if_begin: String::from("if_begin").into(),
            label_if_end: String::from("if_end").into(),
        }
    );
    assert_eq!(
        st_ch_ctx1[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin").into()
        }
    );
    assert_eq!(
        st_ch_ctx1[2],
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
        st_ch_ctx1[3],
        SemanticStackContext::JumpTo {
            label: String::from("if_end").into()
        }
    );
    assert_eq!(
        st_ch_ctx1[4],
        SemanticStackContext::SetLabel {
            label: String::from("if_end").into()
        }
    );

    let st_ch_ctx2 = ch_ctx2.borrow().stack().clone().get();
    assert_eq!(st_ch_ctx2.len(), 5);
    assert_eq!(
        st_ch_ctx2[0],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        st_ch_ctx2[1],
        SemanticStackContext::SetLabel {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        st_ch_ctx2[2],
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
        st_ch_ctx2[3],
        SemanticStackContext::JumpTo {
            label: String::from("loop_begin").into()
        }
    );
    assert_eq!(
        st_ch_ctx2[4],
        SemanticStackContext::SetLabel {
            label: String::from("loop_end").into()
        }
    );

    // Global semantic stack context
    let st_global_context = t.state.global.context.get();
    assert_eq!(st_global_context.len(), 4);
    assert_eq!(
        st_global_context[0],
        SemanticStackContext::Types { type_decl: ty }
    );
    assert_eq!(
        st_global_context[1],
        SemanticStackContext::Constant {
            const_decl: constant1
        }
    );
    assert_eq!(
        st_global_context[2],
        SemanticStackContext::FunctionDeclaration { fn_decl: fn1 }
    );
    assert_eq!(
        st_global_context[3],
        SemanticStackContext::FunctionDeclaration { fn_decl: fn2 }
    );

    // Semantic stack context for the block fn1
    let st_ctx1 = ctx1.stack().clone().get();
    assert_eq!(st_ctx1.len(), 14);
    assert_eq!(
        st_ctx1[0],
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
        st_ctx1[1],
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
        st_ctx1[2],
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
    assert_eq!(st_ctx1[3], st_ch_ctx1[0]);
    assert_eq!(st_ctx1[4], st_ch_ctx1[1]);
    assert_eq!(st_ctx1[5], st_ch_ctx1[2]);
    assert_eq!(st_ctx1[6], st_ch_ctx1[3]);
    assert_eq!(st_ctx1[7], st_ch_ctx1[4]);
    assert_eq!(st_ctx1[8], st_ch_ctx2[0]);
    assert_eq!(st_ctx1[9], st_ch_ctx2[1]);
    assert_eq!(st_ctx1[10], st_ch_ctx2[2]);
    assert_eq!(st_ctx1[11], st_ch_ctx2[3]);
    assert_eq!(st_ctx1[12], st_ch_ctx2[4]);
    assert_eq!(
        st_ctx1[13],
        SemanticStackContext::ExpressionFunctionReturn {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
        }
    );
}

#[test]
fn double_return() {
    let mut t = SemanticTest::new();
    let body_return = BodyStatement::Return(Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    });
    let body_expr = BodyStatement::Expr(Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    });
    let fn1 = FunctionDecl::new(
        FunctionName::new("fn1"),
        vec![],
        Type::Bool,
        vec![body_return, body_expr],
    );
    let fn_stm = MainStatement::Function(fn1);
    let main_stm: Main<CustomExpression<CustomExpressionInstruction>> = vec![fn_stm];
    t.state.run(&main_stm);
    assert!(t.check_errors_len(2), "Errors: {:?}", t.state.errors.len());
    assert!(t.check_error_index(0, StateErrorKind::ForbiddenCodeAfterReturnDeprecated));
    assert!(t.check_error_index(1, StateErrorKind::ReturnAlreadyCalled));
}

#[test]
fn wrong_return_type() {
    let mut t = SemanticTest::new();
    let body_return = BodyStatement::Return(Expression {
        value: ExprValue::Literal(Literal::I8(10)),
        operation: None,
    });
    let fn1 = FunctionDecl::new(
        FunctionName::new("fn1"),
        vec![],
        Type::Bool,
        vec![body_return],
    );
    let fn_stm = MainStatement::Function(fn1);
    let main_stm: Main<CustomExpression<CustomExpressionInstruction>> = vec![fn_stm];
    t.state.run(&main_stm);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::WrongReturnType),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn expression_as_return() {
    let mut t = SemanticTest::new();
    let body_expr = BodyStatement::Expr(Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    });
    let fn1 = FunctionDecl::new(
        FunctionName::new("fn1"),
        vec![],
        Type::Bool,
        vec![body_expr],
    );
    let fn_stm = MainStatement::Function(fn1.clone());
    let main_stm: Main<CustomExpression<CustomExpressionInstruction>> = vec![fn_stm];
    t.state.run(&main_stm);
    assert!(t.is_empty_error());

    // Function body context
    assert_eq!(t.state.blocks.len(), 1);
    let ctx = t.state.blocks[0].borrow();
    assert!(ctx.children.is_empty());
    assert!(ctx.parent.is_none());

    // Semantic stack context for the block
    let st_ctx = ctx.stack().clone().get();
    assert_eq!(st_ctx.len(), 1);
    assert_eq!(
        st_ctx[0],
        SemanticStackContext::ExpressionFunctionReturn {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
        }
    );

    // Global semantic stack context
    let st_global_context = t.state.global.context.get();
    assert_eq!(st_global_context.len(), 1);
    assert_eq!(
        st_global_context[0],
        SemanticStackContext::FunctionDeclaration { fn_decl: fn1 }
    );
}

#[test]
fn if_return_from_function() {
    let mut t = SemanticTest::new();
    let body_expr_return = BodyStatement::Expr(Expression {
        value: ExprValue::Literal(Literal::I8(5)),
        operation: None,
    });
    let if_expr_return = Expression {
        value: ExprValue::Literal(Literal::I8(10)),
        operation: None,
    };
    let if_expr = Expression {
        value: ExprValue::Literal(Literal::Bool(true)),
        operation: None,
    };
    let body_if = Stmt::If(IfStatement {
        condition: IfCondition::Single(if_expr),
        body: IfBodyStatements::If(vec![IfBodyStatement::Return(if_expr_return)]),
        else_statement: None,
        else_if_statement: None,
    });
    let fn1 = FunctionDecl::new(
        FunctionName::new("fn1"),
        vec![],
        Type::I8,
        vec![BodyStatement::Stmt(body_if), body_expr_return],
    );
    let fn_stm = MainStatement::Function(fn1.clone());
    let main_stm: Main<CustomExpression<CustomExpressionInstruction>> = vec![fn_stm];
    t.state.run(&main_stm);
    assert!(t.is_empty_error());

    // Function body context
    assert_eq!(t.state.blocks.len(), 1);
    let ctx = t.state.blocks[0].borrow();
    assert_eq!(ctx.children.len(), 1);
    assert!(ctx.parent.is_none());

    // Children block context
    let children_ctx = ctx.children[0].borrow();
    assert!(children_ctx.children.is_empty());
    assert!(children_ctx.parent.is_some());

    // Children semantic stack context for the block
    let st_children_ctx = children_ctx.stack().clone().get();
    assert_eq!(st_children_ctx.len(), 4);
    assert_eq!(
        st_children_ctx[0],
        SemanticStackContext::IfConditionExpression {
            expr_result: ExprResult::Literal(Literal::Bool(true)),
            label_if_begin: String::from("if_begin").into(),
            label_if_end: String::from("if_end").into(),
        }
    );
    assert_eq!(
        st_children_ctx[1],
        SemanticStackContext::SetLabel {
            label: String::from("if_begin").into()
        }
    );
    assert_eq!(
        st_children_ctx[2],
        SemanticStackContext::JumpFunctionReturn {
            expr_result: ExprResult::Literal(Literal::I8(10)),
        }
    );
    assert_eq!(
        st_children_ctx[3],
        SemanticStackContext::SetLabel {
            label: String::from("if_end").into()
        }
    );

    // Semantic stack context for the block
    let st_ctx = ctx.stack().get();
    assert_eq!(st_ctx.len(), 5);
    assert_eq!(st_ctx[0], st_children_ctx[0]);
    assert_eq!(st_ctx[1], st_children_ctx[1]);
    assert_eq!(st_ctx[2], st_children_ctx[2]);
    assert_eq!(st_ctx[3], st_children_ctx[3]);
    assert_eq!(
        st_ctx[4],
        SemanticStackContext::ExpressionFunctionReturnWithLabel {
            expr_result: ExprResult::Literal(Literal::I8(5)),
        }
    );

    // Global semantic stack context
    let st_global_context = t.state.global.context.get();
    assert_eq!(st_global_context.len(), 1);
    assert_eq!(
        st_global_context[0],
        SemanticStackContext::FunctionDeclaration { fn_decl: fn1 }
    );
}

#[test]
fn function_args_and_let_binding() {
    let mut t = SemanticTest::new();
    let body_let_binding = Stmt::LetBinding {
        name: ValueName::new("y"),
        mutable: true,
        ty: Some(Type::U64),
        value: Box::new(Expression {
            value: ExprValue::Literal(Literal::U64(23)),
            operation: Some((
                ExpressionOperations::Plus,
                Box::new(Expression {
                    value: ExprValue::Variable(ValueName::new("x")),
                    operation: None,
                }),
            )),
        }),
    };
    let body_expr_return = BodyStatement::Expr(Expression {
        value: ExprValue::Variable(ValueName::new("y")),
        operation: None,
    });

    let fn_param1_name = String::from("x");
    let fn_param1_ty = Type::U64;
    let fn1 = FunctionDecl::new(
        FunctionName::new("fn1"),
        vec![(fn_param1_name.clone(), fn_param1_ty.clone())],
        Type::U64,
        vec![BodyStatement::Stmt(body_let_binding), body_expr_return],
    );
    let fn_stm = MainStatement::Function(fn1.clone());
    let main_stm: Main<CustomExpression<CustomExpressionInstruction>> = vec![fn_stm];
    t.state.run(&main_stm);
    assert!(t.is_empty_error());

    assert_eq!(t.state.blocks.len(), 1);
    let ctx = t.state.blocks[0].borrow();
    assert!(ctx.children.is_empty());
    assert!(ctx.parent.is_none());

    let stm_ctx = ctx.stack().get();
    let ty = Type::U64;
    let value_x = Value {
        inner_name: "x".into(),
        inner_type: ty.clone(),
        mutable: false,
        alloca: false,
        malloc: false,
    };
    let value_y = Value {
        inner_name: "y.0".into(),
        inner_type: ty.clone(),
        mutable: true,
        alloca: false,
        malloc: false,
    };
    assert_eq!(
        stm_ctx[0],
        SemanticStackContext::FunctionArg {
            value: value_x.clone(),
            name: fn_param1_name,
            ty: fn_param1_ty,
        }
    );
    assert_eq!(
        stm_ctx[1],
        SemanticStackContext::ExpressionValue {
            expression: value_x,
            register: 1,
        }
    );
    assert_eq!(
        stm_ctx[2],
        SemanticStackContext::ExpressionOperation {
            operation: ExpressionOperations::Plus,
            left_value: ExprResult::Literal(Literal::U64(23)),
            right_value: ExprResult::Register(ty.clone(), 1),
            register: 2,
        }
    );
    assert_eq!(
        stm_ctx[3],
        SemanticStackContext::LetBinding {
            let_decl: value_y.clone(),
            expr_result: ExprResult::Register(ty.clone(), 2),
        }
    );
    assert_eq!(
        stm_ctx[4],
        SemanticStackContext::ExpressionValue {
            expression: value_y,
            register: 3,
        }
    );
    assert_eq!(
        stm_ctx[5],
        SemanticStackContext::ExpressionFunctionReturn {
            expr_result: ExprResult::Register(ty.clone(), 3),
        }
    );

    // Verify global entities
    let fn_state = t.state.global.functions.get(&fn1.name).unwrap();
    assert_eq!(fn_state.name, fn1.name);
    assert_eq!(fn_state.result, fn1.result.clone());
    assert_eq!(fn_state.args.len(), 1);
    assert_eq!(fn_state.args[0], ty);
    let global_ctx = t.state.global.context.get();
    assert_eq!(global_ctx.len(), 1);
    assert_eq!(
        global_ctx[0],
        SemanticStackContext::FunctionDeclaration { fn_decl: fn1 }
    );
}

#[test]
fn function_args_duplication() {
    let mut t = SemanticTest::new();
    let body_expr_return = BodyStatement::Expr(Expression {
        value: ExprValue::Literal(Literal::U64(10)),
        operation: None,
    });

    let fn1 = FunctionDecl::new(
        FunctionName::new("fn1"),
        vec![
            (String::from("x"), Type::U64),
            (String::from("x"), Type::U64),
        ],
        Type::U64,
        vec![body_expr_return],
    );
    let fn_stm = MainStatement::Function(fn1.clone());
    let main_stm: Main<CustomExpression<CustomExpressionInstruction>> = vec![fn_stm];
    t.state.run(&main_stm);
    assert!(t.check_errors_len(1));
    assert!(t.check_error(StateErrorKind::FunctionArgumentNameDuplicated));
}
