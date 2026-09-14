//! Type checker for Roto scripts
//!
//! This type checker performs simplified Hindley-Milner type inference.
//! The simplification is that we only have one situation in which general
//! type variables are generated: method instantiation. Otherwise, we do
//! not have to deal with polymorphism at all. However, we might still
//! extend the type system later to accommodate for that.
//!
//! The current implementation is based on Algorithm M, described in
//! <https://dl.acm.org/doi/pdf/10.1145/291891.291892>. The advantage of
//! Algorithm M over the classic Algorithm W is that it yields errors
//! earlier in the type inference, so that errors appear more often where
//! they are caused.
//!
//! See also <https://en.wikipedia.org/wiki/Hindley%E2%80%93Milner_type_system>.
//!
//! # Type checking steps
//!
//! There are several type checking steps that happen at each compilation. The
//! order of these steps is fixed for a variety of reasons.
//!
//! ## Declaring built-in types
//!
//! We start by declaring all the built-in types, since these need to be
//! available for things in the runtime and in the script. This includes
//! primitives (`bool`, `u32`, etc.), `Option[T]`, `Verdict[A, R]` and
//! things like that.
//!
//! The methods of these types are also declared.
//!
//! See [`TypeChecker::declare_builtin_types`].
//!
//! ## Declaring runtime types and functions
//!
//! After the built-in types, we move on to the runtime types, methods
//! and functions. These can reference built-in types. We now have everything
//! we need up front, so we can move on to type checking the script itself.
//!
//! See [`TypeChecker::declare_runtime_items`].
//!
//! ## Declaring modules
//!
//! We first determine the general structure of the script. Meaning that we
//! build the scope tree for the modules and add a [`Declaration`] for
//! the items in it. At this stage, we do not have all the information to
//! resolve the contents of each declaration, so each [`Declaration`] only
//! contains the minimal information we need for name resolution.
//!
//! See [`TypeChecker::declare_modules`].
//!
//! ## Resolving imports
//!
//! With the modules in place, it's possible to resolve the module-level imports
//! in script. An important detail is that the order of imports does not impact
//! the semantics of the script. Therefore, we keep trying to resolve each of them
//! until we either have none left or we can't resolve further.
//!
//! See [`TypeChecker::declare_imports`].
//!
//! ## Declaring types
//!
//! The full structure for name resolution is now in place, which means we can
//! start filling in the each [`Declaration`] we found before and add its
//! with its actual definition. We start with the types declared in the script.
//!
//! See [`TypeChecker::declare_types`].
//!
//! ## Detecting type cycles
//!
//! It's important to ensure that types are not recursive, because a recursive
//! type has an infinite size, so we have an additional check for this.
//!
//! See [`detect_type_cycles`].
//!
//! ## Declaring functions
//!
//! Since functions can call each other, we first declare each function and
//! its type without type checking its body.
//!
//! See [`TypeChecker::declare_functions`].
//!
//! ## Type checking function bodies
//!
//! We can now type check the actual expressions in the script.
//!
//! See [`TypeChecker::tree`]
//!
//! ## Force filtermap types to unit
//!
//! Some filtermaps only `accept` or `reject`, leaving the other type
//! undetermined. We force those to the unit type.
//!
//! See [`TypeChecker::force_filtermap_types`]
//!
//! [`Declaration`]: scope::Declaration

mod cycle;
mod error;
mod expr;
mod function;
mod info;
mod scope;
mod typifier;

pub use self::error::{Level, TypeError};
pub use self::expr::{PathValue, ResolvedPath};
pub use self::info::TypeInfo;
pub use self::scope::{
    DeclKind, Declaration, ModuleScope, ResolvedName, ScopeGraph, ScopeRef, ScopeType, TypeOrStub,
    ValueKind,
};
pub use self::typifier::{TypeResult, Typifier, typecheck};

use crate::{
    ast,
    runtime::{RuntimeFunctionRef, layout::Layout},
};
use std::{any::TypeId, borrow::Borrow};

/// Whether an integer type must be signed
///
/// We have to track this because the unary minus operator only allows signed integers.
/// So, if we find a unary minus with an `IntVar` as argument, we set `MustBySigned` to `Yes`.
/// If `MustBeSigned` is set to `Yes`, it will only unify with signed integer types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MustBeSigned {
    Yes,
    No,
}

/// Types that the type checker deals with
///
/// This might represent unconcrete types.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Never,
    Var(usize),
    ExplicitVar(ast::ident::Ident),
    IVar(usize, MustBeSigned),
    FVar(usize),
    RecordVar(usize, Vec<(ast::Ident, Self)>),
    Record(Vec<(ast::Ident, Self)>),
    Function(Vec<Self>, Box<Self>),
    Name(TypeName),
}

impl Type {
    pub fn unit() -> Self {
        Self::Unit
    }

    pub fn bool() -> Self {
        Self::named("bool", Vec::new())
    }

    pub fn discriminant() -> Self {
        Self::named("u8", Vec::new())
    }

