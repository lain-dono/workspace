//! Type information on Rust types
//!
//! The [`TypeRegistry`] holds information on Rust types that we have seen,
//! so we can match them to Roto types and use the names in error messages.
//!
//! The registry should initially hold all registered types and primitives.
//! On demand, we add more complex types via the [`Reflect`] trait, which
//! is implemented for types that have a Roto equivalent. This is necessary
//! for mapping a complex Rust type to Roto types.

use crate::{
    lir::{self, Memory},
    runtime::{
        basic::{OptionWrapper, Verdict},
        layout::Layout,
        val::Val,
    },
};
use std::{
    any::{TypeId, type_name},
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeDescription {
    /// Some type that we don't know how to decompose
    Leaf,

    /// `Option<T>`
    Option(TypeId),

    /// `Verdict<A, R>`
    Verdict(TypeId, TypeId),

    /// `Val<T>`
    Val(TypeId),
}

#[derive(Clone)]
pub struct Ty {
    /// The name of the type in Rust, mostly for diagnostic purposes
    pub rust_name: &'static str,

    /// The memory alignment of the type in bytes
    pub layout: Layout,

    /// The [`TypeId`] corresponding to this type
    pub type_id: TypeId,

    /// Description of the structure of the type
    pub description: TypeDescription,
}

impl Ty {
    fn new<T: 'static>(description: TypeDescription) -> Self {
        Self {
            rust_name: type_name::<T>(),
            layout: Layout::of::<T>(),
            type_id: TypeId::of::<T>(),
            description,
        }
    }
}

static GLOBAL_TYPE_REGISTRY: LazyLock<Mutex<TypeRegistry>> =
    LazyLock::new(|| Mutex::new(TypeRegistry::default()));

/// A map from [`TypeId`] to a [`Ty`], which is a description of the type
#[derive(Clone, Default)]
pub struct TypeRegistry {
    map: HashMap<TypeId, &'static Ty>,
}

impl TypeRegistry {
    pub fn store<T: 'static>(description: TypeDescription) -> Ty {
        let ty = Ty::new::<T>(description);
        GLOBAL_TYPE_REGISTRY
            .lock()
            .unwrap()
            .map
            .entry(ty.type_id)
            // Leaking is fine because we only store each type once in the
            // global TypeRegistry.
            .or_insert_with(|| Box::leak(Box::new(ty)))
            .clone()
    }

    pub fn get(id: TypeId) -> Option<&'static Ty> {
        let registry = GLOBAL_TYPE_REGISTRY.lock().unwrap();
        registry.map.get(&id).map(|v| &**v)
    }

    /// Register a type implementing [`Reflect`]
    pub fn resolve<T: Reflect>() -> Ty {
        T::resolve()
    }
}

/// A type that can be passed to Roto.
///
/// The `Reflect::Transformed` type represents the type that this type will be
/// converted into before being passed to Roto. For example, `Option` will be
/// converted into a type with the same variants, but a fixed layout.
///
/// The `Reflect::AsParam` then specifies how this value is passed to a Roto
/// function. Most primitives are simply passed by value, but many other types
/// are passed by `*mut Reflect::Transformed`.
pub trait Reflect: Sized + 'static {
    /// Intermediate type that can be used to convert a type to a Roto type
    type Transformed;

    /// The type that this type should be converted into when passed to Roto
    type AsParam: Param<Self::Transformed>;

    /// Transform this value into a value that Roto understands
    fn transform(self) -> Self::Transformed;

    /// Transform this a Roto value back into this type
    fn untransform(transformed: Self::Transformed) -> Self;

    fn as_param(transformed: &mut Self::Transformed) -> Self::AsParam {
        Self::AsParam::as_param(transformed)
    }

    fn to_value(param: Self::AsParam) -> Self::Transformed {
        Self::AsParam::to_value(param)
    }

    /// Attempt to convert an IR value into `Self`
    fn from_ir_value(
        mem: &mut Memory,
        value: lir::Value,
    ) -> Result<Self::AsParam, ValueDoesNotMatchType> {
        Self::AsParam::from_ir_value(mem, value)
    }

    /// Put information about this type into the global type registry
    ///
    /// The information is also returned for direct use.
    fn resolve() -> Ty;

    #[must_use]
    fn name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub struct ValueDoesNotMatchType;

pub trait Param<T>: Sized {
    fn as_param(value: &mut T) -> Self;

    fn to_value(self) -> T;

    fn from_ir_value(mem: &mut Memory, value: lir::Value) -> Result<Self, ValueDoesNotMatchType>;
}

impl<T> Param<T> for *mut T {
    fn as_param(transformed: &mut T) -> Self {
        std::ptr::from_mut::<T>(transformed)
    }

    fn to_value(self) -> T {
        unsafe { std::ptr::read(self) }
    }

    fn from_ir_value(mem: &mut Memory, value: lir::Value) -> Result<Self, ValueDoesNotMatchType> {
        let lir::Value::Ptr(p) = value else {
            return Err(ValueDoesNotMatchType);
        };
        Ok(mem.read_slice(p, std::mem::size_of::<T>()).as_ptr() as *mut T)
    }
}

