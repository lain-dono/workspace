mod utils;

#[cfg(test)]
#[cfg(feature = "codec")]
mod test {
    use std::collections::HashMap;

    use crate::utils::{CustomExpression, CustomExpressionInstruction, SemanticTest};
    use semantic::block_state::BlockState;
    use semantic::constants::{Constant, ConstantExpression, ConstantValue};
    use semantic::error::{StateErrorKind, StateErrorResult};
    use semantic::expression::{ExprResult, ExprValue, Expression, ExpressionOperations};
    use semantic::function::{FunctionCall, FunctionDecl};
    use semantic::handle::Handle;
    use semantic::names::{
        ConstantName, FunctionName, ImportName, InnerValueName, LabelName, TypeName, ValueName,
    };
    use semantic::stmt::{
        BodyStatement, Condition, ExpressionCondition, ExpressionLogicCondition, IfBodyStatement,
        IfBodyStatements, IfCondition, IfLoopBodyStatement, IfStatement, Logic, LoopBodyStatement,
        Stmt,
    };
    use semantic::types::{Literal, StructAttributeType, StructTypes, Type, Value};
    use semantic::{ImportPath, Main, MainStatement};

    #[test]
    fn basic_ast_serialize() {
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
            attributes: {
                let mut map = HashMap::new();
                map.insert(
                    ValueName::new("y"),
                    StructAttributeType {
                        name: ValueName::new("y"),
                        ty: Type::U64,
                        index: 0,
                    },
                );
                map
            },
            methods: Default::default(),
        };
        let ty_stm = MainStatement::Types(ty.clone());

        let let_binding = Stmt::LetBinding {
            name: ValueName::new("x"),
            mutable: true,
            ty: None,
            value: Box::new(Expression {
                value: ExprValue::Literal(Literal::Bool(false)),
                operation: None,
            }),
        };
        let body_let_binding = let_binding.clone();
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
                operation: Some((
                    ExpressionOperations::And,
                    Box::new(Expression {
                        value: ExprValue::Literal(Literal::Bool(true)),
                        operation: None,
                    }),
                )),
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
        let body_loop = Stmt::Loop(vec![
            LoopBodyStatement::Stmt(Stmt::If(IfStatement {
                condition: IfCondition::Logic(ExpressionLogicCondition {
                    left: ExpressionCondition {
                        left: Expression {
                            value: ExprValue::Literal(Literal::U32(10)),
                            operation: None,
                        },
                        condition: Condition::Gt,
                        right: Expression {
                            value: ExprValue::Literal(Literal::U32(20)),
                            operation: None,
                        },
                    },
                    right: Some((
                        Logic::Or,
                        Box::new(ExpressionLogicCondition {
                            left: ExpressionCondition {
                                left: Expression {
                                    value: ExprValue::Literal(Literal::U32(30)),
                                    operation: None,
                                },
                                condition: Condition::Le,
                                right: Expression {
                                    value: ExprValue::Literal(Literal::U32(40)),
                                    operation: None,
                                },
                            },
                            right: None,
                        }),
                    )),
                }),
                else_statement: None,
                else_if_statement: None,
                body: IfBodyStatements::Loop(vec![
                    IfLoopBodyStatement::Stmt(let_binding.clone()),
                    IfLoopBodyStatement::Break,
                ]),
            })),
            LoopBodyStatement::Stmt(Stmt::FunctionCall(FunctionCall {
                name: FunctionName::new("fn2"),
                args: vec![],
            })),
        ]);
        let body_return = BodyStatement::Return(Expression {
            value: ExprValue::Literal(Literal::Bool(true)),
            operation: None,
        });
        let fn1 = FunctionDecl::new(
            FunctionName::new("fn1"),
            vec![],
            Type::Bool,
            vec![
                BodyStatement::Stmt(body_let_binding.clone()),
                BodyStatement::Stmt(body_binding),
                BodyStatement::Stmt(body_fn_call),
                BodyStatement::Stmt(body_if),
                BodyStatement::Stmt(body_loop),
                body_return.clone(),
            ],
        );
        let fn1_stm = MainStatement::Function(fn1);

        let body_expr_return = BodyStatement::Expr(Expression {
            value: ExprValue::Literal(Literal::U32(23)),
            operation: None,
        });
        let fn2 = FunctionDecl::new(
            FunctionName::new("fn2"),
            vec![(String::from("x"), Type::U32)],
            Type::U32,
            vec![body_expr_return],
        );
        let fn2_stm = MainStatement::Function(fn2.clone());

        let main_stm: Main<CustomExpression<CustomExpressionInstruction>> =
            vec![import_stm, constant_stm, ty_stm, fn1_stm, fn2_stm];
        let json = serde_json::to_string(&main_stm).unwrap();
        let ser_ast: Main<CustomExpression<CustomExpressionInstruction>> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(main_stm, ser_ast);

        t.state.run(&main_stm);
        assert!(t.is_empty_error());
        let _json_state = serde_json::to_string(&t.state).unwrap();
    }

    #[test]
    fn semantic_extended_serde_check() {
        // It covers uncovered serde parts
        let iv: InnerValueName = "x".into();
        let to_json = serde_json::to_string(&iv).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(iv, to_val);

        let lbl: LabelName = String::from("lbl").into();
        let to_json = serde_json::to_string(&lbl).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(lbl, to_val);

        let v = Value {
            inner_name: "x".into(),
            inner_type: Type::Ptr,
            mutable: false,
            alloca: false,
            malloc: false,
        };
        let to_json = serde_json::to_string(&v).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(v, to_val);

        let parent_bs: BlockState<CustomExpressionInstruction, ()> = BlockState::new(None);
        let bs = BlockState::new(Some(Handle::new(parent_bs)));
        let to_json = serde_json::to_string(&bs).unwrap();
        let _to_val: BlockState<CustomExpressionInstruction, ()> =
            serde_json::from_str(&to_json).unwrap();

        let lcond = Logic::Or;
        let to_json = serde_json::to_string(&lcond).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(lcond, to_val);

        let cond = Condition::Gt;
        let to_json = serde_json::to_string(&cond).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(cond, to_val);

        let lbs = LoopBodyStatement::<()>::Break;
        let to_json = serde_json::to_string(&lbs).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(lbs, to_val);

        let lbs = IfLoopBodyStatement::<()>::Break;
        let to_json = serde_json::to_string(&lbs).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(lbs, to_val);

        let pv = Literal::Ptr;
        let to_json = serde_json::to_string(&pv).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(pv, to_val);

        let ex_res = ExprResult::Literal(Literal::Ptr);
        let to_json = serde_json::to_string(&ex_res).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(ex_res, to_val);

        let ex_s = ExprValue::<()>::Struct {
            name: "x".to_string().into(),
            attr: "y".to_string().into(),
        };
        let to_json = serde_json::to_string(&ex_s).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(ex_s, to_val);

        let exp_op = ExpressionOperations::And;
        let to_json = serde_json::to_string(&exp_op).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(exp_op, to_val);

        let state_err = StateErrorResult {
            kind: StateErrorKind::Common,
            value: "test".to_string(),
        };
        let to_json = serde_json::to_string(&state_err).unwrap();
        let to_val = serde_json::from_str(&to_json).unwrap();
        assert_eq!(state_err, to_val);
    }
}
