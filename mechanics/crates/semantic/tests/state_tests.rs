use crate::utils::{CustomExpression, CustomExpressionInstruction};
use semantic::block_state::BlockState;
use semantic::constants::{Constant, ConstantExpression, ConstantValue};
use semantic::context::{ExtendedSemanticContext, SemanticContext, SemanticStack};
use semantic::expression::{ExprResult, ExpressionOperations};
use semantic::function::{FunctionDecl, FunctionHeader};
use semantic::handle::Handle;
use semantic::names::{FunctionName, InnerValueName, LabelName, ValueName};
use semantic::semantic::State;
use semantic::stmt::{Condition, Logic};
use semantic::types::{Type, Value};

mod utils;

#[test]
fn state_init() {
    let st =
        State::<CustomExpression<CustomExpressionInstruction>, CustomExpressionInstruction>::default();
    // For grcov
    let _ = format!("{st:?}");
    assert!(st.global.types.is_empty());
    assert!(st.global.constants.is_empty());
    assert!(st.global.functions.is_empty());
    assert_eq!(st.global.context, SemanticStack::new());
    assert!(st.blocks.is_empty());
    assert!(st.errors.is_empty());
}

#[test]
fn state_block_state_count() {
    let bst1 = Handle::new(BlockState::new(None));
    let bst2 = Handle::new(BlockState::new(Some(bst1.clone())));
    let mut st1 =
        State::<CustomExpression<CustomExpressionInstruction>, CustomExpressionInstruction>::default();
    st1.blocks.push(bst1);
    st1.blocks.push(bst2);
    assert_eq!(st1.blocks.len(), 2);
    let fs = FunctionDecl::new(FunctionName::new("fn1"), vec![], Type::Bool, vec![]);
    st1.function_body(&fs);
    // Should contain error
    assert_eq!(st1.errors.len(), 1);
    assert_eq!(st1.blocks.len(), 3);
}

#[test]
fn block_state_fields() {
    let mut bst: BlockState<CustomExpressionInstruction, ()> = BlockState::new(None);
    //  assert!(bst.values.is_empty());
    //  assert!(bst.inner.inner_values_name.is_empty());
    //  assert!(bst.inner.labels.is_empty());
    assert!(!bst.inner.manual_return);
    //  assert!(bst.parent.is_none());
    //  assert!(bst.children.is_empty());
    assert_eq!(bst.stack(), SemanticStack::new());
    bst.add_child(Handle::new(BlockState::new(None)));
    assert_eq!(bst.children.len(), 1);

    let bst1 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(None));
    let bst2 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(Some(
        bst1.clone(),
    )));
    bst1.add_child(bst1.clone());
    bst2.set_return();
    assert!(bst1.borrow().inner.manual_return);
    assert!(bst2.borrow().inner.manual_return);
}

#[test]
fn inner_value_name_transform() {
    let value_name: ValueName = "x1".into();
    let inner_value_name1: InnerValueName = value_name.into();
    assert_eq!(inner_value_name1.to_string(), "x1");
    let inner_value_name2: InnerValueName = "x2".into();
    assert_eq!(inner_value_name2.to_string(), "x2");
    let inner_value_name3: InnerValueName = String::from("x3").into();
    assert_eq!(inner_value_name3.to_string(), "x3");
}

#[test]
fn block_state_inner_value_name() {
    let bst1 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(None));
    let bst2 = Handle::new(BlockState::new(Some(bst1.clone())));
    bst1.add_child(bst1.clone());
    assert!(bst2.borrow().parent.is_some());
    assert_eq!(bst1.borrow().children.len(), 1);

    let inner_value_name1: InnerValueName = "x1".into();
    let inner_value_name2: InnerValueName = "x2".into();
    let inner_value_name3: InnerValueName = "x3".into();

    bst1.set_inner_value_name(&inner_value_name1);
    bst2.set_inner_value_name(&inner_value_name2);

    assert_eq!(bst1.borrow().inner.inner_values_name.len(), 2);
    assert!(bst1.borrow().is_inner_value_name_exist(&inner_value_name1));
    assert!(bst1.borrow().is_inner_value_name_exist(&inner_value_name2));
    assert!(!bst1.borrow().is_inner_value_name_exist(&inner_value_name3));

    assert_eq!(bst2.borrow().inner.inner_values_name.len(), 1);
    assert!(bst2.borrow().is_inner_value_name_exist(&inner_value_name1));
    assert!(bst2.borrow().is_inner_value_name_exist(&inner_value_name2));
    assert!(!bst2.borrow().is_inner_value_name_exist(&inner_value_name3));

    assert_eq!(
        bst1.borrow()
            .get_next_inner_name(&inner_value_name1)
            .to_string(),
        "x1.0"
    );
    assert_eq!(
        bst2.borrow()
            .get_next_inner_name(&inner_value_name1)
            .to_string(),
        "x1.0"
    );
    assert_eq!(
        bst1.borrow()
            .get_next_inner_name(&inner_value_name3)
            .to_string(),
        "x3.0"
    );
    let inner_value_name3_0: InnerValueName = "x3.0".into();
    assert_eq!(
        bst1.borrow()
            .get_next_inner_name(&inner_value_name3_0)
            .to_string(),
        "x3.1"
    );

    let inner_value_name2_0 = "x2.0".into();
    bst2.set_inner_value_name(&inner_value_name2_0);
    assert!(
        bst2.borrow()
            .is_inner_value_name_exist(&inner_value_name2_0)
    );
    assert_eq!(
        bst1.borrow()
            .get_next_inner_name(&inner_value_name2)
            .to_string(),
        "x2.1"
    );
    assert_eq!(
        bst2.borrow()
            .get_next_inner_name(&inner_value_name2)
            .to_string(),
        "x2.1"
    );
}

