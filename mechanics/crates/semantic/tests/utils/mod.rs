use semantic::block_state::BlockState;
use semantic::context::{ExtendedExpression, SemanticContextInstruction};
use semantic::error::StateErrorKind;
use semantic::expression::ExprResult;
use semantic::handle::Handle;
use semantic::semantic::State;
use semantic::types::Literal;
use std::fmt::Display;
use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomExpression<I: SemanticContextInstruction> {
    _marker: PhantomData<I>,
}

impl<I: SemanticContextInstruction> ExtendedExpression<I> for CustomExpression<I> {
    fn expression(
        &self,
        _state: &mut State<Self, I>,
        _block_state: &Handle<BlockState<I, Self>>,
    ) -> ExprResult {
        ExprResult::Literal(Literal::Ptr)
    }
}

impl<I: SemanticContextInstruction> Display for CustomExpression<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CUSTOM")
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomExpressionInstruction;

impl SemanticContextInstruction for CustomExpressionInstruction {}

pub struct SemanticTest<I: SemanticContextInstruction> {
    pub state: State<CustomExpression<I>, I>,
}

impl Default for SemanticTest<CustomExpressionInstruction> {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticTest<CustomExpressionInstruction> {
    pub fn new() -> Self {
        Self {
            state: State::default(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty_error(&self) -> bool {
        self.state.errors.is_empty()
    }

    #[allow(dead_code)]
    pub fn clean_errors(&mut self) {
        self.state.errors = vec![]
    }

    #[allow(dead_code)]
    pub fn check_errors_len(&self, len: usize) -> bool {
        self.state.errors.len() == len
    }

    #[allow(dead_code)]
    pub fn check_error(&self, err_kind: StateErrorKind) -> bool {
        self.state.errors.first().unwrap().kind == err_kind
    }

    #[allow(dead_code)]
    pub fn check_error_index(&self, index: usize, err_kind: StateErrorKind) -> bool {
        self.state.errors.get(index).unwrap().kind == err_kind
    }
}
