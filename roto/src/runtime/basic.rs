use super::{CloneDrop, Movability};
use crate::{
    Constant, Function, Impl, Library, Module, Reflect, RegisterableFn, Runtime, Type, Use,
    ast::ident::Ident,
};
use std::sync::Arc;

fn to_string_impl<T: ToString + Reflect>(lib: &mut Library) {
    let func = |this: T| -> Arc<str> { this.to_string().into() };

    let doc = "Convert this value into a `String`";
    let mut block = Impl::new::<T>();
    block.define_func("to_string", doc, ["self"], func);
    lib.add(block.into());
}

impl Library {
    /// A type implementing `Clone`.
    ///
    /// The `name` must be a valid identifier. The `doc` parameter is the
    /// docstring that will be displayed in the documentation.
    pub fn define_clone_ty<T: Reflect + Clone>(&mut self, name: &str, doc: &str) {
        let ty = Type::new::<T>(name, doc, Movability::CloneDrop(CloneDrop::of::<T>()));
        self.add(ty.unwrap().into());
    }

    /// A type implementing `Copy`.
    ///
    /// The `name` must be a valid identifier. The `doc` parameter is the
    /// docstring that will be displayed in the documentation.
    pub fn define_copy_ty<T: Reflect + Copy>(&mut self, name: &str, doc: &str) {
        let ty = Type::new::<T>(name, doc, Movability::Copy);
        self.add(ty.unwrap().into());
    }

    pub(crate) fn define_value<T: Reflect + Copy + ToString>(&mut self, name: &str, doc: &str) {
        let ty = Type::new::<T>(name, doc, Movability::Value);
        self.add(ty.unwrap().into());
        to_string_impl::<T>(self);
    }

    pub fn define_const<T: Reflect>(&mut self, name: impl Into<Ident>, doc: impl AsRef<str>, val: T)
    where
        T::Transformed: Send + Sync + 'static,
    {
        self.add(Constant::new(name, doc, val).unwrap().into());
    }

    pub fn define_func<'a, A, R>(
        &mut self,
        name: &str,
        doc: &str,
        params: impl Into<Vec<&'a str>>,
        func: impl RegisterableFn<A, R>,
    ) {
        let params = params.into();
        let func = Function::new(name, doc, params, func);
        self.add(func.unwrap().into());
    }
}

impl Impl {
    pub fn define_func<'a, A, R>(
        &mut self,
        name: &str,
        doc: &str,
        params: impl Into<Vec<&'a str>>,
        func: impl RegisterableFn<A, R>,
    ) {
        let params = params.into();
        let func = Function::new(name, doc, params, func);
        self.add(func.unwrap());
    }
}

impl Module {
    pub fn define_const<T: Reflect>(&mut self, name: impl Into<Ident>, doc: impl AsRef<str>, val: T)
    where
        T::Transformed: Send + Sync + 'static,
    {
        self.add(Constant::new(name, doc, val).unwrap());
    }

    pub fn define_func<'a, A, R>(
        &mut self,
        name: &str,
        doc: &str,
        params: impl Into<Vec<&'a str>>,
        func: impl RegisterableFn<A, R>,
    ) {
        let params = params.into();
        let func = Function::new(name, doc, params, func);
        self.add(func.unwrap());
    }
}

