use std::collections::HashSet;

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum TestError {
    #[error("not found")]
    NotFound,
    #[error("type mismatch")]
    TypeMismatch,
}

pub type TestResult<T = bool> = Result<T, TestError>;

#[derive(Clone)]
pub enum Filter<T> {
    All(Vec<Where<T>>),
    Any(Vec<Where<T>>),
}

impl<T> Filter<T> {
    pub fn all(items: impl IntoIterator<Item = Where<T>>) -> Self {
        Self::All(items.into_iter().collect())
    }

    pub fn any(items: impl IntoIterator<Item = Where<T>>) -> Self {
        Self::Any(items.into_iter().collect())
    }

    pub fn is<'a>(&'a self, view: impl ViewRef<'a, T>) -> TestResult {
        match self {
            Self::All(items) => items
                .iter()
                .try_fold(true, |acc, item| Ok(acc && item.is(view)?)),

            Self::Any(items) => items
                .iter()
                .try_fold(false, |acc, item| Ok(acc || item.is(view)?)),
        }
    }
}

pub type Field = &'static str;

#[derive(Clone)]
pub enum Operand {
    Field(Field),
    Value(Value),
}

impl From<Field> for Operand {
    fn from(field: Field) -> Self {
        Self::Field(field)
    }
}

impl From<Value> for Operand {
    fn from(value: Value) -> Self {
        Self::Value(value)
    }
}

#[derive(Clone)]
pub enum Where<T> {
    Filter(Filter<T>),

    Includes(T, T),
    Excludes(T, T),

    IsDisjoint(T, T),
    IsSubset(T, T),
    IsSuperset(T, T),

    Compare(T, Compare, T),
}

impl<T> Where<T> {
    pub fn cmp(lhs: impl Into<T>, cmp: Compare, rhs: impl Into<T>) -> Self {
        Self::Compare(lhs.into(), cmp, rhs.into())
    }

    pub fn is<'a>(&'a self, view: impl ViewRef<'a, T>) -> TestResult {
        match *self {
            Self::Filter(ref filter) => filter.is(view),

            Self::Includes(ref lhs, ref rhs) => match (view.get(lhs)?, view.get(rhs)?) {
                (Value::Enum(lhs), Value::Text(rhs)) => Ok(lhs.contains(rhs)),
                _ => Err(TestError::TypeMismatch),
            },
            Self::Excludes(ref lhs, ref rhs) => match (view.get(lhs)?, view.get(rhs)?) {
                (Value::Enum(lhs), Value::Text(rhs)) => Ok(!lhs.contains(rhs)),
                _ => Err(TestError::TypeMismatch),
            },

            Self::IsDisjoint(ref lhs, ref rhs) => match (view.get(lhs)?, view.get(rhs)?) {
                (Value::Enum(lhs), Value::Enum(rhs)) => Ok(lhs.is_disjoint(rhs)),
                _ => Err(TestError::TypeMismatch),
            },
            Self::IsSubset(ref lhs, ref rhs) => match (view.get(lhs)?, view.get(rhs)?) {
                (Value::Enum(lhs), Value::Enum(rhs)) => Ok(lhs.is_subset(rhs)),
                _ => Err(TestError::TypeMismatch),
            },
            Self::IsSuperset(ref lhs, ref rhs) => match (view.get(lhs)?, view.get(rhs)?) {
                (Value::Enum(lhs), Value::Enum(rhs)) => Ok(lhs.is_subset(rhs)),
                _ => Err(TestError::TypeMismatch),
            },

            Self::Compare(ref lhs, cmp, ref rhs) => cmp.is(view.get(lhs)?, view.get(rhs)?),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Uint(u32),
    Sint(i32),
    Float(f32),
    Text(String),
    Enum(HashSet<String>),
}

#[derive(Clone, Copy)]
pub enum Compare {
    Eq, // ==
    Ne, // !=

    Ge, // <=
    Gt, // <
    Le, // >=
    Lt, // >
}

impl Compare {
    pub fn is(self, a: &Value, b: &Value) -> TestResult {
        match (self, a, b) {
            (Self::Eq, &Value::Bool(a), &Value::Bool(b)) => Ok(a == b),
            (Self::Ne, &Value::Bool(a), &Value::Bool(b)) => Ok(a != b),

            (Self::Eq, Value::Text(a), Value::Text(b)) => Ok(a == b),
            (Self::Ne, Value::Text(a), Value::Text(b)) => Ok(a != b),

            (Self::Eq, Value::Enum(a), Value::Enum(b)) => Ok(a == b),
            (Self::Ne, Value::Enum(a), Value::Enum(b)) => Ok(a != b),

            (Self::Eq, &Value::Uint(a), &Value::Uint(b)) => Ok(a == b),
            (Self::Ne, &Value::Uint(a), &Value::Uint(b)) => Ok(a != b),
            (Self::Ge, &Value::Uint(a), &Value::Uint(b)) => Ok(a <= b),
            (Self::Gt, &Value::Uint(a), &Value::Uint(b)) => Ok(a < b),
            (Self::Le, &Value::Uint(a), &Value::Uint(b)) => Ok(a >= b),
            (Self::Lt, &Value::Uint(a), &Value::Uint(b)) => Ok(a > b),

            (Self::Eq, &Value::Sint(a), &Value::Sint(b)) => Ok(a == b),
            (Self::Ne, &Value::Sint(a), &Value::Sint(b)) => Ok(a != b),
            (Self::Ge, &Value::Sint(a), &Value::Sint(b)) => Ok(a <= b),
            (Self::Gt, &Value::Sint(a), &Value::Sint(b)) => Ok(a < b),
            (Self::Le, &Value::Sint(a), &Value::Sint(b)) => Ok(a >= b),
            (Self::Lt, &Value::Sint(a), &Value::Sint(b)) => Ok(a > b),

            (Self::Eq, &Value::Float(a), &Value::Float(b)) => Ok(a == b),
            (Self::Ne, &Value::Float(a), &Value::Float(b)) => Ok(a != b),
            (Self::Ge, &Value::Float(a), &Value::Float(b)) => Ok(a <= b),
            (Self::Gt, &Value::Float(a), &Value::Float(b)) => Ok(a < b),
            (Self::Le, &Value::Float(a), &Value::Float(b)) => Ok(a >= b),
            (Self::Lt, &Value::Float(a), &Value::Float(b)) => Ok(a > b),

            _ => Err(TestError::TypeMismatch),
        }
    }
}

// #[derive(Clone, Copy, Debug)]
// pub struct Field;

#[derive(Clone, Copy)]
pub struct OperandView<'a>(&'a std::collections::HashMap<Field, Value>);

impl<'a> ViewRef<'a, Operand> for OperandView<'a> {
    fn get(&self, op: &'a Operand) -> TestResult<&'a Value> {
        match op {
            Operand::Field(field) => self.0.get(field).ok_or(TestError::NotFound),
            Operand::Value(value) => Ok(value),
        }
    }
}

pub trait ViewRef<'a, T>: Copy {
    fn get(&self, op: &'a T) -> TestResult<&'a Value>;
}

#[cfg(test)]
#[test]
fn filter() {
    let age_value = Value::Uint(14);
    let context = [
        ("age", age_value.clone()),
        ("age_group", Value::Text(String::from("young_adult"))),
    ];
    let data: std::collections::HashMap<Field, Value> = context.into_iter().collect();

    let filter = Filter::all([Where::cmp("age", Compare::Eq, age_value)]);

    assert!(filter.is(OperandView(&data)).unwrap());
}
