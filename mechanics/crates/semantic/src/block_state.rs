//! # Block State types
//! Block state Semantic types.

use crate::constants::Constant;
use crate::context::{
    ExtendedSemanticContext, SemanticContext, SemanticContextInstruction, SemanticStack,
};
use crate::expression::{ExprResult, ExpressionOperations};
use crate::function::FunctionHeader;
use crate::handle::Handle;
use crate::names::{InnerValueName, LabelName, ValueName};
use crate::stmt::{Condition, Logic};
use crate::types::{Type, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockStateInner {
    /// Used to keep all names in the block state (and parent) as unique
    pub inner_values_name: HashSet<InnerValueName>,
    /// State labels for conditional operations
    labels: HashSet<LabelName>,
    /// Last register for unique register representation
    last_register_number: u64,
    /// Manual return from other states
    pub manual_return: bool,
}

/// # Block state
/// - `values` - contains unique values map for current state but not unique
///   for parent states. The map contains key-value: `value_name` (unique
///   only for current state); and `Value` itself - value parameters.
/// - `inner_values_name` - is entity that represent inner value name - it
///   can be different from `Value` name because it should be unique for all
///   parent states. For example, of 3 values with name `x`, inner value
///   name will be: [`x`, `x.0`, `x.1`]. It mean, inner value name can
///   contain `value counter` as end of the name.
/// - `labels` - labels set, for conditional operation. Unique for current
///   and all paren states.
/// - `last_register_number` - represent register counter for current and
///   all parent states for `Codegen`. Register represented as `u64` and
///   should be linearly incremented.
/// - `manual_return` - flag indicated, that return was invoked from
/// other state, for example: if-flow, loop-flow
/// - `parent` - represent parent states.
#[derive(Debug)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockState<I: SemanticContextInstruction, E> {
    /// State values
    pub values: HashMap<ValueName, Value>,

    pub inner: BlockStateInner,

    /// Parent state
    pub parent: Option<Handle<BlockState<I, E>>>,

    /// children states
    pub children: Vec<Handle<BlockState<I, E>>>,

    /// Semantic stack context for Block state
    stack: SemanticStack<I, E>,
}

impl<I: SemanticContextInstruction, E: Clone> BlockState<I, E> {
    /// Init block state with optional `parent` state
    #[must_use]
    pub fn new(parent: Option<Handle<Self>>) -> Self {
        // Get values from parent
        let inner = parent
            .as_ref()
            .map(|parent| parent.borrow().inner.clone())
            .unwrap_or_default();

        Self {
            values: HashMap::new(),
            children: vec![],
            inner,
            parent,
            stack: SemanticStack::new(),
        }
    }

    #[must_use]
    pub fn stack(&self) -> SemanticStack<I, E> {
        self.stack.clone()
    }

    /// Set `last_register_number` for current and parent states
    fn set_register(&mut self, last_register_number: u64) {
        self.inner.last_register_number = last_register_number;
        // Set `last_register_number` for parents
        if let Some(parent) = &self.parent {
            parent.borrow_mut().set_register(last_register_number);
        }
    }

    /// Increment register
    pub fn inc_register(&mut self) -> u64 {
        self.set_register(self.inner.last_register_number + 1);
        self.inner.last_register_number
    }

    /// Last register number
    #[must_use]
    pub const fn register(&self) -> u64 {
        self.inner.last_register_number
    }

    /// Get child Block state
    pub fn add_child(&mut self, child: Handle<Self>) {
        self.children.push(child);
    }

    /// Set value inner name to current state and parent states
    pub fn set_inner_value_name(&mut self, name: &InnerValueName) {
        self.inner.inner_values_name.insert(name.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().set_inner_value_name(name);
        }
    }