impl Default for Library {
    fn default() -> Self {
        let mut lib = Self { items: vec![] };

        let imports = vec![
            vec![String::from("Option"), String::from("Some")],
            vec![String::from("Option"), String::from("None")],
        ];
        lib.add(Use::new(imports).into());

        lib.define_value::<bool>("bool", "The boolean type");
        lib.define_clone_ty::<Arc<str>>("String", "The string type");
        lib.define_value::<f32>("f32", "The 32-bit floating point type");
        lib.define_value::<f64>("f64", "The 64-bit floating point type");

        lib.define_value::<u8>("u8", "The 8-bit unsigned integer");
        lib.define_value::<u16>("u16", "The 16-bit unsigned integer");
        lib.define_value::<u32>("u32", "The 32-bit unsigned integer");
        lib.define_value::<u64>("u64", "The 64-bit unsigned integer");

        lib.define_value::<i8>("i8", "The 8-bit signed integer");
        lib.define_value::<i16>("i16", "The 16-bit signed integer");
        lib.define_value::<i32>("i32", "The 32-bit signed integer");
        lib.define_value::<i64>("i64", "The 64-bit signed integer");

        // impl string
        {
            let mut block = Impl::new::<Arc<str>>();

            let doc = "Convert this value into a `String`";
            let func = |this: Arc<str>| -> Arc<str> { this };
            block.define_func("to_string", doc, ["self"], func);

            let doc = "Append a string to another, creating a new string";
            let func = |this: Arc<str>, other: Arc<str>| -> Arc<str> {
                (this.to_string() + &other).into()
            };
            block.define_func("append", doc, ["self", "other"], func);

            let doc = "Check whether a string contains another string";
            let func =
                |this: Arc<str>, needle: Arc<str>| -> bool { this.contains(needle.as_ref()) };
            block.define_func("contains", doc, ["self", "needle"], func);

            let doc = "Check whether a string starts with a given prefix";
            let func =
                |this: Arc<str>, suffix: Arc<str>| -> bool { this.starts_with(suffix.as_ref()) };
            block.define_func("starts_with", doc, ["self", "suffix"], func);

            let doc = "Check whether a string ends with a given suffix";
            let func =
                |this: Arc<str>, suffix: Arc<str>| -> bool { this.ends_with(suffix.as_ref()) };
            block.define_func("ends_with", doc, ["self", "suffix"], func);

            let doc = "Create a new string with all characters converted to lowercase";
            let func = |this: Arc<str>| -> Arc<str> { this.to_lowercase().into() };
            block.define_func("to_lowercase", doc, ["self"], func);

            let doc = "Create a new string with all characters converted to uppercase";
            let func = |this: Arc<str>| -> Arc<str> { this.to_uppercase().into() };
            block.define_func("to_uppercase", doc, ["self"], func);

            let doc = "Repeat a string `n` times and join them";
            let func = |this: Arc<str>, n: u32| -> Arc<str> { this.repeat(n as usize).into() };
            block.define_func("repeat", doc, ["self", "n"], func);

            let doc = "Tests for string values to be equal, and is used by `==`";
            let func = |this: Arc<str>, other: Arc<str>| -> bool { this == other };
            block.define_func("eq", doc, ["self", "other"], func);

            let doc = "Tests for string values to be not equal, and is used by `!=`";
            let func = |this: Arc<str>, other: Arc<str>| -> bool { this != other };
            block.define_func("ne", doc, ["self", "other"], func);

            lib.add(block.into());
        }

        // impl f32
        {
            type Float = f32;
            let mut block = Impl::new::<Float>();

            let doc = "Returns the smallest integer greater than or equal to self";
            block.define_func::<_, Float>("floor", doc, ["self"], |this: Float| this.floor());

            let doc = "Returns the smallest integer greater than or equal to self";
            block.define_func::<_, Float>("ceil", doc, ["self"], |this: Float| this.ceil());

            let doc = "Returns the nearest integer to self. If a value is half-way between two integers, round away from 0.0";
            block.define_func::<_, Float>("round", doc, ["self"], |this: Float| this.round());

            let doc = "Computes the absolute value of self";
            block.define_func::<_, Float>("abs", doc, ["self"], |this: Float| this.abs());

            let doc = "Returns the square root of a number";
            block.define_func::<_, Float>("sqrt", doc, ["self"], |this: Float| this.sqrt());

            let doc = "Raises a number to a floating point power";
            block.define_func::<_, Float>("pow", doc, ["self"], |this: Float, exp: Float| {
                this.powf(exp)
            });

            let doc = "Returns true if this value is NaN";
            block.define_func::<_, bool>("is_nan", doc, ["self"], |this: Float| this.is_nan());

            let doc = "Returns true if this number is neither infinite nor NaN";
            block
                .define_func::<_, bool>("is_finite", doc, ["self"], |this: Float| this.is_finite());

            let doc = "Returns true if this value is positive infinity or negative infinity, and false otherwise";
            block.define_func::<_, bool>("is_infinite", doc, ["self"], |this: Float| {
                this.is_infinite()
            });

            lib.add(block.into());
        }

        // impl f64
        {
            type Float = f64;
            let mut block = Impl::new::<Float>();

            let doc = "Returns the smallest integer greater than or equal to self";
            block.define_func::<_, Float>("floor", doc, ["self"], |this: Float| this.floor());

            let doc = "Returns the smallest integer greater than or equal to self";
            block.define_func::<_, Float>("ceil", doc, ["self"], |this: Float| this.ceil());

            let doc = "Returns the nearest integer to self. If a value is half-way between two integers, round away from 0.0";
            block.define_func::<_, Float>("round", doc, ["self"], |this: Float| this.round());

            let doc = "Computes the absolute value of self";
            block.define_func::<_, Float>("abs", doc, ["self"], |this: Float| this.abs());

            let doc = "Returns the square root of a number";
            block.define_func::<_, Float>("sqrt", doc, ["self"], |this: Float| this.sqrt());

            let doc = "Raises a number to a floating point power";
            block.define_func::<_, Float>("pow", doc, ["self"], |this: Float, exp: Float| {
                this.powf(exp)
            });

            let doc = "Returns true if this value is NaN";
            block.define_func::<_, bool>("is_nan", doc, ["self"], |this: Float| this.is_nan());

            let doc = "Returns true if this number is neither infinite nor NaN";
            block
                .define_func::<_, bool>("is_finite", doc, ["self"], |this: Float| this.is_finite());

            let doc = "Returns true if this value is positive infinity or negative infinity, and false otherwise";
            block.define_func::<_, bool>("is_infinite", doc, ["self"], |this: Float| {
                this.is_infinite()
            });

            lib.add(block.into());
        }

        lib
    }
}

