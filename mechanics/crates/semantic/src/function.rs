use crate::{expression::Expression, names::FunctionName, stmt::BodyStatement, types::Type};

/// Function declaration is basic type that represent function itself.
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionDecl<E> {
    /// Name
    pub name: FunctionName,
    /// Arguments
    pub args: Vec<(String, Type)>,
    /// Result type
    pub result: Type,
    /// Body statements
    pub body: Vec<BodyStatement<E>>,
}

impl<E> FunctionDecl<E> {
    #[must_use]
    pub const fn new(
        name: FunctionName,
        args: Vec<(String, Type)>,
        result: Type,
        body: Vec<BodyStatement<E>>,
    ) -> Self {
        Self {
            name,
            args,
            result,
            body,
        }
    }

    #[must_use]
    pub fn header(&self) -> FunctionHeader {
        FunctionHeader {
            name: self.name.clone(),
            args: self.args.iter().map(|(_, ty)| ty.clone()).collect(),
            result: self.result.clone(),
        }
    }
}

/// Function declaration header
///
/// It used to detect functions in state and
/// their parameters to use in normal execution
/// flog.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionHeader {
    /// Inner function name
    pub name: FunctionName,
    /// Function parameters types
    pub args: Vec<Type>,
    /// Inner (return) type
    pub result: Type,
}

/// # Function call
/// Basic struct for function call representation
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionCall<E> {
    /// Call function name
    pub name: FunctionName,
    /// Call function parameters contains expressions
    pub args: Vec<Expression<E>>,
}

impl<E> std::fmt::Display for FunctionCall<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
