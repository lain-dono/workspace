use super::{
    DeclKind, EnumVariant, Function, ModuleScope, MustBeSigned, ResolvedName, ScopeGraph, ScopeRef,
    ScopeType, Type, TypeDef, TypeError, TypeInfo, TypeName, TypeOrStub, ValueKind,
    cycle::detect_type_cycles, default_types,
};
use crate::{
    ast::{self, Meta, MetaId},
    ice,
    module::{Module, ModuleTree},
    print::TypeDisplay,
    runtime::{
        Runtime,
        ty::{TypeDescription, TypeRegistry},
    },
};
use std::{any::TypeId, borrow::Borrow};

#[derive(Clone)]
pub(crate) enum Obligation {
    ResolveMethod {
        id: MetaId,
        receiver: Type,
        ident: ast::ident::Ident,
        parameter_types: Vec<Type>,
        return_type: Type,
    },
}

/// Result of type checking
pub type TypeResult<T> = Result<T, TypeError>;

pub fn typecheck(runtime: &Runtime, module_tree: &ModuleTree) -> TypeResult<TypeInfo> {
    runtime.type_checker.clone().check_module_tree(module_tree)
}

/// Holds the state for type checking
///
/// Most type checking steps are methods on this type.
#[derive(Clone)]
pub struct Typifier {
    /// The list of built-in functions, methods and static methods.
    pub(super) functions: Vec<Function>,
    pub(crate) types: TypeInfo,
    pub(super) match_counter: usize,
    pub(super) if_else_counter: usize,
    pub(super) while_counter: usize,
    /// Set of obligations that we have to satisfy at the end of type checking a function
    pub(super) obligations: Vec<Obligation>,
}

impl Typifier {
    pub fn new() -> Self {
        let mut checker = Typifier {
            functions: Vec::new(),
            types: TypeInfo::default(),
            match_counter: 0,
            if_else_counter: 0,
            while_counter: 0,
            obligations: Vec::new(),
        };
        checker.declare_builtin_types().unwrap();
        checker
    }

    pub fn scopes(&self) -> &ScopeGraph {
        &self.types.scopes
    }

    /// Perform type checking for a module tree (i.e. the entire program)
    pub fn check_module_tree(mut self, tree: &ModuleTree) -> Result<TypeInfo, TypeError> {
        let modules = self.declare_modules(tree)?;
        self.declare_imports(&modules)?;
        self.declare_types(&modules)?;

        detect_type_cycles(&self.types.types).map_err(|description| {
            TypeError::simple(
                description,
                "type cycle detected",
                MetaId::INTRINSIC, // TODO: make a more useful error here with the recursive chain
            )
        })?;

        self.declare_functions(&modules)?;
        self.tree(&modules)?;
        self.force_fmap_types(&modules);

        Ok(self.types)
    }

    pub(crate) fn scope_of(&self, scope: ScopeRef, ident: ast::ident::Ident) -> Option<ScopeRef> {
        let ident = Meta::intrinsic(ident);
        self.types.resolve_name(scope, &ident, false)?.scope
    }

    fn declare_builtin_types(&mut self) -> TypeResult<()> {
        for (ident, doc, ty) in default_types() {
            let ident = Meta::intrinsic(ident);
            let name = ResolvedName::global(*ident);
            self.types
                .insert_type(ScopeRef::GLOBAL, &ident, doc, ty.clone())
                .map_err(|id| TypeError::declared_twice(&ident, id))?;

            if let TypeDef::Enum(_, variants) = &ty {
                let dec = self.types.get_declaration(name);
                let DeclKind::Type(TypeOrStub::Type(type_def)) = dec.kind else {
                    ice!();
                };
                let scope = dec.scope.unwrap();
                for variant in variants {
                    self.types
                        .decl(
                            scope,
                            &Meta::intrinsic(variant.name),
                            DeclKind::Variant(Some((type_def.clone(), variant.clone()))),
                        )
                        .unwrap();
                }
            }

            self.types.types.insert(name, ty);
        }
        Ok(())
    }

