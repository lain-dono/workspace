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

IfBodyStatement::Return(expression) => {
    if let Some(result) = self.expression(expression, state) {
        // Jump to return label in codegen and set return
        // status to indicate function, that it's manual return
        state.jump_function_return(result);
        state.set_return();
        return_is_called = true;
    }
}

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
    // Skip next loop  step and jump to the start of loop
    state.jump_to(label_loop_start.clone());
    continue_is_called = true;
}
IfLoopBodyStatement::Break => {
    // Break loop and jump to the end of loop
    state.jump_to(label_loop_end.clone());
    break_is_called = true;
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
LoopBodyStatement::Continue => {
    // Skip next loop  step and jump to the start of loop
    state.jump_to(begin.clone());
    continue_is_called = true;
}
LoopBodyStatement::Break => {
    // Break loop and jump to the end of loop
    state.jump_to(end.clone());
    break_is_called = true;
}