    /// Check is `inner_value_name` exist in current and parent states
    #[must_use]
    pub fn is_inner_value_name_exist(&self, name: &InnerValueName) -> bool {
        self.inner.inner_values_name.contains(name)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.borrow().is_inner_value_name_exist(name))
    }

    /// Get `Value` by value name from current state.
    /// If not found on current state - recursively find in parent states.
    #[must_use]
    pub fn get_value_name(&self, name: &ValueName) -> Option<Value> {
        self.values.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.borrow().get_value_name(name))
        })
    }

    /// Check is label name exist in current and parent states
    #[must_use]
    pub fn is_label_name_exist(&self, name: &LabelName) -> bool {
        self.inner.labels.contains(name)
            || self
                .parent
                .as_ref()
                .is_some_and(|parent| parent.borrow().is_label_name_exist(name))
    }

    /// Set label name to current and all parent states
    pub fn set_label_name(&mut self, name: &LabelName) {
        self.inner.labels.insert(name.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().set_label_name(name);
        }
    }

    /// Set attribute counter - increment, if counter exist.
    #[must_use]
    pub fn set_attr_counter(label: &str) -> String {
        let (label, counter) = if let Some((label, counter)) = label.split_once('.') {
            (label, 1 + counter.parse::<usize>().unwrap_or_default())
        } else {
            (label, 0)
        };

        format!("{label}.{counter:?}")
    }

    /// Get and set next label for condition operations
    /// - If label doesn't exist in State - just insert to State and
    ///   self return
    /// - if label exists, get label counter
    pub fn get_and_set_next_label(&mut self, label: &LabelName) -> LabelName {
        // Check is label exists. If doesn't set it to State and return self
        if self.is_label_name_exist(label) {
            // If label exists, split and get number of label counter
            let name = LabelName::from(Self::set_attr_counter(&label.to_string()));
            if self.is_label_name_exist(&name) {
                self.get_and_set_next_label(&name)
            } else {
                self.set_label_name(&name);
                name
            }
        } else {
            self.set_label_name(label);
            label.clone()
        }
    }

    /// Get next `inner_value_name` by name counter for current and
    /// parent states. The `inner_value_name` should always be unique.
    #[must_use]
    pub fn get_next_inner_name(&self, val: &InnerValueName) -> InnerValueName {
        // Increment inner value name counter for shadowed variable
        let name = InnerValueName::from(Self::set_attr_counter(&val.to_string()));
        if self.is_inner_value_name_exist(&name) {
            self.get_next_inner_name(&name)
        } else {
            name
        }
    }

    /// Set return status flag for current and parent states
    fn set_return(&mut self) {
        self.inner.manual_return = true;
        if let Some(parent) = &self.parent {
            parent.borrow_mut().set_return();
        }
    }
}

impl<I: SemanticContextInstruction, E: Clone> Handle<BlockState<I, E>> {
    /// Set return status flag for current and parent states
    pub fn set_return(&self) {
        self.borrow_mut().set_return();
    }

    pub fn set_inner_value_name(&self, name: &InnerValueName) {
        self.borrow_mut().set_inner_value_name(name);
    }

    #[must_use]
    pub fn inc_register(&self) -> u64 {
        self.borrow_mut().inc_register()
    }

    pub fn add_child(&self, child: Self) {
        self.borrow_mut().add_child(child);
    }

    #[must_use]
    pub fn get_and_set_next_label(&self, label: &LabelName) -> LabelName {
        self.borrow_mut().get_and_set_next_label(label)
    }

    pub fn set_label_name(&self, name: &LabelName) {
        self.borrow_mut().set_label_name(name);
    }

    #[must_use]
    pub fn get_value_name(&self, name: &ValueName) -> Option<Value> {
        self.borrow_mut().get_value_name(name)
    }

    pub fn insert_value(&self, name: impl Into<ValueName>, value: Value) {
        self.borrow_mut().values.insert(name.into(), value);
    }

    //

    pub fn expression_value(&self, expression: Value, register_number: u64) {
        self.borrow_mut()
            .expression_value(expression, register_number);
    }

    pub fn expression_const(&self, expression: Constant, register_number: u64) {
        self.borrow_mut()
            .expression_const(expression, register_number);
    }

