//! Runtime definition for Roto

pub mod basic;
pub mod docs;
pub mod func;
pub mod items;
pub mod layout;
pub mod ty;
pub mod val;

use self::func::FunctionDescription;
use self::layout::Layout;
use crate::{
    Impl, RotoReport,
    ast::{Lexer, Token, ident::Ident},
    file_tree::FileTree,
    runtime::items::{Constant, Function, Item, Module, Type, Use},
    types::{ResolvedName, ScopeRef, Typifier},
};
use std::{any::TypeId, collections::HashMap, path::Path, sync::Arc};

/// Provides the types and functions that Roto can access via FFI
///
/// Roto is an embedded language, therefore, it must be hooked up to the
/// outside world to do anything useful besides pure computation. This
/// connection is provided by the [`Runtime`], which exposes types, methods
/// and functions to Roto.
///
/// Roto can run in different [`Runtime`]s, depending on the situation.
/// Extending the default [`Runtime`] is the primary way to extend the
/// capabilities of Roto.
///
/// ## Compiling a script
///
/// - [`Runtime::compile`]
///
/// ## Registering types, functions and constants.
///
/// - [`Runtime::add`]
///
/// ## Registering context type
///
/// - [`Runtime::register_context_type`]
///
/// ## Printing documentation
///
/// - [`Runtime::print_documentation`]
#[derive(Clone)]
pub struct Runtime {
    pub(crate) type_checker: Typifier,
    types: Vec<RuntimeType>,
    functions: Vec<RuntimeFunction>,
    constants: HashMap<ResolvedName, RuntimeConstant>,
}

impl core::fmt::Debug for Runtime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Runtime").finish()
    }
}

/// Compiling a script
impl Runtime {
    /// Compile a script from a path and return the result.
    ///
    /// If the path is a file, then that file will be loaded. If the path is a
    /// directory, the directory will be scanned for modules.
    pub fn compile(&self, path: impl AsRef<Path>) -> Result<crate::codegen::Package, RotoReport> {
        FileTree::read(path)?.compile(self)
    }
}

/// Creating a [`Runtime`]
impl Runtime {
    /// A Runtime that is as empty as possible.
    ///
    /// This contains only type information for Roto primitives.
    #[must_use]
    pub fn builtin() -> Self {
        Self::from_lib(self::items::Library::default()).unwrap()
    }

    /// Create a new [`Runtime`] with a given library.
    ///
    /// This is nothing more than a convenience function around
    /// [`Runtime::new`] followed by [`Runtime::add`].
    pub fn from_lib(lib: self::items::Library) -> Result<Self, RegistrationError> {
        let mut rt = Self {
            type_checker: Typifier::new(),
            types: vec![],
            functions: vec![],
            constants: HashMap::default(),
        };
        rt.add(&lib.items)?;
        Ok(rt)
    }
}

/// Inspecting and modifying the [`Runtime`]
impl Runtime {
    #[cfg(feature = "cli")]
    pub fn cli(&self) {
        crate::cli(self);
    }
}

#[derive(Clone, Debug)]
pub enum Movability {
    /// This type is passed by value, only available for built-in types.
    Value,

    /// This type can be copied without calling clone and drop.
    Copy,

    /// This type needs a clone and drop function.
    CloneDrop(CloneDrop),
}

#[derive(Clone, Debug)]
pub struct CloneDrop {
    pub clone: unsafe extern "C" fn(*const (), *mut ()),
    pub drop: unsafe extern "C" fn(*mut ()),
}

impl CloneDrop {
    pub const fn of<T: Clone>() -> Self {
        Self {
            clone: extern_clone::<T> as _,
            drop: extern_drop::<T> as _,
        }
    }
}

unsafe extern "C" fn extern_clone<T: Clone>(from: *const (), to: *mut ()) {
    // (*to) is uninitialized so we *must* use pointer::write instead of using a pointer assignment.
    unsafe { to.cast::<T>().write(T::clone(&*from.cast::<T>())) };
}

unsafe extern "C" fn extern_drop<T>(x: *mut ()) {
    unsafe { x.cast::<T>().drop_in_place() };
}