impl<A: Reflect, R: Reflect> Reflect for Verdict<A, R>
where
    A::Transformed: Clone,
    R::Transformed: Clone,
{
    type Transformed = Verdict<A::Transformed, R::Transformed>;
    type AsParam = *mut Self::Transformed;

    fn transform(self) -> Self::Transformed {
        match self {
            Self::Accept(a) => Verdict::Accept(a.transform()),
            Self::Reject(r) => Verdict::Reject(r.transform()),
        }
    }

    fn untransform(transformed: Self::Transformed) -> Self {
        match transformed {
            Verdict::Accept(a) => Self::Accept(A::untransform(a)),
            Verdict::Reject(r) => Self::Reject(R::untransform(r)),
        }
    }

    fn resolve() -> Ty {
        let t = A::resolve().type_id;
        let e = R::resolve().type_id;

        let desc = TypeDescription::Verdict(t, e);
        TypeRegistry::store::<Self>(desc)
    }
}

impl<T: Reflect> Reflect for Option<T> {
    type Transformed = OptionWrapper<T::Transformed>;
    type AsParam = *mut Self::Transformed;

    fn transform(self) -> Self::Transformed {
        match self {
            Some(t) => OptionWrapper::Some(t.transform()),
            None => OptionWrapper::None,
        }
    }

    fn untransform(transformed: Self::Transformed) -> Self {
        match transformed {
            OptionWrapper::Some(t) => Some(T::untransform(t)),
            OptionWrapper::None => None,
        }
    }

    fn resolve() -> Ty {
        let t = T::resolve().type_id;

        let desc = TypeDescription::Option(t);
        TypeRegistry::store::<Self>(desc)
    }
}

impl<T> Param<Val<T>> for *mut T {
    fn as_param(value: &mut Val<T>) -> Self {
        &raw mut value.0
    }

    fn to_value(self) -> Val<T> {
        Val(unsafe { std::ptr::read(self) })
    }

    fn from_ir_value(mem: &mut Memory, value: lir::Value) -> Result<Self, ValueDoesNotMatchType> {
        if let lir::Value::Ptr(p) = value {
            Ok(mem.read_slice(p, std::mem::size_of::<T>()).as_ptr() as *mut T)
        } else {
            Err(ValueDoesNotMatchType)
        }
    }
}

impl<T: 'static + Clone> Reflect for Val<T> {
    type Transformed = Self;
    type AsParam = *mut T;

    fn transform(self) -> Self::Transformed {
        self
    }

    fn untransform(transformed: Self::Transformed) -> Self {
        transformed
    }

    fn resolve() -> Ty {
        let desc = TypeDescription::Val(TypeId::of::<T>());
        TypeRegistry::store::<Self>(desc)
    }

    fn name() -> &'static str {
        std::any::type_name::<T>()
    }
}

impl Reflect for Arc<str> {
    type Transformed = Self;
    type AsParam = *mut Self;

    fn transform(self) -> Self::Transformed {
        self
    }

    fn untransform(transformed: Self::Transformed) -> Self {
        transformed
    }

    fn resolve() -> Ty {
        TypeRegistry::store::<Self>(TypeDescription::Leaf)
    }
}

impl TryFrom<&lir::Value> for () {
    type Error = ();

    fn try_from(_: &lir::Value) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl Param<()> for () {
    fn as_param((): &mut ()) -> Self {}
    fn to_value(self) {}
    fn from_ir_value(_mem: &mut Memory, _value: lir::Value) -> Result<Self, ValueDoesNotMatchType> {
        Ok(())
    }
}

impl Reflect for () {
    type Transformed = Self;
    type AsParam = Self;

    fn transform(self) -> Self::Transformed {}
    fn untransform((): Self::Transformed) -> Self {}
    fn resolve() -> Ty {
        TypeRegistry::store::<Self>(TypeDescription::Leaf)
    }
}

macro_rules! simple_reflect {
    ($( $t:ty : $ir:ident($i:ty),)+) => {
        $(
        impl From<$t> for lir::Value {
            fn from(value: $t) -> Self {
                lir::Value::$ir(value as $i)
            }
        }

        impl TryFrom<&lir::Value> for $t {
            type Error = ();

            fn try_from(value: &lir::Value) -> Result<Self, Self::Error> {
                match value {
                    lir::Value::$ir(x) => Ok(*x as $t),
                    _ => Err(()),
                }
            }
        }

        impl Param<$t> for $t {
            fn as_param(value: &mut $t) -> Self {
                *value
            }

            fn to_value(self) -> $t {
                self
            }

            fn from_ir_value(
                _: &mut Memory,
                value: lir::Value,
            ) -> Result<Self, ValueDoesNotMatchType> {
                let lir::Value::$ir(p) = value else {
                    return Err(ValueDoesNotMatchType);
                };
                Ok(p as $t)
            }
        }

        impl Reflect for $t {
            type Transformed = Self;
            type AsParam = Self;

            fn transform(self) -> Self::Transformed {
                self
            }

            fn untransform(transformed: Self::Transformed) -> Self {
                transformed
            }

            fn resolve() -> Ty {
                TypeRegistry::store::<Self>(TypeDescription::Leaf)
            }
        }
    )+
    };
}

simple_reflect!(
    bool: Bool(bool),

    u8 : I8(i8 ), i8 :I8 (i8 ),
    u16:I16(i16), i16:I16(i16),
    u32:I32(i32), i32:I32(i32),
    u64:I64(i64), i64:I64(i64),

    f32:F32(f32),
    f64:F64(f64),
);