    pub fn expression_struct_value(&self, expression: Value, index: u32, register_number: u64) {
        self.borrow_mut()
            .expression_struct_value(expression, index, register_number);
    }

    pub fn expression_operation(
        &self,
        operation: ExpressionOperations,
        left_value: ExprResult,
        right_value: ExprResult,
        register_number: u64,
    ) {
        self.borrow_mut()
            .expression_operation(operation, left_value, right_value, register_number);
    }

    pub fn call(&self, call: FunctionHeader, params: Vec<ExprResult>, register_number: u64) {
        self.borrow_mut().call(call, params, register_number);
    }

    pub fn let_binding(&self, let_decl: Value, expr_result: ExprResult) {
        self.borrow_mut().let_binding(let_decl, expr_result);
    }

    pub fn binding(&self, val: Value, expr_result: ExprResult) {
        self.borrow_mut().binding(val, expr_result);
    }

    pub fn expression_function_return(&self, expr_result: ExprResult) {
        self.borrow_mut().expression_function_return(expr_result);
    }

    pub fn expression_function_return_with_label(&self, expr_result: ExprResult) {
        self.borrow_mut()
            .expression_function_return_with_label(expr_result);
    }

    pub fn set_label(&self, label: LabelName) {
        self.borrow_mut().set_label(label);
    }

    pub fn jump_to(&self, label: LabelName) {
        self.borrow_mut().jump_to(label);
    }

    pub fn if_condition_expression(
        &self,
        expr_result: ExprResult,
        label_if_begin: LabelName,
        label_if_end: LabelName,
    ) {
        self.borrow_mut()
            .if_condition_expression(expr_result, label_if_begin, label_if_end);
    }

    pub fn condition_expression(
        &self,
        left_result: ExprResult,
        right_result: ExprResult,
        condition: Condition,
        register_number: u64,
    ) {
        self.borrow_mut().condition_expression(
            left_result,
            right_result,
            condition,
            register_number,
        );
    }

    pub fn jump_function_return(&self, expr_result: ExprResult) {
        self.borrow_mut().jump_function_return(expr_result);
    }

    pub fn logic_condition(
        &self,
        logic_condition: Logic,
        left_register_result: u64,
        right_register_result: u64,
        register_number: u64,
    ) {
        self.borrow_mut().logic_condition(
            logic_condition,
            left_register_result,
            right_register_result,
            register_number,
        );
    }

    pub fn if_condition_logic(
        &self,
        label_if_begin: LabelName,
        label_if_end: LabelName,
        result_register: u64,
    ) {
        self.borrow_mut()
            .if_condition_logic(label_if_begin, label_if_end, result_register);
    }

    pub fn function_arg(&self, value: Value, name: String, ty: Type) {
        self.borrow_mut().function_arg(value, name, ty);
    }
}

impl<I: SemanticContextInstruction, E: Clone> SemanticContext for BlockState<I, E> {
    fn expression_value(&mut self, expression: Value, register_number: u64) {
        self.stack
            .expression_value(expression.clone(), register_number);
        if let Some(parent) = &self.parent {
            parent.expression_value(expression, register_number);
        }
    }

    fn expression_const(&mut self, expression: Constant, register_number: u64) {
        self.stack
            .expression_const(expression.clone(), register_number);
        if let Some(parent) = &self.parent {
            parent.expression_const(expression, register_number);
        }
    }

    fn expression_struct_value(&mut self, expression: Value, index: u32, register_number: u64) {
        self.stack
            .expression_struct_value(expression.clone(), index, register_number);
        if let Some(parent) = &self.parent {
            parent.expression_struct_value(expression, index, register_number);
        }
    }

    fn expression_operation(
        &mut self,
        operation: ExpressionOperations,
        left_value: ExprResult,
        right_value: ExprResult,
        register_number: u64,
    ) {
        self.stack.expression_operation(
            operation.clone(),
            left_value.clone(),
            right_value.clone(),
            register_number,
        );
        if let Some(parent) = &self.parent {
            parent.borrow_mut().expression_operation(
                operation,
                left_value,
                right_value,
                register_number,
            );
        }
    }