#[test]
fn block_state_label_name() {
    let bst1 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(None));
    let bst2 = Handle::new(BlockState::new(Some(bst1.clone())));
    bst1.add_child(bst1.clone());

    let lbl1 = LabelName::new("lbl1");
    let lbl2 = LabelName::new("lbl2");
    assert_eq!(lbl1.to_string(), "lbl1");
    bst1.set_label_name(&lbl1);
    bst2.set_label_name(&lbl2);
    assert!(bst1.borrow().is_label_name_exist(&lbl1));
    assert!(bst2.borrow().is_label_name_exist(&lbl1));
    assert!(bst1.borrow().is_label_name_exist(&lbl2));
    assert!(bst2.borrow().is_label_name_exist(&lbl2));

    let lbl3 = bst1.get_and_set_next_label(&String::from("lbl3").into());
    assert_eq!(lbl3.to_string(), "lbl3");

    let lbl1_0 = bst1.get_and_set_next_label(&lbl1);
    assert_eq!(lbl1_0.to_string(), "lbl1.0");

    let lbl1_1 = bst2.get_and_set_next_label(&lbl1);
    assert_eq!(lbl1_1.to_string(), "lbl1.1");
    let lbl1_2 = bst2.get_and_set_next_label(&lbl1_1);
    assert_eq!(lbl1_2.to_string(), "lbl1.2");
    let lbl1_3 = bst2.get_and_set_next_label(&lbl1);
    assert_eq!(lbl1_3.to_string(), "lbl1.3");
}

#[test]
fn block_state_value() {
    let bst1 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(None));
    let bst2 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(Some(
        bst1.clone(),
    )));
    let bst3 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(None));
    bst1.add_child(bst1.clone());

    // Insert Value
    let vn1 = ValueName::new("x5");
    let val = Value {
        inner_name: "x5.0".into(),
        inner_type: Type::Bool,
        mutable: false,
        alloca: false,
        malloc: false,
    };
    bst1.insert_value(vn1.clone(), val.clone());
    assert_eq!(bst1.borrow().get_value_name(&vn1).unwrap(), val.clone());
    assert_eq!(bst2.borrow().get_value_name(&vn1).unwrap(), val);
    assert!(bst3.borrow().get_value_name(&vn1).is_none());

    assert_eq!(bst1.borrow().register(), 0);
    assert_eq!(bst2.borrow().register(), 0);
    bst2.inc_register();
    assert_eq!(bst1.borrow().register(), 1);
    assert_eq!(bst2.borrow().register(), 1);
}

#[test]
fn block_state_last_register_inc() {
    let bst1 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(None));
    let bst2 = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(Some(
        bst1.clone(),
    )));
    let mut bst3: BlockState<CustomExpressionInstruction, ()> = BlockState::new(None);

    assert_eq!(bst1.borrow().register(), 0);
    assert_eq!(bst2.borrow().register(), 0);
    bst2.inc_register();
    assert_eq!(bst1.borrow().register(), 1);
    assert_eq!(bst2.borrow().register(), 1);

    assert_eq!(bst3.register(), 0);
    bst3.inc_register();
    assert_eq!(bst3.register(), 1);
    // For grcov
    let _ = format!("{bst3:?}");
}

#[test]
fn block_state_instructions_with_parent() {
    let parent_bst = Handle::new(BlockState::<CustomExpressionInstruction, ()>::new(None));
    let mut bst = BlockState::new(Some(parent_bst.clone()));
    let val = Value {
        inner_name: String::from("x").into(),
        inner_type: Type::Ptr,
        mutable: false,
        alloca: false,
        malloc: false,
    };
    bst.expression_value(val.clone(), 1);
    let expr_const: Constant = Constant {
        name: String::from("cnt1").into(),
        ty: Type::Ptr,
        value: ConstantExpression {
            value: ConstantValue::Constant(String::from("cnt2").into()),
            operation: None,
        },
    };
    bst.expression_const(expr_const, 1);
    bst.expression_struct_value(val.clone(), 1, 1);
    let expr_op = ExpressionOperations::Plus;
    let expr_res = ExprResult::Register(Type::Ptr, 1);
    bst.expression_operation(expr_op, expr_res.clone(), expr_res.clone(), 1);
    let call_fn = FunctionHeader {
        name: String::from("fn1").into(),
        result: Type::Ptr,
        args: vec![],
    };
    bst.call(call_fn, vec![], 1);
    bst.let_binding(val.clone(), expr_res.clone());
    bst.binding(val.clone(), expr_res.clone());
    bst.expression_function_return(expr_res.clone());
    bst.expression_function_return_with_label(expr_res.clone());

    let label: LabelName = String::from("label").into();
    bst.set_label(label.clone());
    bst.jump_to(label.clone());
    bst.if_condition_expression(expr_res.clone(), label.clone(), label.clone());

    let condition = Condition::Gt;
    bst.condition_expression(expr_res.clone(), expr_res.clone(), condition, 1);
    bst.jump_function_return(expr_res);

    let logic_condition = Logic::And;
    bst.logic_condition(logic_condition, 1, 2, 3);
    bst.if_condition_logic(label.clone(), label, 1);
    bst.function_arg(val, String::from("x"), Type::Ptr);

    let custom_instr = CustomExpressionInstruction;
    bst.extended_expression(&custom_instr);

    let parent_ctx = parent_bst.borrow().stack().get();
    assert_eq!(parent_ctx.len(), 18);
}
