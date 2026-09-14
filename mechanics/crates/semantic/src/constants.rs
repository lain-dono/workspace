use crate::expression::ExpressionOperations;
use crate::names::ConstantName;
use crate::types::{Literal, Type};

/// Constant value can contain other constant or primitive value
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum ConstantValue {
    Constant(ConstantName),
    Literal(Literal),
}

/// Constant expression represent expression operation between
/// constant values, and represent flat tree
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstantExpression {
    /// Constant value for expression operation
    pub value: ConstantValue,
    /// Optional expression operation and next constant expression entry point
    pub operation: Option<(ExpressionOperations, Box<ConstantExpression>)>,
}

/// # Constant
/// Can contain: name, type
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct Constant {
    /// Constant name
    pub name: ConstantName,
    /// Constant type
    pub ty: Type,
    /// Constant value represented through constant expression
    pub value: ConstantExpression,
}