    fn call(&mut self, call: FunctionHeader, params: Vec<ExprResult>, register_number: u64) {
        self.stack
            .call(call.clone(), params.clone(), register_number);
        if let Some(parent) = &self.parent {
            parent.borrow_mut().call(call, params, register_number);
        }
    }

    fn let_binding(&mut self, let_decl: Value, expr_result: ExprResult) {
        self.stack
            .let_binding(let_decl.clone(), expr_result.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().let_binding(let_decl, expr_result);
        }
    }

    fn binding(&mut self, val: Value, expr_result: ExprResult) {
        self.stack.binding(val.clone(), expr_result.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().binding(val, expr_result);
        }
    }

    fn expression_function_return(&mut self, expr_result: ExprResult) {
        self.stack.expression_function_return(expr_result.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().expression_function_return(expr_result);
        }
    }

    fn expression_function_return_with_label(&mut self, expr_result: ExprResult) {
        self.stack
            .expression_function_return_with_label(expr_result.clone());
        if let Some(parent) = &self.parent {
            parent
                .borrow_mut()
                .expression_function_return_with_label(expr_result);
        }
    }

    fn set_label(&mut self, label: LabelName) {
        self.stack.set_label(label.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().set_label(label);
        }
    }

    fn jump_to(&mut self, label: LabelName) {
        self.stack.jump_to(label.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().jump_to(label);
        }
    }

    fn if_condition_expression(
        &mut self,
        expr_result: ExprResult,
        label_if_begin: LabelName,
        label_if_end: LabelName,
    ) {
        self.stack.if_condition_expression(
            expr_result.clone(),
            label_if_begin.clone(),
            label_if_end.clone(),
        );
        if let Some(parent) = &self.parent {
            parent
                .borrow_mut()
                .if_condition_expression(expr_result, label_if_begin, label_if_end);
        }
    }

    fn condition_expression(
        &mut self,
        left_result: ExprResult,
        right_result: ExprResult,
        condition: Condition,
        register_number: u64,
    ) {
        self.stack.condition_expression(
            left_result.clone(),
            right_result.clone(),
            condition.clone(),
            register_number,
        );
        if let Some(parent) = &self.parent {
            parent.borrow_mut().condition_expression(
                left_result,
                right_result,
                condition,
                register_number,
            );
        }
    }

    fn jump_function_return(&mut self, expr_result: ExprResult) {
        self.stack.jump_function_return(expr_result.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().jump_function_return(expr_result);
        }
    }

    fn logic_condition(
        &mut self,
        logic_condition: Logic,
        left_register_result: u64,
        right_register_result: u64,
        register_number: u64,
    ) {
        self.stack.logic_condition(
            logic_condition.clone(),
            left_register_result,
            right_register_result,
            register_number,
        );
        if let Some(parent) = &self.parent {
            parent.borrow_mut().logic_condition(
                logic_condition,
                left_register_result,
                right_register_result,
                register_number,
            );
        }
    }

    fn if_condition_logic(
        &mut self,
        label_if_begin: LabelName,
        label_if_end: LabelName,
        result_register: u64,
    ) {
        self.stack.if_condition_logic(
            label_if_begin.clone(),
            label_if_end.clone(),
            result_register,
        );
        if let Some(parent) = &self.parent {
            parent
                .borrow_mut()
                .if_condition_logic(label_if_begin, label_if_end, result_register);
        }
    }

    fn function_arg(&mut self, value: Value, name: String, ty: Type) {
        self.stack
            .function_arg(value.clone(), name.clone(), ty.clone());
        if let Some(parent) = &self.parent {
            parent.borrow_mut().function_arg(value, name, ty);
        }
    }
}

impl<I: SemanticContextInstruction, E> ExtendedSemanticContext<I> for BlockState<I, E> {
    fn extended_expression(&mut self, expr: &I) {
        self.stack.extended_expression(expr);
        if let Some(parent) = &self.parent {
            parent.borrow_mut().extended_expression(expr);
        }
    }
}
