use bevy::reflect::{FromType, TypeData};
use std::any::Any;
use std::collections::{HashMap, VecDeque};

/// Descriptor of a path into a struct/enum. Either a `Field` (`.foo`) or a `VariantField` (`RGBA.r`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
enum Target {
    Field(usize),
    VariantField(usize, usize),
}

/// Map of [`Target`]s to arbitrary [`TypeData`] used to control how the value is displayed, e.g. [`NumberOptions`](crate::inspector_options::std_options::NumberOptions).
///
/// Comes with a [derive macro](derive@Options), which generates a `FromType<T> for Options` impl:
/// ```rust
/// use bevy_inspector_egui::prelude::*;
/// use bevy_reflect::Reflect;
///
/// #[derive(Reflect, Default, Options)]
/// #[reflect(Options)]
/// struct Config {
///     #[inspector(min = 10.0, max = 70.0)]
///     font_size: f32,
///     option: Option<f32>,
/// }
/// ```
/// will expand roughly to
/// ```rust
/// # use bevy_inspector_egui::inspector_options::{Options, Target, std_options::NumberOptions};
/// let mut options = Options::default();
/// let mut field_options = NumberOptions::default();
/// field_options.min = 10.0.into();
/// field_options.max = 70.0.into();
/// options.insert(Target::Field(0usize), field_options);
/// ```
#[derive(Default)]
pub struct Options {
    options: HashMap<Target, Box<dyn TypeData>>,
}

impl std::fmt::Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut options = f.debug_struct("Options");
        for entry in self.options.keys() {
            options.field(&format!("{entry:?}"), &"..");
        }
        options.finish()
    }
}

impl Clone for Options {
    fn clone(&self) -> Self {
        let options = self.options.iter();
        let options = options.map(|(&target, data)| (target, TypeData::clone_type_data(&**data)));
        let options = options.collect();
        Self { options }
    }
}

impl Options {
    pub fn field(&mut self, field: usize, options: Box<dyn TypeData>) {
        let target = Target::Field(field);
        self.options.insert(target, options);
    }
    pub fn variant_field(&mut self, variant: usize, field: usize, options: Box<dyn TypeData>) {
        let target = Target::VariantField(variant, field);
        self.options.insert(target, options);
    }

    fn insert(&mut self, target: Target, options: Box<dyn TypeData>) {
        self.options.insert(target, options);
    }

    fn get(&self, target: Target) -> Option<&dyn Any> {
        self.options.get(&target).map(|value| value.as_any())
    }

    fn _iter(&self) -> impl Iterator<Item = (Target, &dyn TypeData)> + '_ {
        self.options.iter().map(|(&target, data)| (target, &**data))
    }
}

#[derive(Clone, Copy)]
pub struct AnyOptions<'a>(pub(super) &'a dyn Any);

impl<'a> From<&'a dyn Any> for AnyOptions<'a> {
    fn from(value: &'a dyn Any) -> Self {
        Self(value)
    }
}

impl<'a> AnyOptions<'a> {
    pub const EMPTY: AnyOptions<'static> = AnyOptions(&());

    pub fn new<T: Any + Clone + Default>(value: &'a T) -> Self {
        Self(value)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is::<()>()
    }

    pub fn downcast_ref<T: Any + Clone + Default>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }

    pub fn downcast_or_default<T: Any + Clone + Default>(&self) -> T {
        self.0.downcast_ref::<T>().cloned().unwrap_or_default()
    }

    pub fn field(self, field: usize) -> Self {
        self.get(Target::Field(field))
    }

    pub fn variant_field(self, variant_index: usize, field_index: usize) -> Self {
        self.get(Target::VariantField(variant_index, field_index))
    }

    fn get(self, target: Target) -> Self {
        self.0
            .downcast_ref::<Options>()
            .and_then(|opt| opt.get(target))
            .map(Self)
            .unwrap_or(Self::EMPTY)
    }
}

/// Wrapper of [`struct@Options`] to be stored in the [`TypeRegistry`](bevy_reflect::TypeRegistry)
#[derive(Clone)]
pub struct ReflectOptions(pub Options);

impl<T> FromType<T> for ReflectOptions
where
    Options: FromType<T>,
{
    fn from_type() -> Self {
        Self(Options::from_type())
    }
}

/// Helper trait for the [`derive@Options`] macro.
///
/// ```skip
///     #[inspector(min = 0.0, max = 1.0)]
///     field: f32
/// ```
/// will expand to this:
/// ```rust
/// # use std::convert::Into;
/// # use bevy_inspector_egui::inspector_options::{Options, OptionsType, Target};
/// let mut options = Options::default();
/// let mut field_options =  <f32 as OptionsType>::DeriveOptions::default();
/// field_options.min = Into::into(2.0);
/// field_options.max = Into::into(3.0);
/// options.insert(
///     Target::Field(0usize),
///     <f32 as OptionsType>::options_from_derive(field_options),
/// );
pub trait OptionsType {
    type Derive: Default;
    /// Can be arbitrary types which will be passed to [`InspectorImpl`](crate::inspector_egui_impls::InspectorImpl) like [`NumberOptions`](crate::inspector_options::std_options::NumberOptions),
    /// or nested [`struct@Options`] which will be passed to children (see [`impl OptionsType for Option`](trait.OptionsType.html#impl-OptionsType-for-Option<T>)).
    type Options: TypeData;

    fn options_from_derive(derive: Self::Derive) -> Self::Options;
}

impl<T: OptionsType> OptionsType for Option<T> {
    type Derive = T::Derive;
    type Options = Options;

    fn options_from_derive(derive: Self::Derive) -> Self::Options {
        let target = Target::VariantField(1, 0); // Some;
        let mut options = Options::default();
        options.insert(target, Box::new(T::options_from_derive(derive)));
        options
    }
}

impl<T: OptionsType> OptionsType for Vec<T> {
    type Derive = T::Derive;
    type Options = T::Options;

    fn options_from_derive(derive: Self::Derive) -> Self::Options {
        T::options_from_derive(derive)
    }
}

impl<T: OptionsType> OptionsType for VecDeque<T> {
    type Derive = T::Derive;
    type Options = T::Options;

    fn options_from_derive(derive: Self::Derive) -> Self::Options {
        T::options_from_derive(derive)
    }
}

impl<T: OptionsType, const N: usize> OptionsType for [T; N] {
    type Derive = T::Derive;
    type Options = T::Options;

    fn options_from_derive(derive: Self::Derive) -> Self::Options {
        T::options_from_derive(derive)
    }
}
