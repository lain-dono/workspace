use crate::utils::SemanticTest;
use semantic::context::SemanticStackContext;
use semantic::error::StateErrorKind;
use semantic::function::FunctionDecl;
use semantic::names::{FunctionName, TypeName};
use semantic::types::{StructTypes, Type};

mod utils;

#[test]
fn function_declaration_without_body() {
    let mut t = SemanticTest::new();
    let fn_name = FunctionName::new("fn1");
    let fn_statement = FunctionDecl::new(fn_name.clone(), vec![], Type::I8, vec![]);
    t.state.function_declaration(&fn_statement);
    assert!(t.is_empty_error());
    assert!(t.state.global.functions.contains_key(&fn_name));
    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::FunctionDeclaration {
            fn_decl: fn_statement.clone()
        }
    );

    t.state.function_declaration(&fn_statement);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::FunctionAlreadyExist),
        "Errors: {:?}",
        t.state.errors[0]
    );
}

#[test]
fn function_declaration_wrong_type() {
    let mut t = SemanticTest::new();
    let fn_name = FunctionName::new("fn2");

    let type_decl = StructTypes {
        name: TypeName::new("type1"),
        attributes: Default::default(),
        methods: Default::default(),
    };

    let fn_statement = FunctionDecl::new(
        fn_name.clone(),
        vec![(String::from("x"), Type::I64)],
        Type::Struct(type_decl.clone()),
        vec![],
    );
    t.state.function_declaration(&fn_statement);
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::TypeNotFound),
        "Errors: {:?}",
        t.state.errors[0]
    );

    assert!(!t.state.global.functions.contains_key(&fn_name));
    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 0);

    let fn_statement2 = FunctionDecl::new(
        fn_statement.name,
        vec![(String::from("x"), Type::Struct(type_decl.clone()))],
        Type::I64,
        fn_statement.body,
    );
    t.state.function_declaration(&fn_statement2);
    assert!(t.check_errors_len(2), "Errors: {:?}", t.state.errors.len());
    assert!(t.check_error_index(1, StateErrorKind::TypeNotFound));
    t.clean_errors();

    t.state.types(&type_decl);
    assert!(t.is_empty_error());
    t.state.function_declaration(&fn_statement2);
    assert!(t.is_empty_error());
    assert!(t.state.global.functions.contains_key(&fn_name));
    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 2);
    assert_eq!(
        state[1],
        SemanticStackContext::FunctionDeclaration {
            fn_decl: fn_statement2
        }
    );
}
