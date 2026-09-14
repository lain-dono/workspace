//! # Errors types
//! Errors types for Semantic analyzer result of Error state.

/// Common errors kind for the State.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum StateErrorKind {
    /// Common error indicate errors in the State
    Common,
    ConstantAlreadyExist,
    ConstantNotFound,
    WrongLetType,
    WrongExpressionType,
    TypeAlreadyExist,
    FunctionAlreadyExist,
    ValueNotFound,
    ValueNotStruct,
    ValueNotStructField,
    ValueIsNotMutable,
    FunctionNotFound,
    FunctionParameterTypeWrong,
    ReturnNotFound,
    ReturnAlreadyCalled,
    IfElseDuplicated,
    TypeNotFound,
    WrongReturnType,
    ConditionExpressionWrongType,
    ConditionIsEmpty,
    ConditionExpressionNotSupported,
    ForbiddenCodeAfterReturnDeprecated,
    ForbiddenCodeAfterContinueDeprecated,
    ForbiddenCodeAfterBreakDeprecated,
    FunctionArgumentNameDuplicated,
}

/// State error result data representation
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct StateErrorResult {
    /// Kind of error
    pub kind: StateErrorKind,
    /// Error value
    pub value: String,
}

impl StateErrorResult {
    /// Get state trace data from error result as string
    #[must_use]
    pub fn trace_state(&self) -> String {
        format!("[{:?}] for value {:?}", self.kind, self.value,)
    }
}
