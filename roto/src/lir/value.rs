//! Values and types for the IR

/// A Roto value with type information at runtime
///
/// The purpose of [`Value`] is to provide a safe way to test our
/// generated code. It is the value that is generally used by the IR.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub enum Value {
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Ptr(usize),
}

/// The types for [`Value`]s
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[must_use]
pub enum Type {
    Bool,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Ptr,
}

impl Type {
    pub const IVAR: Self = Self::I32;
    pub const FVAR: Self = Self::F64;

    /// The size of the type in bytes
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::I8 | Self::Bool => 1,
            Self::I16 => 2,
            Self::I32 | Self::F32 => 4,
            Self::I64 | Self::F64 => 8,
            Self::Ptr => (usize::BITS / 8) as usize,
        }
    }

    #[must_use]
    pub const fn align(self) -> usize {
        match self {
            Self::I8 | Self::Bool => 1,
            Self::I16 => 2,
            Self::I32 | Self::F32 => 4,
            Self::I64 | Self::F64 => 8,
            Self::Ptr => (usize::BITS / 8) as usize,
        }
    }
}

impl core::fmt::Display for Type {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Bool => "bool",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Ptr => "ptr",
        })
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::I8(l), Self::I8(r)) => l == r,
            (Self::I16(l), Self::I16(r)) => l == r,
            (Self::I32(l), Self::I32(r)) => l == r,
            (Self::I64(l), Self::I64(r)) => l == r,
            (Self::Ptr(l), Self::Ptr(r)) => l == r,
            _ => panic!("tried comparing different types"),
        }
    }
}

impl Value {
    pub fn ty(&self) -> Type {
        match self {
            Self::Bool(_) => Type::Bool,
            Self::I8(_) => Type::I8,
            Self::I16(_) => Type::I16,
            Self::I32(_) => Type::I32,
            Self::I64(_) => Type::I64,
            Self::F32(_) => Type::F32,
            Self::F64(_) => Type::F64,
            Self::Ptr(_) => Type::Ptr,
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Bool(t) => bytemuck::bytes_of(t),
            Self::I8(t) => bytemuck::bytes_of(t),
            Self::I16(t) => bytemuck::bytes_of(t),
            Self::I32(t) => bytemuck::bytes_of(t),
            Self::I64(t) => bytemuck::bytes_of(t),
            Self::F32(t) => bytemuck::bytes_of(t),
            Self::F64(t) => bytemuck::bytes_of(t),
            Self::Ptr(t) => bytemuck::bytes_of(t),
        }
    }

    pub fn from_slice(ty: Type, s: &[u8]) -> Self {
        Self::try_from_slice(ty, s).unwrap()
    }

    pub fn try_from_slice(ty: Type, s: &[u8]) -> Result<Self, bytemuck::PodCastError> {
        Ok(match ty {
            Type::Bool => Self::Bool(!matches!(*bytemuck::try_from_bytes::<i8>(s)?, 0)),
            Type::I8 => Self::I8(*bytemuck::try_from_bytes(s)?),
            Type::I16 => Self::I16(*bytemuck::try_from_bytes(s)?),
            Type::I32 => Self::I32(*bytemuck::try_from_bytes(s)?),
            Type::I64 => Self::I64(*bytemuck::try_from_bytes(s)?),
            Type::F32 => Self::F32(*bytemuck::try_from_bytes(s)?),
            Type::F64 => Self::F64(*bytemuck::try_from_bytes(s)?),
            Type::Ptr => Self::Ptr(*bytemuck::try_from_bytes(s)?),
        })
    }

    #[must_use]
    pub fn as_bool(self) -> bool {
        match self {
            Self::Bool(x) => x,
            _ => panic!("Invalid value!"),
        }
    }

    #[must_use]
    pub fn as_u64(self) -> u64 {
        match self {
            Self::I8(x) => x as u64,
            Self::I16(x) => x as u64,
            Self::I32(x) => x as u64,
            Self::I64(x) => x as u64,
            _ => panic!("Invalid value!"),
        }
    }

    #[must_use]
    pub fn as_i64(self) -> i64 {
        match self {
            Self::I8(x) => x as i64,
            Self::I16(x) => x as i64,
            Self::I32(x) => x as i64,
            Self::I64(x) => x,
            _ => panic!("Invalid value!"),
        }
    }

    #[must_use]
    pub fn as_f64(self) -> f64 {
        match self {
            Self::F32(x) => x as f64,
            Self::F64(x) => x,
            _ => panic!("Invalid value!"),
        }
    }

    #[must_use]
    pub fn switch_on(self) -> usize {
        const { assert!(size_of::<usize>() == 8, "only non-toy arch's are supported") };
        match self {
            Self::Bool(b) => b as usize,
            Self::I8(x) => x as usize,
            Self::I16(x) => x as usize,
            Self::I32(x) => x as usize,
            Self::I64(x) => x as usize,
            _ => todo!(),
        }
    }
}

impl core::fmt::Display for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bool(x) => write!(f, "bool({x})"),
            Self::I8(x) => write!(f, "i8({x})"),
            Self::I16(x) => write!(f, "i16({x})"),
            Self::I32(x) => write!(f, "i32({x})"),
            Self::I64(x) => write!(f, "i64({x})"),
            Self::F32(x) => write!(f, "f32({x})"),
            Self::F64(x) => write!(f, "f64({x})"),
            Self::Ptr(x) => write!(f, "ptr({x})"),
        }
    }
}
