//! # Semantic types
//! Type-system types for Semantic analyzer State results.

use crate::names::{FunctionName, InnerValueName, TypeName, ValueName};
use std::collections::HashMap;
use std::fmt::Display;

/// # Values
/// Can contain inner data: name, type, memory allocation status:
/// - alloca - stack allocation
/// - malloc - malloc allocation
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct Value {
    /// Inner value name
    pub inner_name: InnerValueName,
    /// Inner value type
    pub inner_type: Type,

    /// Mutability flag
    pub mutable: bool,
    /// Stack allocation flag
    pub alloca: bool,
    /// Memory allocation flag
    pub malloc: bool,
}

/// `TypeAttributes` type attributes trait.
/// Used for types declarations.
pub trait TypeAttributes {
    /// Get attribute index by value name for the parent type
    fn attribute_index(&self, attr_name: &ValueName) -> Option<u32>;
    /// Get attribute type by value name for the parent type
    fn attribute_type(&self, attr_name: &ValueName) -> Option<Type>;
    /// Get function name for the parent type by method name
    fn method(&self, method_name: String) -> Option<FunctionName>;
    /// Check is value attribute
    fn is_attribute(&self, name: &ValueName) -> bool;
    /// Check is name is method
    fn is_method(&self, name: String) -> bool;
}

/// # Type
/// Basic representation of Type. Basic entities:
/// - primitive type
/// - struct type
/// - array type
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum Type {
    U8,
    U16,
    U32,
    U64,

    I8,
    I16,
    I32,
    I64,

    F32,
    F64,

    Bool,
    Char,
    Ptr,
    None,

    Struct(StructTypes),
    Array(Box<Self>, u32),
}

impl Type {
    /// Get type name
    #[must_use]
    pub fn name(&self) -> TypeName {
        self.to_string().into()
    }

    /// Get structure type if it is
    #[must_use]
    pub fn get_struct(&self) -> Option<StructTypes> {
        match self {
            Self::Struct(ty) => Some(ty.clone()),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_primitive(&self) -> bool {
        !matches!(self, Self::Array(_, _) | Self::Struct(_))
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8 => f.write_str("u8"),
            Self::U16 => f.write_str("u16"),
            Self::U32 => f.write_str("u32"),
            Self::U64 => f.write_str("u64"),
            Self::I8 => f.write_str("i8"),
            Self::I16 => f.write_str("i16"),
            Self::I32 => f.write_str("i32"),
            Self::I64 => f.write_str("i64"),
            Self::F32 => f.write_str("f32"),
            Self::F64 => f.write_str("f64"),
            Self::Bool => f.write_str("bool"),
            Self::Char => f.write_str("char"),
            Self::Ptr => f.write_str("ptr"),
            Self::None => f.write_str("()"),

            Self::Struct(struct_type) => write!(f, "{}", struct_type.name),
            Self::Array(array_type, size) => write!(f, "[{array_type};{size:?}]"),
        }
    }
}

impl TypeAttributes for Type {
    fn attribute_index(&self, attr_name: &ValueName) -> Option<u32> {
        match self {
            Self::Struct(st) => st.attribute_index(attr_name),
            _ => None,
        }
    }
    fn attribute_type(&self, attr_name: &ValueName) -> Option<Type> {
        match self {
            Self::Struct(st) => st.attribute_type(attr_name),
            _ => None,
        }
    }
    fn method(&self, method_name: String) -> Option<FunctionName> {
        match self {
            Self::Struct(st) => st.method(method_name),
            _ => None,
        }
    }
    fn is_attribute(&self, attr_name: &ValueName) -> bool {
        match self {
            Self::Struct(st) => st.is_attribute(attr_name),
            _ => false,
        }
    }
    fn is_method(&self, method_name: String) -> bool {
        match self {
            Self::Struct(st) => st.is_method(method_name),
            _ => false,
        }
    }
}

/// # Struct types
/// Basic entity for struct type itself.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct StructTypes {
    /// Type name
    pub name: TypeName,
    /// Struct attributes
    pub attributes: HashMap<ValueName, StructAttributeType>,
    /// Struct methods
    pub methods: HashMap<String, FunctionName>,
}

impl TypeAttributes for StructTypes {
    fn attribute_index(&self, attr_name: &ValueName) -> Option<u32> {
        self.attributes.get(attr_name).map(|attr| attr.index)
    }
    fn attribute_type(&self, attr_name: &ValueName) -> Option<Type> {
        self.attributes.get(attr_name).map(|attr| attr.ty.clone())
    }
    fn method(&self, method_name: String) -> Option<FunctionName> {
        self.methods.get(&method_name).cloned()
    }
    fn is_attribute(&self, attr_name: &ValueName) -> bool {
        self.attributes.contains_key(attr_name)
    }
    fn is_method(&self, method_name: String) -> bool {
        self.methods.contains_key(&method_name)
    }
}

/// `StructAttributeType` is type for Struct attributes fields.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
pub struct StructAttributeType {
    /// Attribute name for struct type
    pub name: ValueName,
    /// Attribute index representation for struct type
    pub index: u32,
    /// Attribute type for struct type
    pub ty: Type,
}

/// `Literal` represents primitive value element of AST.
/// Values based on primitive types.
/// Used for `ConstantValue` and `ExpressionValue`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "codec", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "codec", serde(tag = "type", content = "content"))]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Char(char),
    Ptr,
    None,
}

impl Literal {
    #[must_use]
    pub const fn ty(&self) -> Type {
        match self {
            Self::U8(_) => Type::U8,
            Self::U16(_) => Type::U16,
            Self::U32(_) => Type::U32,
            Self::U64(_) => Type::U64,
            Self::I8(_) => Type::I8,
            Self::I16(_) => Type::I16,
            Self::I32(_) => Type::I32,
            Self::I64(_) => Type::I64,
            Self::F32(_) => Type::F32,
            Self::F64(_) => Type::F64,
            Self::Char(_) => Type::Char,
            Self::Bool(_) => Type::Bool,
            Self::Ptr => Type::Ptr,
            Self::None => Type::None,
        }
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(val) => write!(f, "{val}"),
            Self::U16(val) => write!(f, "{val}"),
            Self::U32(val) => write!(f, "{val}"),
            Self::U64(val) => write!(f, "{val}"),
            Self::I8(val) => write!(f, "{val}"),
            Self::I16(val) => write!(f, "{val}"),
            Self::I32(val) => write!(f, "{val}"),
            Self::I64(val) => write!(f, "{val}"),
            Self::F32(val) => write!(f, "{val}"),
            Self::F64(val) => write!(f, "{val}"),
            Self::Bool(val) => write!(f, "{val}"),
            Self::Char(val) => write!(f, "{val}"),
            Self::Ptr => f.write_str("ptr"),
            Self::None => f.write_str("None"),
        }
    }
}
