use bevy_reflect::{PartialReflect, Reflect, ReflectKind, ReflectRef, VariantType};

#[derive(thiserror::Error, Debug)]
pub enum ScriptAccessError {
    #[error("expected Struct, actual {0}")]
    Struct(ReflectKind),
    #[error("expected Tuple, actual {0}")]
    Tuple(ReflectKind),
    #[error("expected List, actual {0}")]
    List(ReflectKind),

    #[error("expected Struct, actual {0:?}")]
    StructVariant(VariantType),
    #[error("expected Tuple, actual {0:?}")]
    TupleVariant(VariantType),
}

#[derive(Debug, Clone, Reflect)]
pub enum ScriptAccess {
    Field(&'static str),
    FieldIndex(usize),
    TupleIndex(usize),
    ListIndex(usize),
}

impl ScriptAccess {
    pub fn field(field: &'static str) -> Self {
        Self::Field(field)
    }

    pub fn access<'a>(
        &self,
        base: &'a dyn PartialReflect,
    ) -> Result<Option<&'a dyn PartialReflect>, ScriptAccessError> {
        match self {
            ScriptAccess::Field(field) => Self::access_field(base, field),
            &ScriptAccess::FieldIndex(index) => Self::access_field_index(base, index),
            &ScriptAccess::TupleIndex(index) => Self::access_tuple_index(base, index),
            &ScriptAccess::ListIndex(index) => Self::access_list_index(base, index),
        }
    }

    /// A name-based field access on a struct.
    pub fn access_field<'a>(
        base: &'a dyn PartialReflect,
        field: &str,
    ) -> Result<Option<&'a dyn PartialReflect>, ScriptAccessError> {
        use ReflectRef::{Enum, Struct};

        match base.reflect_ref() {
            Struct(struct_ref) => Ok(struct_ref.field(field)),
            Enum(enum_ref) => match enum_ref.variant_type() {
                VariantType::Struct => Ok(enum_ref.field(field)),
                actual => Err(ScriptAccessError::StructVariant(actual)),
            },
            actual => Err(ScriptAccessError::Struct(actual.into())),
        }
    }

    /// A index-based field access on a struct.
    pub fn access_field_index(
        base: &dyn PartialReflect,
        index: usize,
    ) -> Result<Option<&dyn PartialReflect>, ScriptAccessError> {
        use ReflectRef::{Enum, Struct};

        match base.reflect_ref() {
            Struct(struct_ref) => Ok(struct_ref.field_at(index)),
            Enum(enum_ref) => match enum_ref.variant_type() {
                VariantType::Struct => Ok(enum_ref.field_at(index)),
                actual => Err(ScriptAccessError::StructVariant(actual)),
            },
            actual => Err(ScriptAccessError::Struct(actual.into())),
        }
    }

    /// An index-based access on a tuple.
    pub fn access_tuple_index(
        base: &dyn PartialReflect,
        index: usize,
    ) -> Result<Option<&dyn PartialReflect>, ScriptAccessError> {
        use ReflectRef::{Enum, Tuple, TupleStruct};

        match base.reflect_ref() {
            TupleStruct(tuple) => Ok(tuple.field(index)),
            Tuple(tuple) => Ok(tuple.field(index)),
            Enum(enum_ref) => match enum_ref.variant_type() {
                VariantType::Tuple => Ok(enum_ref.field_at(index)),
                actual => Err(ScriptAccessError::TupleVariant(actual)),
            },
            actual => Err(ScriptAccessError::Tuple(actual.into())),
        }
    }

    /// An index-based access on a list.
    pub fn access_list_index(
        base: &dyn PartialReflect,
        index: usize,
    ) -> Result<Option<&dyn PartialReflect>, ScriptAccessError> {
        use ReflectRef::{Array, List};

        match base.reflect_ref() {
            List(list) => Ok(list.get(index)),
            Array(list) => Ok(list.get(index)),
            actual => Err(ScriptAccessError::List(actual.into())),
        }
    }
}