    pub(crate) fn declare_runtime_module(
        &mut self,
        parent: Option<ScopeRef>,
        ident: ast::ident::Ident,
        doc: String,
    ) -> Result<ScopeRef, String> {
        let mod_scope = ModuleScope {
            name: ResolvedName {
                ident,
                scope: parent.unwrap_or(ScopeRef::GLOBAL),
            },
            parent_module: parent,
        };
        let mod_scope = self
            .types
            .wrap(ScopeRef::GLOBAL, ScopeType::Module(mod_scope));

        let scope = parent.unwrap_or(ScopeRef::GLOBAL);

        let ident = Meta::intrinsic(ident);

        self.types
            .insert_module(scope, &ident, doc, mod_scope)
            .map_err(|_| format!("An item with the name `{ident}` already exists"))?;

        Ok(mod_scope)
    }

    pub(crate) fn declare_runtime_type(
        &mut self,
        scope: ScopeRef,
        ident: ast::ident::Ident,
        type_id: TypeId,
        doc: String,
    ) -> Result<(), String> {
        let name = ResolvedName { scope, ident };
        let ident = Meta::intrinsic(ident);

        // Small edge case: the primitives are already in the typechecker, so we
        // skip them, but we should override the documentation.
        if let Some(other) = self.types.resolve_name(scope, &ident, true)
            && let DeclKind::Type(TypeOrStub::Type(
                TypeDef::Scalar(_) | TypeDef::Bool | TypeDef::String,
            )) = other.kind
        {
            let dec = self.types.get_declaration_mut(other.name);
            dec.doc = doc;
            return Ok(());
        }

        let ty = TypeDef::Runtime(name, type_id);
        self.types
            .insert_type(ScopeRef::GLOBAL, &ident, doc, ty.clone())
            .map_err(|_id| format!("Item `{ident}` already exists in this scope"))?;

        self.types.types.insert(name, ty);
        Ok(())
    }

    pub(crate) fn rust_type_to_roto_type(runtime: &Runtime, t: TypeId) -> Result<Type, String> {
        let ty = TypeRegistry::get(t).unwrap();

        if ty.type_id == TypeId::of::<()>() {
            return Ok(Type::Unit);
        }

        match ty.description {
            TypeDescription::Leaf => {
                let ty = runtime
                    .runtime_type(ty.type_id)
                    .ok_or_else(|| format!("unregistered type: {}", ty.rust_name))?;
                Ok(Type::Name(TypeName {
                    name: ty.name(),
                    args: Vec::new(),
                }))
            }
            TypeDescription::Option(t) => {
                Ok(Type::option(Self::rust_type_to_roto_type(runtime, t)?))
            }
            TypeDescription::Verdict(a, r) => Ok(Type::verdict(
                Self::rust_type_to_roto_type(runtime, a)?,
                Self::rust_type_to_roto_type(runtime, r)?,
            )),
            TypeDescription::Val(_) => {
                let ty = runtime
                    .runtime_type(ty.type_id)
                    .ok_or_else(|| format!("unregistered type: {}", ty.rust_name))?;
                Ok(Type::Name(TypeName {
                    name: ty.name(),
                    args: Vec::new(),
                }))
            }
        }
    }

    pub(crate) fn declare_runtime_constant(
        &mut self,
        scope: ScopeRef,
        ident: ast::ident::Ident,
        ty: Type,
        doc: String,
    ) -> Result<(), String> {
        let ident = Meta::intrinsic(ident);
        if self.types.insert_constant(scope, &ident, &ty, doc).is_err() {
            // TODO: Improve error message
            return Err("Name declared twice!".into());
        }
        Ok(())
    }

    pub(crate) fn declare_runtime_import(
        &mut self,
        scope: ScopeRef,
        name: ResolvedName,
    ) -> Result<(), String> {
        if self
            .types
            .insert_import(scope, MetaId::INTRINSIC, name)
            .is_err()
        {
            // TODO: Improve error message
            return Err("Name declared twice!".into());
        }
        Ok(())
    }

