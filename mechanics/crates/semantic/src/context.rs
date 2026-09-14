//! # Semantic types
//! Semantic analyzer result state types.
//! It contains `SemanticStack` as Semantic results Context data.

use crate::block_state::BlockState;
use crate::constants::Constant;
use crate::expression::{ExprResult, ExpressionOperations};
use crate::function::{FunctionDecl, FunctionHeader};
use crate::handle::Handle;
use crate::names::LabelName;
use crate::semantic::State;
use crate::stmt::{Condition, Logic};
use crate::types::{StructTypes, Type, Value};
use std::fmt::Debug;

/// Semantic Context trait contain instructions set functions
/// for Global Stack context. It includes:
/// - functions
/// - types
/// - constants
pub trait GlobalSemanticContext<E> {
    fn function_declaration(&mut self, fn_decl: FunctionDecl<E>);
    fn constant(&mut self, const_decl: Constant);
    fn types(&mut self, type_decl: StructTypes);
}

/// Semantic Context trait contain instructions set functions
/// for the Stack context.
pub trait SemanticContext {
    fn expression_value(&mut self, expression: Value, register: u64);
    fn expression_const(&mut self, expression: Constant, register: u64);
    fn expression_struct_value(&mut self, expression: Value, index: u32, register: u64);
    fn expression_operation(
        &mut self,
        operation: ExpressionOperations,
        left_value: ExprResult,
        right_value: ExprResult,
        register: u64,
    );
    fn call(&mut self, call: FunctionHeader, params: Vec<ExprResult>, register: u64);
    fn let_binding(&mut self, let_decl: Value, expr_result: ExprResult);
    fn binding(&mut self, val: Value, expr_result: ExprResult);
    fn expression_function_return(&mut self, expr_result: ExprResult);
    fn expression_function_return_with_label(&mut self, expr_result: ExprResult);
    fn set_label(&mut self, label: LabelName);
    fn jump_to(&mut self, label: LabelName);
    fn if_condition_expression(
        &mut self,
        expr_result: ExprResult,
        label_if_begin: LabelName,
        label_if_end: LabelName,
    );
    fn condition_expression(
        &mut self,
        left_result: ExprResult,
        right_result: ExprResult,
        condition: Condition,
        register: u64,
    );
    fn jump_function_return(&mut self, expr_result: ExprResult);
    fn logic_condition(
        &mut self,
        logic_condition: Logic,
        left_register_result: u64,
        right_register_result: u64,
        register: u64,
    );
    fn if_condition_logic(
        &mut self,
        label_if_begin: LabelName,
        label_if_end: LabelName,
        result_register: u64,
    );
    fn function_arg(&mut self, value: Value, name: String, ty: Type);
}

/// Extended Semantic Context trait contain instructions set functions
/// for the Extended Stack context.
pub trait ExtendedSemanticContext<I: SemanticContextInstruction> {
    fn extended_expression(&mut self, expr: &I);
}

/// Semantic Context trait contains custom instruction implementation
/// to flexibly extend context instructions. It represents ivs derided
/// traits: `Debug` and `Serialize` + `Deserialize`
pub trait SemanticContextInstruction: Debug + Clone + PartialEq {}

/// Extended Expression for semantic analyzer.
pub trait ExtendedExpression<I: SemanticContextInstruction>: Debug + Clone + PartialEq {
    /// Custom expression. Ast should be received from `GetAst` trait.
    fn expression(
        &self,
        state: &mut State<Self, I>,
        block_state: &Handle<BlockState<I, Self>>,
    ) -> ExprResult;
}

/// # Semantic stack
/// Semantic stack represent stack of Semantic Context results
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticStack<I, E>(Vec<SemanticStackContext<I, E>>);

impl<I, E> Default for SemanticStack<I, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, E> SemanticStack<I, E> {
    /// Init Semantic stack
    #[must_use]
    pub const fn new() -> Self {
        Self(vec![])
    }

