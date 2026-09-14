//! Type checking scopes

use super::{EnumVariant, FunctionDefinition, Type, TypeDef};
use crate::{ast, ice};
use std::collections::btree_map::BTreeMap;

/// A reference to a [`Scope`] in a [`ScopeGraph`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeRef(pub(super) usize);

impl ScopeRef {
    /// The scope at the root of a [`ScopeGraph`]
    pub const GLOBAL: Self = Self(0);
}

/// Combination of a scope and an identifier
///
/// This forms a unique name for an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedName {
    pub scope: ScopeRef,
    pub ident: ast::ident::Ident,
}

impl ResolvedName {
    pub fn new(scope: ScopeRef, ident: impl Into<ast::ident::Ident>) -> Self {
        Self {
            scope,
            ident: ident.into(),
        }
    }

    pub fn global(ident: impl Into<ast::ident::Ident>) -> Self {
        Self::new(ScopeRef::GLOBAL, ident)
    }
}

/// A declaration in a [`ScopeGraph`]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Declaration {
    pub name: ResolvedName,
    pub kind: DeclKind,
    pub id: ast::MetaId,
    pub scope: Option<ScopeRef>,
    pub doc: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeclKind {
    Value(ValueKind, Type),
    Type(TypeOrStub),
    Function(Option<FunctionDeclaration>),
    Module,
    Method(Option<FunctionDeclaration>),
    Variant(Option<(TypeDef, EnumVariant)>),
    TypeParam(ast::ident::Ident),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionDeclaration {
    pub def: FunctionDefinition,
    pub params: Vec<ast::ident::Ident>,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeOrStub {
    Type(TypeDef),
    Stub { num_params: usize },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Local,
    Constant,
}

#[derive(Clone)]
pub struct ScopeGraph {
    pub(super) declarations: BTreeMap<ResolvedName, Declaration>,
    pub(super) scopes: Vec<Scope>,
}

/// A type checking scope
#[derive(Clone)]
pub(super) struct Scope {
    pub kind: ScopeType,
    pub parent: Option<ScopeRef>,
    pub imports: BTreeMap<ast::ident::Ident, (ast::MetaId, ResolvedName)>,
}

/// The syntactic structure that a scope represents
///
/// This is used primarily for printing a roughly human-readable name.
#[derive(Clone)]
pub enum ScopeType {
    Root,
    Then(usize),
    Else(usize),
    WhileBody(usize),
    Module(ModuleScope),
    Function(ast::ident::Ident),
    MatchArm(usize, Option<usize>),
    Type(ast::ident::Ident),
    TypeParams,
}

#[derive(Clone)]
pub struct ModuleScope {
    pub name: ResolvedName,
    pub parent_module: Option<ScopeRef>,
}

impl ScopeGraph {
    /// Create a new scope over `scope`
    pub fn wrap(&mut self, parent: impl Into<ScopeRef>, kind: ScopeType) -> ScopeRef {
        let scope = ScopeRef(self.scopes.len());
        self.scopes.push(Scope {
            kind,
            parent: Some(parent.into()),
            imports: BTreeMap::new(),
        });
        scope
    }

    pub fn declarations_in(&self, scope: ScopeRef) -> impl Iterator<Item = &Declaration> {
        self.declarations
            .iter()
            .filter(move |(n, _)| n.scope == scope)
            .map(|(_, d)| d)
    }

    pub fn parent(&self, scope: ScopeRef) -> Option<ScopeRef> {
        self.scopes[scope.0].parent
    }

    pub fn resolve_name(
        &self,
        mut scope: ScopeRef,
        ident: &ast::Ident,
        recurse: bool,
    ) -> Option<Declaration> {
        loop {
            let name = ResolvedName::new(scope, **ident);
            if let Some(d) = self.declarations.get(&name) {
                return Some(d.clone());
            }

            if !recurse {
                return None;
            }

            if let Some(x) = self.scopes[scope.0].imports.get(ident) {
                return Some(self.declarations.get(&x.1).unwrap().clone());
            }

            scope = self.parent(scope)?;
        }
    }

    pub fn get_declaration(&self, name: ResolvedName) -> Declaration {
        let Some(dec) = self.declarations.get(&name) else {
            ice!("Could not get declaration: {}", name.ident);
        };
        dec.clone()
    }

    pub fn get_declaration_mut(&mut self, name: ResolvedName) -> &mut Declaration {
        let Some(dec) = self.declarations.get_mut(&name) else {
            ice!("Could not get declaration: {}", name.ident);
        };
        dec
    }

    pub fn parent_module(&self, mut scope: ScopeRef) -> Option<Declaration> {
        loop {
            let s = &self.scopes[scope.0];

            if let ScopeType::Module(m) = &s.kind {
                let ScopeType::Module(parent) = &self.scopes[m.parent_module?.0].kind else {
                    unreachable!();
                };
                return Some(self.get_declaration(parent.name));
            }

            scope = self.parent(scope)?;
        }
    }
}

impl Default for ScopeGraph {
    fn default() -> Self {
        Self {
            declarations: BTreeMap::new(),
            scopes: vec![Scope {
                kind: ScopeType::Root,
                parent: None,
                imports: BTreeMap::new(),
            }],
        }
    }
}

impl ScopeGraph {
    pub fn module_name(&self, m: &ModuleScope) -> String {
        let mut idents = Vec::new();

        let mut m = Some(m);
        while let Some(current) = m {
            idents.push(current.name.ident.to_string());
            m = current.parent_module.map(|idx| {
                let ScopeType::Module(m) = &self.scopes[idx.0].kind else {
                    panic!();
                };
                m
            });
        }

        idents.reverse();
        idents.join(".")
    }

    pub fn print_scope(&self, scope: ScopeRef) -> String {
        let mut idents = Vec::new();
        let mut scope = Some(scope);
        while let Some(s) = scope {
            let s = &self.scopes[s.0];
            let ident = match &s.kind {
                ScopeType::Root => break,
                ScopeType::Module(m) => self.module_name(m),
                ScopeType::Function(fn_name) => fn_name.as_str().to_string(),
                ScopeType::Then(idx) => format!("$if_{idx}_then"),
                ScopeType::Else(idx) => format!("$if_{idx}_else"),
                ScopeType::MatchArm(idx, Some(arm)) => format!("$match_{idx}_arm_{arm}"),
                ScopeType::MatchArm(idx, None) => format!("$match_{idx}_arm_default"),

                ScopeType::WhileBody(idx) => format!("$loop_body_{idx}"),
                ScopeType::Type(ty_name) => ty_name.as_str().to_string(),
                ScopeType::TypeParams => "$type_params".into(),
            };
            idents.push(ident);
            scope = s.parent;
        }
        idents.reverse();
        idents.join(".")
    }
}
