//! # Condition types
//! Condition types for Semantic analyzer result state.

use crate::expression::Expression;
use crate::function::FunctionCall;
use crate::names::ValueName;
use crate::types::Type;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum Stmt<E> {
    /// Value initialization through binding.
    LetBinding {
        /// Value name
        name: ValueName,
        /// Value mutability flag
        mutable: bool,
        /// Value type
        ty: Option<Type>,
        /// Value bind expression
        value: Box<Expression<E>>,
    },
    /// `Binding` represents mutable binding for previously bind values
    Binding {
        /// Binding value name
        name: ValueName,
        /// Value expression representation
        value: Box<Expression<E>>,
    },
    FunctionCall(FunctionCall<E>),
    If(IfStatement<E>),
    Loop(Vec<LoopBodyStatement<E>>),
}

/// # Body statement
/// Statement of body. Body is basic entity for functions and
/// represent basic functions elements.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum BodyStatement<E> {
    Stmt(Stmt<E>),

    Expr(Expression<E>),
    Return(Expression<E>),
}

/// Basic logical conditions mostly for compare expressions
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum Condition {
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=
}

/// Logical conditions type representation.
/// Usefulf for logical expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum Logic {
    And,
    Or,
}

/// Expression condition for two expressions
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionCondition<E> {
    /// Left expression
    pub left: Expression<E>,
    /// Condition for expressions
    pub condition: Condition,
    /// Right expression
    pub right: Expression<E>,
}

/// Expression logic condition for expression conditions.
/// It's build chain of expression conditions and
/// expression logic conditions as flat tree.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionLogicCondition<E> {
    /// Left expression condition
    pub left: ExpressionCondition<E>,
    /// Optional right expression condition with logic condition
    pub right: Option<(Logic, Box<ExpressionLogicCondition<E>>)>,
}

/// If-condition representation. It can be:
/// - simple - just expression
/// - logic - represented through `ExpressionLogicCondition`
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum IfCondition<E> {
    Single(Expression<E>),
    Logic(ExpressionLogicCondition<E>),
}

/// # If statement
/// Basic entity that represent if-statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct IfStatement<E> {
    /// If-condition
    pub condition: IfCondition<E>,
    /// Basic if-body, if if-condition is true
    pub body: IfBodyStatements<E>,
    /// Basic else-body, if if-condition is false
    pub else_statement: Option<IfBodyStatements<E>>,
    /// Basic else-if-body
    pub else_if_statement: Option<Box<IfStatement<E>>>,
}

/// If-body statement can be:
/// - if-body-statement related only
/// - loop-body-statement related - special case for the loops
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum IfBodyStatements<E> {
    If(Vec<IfBodyStatement<E>>),
    Loop(Vec<IfLoopBodyStatement<E>>),
}

/// Loop body statement represents body for the loop
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum LoopBodyStatement<E> {
    Stmt(Stmt<E>),

    Return(Expression<E>),
    Break,
    Continue,
}

/// If-body statement represents body for the if-body
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum IfBodyStatement<E> {
    Stmt(Stmt<E>),

    Return(Expression<E>),
}

/// If-loop body statements represent body of if-body in the loops
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum IfLoopBodyStatement<E> {
    Stmt(Stmt<E>),

    Return(Expression<E>),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtFull<E> {
    Stmt(Stmt<E>),

    Expr(Expression<E>),
    Return(Expression<E>),
    Break,
    Continue,
}

impl<E> From<Stmt<E>> for StmtFull<E> {
    fn from(value: Stmt<E>) -> Self {
        Self::Stmt(value)
    }
}

impl<E> From<BodyStatement<E>> for StmtFull<E> {
    fn from(value: BodyStatement<E>) -> Self {
        match value {
            BodyStatement::Stmt(stmt) => Self::Stmt(stmt),
            BodyStatement::Expr(expression) => Self::Expr(expression),
            BodyStatement::Return(expression) => Self::Return(expression),
        }
    }
}

impl<E> From<LoopBodyStatement<E>> for StmtFull<E> {
    fn from(value: LoopBodyStatement<E>) -> Self {
        match value {
            LoopBodyStatement::Stmt(stmt) => Self::Stmt(stmt),
            LoopBodyStatement::Return(expression) => Self::Return(expression),
            LoopBodyStatement::Break => Self::Break,
            LoopBodyStatement::Continue => Self::Continue,
        }
    }
}

impl<E> From<IfBodyStatement<E>> for StmtFull<E> {
    fn from(value: IfBodyStatement<E>) -> Self {
        match value {
            IfBodyStatement::Stmt(stmt) => Self::Stmt(stmt),
            IfBodyStatement::Return(expression) => Self::Return(expression),
        }
    }
}

impl<E> From<IfLoopBodyStatement<E>> for StmtFull<E> {
    fn from(value: IfLoopBodyStatement<E>) -> Self {
        match value {
            IfLoopBodyStatement::Stmt(stmt) => Self::Stmt(stmt),
            IfLoopBodyStatement::Return(expression) => Self::Return(expression),
            IfLoopBodyStatement::Break => Self::Break,
            IfLoopBodyStatement::Continue => Self::Continue,
        }
    }
}