    /// Push Context data to the stack
    fn push(&mut self, value: SemanticStackContext<I, E>) {
        self.0.push(value);
    }

    /// Get all context stack data as array data
    #[must_use]
    pub fn get(self) -> Vec<SemanticStackContext<I, E>> {
        self.0
    }
}

impl<I: SemanticContextInstruction, E> GlobalSemanticContext<E> for SemanticStack<I, E> {
    /// Push Context to the stack as function declaration data.
    /// Function declaration instruction.
    ///
    /// ## Parameters
    /// - `fn_decl` - function declaration parameters
    fn function_declaration(&mut self, fn_decl: FunctionDecl<E>) {
        self.push(SemanticStackContext::FunctionDeclaration { fn_decl });
    }

    /// Push Context to the stack as constant data.
    /// Constant declaration instruction.
    ///
    /// ## Parameters
    /// - `const_decl` - constant declaration parameters
    fn constant(&mut self, const_decl: Constant) {
        self.push(SemanticStackContext::Constant { const_decl });
    }

    /// Push Context to the stack as types data.
    /// Types declaration instruction.
    ///
    /// ## Parameters
    /// - `type_decl` - type declaration parameters
    fn types(&mut self, type_decl: StructTypes) {
        self.push(SemanticStackContext::Types { type_decl });
    }
}

impl<I: SemanticContextInstruction, E> SemanticContext for SemanticStack<I, E> {
    /// Push Context to the stack as expression value data.
    ///
    /// ## Parameters
    /// - `expression` - contains expression value
    /// - `register` - register to store result data
    fn expression_value(&mut self, expression: Value, register: u64) {
        self.push(SemanticStackContext::ExpressionValue {
            expression,
            register,
        });
    }

    /// Push Context to the stack as expression const data.
    ///
    /// ## Parameters
    /// - `expression` - contains expression constant
    /// - `register` - register to store result data
    fn expression_const(&mut self, expression: Constant, register: u64) {
        self.push(SemanticStackContext::ExpressionConst {
            expression,
            register,
        });
    }

    /// Push Context to the stack as expression struct value data.
    ///
    /// ## Parameters
    /// - `expression` - contains expression value for specific `Structure` attribute
    /// - `index` - represent attribute index in the `Structure` type
    /// - `register` - register to store result data
    fn expression_struct_value(&mut self, expression: Value, index: u32, register: u64) {
        self.push(SemanticStackContext::ExpressionStructValue {
            expression,
            index,
            register,
        });
    }

    /// Push Context to the stack as expression operation data.
    /// `expression_operation` imply operation between `left_value` and
    /// `right_value` and store result to `register`.
    ///
    /// ## Parameters
    /// - `operation` - specific operation
    /// - `left_value` - left expression result
    /// - `right_value` - right expression result
    /// - `register` - register to store result of expression operation
    fn expression_operation(
        &mut self,
        operation: ExpressionOperations,
        left_value: ExprResult,
        right_value: ExprResult,
        register: u64,
    ) {
        self.push(SemanticStackContext::ExpressionOperation {
            operation,
            left_value,
            right_value,
            register,
        });
    }

    /// Push Context to the stack as function call data.
    /// Function call instruction with parameters and result data.
    ///
    /// ## Parameters
    /// - `call` - function declaration data
    /// - `params` - function parameters
    ///  - `register` - register to store result of function call
    fn call(&mut self, call: FunctionHeader, params: Vec<ExprResult>, register: u64) {
        self.push(SemanticStackContext::Call {
            call,
            params,
            register,
        });
    }

    /// Push Context to the stack as let-binding data.
    /// Let binding instruction that "bind" expression result to
    /// the new value.
    ///
    /// ## Parameters
    /// - `let_decl` - value declaration
    /// -  `expr_result` - expression result that will be bind to the value
    fn let_binding(&mut self, let_decl: Value, expr_result: ExprResult) {
        self.push(SemanticStackContext::LetBinding {
            let_decl,
            expr_result,
        });
    }