    pub fn ivar() -> Self {
        Self::named("i32", Vec::new())
    }

    pub fn fvar() -> Self {
        Self::named("f64", Vec::new())
    }

    pub fn string() -> Self {
        Self::named("String", Vec::new())
    }

    pub fn verdict(a: impl Borrow<Self>, b: impl Borrow<Self>) -> Self {
        Self::named("Verdict", vec![a.borrow().clone(), b.borrow().clone()])
    }

    pub fn option(t: impl Borrow<Self>) -> Self {
        Self::named("Option", vec![t.borrow().clone()])
    }

    pub fn list(t: impl Borrow<Self>) -> Self {
        Self::named("List", vec![t.borrow().clone()])
    }

    /// Create a named type in the global scope
    pub fn named(ident: impl Into<ast::ident::Ident>, args: Vec<Self>) -> Self {
        let name = ResolvedName::global(ident);
        Self::Name(TypeName { name, args })
    }

    pub fn substitute(&self, from: &Self, to: &Self) -> Self {
        if self == from {
            return to.clone();
        }

        let f = |x: &Self| x.substitute(from, to);

        match self {
            Self::RecordVar(x, fields) => {
                Self::RecordVar(*x, fields.iter().map(|(n, t)| (n.clone(), f(t))).collect())
            }
            Self::Record(fields) => {
                Self::Record(fields.iter().map(|(n, t)| (n.clone(), f(t))).collect())
            }
            Self::Name(name) => Self::Name(name.substitute(from, to)),
            other => other.clone(),
        }
    }

    pub fn substitute_iter<'a>(&self, iter: impl Iterator<Item = (&'a Self, &'a Self)>) -> Self {
        iter.fold(self.clone(), |me, (from, to)| me.substitute(from, to))
    }
}

/// A definition of a named type
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeDef {
    Bool,
    Scalar(Num),
    String,
    Enum(TypeName, Vec<EnumVariant>),
    Struct(TypeName, Vec<(ast::Ident, Type)>),
    Runtime(ResolvedName, TypeId),
}

impl TypeDef {
    pub const fn is_sint(&self) -> bool {
        matches!(self, Self::Scalar(Num::I8 | Num::I16 | Num::I32 | Num::I64))
    }

    pub const fn is_uint(&self) -> bool {
        matches!(self, Self::Scalar(Num::U8 | Num::U16 | Num::U32 | Num::U64))
    }

    pub const fn is_int(&self) -> bool {
        self.is_sint() || self.is_uint()
    }

    pub const fn is_float(&self) -> bool {
        matches!(self, Self::Scalar(Num::F32 | Num::F64))
    }

    pub fn type_parameters(&self) -> usize {
        match self {
            Self::Enum(name, _) | Self::Struct(name, _) => name.args.len(),
            Self::Runtime(..) | Self::Scalar(..) | Self::Bool | Self::String => 0,
        }
    }

    /// Get the type name that belongs to this type definition
    pub fn type_name(&self) -> TypeName {
        match self {
            Self::Enum(name, _) | Self::Struct(name, _) => name.clone(),
            &Self::Runtime(name, _) => TypeName {
                name,
                args: Vec::new(),
            },
            Self::Bool => TypeName {
                name: ResolvedName::global("bool"),
                args: Vec::new(),
            },
            Self::String => TypeName {
                name: ResolvedName::global("String"),
                args: Vec::new(),
            },
            Self::Scalar(primitive) => TypeName {
                name: ResolvedName::global(primitive.to_string()),
                args: Vec::new(),
            },
        }
    }

    /// Instantiate the type definition with fresh type variables
    pub fn instantiate(&self, fresh_var: impl FnMut() -> Type) -> Type {
        self.type_name().instantiate(fresh_var)
    }

    /// Get the match patterns for this type definition instantiated with the
    /// given type arguments.
    pub fn match_patterns(&self, type_args: &[Type]) -> Option<Vec<EnumVariant>> {
        let TypeDef::Enum(type_name, variants) = self else {
            return None;
        };

        assert_eq!(type_name.args.len(), type_args.len());

        let subs = type_name.args.iter().zip(type_args);

        let mut new_variants = Vec::new();
        for variant in variants {
            new_variants.push(variant.substitute_iter(subs.clone()));
        }
        Some(new_variants)
    }