    fn declare_modules<'a>(
        &mut self,
        tree: &'a ModuleTree,
    ) -> TypeResult<Vec<(ScopeRef, &'a Module)>> {
        let mut modules = Vec::<(ScopeRef, &'a Module)>::new();
        for m in &tree.modules {
            let Module {
                ident,
                ast,
                children: _,
                parent,
            } = m;
            let parent_module = parent.map(|p| modules[p.0].0);
            let mod_scope = ModuleScope {
                name: ResolvedName {
                    ident: **ident,
                    scope: parent_module.unwrap_or(ScopeRef::GLOBAL),
                },
                parent_module,
            };
            let scope = self
                .types
                .wrap(ScopeRef::GLOBAL, ScopeType::Module(mod_scope));

            if let Some(p) = parent_module {
                self.insert_module(p, ident, String::new(), scope)?;
            } else {
                self.insert_module(ScopeRef::GLOBAL, ident, String::new(), scope)?;
            }

            for d in &ast.declarations {
                let (kind, ident) = match d {
                    ast::Decl::Struct(x) => (
                        DeclKind::Type(TypeOrStub::Stub {
                            num_params: x.params.len(),
                        }),
                        x.ident.clone(),
                    ),
                    ast::Decl::Enum(x) => (
                        DeclKind::Type(TypeOrStub::Stub {
                            num_params: x.params.len(),
                        }),
                        x.ident.clone(),
                    ),
                    ast::Decl::Func(x) => (DeclKind::Function(None), x.ident.clone()),
                    ast::Decl::Fmap(x) => (DeclKind::Function(None), x.ident.clone()),
                    ast::Decl::Import(_) | ast::Decl::Test(_) => continue,
                };

                let new_scope = self.types.wrap(scope, ScopeType::Type(*ident));

                let res = self.types.decl(scope, &ident, kind);

                let dec = match res {
                    Ok(dec) => dec,
                    Err(e) => {
                        return Err(TypeError::declared_twice(&ident, e));
                    }
                };

                dec.scope = Some(new_scope);

                // We have to insert stub declarations for all the enum
                // variants, so that they can be imported.
                if let ast::Decl::Enum(x) = d {
                    for variant in &*x.variants {
                        let res =
                            self.types
                                .decl(new_scope, &variant.ident, DeclKind::Variant(None));

                        if let Err(e) = res {
                            return Err(TypeError::declared_twice(&x.ident, e));
                        }
                    }
                }
            }
            modules.push((scope, m));
        }
        Ok(modules)
    }

    fn declare_imports(&mut self, modules: &[(ScopeRef, &Module)]) -> TypeResult<()> {
        for &(scope, module) in modules {
            let mut paths = Vec::new();
            for expr in &module.ast.declarations {
                let ast::Decl::Import(new_paths) = expr else {
                    continue;
                };
                for path in new_paths {
                    paths.push(path);
                }
            }
            self.imports(scope, &paths)?;
        }
        Ok(())
    }

    fn declare_types(&mut self, modules: &[(ScopeRef, &Module)]) -> TypeResult<()> {
        for &(scope, module) in modules {
            for expr in &module.ast.declarations {
                match expr {
                    ast::Decl::Func(_)
                    | ast::Decl::Fmap(_)
                    | ast::Decl::Test(_)
                    | ast::Decl::Import(_) => (),
                    ast::Decl::Enum(ast::EnumDecl {
                        ident,
                        params,
                        variants,
                    }) => {
                        let name = ResolvedName {
                            scope,
                            ident: **ident,
                        };

                        let eval_scope = self.types.wrap(scope, ScopeType::TypeParams);

                        for param in params {
                            if let Err(e) =
                                self.types
                                    .decl(eval_scope, param, DeclKind::TypeParam(**param))
                            {
                                return Err(TypeError::declared_twice(param, e));
                            }
                        }

                        let mut evaluated_variants = Vec::new();

                        for v in &**variants {
                            let fields = v
                                .fields
                                .iter()
                                .map(|ty| self.eval_ty(eval_scope, ty))
                                .collect::<Result<_, _>>()?;

                            evaluated_variants.push(EnumVariant {
                                name: *v.ident,
                                fields,
                            });
                        }

                        let type_def = TypeDef::Enum(
                            TypeName {
                                name,
                                args: params
                                    .iter()
                                    .map(|ident| Type::ExplicitVar(**ident))
                                    .collect(),
                            },
                            evaluated_variants.clone(),
                        );

                        self.types
                            .insert_type(scope, ident, String::new(), type_def.clone())
                            .map_err(|e| TypeError::declared_twice(ident, e))?;

                        let inner_scope = self.scope_of(scope, **ident).unwrap();

                        for variant in evaluated_variants {
                            self.types
                                .insert_decl(
                                    inner_scope,
                                    &Meta::intrinsic(variant.name),
                                    DeclKind::Variant(Some((type_def.clone(), variant.clone()))),
                                    String::new(),
                                    |kind| matches!(kind, DeclKind::Variant(None)),
                                )
                                .unwrap();
                        }
                    }
                    ast::Decl::Struct(ast::StructDecl {
                        ident,
                        params,
                        fields,
                    }) => {
                        let name = ResolvedName {
                            scope,
                            ident: **ident,
                        };

                        let eval_scope = self.types.wrap(scope, ScopeType::TypeParams);

                        for param in params {
                            if let Err(e) =
                                self.types
                                    .decl(eval_scope, param, DeclKind::TypeParam(**param))
                            {
                                return Err(TypeError::declared_twice(param, e));
                            }
                        }

                        let ty = TypeDef::Struct(
                            TypeName {
                                name,
                                args: params
                                    .iter()
                                    .map(|ident| Type::ExplicitVar(**ident))
                                    .collect(),
                            },
                            self.eval_struct(eval_scope, fields)?,
                        );
                        self.types
                            .insert_type(scope, ident, String::new(), ty.clone())
                            .map_err(|e| TypeError::declared_twice(ident, e))?;
                        let opt = self.types.types.insert(name, ty);
                        assert!(opt.is_none());
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn resolve_obligations(&mut self) -> TypeResult<()> {
        let mut obligations = Vec::new();
        std::mem::swap(&mut obligations, &mut self.obligations);
        for obligation in obligations {
            match obligation {
                Obligation::ResolveMethod {
                    id,
                    receiver,
                    ident,
                    parameter_types,
                    return_type,
                } => {
                    let receiver_ty = self.types.resolve(&receiver);
                    match receiver_ty {
                        Type::IVar(_, _) => {
                            self.unify(&receiver_ty, &Type::ivar(), id, None).unwrap();
                        }
                        Type::FVar(_) => {
                            self.unify(&receiver_ty, &Type::fvar(), id, None).unwrap();
                        }
                        _ => {}
                    }
                    let ident = Meta::new(id, ident);
                    let Some(f) = self.get_method(&receiver, &ident) else {
                        return Err(self.error_no_method_on_type(&receiver, &ident));
                    };

                    let sig = &f.signature;

                    let mut correct = true;
                    correct &= sig.parameter_types.len() == parameter_types.len();

                    for (a, b) in sig.parameter_types.iter().zip(&parameter_types) {
                        correct &= self.unify(a, b, id, None).is_ok();
                    }

                    correct &= self.unify(&sig.return_type, &return_type, id, None).is_ok();

                    if !correct {
                        return Err(TypeError::simple(
                            format!(
                                "the `{}` method of type `{}` does not have the right signature",
                                ident,
                                receiver_ty.display(&self.types),
                            ),
                            format!("does not have a valid `{ident}` method"),
                            id,
                        ));
                    }

                    self.types.function_calls.insert(id, f);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn imports(
        &mut self,
        scope: ScopeRef,
        paths: &[&Meta<ast::Path>],
    ) -> TypeResult<()> {
        // We want imports to work in any order and sometimes there are
        // dependencies between them. This means that we process them in a loop
        // where we exit either if we have no unresolved imports anymore or when
        // we can no longer make progress, in which case we error.
        let mut paths = paths.to_vec();
        loop {
            let last_len = paths.len();
            paths.retain(|path| self.import(scope, path).is_err());
            let new_len = paths.len();
            if new_len == 0 {
                return Ok(());
            } else if new_len == last_len {
                for path in &paths {
                    self.import(scope, path)?;
                }
            }
        }
    }

    fn import(&mut self, scope: ScopeRef, path: &ast::Path) -> TypeResult<()> {
        let mut idents = path.idents.iter();
        let (ident, declaration) = self.resolve_module_part_of_path(scope, &mut idents)?;

        // This is a bit of an oversimplification. The
        // resolve_module_part_of_path should just give us the thing
        // to import, but we might expand import functionality to enum
        // constructors.
        if let Some(_ident) = idents.next() {
            return Err(TypeError::expected_module(ident, declaration));
        }

        self.types
            .insert_import(scope, ident.id, declaration.name)
            .map_err(|old| TypeError::declared_twice(ident, old))
    }

    /// Create a fresh variable in the unionfind structure
    pub(crate) fn fresh_var(&mut self) -> Type {
        self.types.unionfind.fresh(Type::Var)
    }

    /// Create a fresh record variable in the unionfind structure
    pub(crate) fn fresh_record(&mut self, fields: Vec<(ast::Ident, Type)>) -> Type {
        let fields = fields.into_iter().map(|(s, t)| (s.clone(), t)).collect();
        self.types
            .unionfind
            .fresh(move |x| Type::RecordVar(x, fields))
    }

    /// Insert a variable into the given scope
    pub(crate) fn insert_var(
        &mut self,
        scope: ScopeRef,
        ident: ast::Ident,
        ty: impl Borrow<Type>,
    ) -> TypeResult<()> {
        let ty = ty.borrow();
        let kind = DeclKind::Value(ValueKind::Local, ty.clone());
        match self.types.decl(scope, &ident, kind) {
            Ok(decl) => {
                let name = decl.name;
                self.types.resolved_names.insert(ident.id, name);
                self.types.expr_types.insert(ident.id, ty.clone());
                Ok(())
            }
            Err(old) => Err(TypeError::declared_twice(&ident, old)),
        }
    }

    /// Insert a variable into the given scope
    fn insert_module(
        &mut self,
        scope: ScopeRef,
        k: &ast::Ident,
        doc: String,
        mod_scope: ScopeRef,
    ) -> TypeResult<()> {
        match self.types.insert_module(scope, k, doc, mod_scope) {
            Ok(()) => Ok(()),
            Err(old) => Err(TypeError::declared_twice(k, old)),
        }
    }

    /// Unify two types
    ///
    /// This function tries to find the most general unification of two
    /// types. If they cannot be unified an error is generated.
    ///
    /// The types do not need to be resolved before this function.
    ///
    /// Note that this function modifies the union find structure. The changes
    /// it makes cannot be undone. This is a possible improvement for the
    /// future, so that we can attempt multiple unifications to find a correct
    /// one.
    pub(crate) fn unify(
        &mut self,
        a: &Type,
        b: &Type,
        span: MetaId,
        cause: Option<MetaId>,
    ) -> TypeResult<Type> {
        if let Some(ty) = self.unify_inner(a, b) {
            Ok(ty)
        } else {
            let a = self.resolve_type(a);
            let b = self.resolve_type(b);
            Err(self.error_mismatched_types(&a, &b, span, cause))
        }
    }

    fn unify_inner(&mut self, a: &Type, b: &Type) -> Option<Type> {
        use Type::{ExplicitVar, FVar, IVar, Name, Never, Record, RecordVar, Var};

        Some(match (self.resolve_type(a), self.resolve_type(b)) {
            // Evidently, if two types are identical, they trivially unify
            (a, b) if a == b => a,

            // Explicit type variables need to be replaced with fresh type variable.
            // If they appear here, something has gone wrong.
            (a @ ExplicitVar(_), b) | (a, b @ ExplicitVar(_)) => {
                let a = a.display(&self.types);
                let b = b.display(&self.types);
                ice!("Cannot unify explicit var: {a}, {b}",)
            }

            // The never type is special and unifies with anything
            (Never, x) | (x, Never) => x,

            (IVar(..), Name(ty)) | (Name(ty), IVar(..)) if !ty.args.is_empty() => return None,
            (FVar(..), Name(ty)) | (Name(ty), FVar(..)) if !ty.args.is_empty() => return None,

            // We have to ensure that `Yes` has priority over `No`.
            // We map `a` to `b` by default, so if `a` has `Yes` and `b` has `No` we need to map `b` to `a` instead.
            (IVar(a, sign @ MustBeSigned::Yes), IVar(b, MustBeSigned::No)) => {
                self.types.unionfind.set(b, Type::IVar(a, sign));
                Type::IVar(a, sign)
            }
            (IVar(a, _), IVar(b, b_signed)) => {
                self.types.unionfind.set(a, Type::IVar(b, b_signed));
                Type::IVar(b, b_signed)
            }
            (IVar(b, s), Name(name)) | (Name(name), IVar(b, s)) => {
                use MustBeSigned::{No, Yes};
                let def = self.types.resolve_type_name(&name);
                let snum = matches!(s, Yes) && def.is_sint();
                let inum = matches!(s, No) && def.is_int();
                if snum || inum {
                    self.types.unionfind.set(b, Name(name.clone()));
                    Name(name)
                } else {
                    return None;
                }
            }
            (FVar(a), b @ FVar(_)) | (Var(a), b) | (b, Var(a)) => {
                self.types.unionfind.set(a, b.clone());
                b.clone()
            }
            (FVar(b), Name(name)) | (Name(name), FVar(b)) => {
                if self.types.resolve_type_name(&name).is_float() {
                    self.types.unionfind.set(b, Name(name.clone()));
                    Name(name)
                } else {
                    return None;
                }
            }
            (
                RecordVar(a_var, ref a_fields),
                ref b @ (RecordVar(_, ref b_fields) | Record(ref b_fields)),
            ) => {
                self.unify_fields(a_fields, b_fields)?;
                self.types.unionfind.set(a_var, b.clone());
                b.clone()
            }
            (ref a @ Record(ref a_fields), RecordVar(b_var, ref b_fields)) => {
                self.unify_fields(a_fields, b_fields)?;
                self.types.unionfind.set(b_var, a.clone());
                a.clone()
            }
            (RecordVar(var, fields), Name(name)) | (Name(name), RecordVar(var, fields)) => {
                let def = self.types.resolve_type_name(&name);
                let named_fields = def.record_fields(&name.args)?;
                self.unify_fields(&fields, &named_fields)?;
                self.types.unionfind.set(var, Name(name.clone()));
                Name(name)
            }
            // Type names unify if they have the same name and their arguments can be unified.
            (Name(a), Name(b)) if a.name != b.name => return None,
            (Name(a), Name(b)) => {
                for (a_param, b_param) in a.args.iter().zip(&b.args) {
                    self.unify_inner(a_param, b_param)?;
                }
                Name(b)
            }
            // Anything else cannot be unified.
            (_a, _b) => return None,
        })
    }

    fn unify_fields(
        &mut self,
        a_fields: &[(ast::Ident, Type)],
        b_fields: &[(ast::Ident, Type)],
    ) -> Option<Vec<(ast::Ident, Type)>> {
        if a_fields.len() != b_fields.len() {
            return None;
        }

        let mut b_fields = b_fields.to_vec();
        let mut new_fields = Vec::new();
        for (name, a_ty) in a_fields {
            let idx = b_fields.iter().position(|(n, _)| n.node == name.node)?;
            let (_, b_ty) = b_fields.remove(idx);
            new_fields.push((name.clone(), self.unify_inner(a_ty, &b_ty)?));
        }

        Some(new_fields)
    }

    /// Resolve a type variable to a type.
    pub(crate) fn resolve_type(&mut self, t: &Type) -> Type {
        if let Type::Var(x) | Type::IVar(x, _) | Type::FVar(x) | Type::RecordVar(x, _) = t {
            self.types.unionfind.find(*x).clone()
        } else {
            t.clone()
        }
    }

    /// Evaluate a type expression into a [`Type`]
    pub(crate) fn eval_ty(&self, scope: ScopeRef, ty: &ast::TypeExpr) -> TypeResult<Type> {
        Ok(match ty {
            ast::TypeExpr::Never => Type::Never,
            ast::TypeExpr::Unit => Type::unit(),
            ast::TypeExpr::Option(ty) => Type::option(self.eval_ty(scope, ty)?),
            ast::TypeExpr::Path(path, params) => {
                let params = params.as_ref().map_or(&[][..], |p| &(&**p)[..]);
                self.resolve_type_path(scope, path, params)?
            }
            ast::TypeExpr::Struct(record_ty) => Type::Record(self.eval_struct(scope, record_ty)?),
        })
    }

    fn eval_struct(
        &self,
        scope: ScopeRef,
        expr: &ast::NamedFields,
    ) -> TypeResult<Vec<(ast::Ident, Type)>> {
        let mut type_fields = Vec::new();

        for (ident, ty) in &**expr.fields {
            let field_type = self.eval_ty(scope, ty)?;
            type_fields.push((ident, field_type));
        }

        let mut unspanned_type_fields = Vec::new();
        for field in &type_fields {
            let same_fields: Vec<_> = type_fields
                .iter()
                .filter_map(|(ident, _typ)| (ident.node == field.0.node).then_some(ident.id))
                .collect();
            if same_fields.len() > 1 {
                let ident = field.0.as_str();
                return Err(TypeError::duplicate_fields(ident, &same_fields));
            }
            unspanned_type_fields.push((field.0.clone(), field.1.clone()));
        }

        Ok(unspanned_type_fields)
    }
}