    /// Push Context to the stack as binding data.
    /// Binding instruction that "bind" expression result to
    /// the old. previously init value.
    ///
    /// ## Parameters
    /// - `val` - value declaration
    /// -  `expr_result` - expression result that will be bind to the value
    fn binding(&mut self, val: Value, expr_result: ExprResult) {
        self.push(SemanticStackContext::Binding { val, expr_result });
    }

    /// Push Context to the stack as expression function return data.
    /// Return instruction, should be used in the end of functions.
    /// Alwats should be only once.
    ///
    /// ## Parameters
    /// - `expr_result` - result data for the return
    fn expression_function_return(&mut self, expr_result: ExprResult) {
        self.push(SemanticStackContext::ExpressionFunctionReturn { expr_result });
    }

    /// Push Context to the stack as `expression function return with label` data.
    /// Return instruction with additional logic. Most useful case when
    /// `return` previously was call from `if-body` or `loop-body.`.
    /// As additional behavior this `expression_function_return_with_label` should
    /// set `return` label. It will allow `jump-to-return` case. Also
    /// before `return` label Codegen, for normal instruction flow, must
    /// jump to `return` label anyway.
    ///
    /// ## Parameters
    /// - `expr_result` - result data for the return
    fn expression_function_return_with_label(&mut self, expr_result: ExprResult) {
        self.push(SemanticStackContext::ExpressionFunctionReturnWithLabel { expr_result });
    }

    /// Push Context to the stack as `set label` data.
    /// Set label. Useful for any kind of jump operations and conditional flow.
    ///
    /// ## Parameters
    /// - `label` - label name
    fn set_label(&mut self, label: LabelName) {
        self.push(SemanticStackContext::SetLabel { label });
    }

    /// Push Context to the stack as `jump to` data.
    /// Unconditional direct jump to label.
    ///
    /// ## Parameters
    /// - `label` - label for the jump
    fn jump_to(&mut self, label: LabelName) {
        self.push(SemanticStackContext::JumpTo { label });
    }

    /// Push Context to the stack as `if condition expression` data.
    /// `if-condition expression` represent if-condition, when if expression
    /// is "true" jump to `label_if_begin` else `label_if_end`.
    ///
    /// ## Parameters
    /// - `expr_result` - expression result of `if-condition` for
    /// conditional instruction
    /// - `label_if_begin` - label for jump if expression is "true"
    /// - `label_if_end` - label for jump if expression is "false"
    fn if_condition_expression(
        &mut self,
        expr_result: ExprResult,
        label_if_begin: LabelName,
        label_if_end: LabelName,
    ) {
        self.push(SemanticStackContext::IfConditionExpression {
            expr_result,
            label_if_begin,
            label_if_end,
        });
    }

    /// Push Context to the stack as `condition expression` data.
    /// Condition expression between left and right condition calculation.
    ///
    /// ## Parameters
    /// - `left_result` - left expression result
    /// - `right_result` - right expression result
    /// - `condition` - condition operation
    /// - `register` - register to store result of expression operation
    fn condition_expression(
        &mut self,
        left_result: ExprResult,
        right_result: ExprResult,
        condition: Condition,
        register: u64,
    ) {
        self.push(SemanticStackContext::ConditionExpression {
            left_result,
            right_result,
            condition,
            register,
        });
    }

    /// Push Context to the stack as `jump function return` data.
    /// Jump to function return with expression result data. Label for jumping
    /// to return position (always end of function) should be always the same
    /// and should be managed by Codegen.
    ///
    /// ## Parameters
    /// - `expr_result` - expression result for return condition
    fn jump_function_return(&mut self, expr_result: ExprResult) {
        self.push(SemanticStackContext::JumpFunctionReturn { expr_result });
    }

    /// Push Context to the stack as `logic condition` data.
    /// Operate with registers: left and right for specific logic condition.
    /// Result of calculation stored to `register`.
    ///
    /// ## Parameters
    /// - `left_register_result` - result of left condition
    /// - `right_register_result` - result of right condition
    /// - `register` - register to store instruction result
    fn logic_condition(
        &mut self,
        logic_condition: Logic,
        left_register_result: u64,
        right_register_result: u64,
        register: u64,
    ) {
        self.push(SemanticStackContext::LogicCondition {
            logic_condition,
            left_register_result,
            right_register_result,
            register,
        });
    }

