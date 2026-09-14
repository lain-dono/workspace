//! # Expression types
//! Expression types for Semantic analyzer result state.

use crate::function::FunctionCall;
use crate::names::ValueName;
use crate::types::{Literal, Type};
use std::fmt::Display;

/// Max priority level fpr expressions operations
pub const MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS: u8 = 9;

/// # Expression
/// Basic expression entity representation. It contains
/// `ExpressionValue` and optional operations with other
/// expressions. So it's represent flat tree.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct Expression<E> {
    /// Expression value
    pub value: ExprValue<E>,
    /// Optional expression operation under other `Expression`
    pub operation: Option<(ExpressionOperations, Box<Expression<E>>)>,
}

impl<E: std::fmt::Display> Display for Expression<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// # Expression result
/// Contains analyzing results of expression:
/// - `expr_type` - result type of expression
/// - `expr_value` - result value of expression
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub enum ExprResult {
    Literal(Literal),
    Register(Type, u64),
}

impl ExprResult {
    #[must_use]
    pub fn ty(&self) -> Type {
        match self {
            Self::Literal(v) => v.ty(),
            Self::Register(ty, _) => ty.clone(),
        }
    }
}

/// Expression value kinds:
/// - value name - initialized through let-binding
/// - primitive value - most primitive value, like integers etc.
/// - struct value - value of struct type
/// - function call - call of function with params
/// - expression - contains other expression
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum ExprValue<E> {
    Variable(ValueName),
    Literal(Literal),
    /// Expression value of struct type. It's represent access to
    /// struct attributes of values with struct type
    Struct {
        /// Value name for structure value
        name: ValueName,
        /// Value attribute for structure value
        attr: ValueName,
    },
    Call(FunctionCall<E>),
    Expr(Box<Expression<E>>),
    Extended(Box<E>),
}

impl<E: std::fmt::Display> Display for ExprValue<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(val) => write!(f, "{val}"),
            Self::Literal(val) => write!(f, "{val}"),
            Self::Struct { name, attr } => write!(f, "{name}:{attr}"),
            Self::Call(fn_call) => write!(f, "{fn_call}"),
            Self::Expr(val) => write!(f, "{val}"),
            Self::Extended(val) => write!(f, "{val}"),
        }
    }
}

/// Basic  expression operations - calculations and
/// logic operations
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum ExpressionOperations {
    Plus,
    Minus,
    Multiply,
    Divide,
    ShiftLeft,
    ShiftRight,
    And,
    Or,
    Xor,
    Eq,
    NotEq,
    Great,
    Less,
    GreatEq,
    LessEq,
}

impl ExpressionOperations {
    /// Get expression operation priority level
    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Plus => 5,
            Self::Minus => 4,
            Self::Divide => 8,
            Self::Multiply | Self::ShiftLeft | Self::ShiftRight => {
                MAX_PRIORITY_LEVEL_FOR_EXPRESSIONS
            }
            Self::Or | Self::Xor => 6,
            Self::And
            | Self::Eq
            | Self::NotEq
            | Self::Great
            | Self::Less
            | Self::GreatEq
            | Self::LessEq => 7,
        }
    }
}
