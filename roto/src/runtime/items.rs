use crate::{
    Reflect,
    ast::ident::Ident,
    runtime::{
        ConstantValue, Movability, RegistrationError, Runtime,
        func::{FunctionDescription, RegisterableFn},
        layout::Layout,
        ty::TypeDescription,
    },
};
use std::any::TypeId;

/// A registerable item
#[derive(Clone, Debug)]
pub enum Item {
    Function(Function),
    Type(Type),
    Module(Module),
    Constant(Constant),
    Impl(Impl),
    Use(Use),
}

impl From<Function> for Item {
    fn from(value: Function) -> Self {
        Self::Function(value)
    }
}

impl From<Type> for Item {
    fn from(value: Type) -> Self {
        Self::Type(value)
    }
}

impl From<Module> for Item {
    fn from(value: Module) -> Self {
        Self::Module(value)
    }
}

impl From<Constant> for Item {
    fn from(value: Constant) -> Self {
        Self::Constant(value)
    }
}

impl From<Impl> for Item {
    fn from(value: Impl) -> Self {
        Self::Impl(value)
    }
}

impl From<Use> for Item {
    fn from(value: Use) -> Self {
        Self::Use(value)
    }
}

/// A collection of registerable items.
#[derive(Clone, Debug)]
pub struct Library {
    pub items: Vec<Item>,
}

impl Library {
    /// Add an item to the [`Library`].
    pub fn add(&mut self, item: Item) {
        self.items.push(item);
    }

    #[must_use]
    pub fn with(mut self, item: impl Into<Item>) -> Self {
        self.add(item.into());
        self
    }
}

/// A module containing other items
///
/// Can be constructed with [`Module::new`].
#[derive(Clone, Debug)]
pub struct Module {
    pub(crate) ident: Ident,
    pub(crate) doc: String,
    pub(crate) children: Vec<Item>,
}

impl Module {
    /// Construct a new [`Module`].
    ///
    /// Items can be added to this module with [`Module::add`].
    ///
    /// The `name` must be a valid Rust identifier. The `doc` parameter is the
    /// docstring that will be displayed in the documentation that Roto can
    /// generate. The `location` is used for error reporting while registering
    /// this item. You should generally pass `roto::location!()` to get a correct
    /// value.
    pub fn new(name: impl Into<Ident>, doc: impl AsRef<str>) -> Result<Self, RegistrationError> {
        Ok(Self {
            ident: Runtime::check_name(name)?,
            doc: doc.as_ref().to_string(),
            children: Vec::new(),
        })
    }

    pub fn add(&mut self, item: impl Into<Item>) {
        self.children.push(item.into());
    }
}

/// A Roto type
///
/// This type will be cloned and dropped many times, so make sure to have
/// a cheap [`Clone`] and [`Drop`] implementations, for example an
/// [`Rc`](std::rc::Rc) or an [`Arc`](std::sync::Arc).
///
/// Use one of the [`Type::clone`] or [`Type::copy`] constructors to construct
/// this type. [`Type::copy`] will generally be more performant than
/// [`Type::clone`], so you should prefer that if the type implements [`Copy`].
#[derive(Clone, Debug)]
pub struct Type {
    pub(crate) ident: Ident,
    pub(crate) rust_name: &'static str,
    pub(crate) doc: String,
    pub(crate) id: TypeId,
    pub(crate) layout: Layout,
    pub(crate) movability: Movability,
}

impl Type {
    pub(crate) fn new<T: Reflect>(
        name: impl Into<Ident>,
        doc: impl AsRef<str>,
        movability: Movability,
    ) -> Result<Self, RegistrationError> {
        let ty = T::resolve();

        let is_allowed = match ty.description {
            TypeDescription::Leaf | TypeDescription::Val(_) => true,
            TypeDescription::Option(_) | TypeDescription::Verdict(_, _) => false,
        };

        if !is_allowed {
            return Err(RegistrationError::new(format!(
                "Cannot register the type `{}`. Only `Val<T>` types can be registered",
                ty.rust_name
            )));
        }

        Ok(Self {
            ident: Runtime::check_name(name)?,
            rust_name: std::any::type_name::<T>(),
            doc: doc.as_ref().into(),
            id: ty.type_id,
            layout: ty.layout,
            movability,
        })
    }
}