    /// Push Context to the stack as `if condition logic` data.
    /// `if_condition_logic` instruction read data from `result_register`
    /// and conditionally jump: if "true' to `label_if_begin` or
    /// `label_if_end` if "false" (data contained as result after
    /// reading `result_register`).
    ///
    /// ## Parameters
    /// - `label_if_begin` - label for a jump if `result_register` contains
    /// result with "true"
    /// - `label_if_end` - label for a jump if `result_register` contains
    ///  result with "false". It can be not only `if_end` but any kind (for
    /// example `if_else`)
    /// - `result_register` - contains register of previous condition logic
    /// calculations.
    fn if_condition_logic(
        &mut self,
        label_if_begin: LabelName,
        label_if_end: LabelName,
        result_register: u64,
    ) {
        self.push(SemanticStackContext::IfConditionLogic {
            label_if_begin,
            label_if_end,
            result_register,
        });
    }

    /// Push Context to the stack as `function argument` data.
    /// This instruction should allocate pointer (if argument type is
    /// not Ptr) and store argument value to the pointer.
    ///
    /// ## Parameters
    /// - `func_arg` - function parameter data
    fn function_arg(&mut self, value: Value, name: String, ty: Type) {
        self.push(SemanticStackContext::FunctionArg { value, name, ty });
    }
}

impl<I: SemanticContextInstruction, E> ExtendedSemanticContext<I> for SemanticStack<I, E> {
    /// Extended Expression instruction.
    /// AS argument trait, that contains instruction method that returns
    /// instruction parameters.
    fn extended_expression(&mut self, expr: &I) {
        self.push(SemanticStackContext::ExtendedExpression(Box::new(
            expr.clone(),
        )));
    }
}

/// # Semantic stack Context
/// Context data of Semantic results. Contains type declarations
/// for specific instructions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum SemanticStackContext<I, E> {
    ExpressionValue {
        expression: Value,
        register: u64,
    },
    ExpressionConst {
        expression: Constant,
        register: u64,
    },
    ExpressionStructValue {
        expression: Value,
        index: u32,
        register: u64,
    },
    ExpressionOperation {
        operation: ExpressionOperations,
        left_value: ExprResult,
        right_value: ExprResult,
        register: u64,
    },
    Call {
        call: FunctionHeader,
        params: Vec<ExprResult>,
        register: u64,
    },
    LetBinding {
        let_decl: Value,
        expr_result: ExprResult,
    },
    Binding {
        val: Value,
        expr_result: ExprResult,
    },

    FunctionDeclaration {
        fn_decl: FunctionDecl<E>,
    },
    Constant {
        const_decl: Constant,
    },
    Types {
        type_decl: StructTypes,
    },

    ExpressionFunctionReturn {
        expr_result: ExprResult,
    },
    ExpressionFunctionReturnWithLabel {
        expr_result: ExprResult,
    },
    SetLabel {
        label: LabelName,
    },
    JumpTo {
        label: LabelName,
    },
    IfConditionExpression {
        expr_result: ExprResult,
        label_if_begin: LabelName,
        label_if_end: LabelName,
    },
    ConditionExpression {
        left_result: ExprResult,
        right_result: ExprResult,
        condition: Condition,
        register: u64,
    },
    JumpFunctionReturn {
        expr_result: ExprResult,
    },
    LogicCondition {
        logic_condition: Logic,
        left_register_result: u64,
        right_register_result: u64,
        register: u64,
    },
    IfConditionLogic {
        label_if_begin: LabelName,
        label_if_end: LabelName,
        result_register: u64,
    },
    FunctionArg {
        value: Value,
        name: String,
        ty: Type,
    },

    ExtendedExpression(Box<I>),
}