#[derive(Clone, Debug)]
pub struct RuntimeType {
    /// The name the type can be referenced by from Roto
    name: ResolvedName,

    /// [`TypeId`] of the registered type
    ///
    /// This can be used to index into the [`TypeRegistry`]
    type_id: TypeId,

    /// Whether this type is `Copy`
    movability: Movability,

    /// Layout of the type
    layout: Layout,

    /// Docstring of the type to display in documentation
    _docstring: String,
}

impl RuntimeType {
    pub fn name(&self) -> ResolvedName {
        self.name
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn movability(&self) -> &Movability {
        &self.movability
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFunction {
    /// Name that the function can be referenced by
    pub(crate) name: ResolvedName,

    /// Description of the signature of the function
    pub(crate) func: FunctionDescription,

    /// Unique identifier for this function
    pub(crate) id: RuntimeFunctionRef,

    /// Documentation for this function
    pub(crate) doc: String,

    /// Names of the parameters of this function for generated documentation
    pub(crate) params: Vec<Ident>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeFunctionRef(pub usize);

impl core::fmt::Display for RuntimeFunctionRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl RuntimeFunction {
    pub fn get_ref(&self) -> RuntimeFunctionRef {
        self.id
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeConstant {
    pub name: ResolvedName,
    pub ty: TypeId,
    pub docstring: String,
    pub value: ConstantValue,
}

#[derive(Clone)]
pub struct ConstantValue(Arc<dyn Send + Sync + 'static>);

impl core::fmt::Debug for ConstantValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Constant")
            .field(&Arc::as_ptr(&self.0))
            .finish()
    }
}

impl ConstantValue {
    pub fn new<T: Send + Sync + 'static>(x: T) -> Self {
        Self(Arc::new(x))
    }

    pub fn ptr(&self) -> *const () {
        Arc::as_ptr(&self.0).cast::<()>()
    }
}

/// An error that arose while registering items from a library
#[derive(Clone, thiserror::Error)]
#[error("Registration Error:\n\t{message}")]
pub struct RegistrationError {
    message: String,
}

impl RegistrationError {
    fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self { message }
    }
}

impl core::fmt::Debug for RegistrationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

impl Runtime {
    /// Add a library of items to this [`Runtime`]
    ///
    /// However, it is -- with a bit of extra boilerplate -- also possible to
    /// register items without using the macro. You can directly register one
    /// of the item types:
    ///
    ///  - [`Module`]
    ///  - [`Type`]
    ///  - [`Function`]
    ///  - [`Constant`]
    ///  - [`Impl`]
    ///  - [`Use`]
    ///
    /// Or you can register an [`Item`] which combines all the above.
    /// Additionally, you can register collections of these types, such
    /// as `Vec<Item>` or `[Item; N]`.
    ///
    /// See also [`Runtime::from_lib`], which combines [`Runtime::new`] and
    /// [`Runtime::add`] into a single function.
    pub fn add(&mut self, items: &[Item]) -> Result<(), RegistrationError> {
        let root = ScopeRef::GLOBAL;
        self.declare_modules(None, items)?;
        self.declare_types(root, items)?;
        self.declare_functions(root, items)?;
        self.declare_constants(root, items)?;
        self.declare_imports(root, items)?;
        Ok(())
    }

    /// Get the registered types.
    #[must_use]
    pub fn types(&self) -> &[RuntimeType] {
        &self.types
    }

    /// Get the registered functions.
    #[must_use]
    pub fn functions(&self) -> &[RuntimeFunction] {
        &self.functions
    }

    #[must_use]
    pub fn get_function(&self, f: RuntimeFunctionRef) -> &RuntimeFunction {
        &self.functions[f.0]
    }

    /// Get the registered constants.
    #[must_use]
    pub fn constants(&self) -> &HashMap<ResolvedName, RuntimeConstant> {
        &self.constants
    }
}

impl Runtime {
    fn declare_modules(
        &mut self,
        scope: Option<ScopeRef>,
        items: &[Item],
    ) -> Result<(), RegistrationError> {
        for item in items {
            if let Item::Module(module) = item {
                self.declare_module(scope, module)?;
            }
        }
        Ok(())
    }

    fn declare_module(
        &mut self,
        scope: Option<ScopeRef>,
        module: &Module,
    ) -> Result<(), RegistrationError> {
        let scope = self
            .type_checker
            .declare_runtime_module(scope, module.ident, module.doc.clone())
            .map_err(RegistrationError::new)?;

        self.declare_modules(Some(scope), &module.children)?;

        Ok(())
    }

    fn declare_types(&mut self, scope: ScopeRef, items: &[Item]) -> Result<(), RegistrationError> {
        for item in items {
            match item {
                Item::Module(module) => {
                    let scope = self.type_checker.scope_of(scope, module.ident).unwrap();
                    self.declare_types(scope, &module.children)?;
                }
                Item::Type(ty) => self.declare_type(scope, ty)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn declare_type(&mut self, scope: ScopeRef, ty: &Type) -> Result<(), RegistrationError> {
        if let Some(old_ty) = self.types.iter().find(|old_ty| old_ty.type_id == ty.id) {
            // TODO: Print scope in this error message
            let (a, b) = (ty.rust_name, old_ty.name.ident);
            return Err(RegistrationError::new(format!(
                "Type {a} is already registered under a different name: {b}`"
            )));
        }

        self.type_checker
            .declare_runtime_type(scope, ty.ident, ty.id, ty.doc.clone())
            .map_err(RegistrationError::new)?;

        let name = ResolvedName::new(scope, ty.ident);

        self.types.push(RuntimeType {
            name,
            type_id: ty.id,
            movability: ty.movability.clone(),
            layout: ty.layout,
            _docstring: ty.doc.clone(),
        });

        Ok(())
    }

    fn declare_functions(
        &mut self,
        scope: ScopeRef,
        items: &[Item],
    ) -> Result<(), RegistrationError> {
        for item in items {
            match item {
                Item::Module(Module {
                    ident, children, ..
                }) => {
                    let scope = self.type_checker.scope_of(scope, *ident).unwrap();
                    self.declare_functions(scope, children)?;
                }
                Item::Function(f) => self.declare_function(scope, f, false)?,
                Item::Impl(items::Impl { ty, children }) => {
                    let ty = self
                        .types
                        .iter()
                        .find(|t| t.type_id == *ty)
                        .ok_or_else(|| {
                            RegistrationError::new("Impl block with unregistered type")
                        })?;

                    let scope = ty.name.scope;
                    let ident = ty.name.ident;
                    let scope = self.type_checker.scope_of(scope, ident).unwrap();
                    self.declare_methods(scope, children)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn declare_methods(
        &mut self,
        scope: ScopeRef,
        items: &[Item],
    ) -> Result<(), RegistrationError> {
        for item in items {
            match item {
                Item::Function(f) => self.declare_function(scope, f, true)?,
                Item::Impl(_) => {
                    return Err(RegistrationError::new("Cannot nest an impl in an impl"));
                }
                Item::Type(_) => {
                    return Err(RegistrationError::new("Cannot nest a type in an impl"));
                }
                Item::Module(_) => {
                    return Err(RegistrationError::new("Cannot nest a module in an impl"));
                }
                Item::Use(_) | Item::Constant(_) => {}
            }
        }

        Ok(())
    }

    fn declare_function(
        &mut self,
        scope: ScopeRef,
        f: &Function,
        method: bool,
    ) -> Result<(), RegistrationError> {
        let ident = Self::check_name(f.ident)?;

        let parameter_types: Vec<_> = f
            .func
            .parameter_types()
            .iter()
            .map(|ty| self.rust_type_to_roto_type(*ty))
            .collect::<Result<_, _>>()?;

        let return_type = self.rust_type_to_roto_type(f.func.return_type())?;

        let id = RuntimeFunctionRef(self.functions.len());
        let func = RuntimeFunction {
            name: ResolvedName::new(scope, ident),
            id,
            func: f.func.clone(),
            doc: f.doc.clone(),
            params: f.params.clone(),
        };
        self.functions.push(func);

        self.type_checker
            .declare_runtime_function(
                scope,
                ident,
                id,
                f.params.clone(),
                parameter_types,
                return_type,
                f.doc.clone(),
                method,
            )
            .map_err(RegistrationError::new)?;

        Ok(())
    }

    fn declare_constants(
        &mut self,
        scope: ScopeRef,
        items: &[Item],
    ) -> Result<(), RegistrationError> {
        for item in items {
            match item {
                Item::Module(module) => {
                    let scope = self.type_checker.scope_of(scope, module.ident).unwrap();
                    self.declare_constants(scope, &module.children)?;
                }
                Item::Impl(Impl { ty, children }) => {
                    let ty = self
                        .types
                        .iter()
                        .find(|t| t.type_id == *ty)
                        .ok_or_else(|| {
                            RegistrationError::new("Impl block with unregistered type")
                        })?;

                    let scope = ty.name.scope;
                    let ident = ty.name.ident;
                    let scope = self.type_checker.scope_of(scope, ident).unwrap();
                    self.declare_constants(scope, children)?;
                }
                Item::Constant(c) => self.declare_constant(scope, c)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn declare_constant(
        &mut self,
        scope: ScopeRef,
        constant: &Constant,
    ) -> Result<(), RegistrationError> {
        let ty = self.rust_type_to_roto_type(constant.type_id)?;

        self.type_checker
            .declare_runtime_constant(scope, constant.ident, ty, constant.doc.clone())
            .map_err(RegistrationError::new)?;

        let name = ResolvedName::new(scope, constant.ident);

        self.constants.insert(
            name,
            RuntimeConstant {
                name,
                ty: constant.type_id,
                docstring: constant.doc.clone(),
                value: constant.value.clone(),
            },
        );

        Ok(())
    }

    fn declare_imports(
        &mut self,
        scope: ScopeRef,
        items: &[Item],
    ) -> Result<(), RegistrationError> {
        for item in items {
            match item {
                Item::Use(use_item) => self.declare_import(scope, use_item)?,
                Item::Module(module) => self.declare_imports(scope, &module.children)?,
                _ => (),
            }
        }
        Ok(())
    }

    fn declare_import(&mut self, scope: ScopeRef, use_item: &Use) -> Result<(), RegistrationError> {
        for import in &use_item.imports {
            let mut new_scope = scope;
            let path = &import[..import.len() - 1];
            let last = &import[import.len() - 1];
            for part in path {
                new_scope = self
                    .type_checker
                    .scope_of(scope, part.into())
                    .ok_or_else(|| {
                        RegistrationError::new(format!("Could not get scope of {part}"))
                    })?;
            }
            self.type_checker
                .declare_runtime_import(scope, ResolvedName::new(new_scope, last))
                .map_err(RegistrationError::new)?;
        }
        Ok(())
    }

    fn rust_type_to_roto_type(
        &self,
        type_id: TypeId,
    ) -> Result<crate::types::Type, RegistrationError> {
        Typifier::rust_type_to_roto_type(self, type_id).map_err(RegistrationError::new)
    }

    pub(crate) fn runtime_type(&self, id: TypeId) -> Option<&RuntimeType> {
        self.types.iter().find(|ty| ty.type_id == id)
    }

    /// Check that the given string is a valid Roto identifier
    #[must_use = ""]
    fn check_name(name: impl Into<Ident>) -> Result<Ident, RegistrationError> {
        Self::check_name_internal(name.into()).map_err(RegistrationError::new)
    }

    fn check_name_internal(name: Ident) -> Result<Ident, String> {
        let mut lexer = Lexer::new(name.as_str());
        let Some((Ok(tok), _)) = lexer.next() else {
            return Err(format!("Name {name:?} is not a valid identifier"));
        };

        if lexer.next().is_some() {
            return Err(format!(
                "Name {name:?} contains multiple tokens and is not a valid identifier"
            ));
        }

        match tok {
            Token::Ident(_) => Ok(name),
            Token::Keyword(_) => Err(format!(
                "Name {name:?} is a keyword and therefore not a valid identifier"
            )),
            _ => Err(format!("Name {name:?} is not a valid identifier.")),
        }
    }
}
