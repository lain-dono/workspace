//! # Semantic analyzer
//! Semantic analyzer provides algorithms to analyze AST for different
//! rules and generate `Semantic State stack` stack results. AST represent tree
//! nodes of language constructions and fully cover all flow of the program
//! represented through AST. And it's **Turing-complete**.
//!
//! ## Semantic State
//! Semantic State contains basic entities:
//! - `Global State` - global state of semantic analyzer results.
//! - `Context` - stack for `Block state` of each functions body state.
//! - `Errors` - semantic analyzes errors.z

use crate::block_state::BlockState;
use crate::constants::{Constant, ConstantExpression, ConstantValue};
use crate::context::{
    ExtendedExpression, GlobalSemanticContext, SemanticContextInstruction, SemanticStack,
};
use crate::error::{StateErrorKind, StateErrorResult};
use crate::expression::{
    ExprResult, ExprValue, Expression, ExpressionOperations, MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS,
};
use crate::function::{FunctionCall, FunctionDecl, FunctionHeader};
use crate::handle::Handle;
use crate::names::{ConstantName, FunctionName, InnerValueName, LabelName, TypeName};
use crate::stmt::{
    BodyStatement, ExpressionLogicCondition, IfBodyStatement, IfBodyStatements, IfCondition,
    IfLoopBodyStatement, IfStatement, LoopBodyStatement, Stmt,
};
use crate::types::{StructTypes, Type, Value};
use crate::{ImportPath, Main, MainStatement};
use std::collections::HashMap;

/// # Global State
/// Global state can contains state declarations of:
/// - Constants
/// - Types
/// - Functions
/// And Semantic State context results for Global State context:
/// - Context
/// The visibility of Global state limited by current module.
/// `Context` contains results of `Semantic` stack, as result of
/// Semantic analyzer for Global State context. It's can be used for
/// post-verification process, linting, Codegen.
#[derive(Debug)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct GlobalState<I, E> {
    /// Constants declarations
    pub constants: HashMap<ConstantName, Constant>,
    /// Types declarations
    pub types: HashMap<TypeName, Type>,
    /// Functions declarations
    pub functions: HashMap<FunctionName, FunctionHeader>,
    /// Context as Semantic Stack Context results contains basic semantic
    /// result tree for Global context state.
    pub context: SemanticStack<I, E>,
}

impl<I, E> Default for GlobalState<I, E> {
    fn default() -> Self {
        Self {
            functions: HashMap::new(),
            types: HashMap::new(),
            constants: HashMap::new(),
            context: SemanticStack::new(),
        }
    }
}

/// # State
/// Basic entity that contains:
/// - `Global State` - types, constants, functions declaration and
///   most important - context results of Semantic State stack, that can be
///   used for post-verification and/or Codegen.
/// - `Context` stack for `Block state` of each functions body state
/// - `Error State` contains errors stack as result of Semantic analyzer
#[derive(Debug)]
#[cfg_attr(feature = "codec", derive(serde::Serialize))]
pub struct State<E, I: SemanticContextInstruction> {
    /// Global State for current State
    pub global: GlobalState<I, E>,

    /// Context for all `Block State` stack that related to concrete functions body.
    #[cfg_attr(feature = "codec", serde(skip))]
    pub blocks: Vec<Handle<BlockState<I, E>>>,

    /// Error state results stack
    pub errors: Vec<StateErrorResult>,
}