/// Option-like type for with a C-representation
///
/// This type cannot make use of niches because it uses the C-representation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptionWrapper<T> {
    // WARNING: Roto relies on the order of these variants.
    Some(T),
    None,
}

impl<T> From<Option<T>> for OptionWrapper<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(x) => Self::Some(x),
            None => Self::None,
        }
    }
}

impl<T> From<OptionWrapper<T>> for Option<T> {
    fn from(value: OptionWrapper<T>) -> Self {
        match value {
            OptionWrapper::Some(x) => Some(x),
            OptionWrapper::None => None,
        }
    }
}

/// A `Verdict` is the output of a filtermap
///
/// It is functionally equivalent to a [`Result`], but it has `repr(u8)` to
/// keep the representation synchronized with Roto.
///
/// The [`Verdict::into_result`] and [`Verdict::into_option`] methods are
/// available to map a [`Verdict`] to more conventional types.
#[repr(u8)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[must_use]
pub enum Verdict<A, R> {
    // WARNING: Roto relies on the order of these variants
    Accept(A),
    Reject(R),
}

impl<A, R> Verdict<A, R> {
    pub fn into_result(self) -> Result<A, R> {
        match self {
            Self::Accept(x) => Ok(x),
            Self::Reject(x) => Err(x),
        }
    }
}

impl<A> Verdict<A, ()> {
    pub fn into_option(self) -> Option<A> {
        match self {
            Self::Accept(x) => Some(x),
            Self::Reject(()) => None,
        }
    }
}

impl Runtime {
    /// Add functions using I/O to the runtime.
    ///
    /// These functions are disabled by default because Roto might be used in a
    /// context where using I/O is not permitted.
    ///
    /// For now, this just adds the `print` function. More functions will be
    /// added in the future.
    pub fn add_io_functions(&mut self) {
        let doc = "Print a string to stdout";
        let func = |s: Arc<str>| println!("{s}");
        let func = Function::new("print", doc, vec!["s"], func).unwrap();
        self.add(&[func.into()]).unwrap();
    }
}