    pub fn record_fields(&self, type_args: &[Type]) -> Option<Vec<(ast::Ident, Type)>> {
        let TypeDef::Struct(type_name, fields) = self else {
            return None;
        };

        assert_eq!(type_name.args.len(), type_args.len());

        let subs = type_name.args.iter().zip(type_args);

        Some(
            fields
                .iter()
                .map(|(ident, ty)| (ident.clone(), ty.substitute_iter(subs.clone())))
                .collect(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnumVariant {
    pub name: ast::ident::Ident,
    pub fields: Vec<Type>,
}

impl EnumVariant {
    pub fn substitute_iter<'a>(
        &self,
        subs: impl Iterator<Item = (&'a Type, &'a Type)> + Clone,
    ) -> Self {
        let fields = self
            .fields
            .iter()
            .map(|t| t.substitute_iter(subs.clone()))
            .collect();
        Self {
            name: self.name,
            fields,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub parameter_types: Vec<Type>,
    pub return_type: Type,
}

/// The definition of a function from several different sources.
///
/// This is used to extract the function pointer and any other information
/// required to generate the code to call this function.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FunctionDefinition {
    Runtime(RuntimeFunctionRef),
    Roto,
}

/// A function that can be called from Roto
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    /// The type signature of this function
    pub signature: Signature,

    /// Function name
    pub name: ResolvedName,

    /// Type variables of this function
    pub vars: Vec<ast::ident::Ident>,

    /// The source of this function
    pub definition: FunctionDefinition,
}

impl Function {
    pub fn new(
        name: ResolvedName,
        vars: &[ast::ident::Ident],
        signature: Signature,
        definition: FunctionDefinition,
    ) -> Self {
        Self {
            name,
            vars: vars.to_vec(),
            signature,
            definition,
        }
    }
}

/// The list of built-in Roto types
pub fn default_types() -> Vec<(ast::ident::Ident, String, TypeDef)> {
    struct VariantType {
        name: &'static str,
        doc: &'static str,
        params: Vec<&'static str>,
        variants: Vec<(&'static str, Vec<Type>)>,
    }

    let primitives = [
        Num::U8,
        Num::U16,
        Num::U32,
        Num::U64,
        Num::I8,
        Num::I16,
        Num::I32,
        Num::I64,
        Num::F32,
        Num::F64,
    ];

    let mut types = Vec::new();

    types.push((
        ast::ident::Ident::from("String"),
        String::new(),
        TypeDef::String,
    ));

    types.push((
        ast::ident::Ident::from("bool"),
        String::new(),
        TypeDef::Bool,
    ));

    for p in primitives {
        types.push((
            ast::ident::Ident::from(p.to_string()),
            String::new(),
            TypeDef::Scalar(p),
        ));
    }

    let compound_types = [
        VariantType {
            name: "Option",
            doc: "An optional value.",
            params: vec!["T"],
            variants: vec![
                ("Some", vec![Type::ExplicitVar("T".into())]),
                ("None", vec![]),
            ],
        },
        VariantType {
            name: "Verdict",
            doc: "The verdict that a filter reaches about a value, that is, whether to accept or reject it.",
            params: vec!["A", "R"],
            variants: vec![
                ("Accept", vec![Type::ExplicitVar("A".into())]),
                ("Reject", vec![Type::ExplicitVar("R".into())]),
            ],
        },
    ];

    for VariantType {
        name,
        doc,
        params,
        variants,
    } in compound_types
    {
        let ident = ast::ident::Ident::from(name);
        let name = ResolvedName::global(ident);

        let params: Vec<_> = params.into_iter().map(ast::ident::Ident::from).collect();
        let args = params.iter().map(|p| Type::ExplicitVar(*p)).collect();

        let variants = variants
            .into_iter()
            .map(|(name, fields)| EnumVariant {
                name: ast::ident::Ident::from(name),
                fields,
            })
            .collect();

        let ty = TypeDef::Enum(TypeName { name, args }, variants);
        types.push((ident, doc.into(), ty));
    }

    types
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Num {
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
}

impl core::fmt::Display for Num {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
        }
    }
}

impl Num {
    pub const IVAR: Self = Self::I32;
    pub const FVAR: Self = Self::F64;

    pub const fn is_sint(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64)
    }

    pub const fn is_uint(self) -> bool {
        matches!(self, Self::U8 | Self::U16 | Self::U32 | Self::U64)
    }

    pub const fn is_int(self) -> bool {
        self.is_sint() || self.is_uint()
    }

    pub const fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }

    /// Layout of the primitive type
    ///
    /// This gives access to the size and alignment
    pub const fn layout(self) -> Layout {
        match self {
            Self::U8 | Self::I8 => Layout::of::<i8>(),
            Self::U16 | Self::I16 => Layout::of::<i16>(),
            Self::U32 | Self::I32 | Self::F32 => Layout::of::<i32>(),
            Self::U64 | Self::I64 | Self::F64 => Layout::of::<i64>(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeName {
    pub name: ResolvedName,
    pub args: Vec<Type>,
}

impl TypeName {
    fn substitute(&self, from: &Type, to: &Type) -> Self {
        Self {
            name: self.name,
            args: self.args.iter().map(|x| x.substitute(from, to)).collect(),
        }
    }

    /// Instantiate a type name with fresh type variables
    pub fn instantiate(&self, mut fresh_var: impl FnMut() -> Type) -> Type {
        let &Self {
            name,
            args: ref arguments,
        } = self;

        let arguments = arguments
            .iter()
            .cloned()
            .map(|a| {
                if matches!(a, Type::ExplicitVar(_)) {
                    fresh_var()
                } else {
                    a
                }
            })
            .collect();

        Type::Name(Self {
            name,
            args: arguments,
        })
    }
}