/// A function that can be registered.
///
/// Can be constructed with `Function::new`.
#[derive(Clone, Debug)]
pub struct Function {
    pub(crate) ident: Ident,
    pub(crate) doc: String,
    pub(crate) params: Vec<Ident>,
    pub(crate) func: FunctionDescription,
}

impl Function {
    /// Construct a new [`Function`].
    ///
    /// The function to be registered is passed as `func` and must implement
    /// [`RegisterableFn`].
    ///
    /// The `name` must be a valid Roto identifier. The `doc` parameter is the
    /// docstring that will be displayed in the documentation that Roto can
    /// generate. With `params`, you can pass the names for each of the
    /// parameters of the function, this is also used for generating
    /// documentation. The `location` is used for error reporting while
    /// registering this item. You should generally pass [`roto::location!()`]
    /// to get a correct value.
    pub fn new<A, R>(
        name: impl Into<Ident>,
        doc: impl AsRef<str>,
        params: Vec<&str>,
        func: impl RegisterableFn<A, R>,
    ) -> Result<Self, RegistrationError> {
        Ok(Self {
            ident: Runtime::check_name(name)?,
            doc: doc.as_ref().into(),
            params: params.into_iter().map(Into::into).collect(),
            func: FunctionDescription::of(func),
        })
    }
}

/// A constant value
///
/// Can be constructed with [`Constant::new`]
#[derive(Clone, Debug)]
pub struct Constant {
    pub(crate) ident: Ident,
    pub(crate) type_id: TypeId,
    pub(crate) doc: String,
    pub(crate) value: ConstantValue,
}

impl Constant {
    /// Construct a new [`Constant`].
    ///
    /// The value to be registered is passed as `val` and must implement
    /// [`Reflect`], like any registerable type.
    ///
    /// The `name` must be a valid Roto identifier. The `doc` parameter is the
    /// docstring that will be displayed in the documentation that Roto can
    /// generate. The `location` is used for error reporting while
    /// registering this item. You should generally pass [`roto::location!()`]
    /// to get a correct value.
    pub(crate) fn new<T: Reflect>(
        name: impl Into<Ident>,
        doc: impl AsRef<str>,
        val: T,
    ) -> Result<Self, RegistrationError>
    where
        T::Transformed: Send + Sync + 'static,
    {
        Ok(Self {
            ident: Runtime::check_name(name)?,
            doc: doc.as_ref().into(),
            type_id: T::resolve().type_id,
            value: ConstantValue::new(val.transform()),
        })
    }
}

/// An impl block, which adds methods to a type.
///
/// Can be constructed with [`Impl::new`].
#[derive(Clone, Debug)]
pub struct Impl {
    pub(crate) ty: TypeId,
    pub(crate) children: Vec<Item>,
}

impl Impl {
    /// Construct a new [`Impl`] block for a given type `T`.
    ///
    /// The `location` is used for error reporting while registering this item.
    /// You should generally pass [`roto::location!()`] to get a correct value.
    #[must_use]
    pub fn new<T: Reflect>() -> Self {
        Self {
            ty: T::resolve().type_id,
            children: Vec::new(),
        }
    }

    /// Add more registerable items to this [`Impl`] block.
    pub fn add(&mut self, item: impl Into<Item>) {
        self.children.push(item.into());
    }
}

/// A use item, representing an import of items.
///
/// Can be constructed with [`Use::new`].
#[derive(Clone, Debug)]
pub struct Use {
    pub(crate) imports: Vec<Vec<String>>,
}

impl Use {
    /// Construct a new [`Use`].
    ///
    /// Each element of `imports` represents a path to an item to import.
    #[must_use]
    pub fn new(imports: Vec<Vec<String>>) -> Self {
        Self { imports }
    }
}
