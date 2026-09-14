use crate::utils::SemanticTest;
use semantic::context::SemanticStackContext;
use semantic::error::StateErrorKind;
use semantic::names::{TypeName, ValueName};
use semantic::types::{StructAttributeType, StructTypes, Type};
use std::collections::HashMap;

mod utils;

#[test]
fn types_declaration() {
    let mut t = SemanticTest::new();
    let type_decl = StructTypes {
        name: TypeName::new("type1"),
        attributes: Default::default(),
        methods: Default::default(),
    };
    t.state.types(&type_decl.clone());
    assert!(t.is_empty_error());

    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 1);
    assert_eq!(
        state[0],
        SemanticStackContext::Types {
            type_decl: type_decl.clone()
        }
    );

    let ty1 = StructAttributeType {
        name: ValueName::new("attr1"),
        ty: Type::Char,
        index: 0,
    };
    let ty2 = StructAttributeType {
        name: ValueName::new("attr2"),
        ty: Type::U8,
        index: 1,
    };
    let type_decl2 = StructTypes {
        name: TypeName::new("type2"),
        attributes: {
            let mut map = HashMap::new();
            map.insert(ty1.name.clone(), ty1);
            map.insert(ty2.name.clone(), ty2);
            map
        },
        methods: Default::default(),
    };
    t.state.types(&type_decl2.clone());
    assert!(t.is_empty_error());
    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 2);
    assert_eq!(state[0], SemanticStackContext::Types { type_decl });
    assert_eq!(
        state[1],
        SemanticStackContext::Types {
            type_decl: type_decl2.clone()
        }
    );

    t.state.types(&type_decl2.clone());
    assert!(t.check_errors_len(1), "Errors: {:?}", t.state.errors.len());
    assert!(
        t.check_error(StateErrorKind::TypeAlreadyExist),
        "Errors: {:?}",
        t.state.errors[0]
    );
    let state = t.state.global.context.clone().get();
    assert_eq!(state.len(), 2);
}