impl<E, I: SemanticContextInstruction> Default for State<E, I> {
    fn default() -> Self {
        Self {
            global: GlobalState::default(),
            blocks: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl<E, I> State<E, I>
where
    E: ExtendedExpression<I> + std::fmt::Display,
    I: SemanticContextInstruction,
{
    /// Add error to Semantic `Errors State`
    fn error(&mut self, kind: StateErrorKind, value: impl ToString) {
        let value = value.to_string();
        self.errors.push(StateErrorResult { kind, value });
    }

    /// Check is value type exists in `Global State`.
    /// `Primitive` type always return true. For other cases if type doesn't
    /// exist in `Global State`, add errors to `Error State` and return `false` result.
    fn check_type_exists(&mut self, type_name: &Type, val_name: impl ToString) -> bool {
        if type_name.is_primitive() {
            return true;
        }
        if self.global.types.contains_key(&type_name.name()) {
            true
        } else {
            self.error(StateErrorKind::TypeNotFound, val_name);
            false
        }
    }

    /// Run semantic analyzer that covers all flow for AST.
    /// It's do not return any results, but fill results fir the `Semantic State`.
    ///
    pub fn run(&mut self, data: &Main<E>) {
        // Execute each kind of analyzing and return errors data.
        // For functions - fetch only declaration for fast-forward
        // identification for using it in functions body.

        // First pass is Imports and Types
        for main in data {
            match main {
                MainStatement::Import(import) => self.import(import),
                MainStatement::Types(types) => self.types(types),
                _ => (),
            }
        }
        // Declaration pass for Constants and Functions
        for main in data {
            match main {
                MainStatement::Constant(constant) => self.constant(constant),
                MainStatement::Function(function) => self.function_declaration(function),
                _ => (),
            }
        }

        // After getting all functions declarations, fetch only functions body
        for main in data {
            if let MainStatement::Function(function) = main {
                self.function_body(function);
            }
        }
    }

    /// Import analyzer (TBD)
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    pub fn import(&self, data: &ImportPath) {
        if !data.is_empty() {
            let _name = &data[0];
        }
    }

    /// Types declaration analyzer. Add types to `Global State`.
    /// Currently only one type kind: Structs. And types can't be part of
    /// the `Block State`.
    pub fn types(&mut self, data: &StructTypes) {
        if self.global.types.contains_key(&data.name) {
            self.error(StateErrorKind::TypeAlreadyExist, &data.name);
            return;
        }
        let struct_type = Type::Struct(data.clone());
        self.global.types.insert(struct_type.name(), struct_type);
        self.global.context.types(data.clone());
    }

    /// Constant analyzer. Add it to `Global State`, because constants
    /// can be only global for `Semantic state`, not for `Block state`.
    pub fn constant(&mut self, constant: &Constant) {
        if self.global.constants.contains_key(&constant.name) {
            self.error(StateErrorKind::ConstantAlreadyExist, &constant.name);
            return;
        }
        if !self.check_constant_value_expression(constant.value.operation.as_ref()) {
            return;
        }
        if !self.check_type_exists(&constant.ty, &constant.name) {
            return;
        }
        self.global
            .constants
            .insert(constant.name.clone(), constant.clone());
        self.global.context.constant(constant.clone());
    }

    /// Check constant value expression.
    /// If expression contains `Constant` check is constant exists.
    /// Values doesn't check as it's just `Primitive Values`.
    /// Also check all expression tree branches.
    /// If `ConstantValue` doesn't exist add error to `Error State` and `return` false result.
    pub fn check_constant_value_expression(
        &mut self,
        data: Option<&(ExpressionOperations, Box<ConstantExpression>)>,
    ) -> bool {
        // For constant expression skip ExpressionOperations
        if let Some((_, child)) = data {
            // Check only Constant value
            match &child.value {
                // Check is ConstantValue already exist in global state
                ConstantValue::Constant(const_name) => {
                    if self.global.constants.contains_key(const_name) {
                        self.check_constant_value_expression(child.operation.as_ref())
                    } else {
                        self.error(StateErrorKind::ConstantNotFound, const_name);
                        false
                    }
                }
                ConstantValue::Literal(_) => true,
            }
        } else {
            true
        }
    }

    /// Function declaration analyze. Add it to Global State/M
    pub fn function_declaration(&mut self, decl: &FunctionDecl<E>) {
        if self.global.functions.contains_key(&decl.name) {
            self.error(StateErrorKind::FunctionAlreadyExist, &decl.name);
            return;
        }

        // Fetch parameters and check types

        if !self.check_type_exists(&decl.result, &decl.name) {
            return;
        }

        let force_quite = decl
            .args
            .iter()
            .any(|(name, ty)| !self.check_type_exists(ty, name));

        // Force quite if errors
        if force_quite {
            return;
        }

        let func = decl.header();

        self.global.functions.insert(decl.name.clone(), func);
        self.global.context.function_declaration(decl.clone());
    }

    /// Function body analyze.
    /// It is basic execution entity for program flow.
    /// It's operate sub analyze for function elements. It's contain
    /// Body State for current and child states.
    pub fn function_body(&mut self, data: &FunctionDecl<E>) {
        // Init empty function body state
        let body_state = Handle::new(BlockState::new(None));

        // Add `State context` with body state context block
        self.blocks.push(body_state.clone());

        // Init function parameters - add to SemanticStackContext
        self.init_func_params(&body_state, &data.args);

        // Flag to indicate is function return called
        let mut return_is_called = false;

        // Fetch function elements and gather errors
        for body in &data.body {
            if return_is_called {
                self.error(
                    StateErrorKind::ForbiddenCodeAfterReturnDeprecated,
                    format!("{body:?}"),
                );
            }
            match body {
                BodyStatement::Stmt(stmt) => self.stmt(stmt, &body_state, None, None),
                BodyStatement::Expr(expr) | BodyStatement::Return(expr) => {
                    // Check is return statement previously called
                    if return_is_called {
                        self.error(StateErrorKind::ReturnAlreadyCalled, expr);
                    }
                    if let Some(result) = self.expression(expr, &body_state) {
                        // Check expression type and do not exist from flow
                        self.check_type_exists(&result.ty(), expr);
                        if data.result != result.ty() {
                            self.error(StateErrorKind::WrongReturnType, expr);
                        }

                        return_is_called = true;

                        // Check is state contain flag of manual
                        // return from other states, for example:
                        // if-flow, loop-flow
                        if body_state.borrow().inner.manual_return {
                            // First we put expression return calculation for case when
                            // before in the state was return statement. So construct
                            // return expression and jump to return label, set return
                            // label and invoke after that read `return` value from all
                            // previous returns and invoke return instruction itself.
                            body_state.expression_function_return_with_label(result);
                        } else {
                            body_state.expression_function_return(result);
                        }
                    }
                }
            }
        }

        // Check is function contain return
        if !return_is_called {
            self.error(StateErrorKind::ReturnNotFound, "");
        }
    }

    /// Init function parameters.
    /// It's init function parameters as values, same as let-binding.
    /// And add instructions to `SemanticStack`.
    fn init_func_params(
        &mut self,
        function_state: &Handle<BlockState<I, E>>,
        fn_params: &Vec<(String, Type)>,
    ) {
        for (arg_name, arg_ty) in fn_params {
            // Find value in current state and parent states
            let value = function_state
                .borrow()
                .get_value_name(&arg_name.clone().into());

            // Calculate `inner_name` as unique for current and all parent states
            let inner_name: InnerValueName = if value.is_none() {
                // if value not found in all states check and set
                // `inner_value` from value name
                // NOTE: value number not incremented
                arg_name.clone().into()
            } else {
                // Function parameter name can't be with the same name.
                // Produce error
                self.error(StateErrorKind::FunctionArgumentNameDuplicated, arg_name);
                return;
            };

            // Set value parameters
            let value = Value {
                inner_name: inner_name.clone(),
                inner_type: arg_ty.clone(),
                mutable: false,
                alloca: false,
                malloc: false,
            };

            // Value inserted only to current state by Value name and Value data
            function_state.insert_value(arg_name.clone(), value.clone());

            // Set `inner_name` to current state and all parent states
            function_state.set_inner_value_name(&inner_name);
            function_state.function_arg(value, arg_name.clone(), arg_ty.clone());
        }
    }

    fn stmt_full(
        &mut self,
        stmt: &crate::stmt::StmtFull<E>,
        state: &Handle<BlockState<I, E>>,

        exit: Option<&LabelName>,
        repeat: Option<(&LabelName, &LabelName)>,
    ) {
        match stmt {
            crate::stmt::StmtFull::Stmt(stmt) => todo!(),
            crate::stmt::StmtFull::Expr(expr) | crate::stmt::StmtFull::Return(expr) => {
                todo!()
            }
            crate::stmt::StmtFull::Break => todo!(),
            crate::stmt::StmtFull::Continue => todo!(),
        }
    }

    pub fn stmt(
        &mut self,
        stmt: &Stmt<E>,
        state: &Handle<BlockState<I, E>>,

        exit: Option<&LabelName>,
        repeat: Option<(&LabelName, &LabelName)>,
    ) {
        match stmt {
            // # Let-binding statement
            // Analyze let-binding statement:
            // 1. Let value bind from expression. First should be analysed
            //    `expression` for binding value.
            // 2. Generate value for current state. Special field `inner_name`
            //    that used as name for `Codegen` should be unique in current
            //    state and for all parent states. For that `inner_name` the
            //    inner value name counter incremented.
            // 3. Set `Value` parameters: `inner_name`,  type and allocation status
            // 4. Insert value to current values state map: value `name` -> `Data`
            // 5. Store `inner_name` in current and parent states
            // 6. Codegen
            Stmt::LetBinding {
                name,
                mutable,
                ty,
                value,
            } => {
                // Call value analytics before putting let-value to state
                let Some(expr_result) = self.expression(value, state) else {
                    return;
                };

                if let Some(ty) = ty {
                    if &expr_result.ty() != ty {
                        self.error(StateErrorKind::WrongLetType, name);
                        return;
                    }
                }

                let let_ty = expr_result.ty();

                // Find value in current state and parent states
                let value = state.borrow().get_value_name(name);

                // Calculate `inner_name` as unique for current and all parent states
                let inner_name = value.map_or_else(
                    // if value not found in all states check and set
                    // `inner_value` from value name
                    || state.borrow().get_next_inner_name(&name.clone().into()),
                    // Increment inner value name counter for shadowed variable
                    // and check variable inner_name for and inner_values in current state
                    |val| state.borrow().get_next_inner_name(&val.inner_name),
                );

                // Set value parameters
                let value = Value {
                    inner_name: inner_name.clone(),
                    inner_type: let_ty,
                    mutable: *mutable,
                    alloca: false,
                    malloc: false,
                };

                // Value inserted only to current state by Value name and Value data
                state.insert_value(name.clone(), value.clone());

                // Set `inner_name` to current state and all parent states
                state.set_inner_value_name(&inner_name);
                state.let_binding(value, expr_result);
            }

            // # Binding statement
            // Analyze binding statement for mutable variables:
            // 1. Bind from expression. First should be analysed `expression` for binding value.
            // 2. Read value for current state.
            // 3. Update value to current values state map: value `name` -> `Data`
            // 4. Codegen with Store action
            Stmt::Binding { name, value } => {
                // Call value analytics before putting let-value to state
                let Some(expr_result) = self.expression(value, state) else {
                    return;
                };

                // Find value in current state and parent states
                let Some(value) = state.borrow().get_value_name(name) else {
                    self.error(StateErrorKind::ValueNotFound, name);
                    return;
                };

                // Check is value mutable
                if !value.mutable {
                    self.error(StateErrorKind::ValueIsNotMutable, name);
                    return;
                }

                state.binding(value, expr_result);
            }
            Stmt::FunctionCall(data) => {
                self.function_call(data, state);
            }
            Stmt::If(data) => self.if_condition(data, state, exit, repeat),
            Stmt::Loop(data) => self.loop_statement(data, state),
        }
    }

    /// # Function-call
    /// Call function with function parameters arguments. Arguments is
    /// expressions.
    /// 1. Check is current function name exists in global state of functions
    /// name.
    /// 2. Analyse expressions for function parameters
    /// 3. Inc register
    /// 4. Generate codegen
    /// Codegen store always result to register even for void result.
    ///
    /// ## Errors
    /// Return error if function name doesn't exist in global state
    pub fn function_call(
        &mut self,
        func_call_data: &FunctionCall<E>,
        body_state: &Handle<BlockState<I, E>>,
    ) -> Option<Type> {
        // Check is function exists in global functions stat
        let Some(func_data) = self.global.functions.get(&func_call_data.name).cloned() else {
            self.error(StateErrorKind::FunctionNotFound, func_call_data);
            return None;
        };
        let fn_type = func_data.result.clone();

        // Analyse function parameters expressions, check their types
        // and set result to array
        let mut params: Vec<ExprResult> = vec![];
        for (i, expr) in func_call_data.args.iter().enumerate() {
            // Types checked in expression, so we don't need additional check
            let expr_result = self.expression(expr, body_state)?;
            if expr_result.ty() != func_data.args[i] {
                let kind = StateErrorKind::FunctionParameterTypeWrong;
                self.error(kind, expr_result.ty());
                continue;
            }
            params.push(expr_result);
        }

        // Result of function call is stored to register
        let register = body_state.inc_register();

        // Store always result to register even for void result
        body_state.call(func_data, params, register);

        Some(fn_type)
    }

    /// # condition-expression
    /// Analyse condition operations.
    /// ## Return
    /// Return result register of `condition-expression` calculation.
    pub fn condition_expression(
        &mut self,
        data: &ExpressionLogicCondition<E>,
        function_body_state: &Handle<BlockState<I, E>>,
    ) -> u64 {
        // Analyse left expression of left condition
        let left_expr = &data.left.left;
        let left_res = self.expression(left_expr, function_body_state);

        // Analyse right expression of left condition
        let right_expr = &data.left.right;
        let right_res = self.expression(right_expr, function_body_state);

        // If some of the `left` or `right` expression is empty just return with error in the state
        let (Some(left_result), Some(right_result)) = (left_res.clone(), right_res.clone()) else {
            let kind = StateErrorKind::ConditionIsEmpty;
            self.error(kind, format!("left={left_res:?}, right={right_res:?}"));
            return function_body_state.borrow().register();
        };

        // Currently strict type comparison
        if left_result.ty() != right_result.ty() {
            let kind = StateErrorKind::ConditionExpressionWrongType;
            self.error(kind, left_result.ty());
            return function_body_state.borrow().register();
        }

        if !left_result.ty().is_primitive() {
            let kind = StateErrorKind::ConditionExpressionNotSupported;
            self.error(kind, left_result.ty());
            return function_body_state.borrow().register();
        }

        // Increment register
        let register = function_body_state.inc_register();

        // Codegen for left condition and set result to register
        function_body_state.condition_expression(
            left_result,
            right_result,
            data.left.condition.clone(),
            register,
        );

        // Analyze right condition
        if let Some(right) = &data.right {
            let left_register_result = function_body_state.borrow().register();
            // Analyse recursively right part of condition
            let right_register_result = self.condition_expression(&right.1, function_body_state);

            // Increment register
            let register = function_body_state.inc_register();

            // Stategen for logical condition for: left [LOGIC-OP] right
            // The result generated from registers, and stored to
            // new register
            function_body_state.logic_condition(
                right.0.clone(),
                left_register_result,
                right_register_result,
                register,
            );
        }
        function_body_state.borrow().register()
    }

    /// # If-condition body
    /// Analyze body for ant if condition:
    /// - if, else, if-else
    /// NOTE: `label_end` - is always already exists
    /// ## Return
    /// Return body statement "return" status
    pub fn if_condition_body(
        &mut self,
        body: &[IfBodyStatement<E>],
        state: &Handle<BlockState<I, E>>,
        label_end: &LabelName,
        label_loop: Option<(&LabelName, &LabelName)>,
    ) -> bool {
        let mut return_is_called = false;
        for body in body {
            if return_is_called {
                let kind = StateErrorKind::ForbiddenCodeAfterReturnDeprecated;
                self.error(kind, format!("{body:?}"));
            }

            match body {
                IfBodyStatement::Stmt(stmt) => {
                    self.stmt(stmt, state, Some(label_end), label_loop);
                }
                IfBodyStatement::Return(expression) => {
                    if let Some(result) = self.expression(expression, state) {
                        // Jump to return label in codegen and set return
                        // status to indicate function, that it's manual return
                        state.jump_function_return(result);
                        state.set_return();
                        return_is_called = true;
                    }
                }
            }
        }
        return_is_called
    }

    /// # If-condition loop body
    /// Analyze body for ant if condition:
    /// - if, else, if-else
    /// ## Return
    /// Return body statement "return" status
    pub fn if_condition_loop_body(
        &mut self,
        body: &[IfLoopBodyStatement<E>],
        state: &Handle<BlockState<I, E>>,
        label_if_end: &LabelName,
        label_loop_start: &LabelName,
        label_loop_end: &LabelName,
    ) -> bool {
        let mut return_is_called = false;
        let mut break_is_called = false;
        let mut continue_is_called = false;
        for body in body {
            if return_is_called {
                let kind = StateErrorKind::ForbiddenCodeAfterReturnDeprecated;
                self.error(kind, format!("{body:?}"));
            }
            if break_is_called {
                let kind = StateErrorKind::ForbiddenCodeAfterBreakDeprecated;
                self.error(kind, format!("{body:?}"));
            }
            if continue_is_called {
                let kind = StateErrorKind::ForbiddenCodeAfterContinueDeprecated;
                self.error(kind, format!("{body:?}"));
            }

            let exit = Some(label_if_end);
            let repeat = Some((label_loop_start, label_loop_end));

            match body {
                IfLoopBodyStatement::Stmt(stmt) => self.stmt(stmt, state, exit, repeat),

                IfLoopBodyStatement::Return(expression) => {
                    if let Some(result) = self.expression(expression, state) {
                        // Jump to return label in codegen and set return
                        // status to indicate function, that it's manual return
                        state.jump_function_return(result);
                        state.set_return();
                        return_is_called = true;
                    }
                }
                IfLoopBodyStatement::Continue => {
                    continue_is_called = true;
                    // Skip next loop  step and jump to the start of loop
                    state.jump_to(label_loop_start.clone());
                }
                IfLoopBodyStatement::Break => {
                    break_is_called = true;
                    // Break loop and jump to the end of loop
                    state.jump_to(label_loop_end.clone());
                }
            }
        }
        return_is_called
    }

    /// # If conditions calculations
    /// Calculate conditions for if-condition. It can contain
    /// simple and logic conditions.
    pub fn if_condition_calculation(
        &mut self,
        condition: &IfCondition<E>,
        state: &Handle<BlockState<I, E>>,
        label_if_begin: &LabelName,
        label_if_else: &LabelName,
        label_if_end: &LabelName,
        is_else: bool,
    ) {
        // Analyse if-conditions
        match condition {
            // if condition represented just as expression
            IfCondition::Single(expr) => {
                // Calculate expression for single if-condition expression
                let Some(expr_result) = self.expression(expr, state) else {
                    return;
                };

                // State for if-condition from expression and if-body start
                let begin = label_if_begin.clone();
                let end = if is_else { label_if_else } else { label_if_end }.clone();

                state.if_condition_expression(expr_result, begin, end);
            }
            // If condition contains logic condition expression
            IfCondition::Logic(expr_logic) => {
                // Analyse if-condition logic
                let result_register = self.condition_expression(expr_logic, state);

                // State for if-condition-logic with if-body start
                let begin = label_if_begin.clone();
                let end = if is_else { label_if_else } else { label_if_end }.clone();

                state.if_condition_logic(begin, end, result_register);
            }
        }
    }

    /// # If-condition
    /// Analyzing includes all variants for if statements:
    /// 1. if
    /// 2. if-else
    /// 3. if-else-if
    /// It creates own state, with parent function-state. in that case
    /// if-state independent from parent state, but csn get access to
    /// parent state.
    /// If condition can't contain `else` and `if-else` on the
    /// same time.
    ///
    /// Special case for `label_end` - it should be set from previous
    /// context, and main goal is to end all of if-condition nodes in
    /// the same flow with same `if-end` label. It's especially important
    /// for `else-if` condition.
    ///
    /// ## Panics
    /// `label_loop` is must be set, it's special case for the Loop,
    /// when `label_loop` should always be set. If it doesn't set, it's
    /// unexpected behavior and program algorithm error
    pub fn if_condition(
        &mut self,
        data: &IfStatement<E>,
        func_state: &Handle<BlockState<I, E>>,
        label_end: Option<&LabelName>,
        label_loop: Option<(&LabelName, &LabelName)>,
    ) {
        // It can't contain `else` and `if-else` on the same time
        if let (Some(_), Some(stm)) = (&data.else_statement, &data.else_if_statement) {
            self.error(StateErrorKind::IfElseDuplicated, "if-condition");
        }
        // Create state for if-body, from parent function state because
        // if-state can contain sub-state, that can be independent from parent
        // state
        let state = Handle::new(BlockState::new(Some(func_state.clone())));

        func_state.add_child(state.clone());

        let begin = LabelName::new("if_begin");
        let r#else = LabelName::new("if_else");
        let end = LabelName::new("if_end");

        // Get labels name for if-begin, and if-end
        let label_if_begin = state.get_and_set_next_label(&begin);
        let label_if_else = state.get_and_set_next_label(&r#else);

        // Set if-end label from previous context
        let label_if_end = label_end
            .cloned()
            .unwrap_or_else(|| state.get_and_set_next_label(&end));

        // To set if-end as single return point check is it previously set
        let is_set_label_if_end = label_end.is_some();
        let is_else = data.else_statement.is_some() || data.else_if_statement.is_some();

        // Analyse if-conditions
        self.if_condition_calculation(
            &data.condition,
            &state,
            &label_if_begin,
            &label_if_else,
            &label_if_end,
            is_else,
        );

        //== If condition main body
        // Set if-begin label
        state.set_label(label_if_begin);

        // Analyze if-conditions body kind.
        // Return flag for current body state, excluding children return claims
        let return_is_called = match &data.body {
            // Analyze if-statement body
            IfBodyStatements::If(body) => {
                self.if_condition_body(body, &state, &label_if_end, label_loop)
            }
            IfBodyStatements::Loop(body) => {
                // It's special case for the Loop, when `label_loop` should always be set.
                // If it doesn't set, it's unexpected behavior and program algorithm error
                let (label_loop_start, label_loop_end) =
                    label_loop.expect("loop label should be set");
                // Analyze if-loop-statement body
                self.if_condition_loop_body(
                    body,
                    &state,
                    &label_if_end,
                    label_loop_start,
                    label_loop_end,
                )
            }
        };

        // Codegen for jump to if-end statement - return to program flow.
        // If return is set do not add jump-to-end label.
        if !return_is_called {
            state.jump_to(label_if_end.clone());
        }

        // Check else statements: else, else-if
        if is_else {
            // Set if-else label
            state.set_label(label_if_else);

            // Analyse if-else body: data.else_statement
            if let Some(else_body) = &data.else_statement {
                // if-else has own state, different from if-state
                let else_state = Handle::new(BlockState::new(Some(func_state.clone())));

                func_state.add_child(else_state.clone());

                let return_is_called = match else_body {
                    // Analyze if-statement body
                    IfBodyStatements::If(body) => {
                        self.if_condition_body(body, &else_state, &label_if_end, label_loop)
                    }
                    // Analyze if-loop-statement body
                    IfBodyStatements::Loop(body) => {
                        let (start, end) = label_loop.expect("label should be set");
                        self.if_condition_loop_body(body, &else_state, &label_if_end, start, end)
                    }
                };

                // Codegen for jump to if-end statement -return to program flow
                // If return is set do not add jump-to-end label.
                if !return_is_called {
                    state.jump_to(label_if_end.clone());
                }
            } else if let Some(else_if_statement) = &data.else_if_statement {
                // Analyse  else-if statement
                // Set `label_if_end` to indicate single if-end point
                self.if_condition(
                    else_if_statement,
                    func_state,
                    Some(&label_if_end),
                    label_loop,
                );
            }
        }

        // End label for all if statement, should be set only once
        if !is_set_label_if_end {
            state.set_label(label_if_end);
        }
    }

    /// # Loop
    /// Loop statement contains logic:
    /// - jump to loop
    /// - loop body
    /// - end of loop
    /// - return, break, continue
    pub fn loop_statement(
        &mut self,
        data: &[LoopBodyStatement<E>],
        func_state: &Handle<BlockState<I, E>>,
    ) {
        // Create state for loop-body, from parent func state because
        // loop-state can contain sub-state, that can be independent from parent state
        let state = Handle::new(BlockState::new(Some(func_state.clone())));
        func_state.add_child(state.clone());

        let begin = LabelName::new("loop_begin");
        let end = LabelName::new("loop_end");

        // Get labels name for loop-begin, and loop-end
        let continue_label = state.get_and_set_next_label(&begin);
        let break_label = state.get_and_set_next_label(&end);

        state.jump_to(continue_label.clone());
        state.set_label(continue_label.clone());

        let mut return_is_called = false;
        let mut break_is_called = false;
        let mut continue_is_called = false;
        for body in data {
            if return_is_called {
                let kind = StateErrorKind::ForbiddenCodeAfterReturnDeprecated;
                self.error(kind, format!("{body:?}"));
            }
            if break_is_called {
                let kind = StateErrorKind::ForbiddenCodeAfterBreakDeprecated;
                self.error(kind, format!("{body:?}"));
            }
            if continue_is_called {
                let kind = StateErrorKind::ForbiddenCodeAfterContinueDeprecated;
                self.error(kind, format!("{body:?}"));
            }

            match body {
                LoopBodyStatement::Stmt(stmt) => {
                    self.stmt(stmt, &state, None, Some((&continue_label, &break_label)));
                }
                LoopBodyStatement::Return(expression) => {
                    if let Some(result) = self.expression(expression, &state) {
                        // Jump to return label in codegen and set return
                        // status to indicate function, that it's manual return
                        state.jump_function_return(result);
                        state.set_return();
                        return_is_called = true;
                    }
                }
                LoopBodyStatement::Break => {
                    // Break loop and jump to the end of loop
                    state.jump_to(break_label.clone());
                    break_is_called = true;
                }
                LoopBodyStatement::Continue => {
                    // Skip next loop  step and jump to the start of loop
                    state.jump_to(continue_label.clone());
                    continue_is_called = true;
                }
            }
        }

        // If return is called do not set loop-specific instructions
        if !return_is_called {
            // Because it's loop jump to loop begin
            state.jump_to(continue_label.clone());
            // Loop ending
            state.set_label(break_label);
        }
    }

    #[allow(clippy::doc_markdown)]
    /// ## Expression
    /// Is basic entity for state operation and state usage.
    /// State correctness verified by expressions call.
    /// Expressions folded by operations priority. For that
    /// expressions tree folded each leaf of tree by priority operation
    /// level. The most striking image is bracketing an expression with
    /// a higher priority, and build tree based on that.
    ///
    /// ## Return
    /// `PrimitiveValue` | `TmpRegister`
    ///
    ///  Possible algorithm conditions:
    ///     1. PrimitiveValue -> PrimitiveValue
    ///     2. Value -> load -> TmpRegister
    ///     3. FuncCall -> call -> TmpRegister
    ///     4. Operations
    ///         4.1. PrimitiveValue
    ///         - PrimitiveValue -> tmp = OP val1, val2 -> TmpRegister
    ///         - Value -> tmp1 = load -> OP val1, tmp1 -> TmpRegister
    ///         - FuncCAll -> tmp1 = call -> OP val1, tmp1 -> TmpRegister
    ///         4.2. TmpRegister (with name tmp1)
    ///         - PrimitiveValue -> tmp2 = OP tmp1, val1 -> TmpRegister
    ///         - Value -> tmp2 = load -> tmp3 = OP tmp1, tmp2 -> TmpRegister
    ///         - FuncCall -> tmp2 = call ->  tmp3 = OP tmp1, tmp2 -> TmpRegister
    ///         4.3. Operations -> recursively invoke 4.2.
    pub fn expression(
        &mut self,
        data: &Expression<E>,
        body_state: &Handle<BlockState<I, E>>,
    ) -> Option<ExprResult> {
        // Fold expression operations priority
        let expr = Self::expression_operations_priority(data.clone());
        // To analyze expression first time, we set:
        // left_value - as None
        // operation - as None
        // And basic expression value is `right_value`, because
        // it can contain sub-operations (`left_value` don't contain
        // and contain Expression result)
        self.expression_operation(None, &expr, None, body_state)
    }

    /// Expression operation semantic logic:
    /// `OP(lhs, rhs)`
    /// Left-value contains optional Expression result for left side of expression.
    #[allow(clippy::too_many_lines)]
    pub fn expression_operation(
        &mut self,
        lhs: Option<&ExprResult>,
        right_expression: &Expression<E>,
        op: Option<&ExpressionOperations>,
        state: &Handle<BlockState<I, E>>,
    ) -> Option<ExprResult> {
        // Get right side value from expression.
        // If expression return error immediately return error
        // because next analyzer should use success result.
        let rhs = match &right_expression.value {
            // Check is expression Value entity
            ExprValue::Variable(value) => {
                // Get value from block state
                let value_from_state = state.get_value_name(value);

                // Register contains result
                let register = state.inc_register();

                // First check value in body state
                let ty = if let Some(val) = value_from_state {
                    state.expression_value(val.clone(), register);
                    val.inner_type
                } else if let Some(val) = self.global.constants.get(&value.to_string().into()) {
                    state.expression_const(val.clone(), register);
                    val.ty.clone()
                } else {
                    // If value doesn't exist in State or as Constant
                    self.error(StateErrorKind::ValueNotFound, value);
                    return None;
                };

                // Return result as register
                ExprResult::Register(ty, register)
            }
            // Check is expression primitive value
            // Just return primitive value itself
            ExprValue::Literal(value) => ExprResult::Literal(value.clone()),
            // Check is expression Function call entity
            ExprValue::Call(fn_call) => {
                // We shouldn't increment register, because it's inside `self.function_call`.
                // And result of function always stored in register.
                let ty = self.function_call(fn_call, state)?;

                // Return result as register
                ExprResult::Register(ty, state.inc_register())
            }
            ExprValue::Struct { name, attr } => {
                // Can be only Value from state, not constant
                // Get value from block state
                let Some(val) = state.get_value_name(name) else {
                    // If value doesn't exist
                    self.error(StateErrorKind::ValueNotFound, name);
                    return None;
                };

                // Check is value type is struct
                let Some(ty) = val.inner_type.get_struct() else {
                    self.error(StateErrorKind::ValueNotStruct, name);
                    return None;
                };

                // Check is type exists
                if !self.check_type_exists(&val.inner_type, name) {
                    return None;
                }
                if &Type::Struct(ty.clone()) != self.global.types.get(&val.inner_type.name())? {
                    self.error(StateErrorKind::WrongExpressionType, name);
                    return None;
                }

                let Some(attributes) = ty.attributes.get(attr).cloned() else {
                    self.error(StateErrorKind::ValueNotStructField, name);
                    return None;
                };

                // Register contains result
                let register = state.inc_register();
                state.expression_struct_value(val.clone(), attributes.index, register);

                ExprResult::Register(attributes.ty, state.inc_register())
            }

            // Subexpression should be analyzed independently
            ExprValue::Expr(expr) => self.expression(expr, state)?,
            ExprValue::Extended(expr) => expr.expression(self, state),
        };

        // Check left expression side and generate expression operation code
        let expression_result = if let (Some(lhs), Some(op)) = (lhs, op) {
            if lhs.ty() != rhs.ty() {
                self.error(StateErrorKind::WrongExpressionType, lhs.ty());
                // Do not fetch other expression flow if type is wrong
                return None;
            }

            // Expression operation is set to register
            let register = state.inc_register();

            // Call expression operation for: OP(lhs, rhs)
            state.expression_operation(op.clone(), lhs.clone(), rhs.clone(), register);

            // Expression result value  for Operations is always should be "register"
            ExprResult::Register(rhs.ty(), register)
        } else {
            rhs
        };

        // Check is for right value contain next operation
        if let Some((operation, expr)) = &right_expression.operation {
            // Recursively call, where current Execution result set as left
            // side expressionf
            self.expression_operation(Some(&expression_result), expr, Some(operation), state)
        } else {
            Some(expression_result)
        }
    }

    /// # Expression operation priority
    /// Fold expression priority.
    /// Pass expressions tree from max priority level to minimum
    /// priority level. If expression priority for concrete branch
    /// founded, it's folded to leaf (same as bracketing).
    ///
    /// ## Return
    /// New folded expressions tree.
    fn expression_operations_priority(data: Expression<E>) -> Expression<E> {
        (0..=MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS)
            .rev()
            .fold(data, Self::fetch_op_priority)
    }

    /// Fetch expression operation priories and fold it.
    /// Expressions folded by operations priority. For that expressions
    /// tree folded each branch of tree to leaf by priority operation
    /// level. The most striking image is bracketing an expression with
    /// a higher priority, and build tree based on that.
    ///
    /// For example: expr = expr1 OP1 expr2 - it has 2 branches
    /// if expr2 contain subbranch (for example: `expr2 OP2 expr3`) we trying
    /// to find priority level for current pass. And if `priority_level == OP1`
    /// - fold it to leaf.
    /// NOTICE: expr1 can't contain subbranches by design. So we pass
    /// expression tree from left to right.
    /// If priority level not equal, we just return income expression, or
    /// if it has subbranch - launch fetching subbranch
    fn fetch_op_priority(data: Expression<E>, priority_level: u8) -> Expression<E> {
        // Check is expression contains right side with operation
        let Some((op, expr)) = data.clone().operation else {
            return data;
        };

        // Check is right expression contain subbranch (sub operation)
        let Some((next_op, next_expr)) = expr.operation.clone() else {
            return data;
        };

        // Check incoming expression operation priority level
        if op.priority() == priority_level {
            // Fold expression to leaf - creating new expression as value
            let expression_value = ExprValue::Expr(Box::new(Expression {
                value: data.value,
                operation: Some((
                    op,
                    Box::new(Expression {
                        value: expr.value,
                        operation: None,
                    }),
                )),
            }));
            // Fetch next expression branch
            let new_expr = Self::fetch_op_priority(*next_expr, priority_level);
            // Create new expression with folded `expression_value`
            Expression {
                value: expression_value,
                operation: Some((next_op, Box::new(new_expr))),
            }
        } else {
            // If priority not equal for current level just
            // fetch right side of expression for next branches
            let new_expr = if next_op.priority() > op.priority() && next_expr.operation.is_none() {
                // Pack expression to leaf
                Expression {
                    value: ExprValue::Expr(expr),
                    operation: None,
                }
            } else {
                Self::fetch_op_priority(*expr, priority_level)
            };

            // Rebuild expression tree
            Expression {
                value: data.value,
                operation: Some((op, Box::new(new_expr))),
            }
        }
    }
}
