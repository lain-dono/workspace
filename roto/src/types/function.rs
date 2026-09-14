//! Type checking function-like items

use super::{
    Function, FunctionDefinition, RuntimeFunctionRef, ScopeRef, ScopeType, Signature, Type,
    TypeError, TypeName, TypeResult, Typifier, expr::ExprContext,
};
use crate::{
    ast, ice,
    module::Module,
    types::{DeclKind, scope::FunctionDeclaration},
};
use core::borrow::Borrow;

impl Typifier {
    pub(crate) fn tree(&mut self, modules: &[(ScopeRef, &Module)]) -> TypeResult<()> {
        for &(scope, module) in modules {
            for expr in &module.ast.declarations {
                match &expr {
                    ast::Decl::Fmap(f) => self.filter_map(scope, f)?,
                    ast::Decl::Func(x) => self.function(scope, x)?,
                    ast::Decl::Test(x) => self.test(scope, x)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub(crate) fn declare_functions(&mut self, modules: &[(ScopeRef, &Module)]) -> TypeResult<()> {
        for &(scope, module) in modules {
            for expr in &module.ast.declarations {
                let def = FunctionDefinition::Roto;
                let (ident, ty, params) = match expr {
                    ast::Decl::Func(decl) => {
                        (&decl.ident, self.func_type(scope, decl)?, &decl.params)
                    }
                    ast::Decl::Fmap(decl) => {
                        (&decl.ident, self.fmap_type(scope, decl)?, &decl.params)
                    }
                    ast::Decl::Test(_test) => continue,
                    ast::Decl::Struct(_) | ast::Decl::Enum(_) | ast::Decl::Import(_) => continue,
                };
                let params = params.0.iter().map(|(i, _)| **i).collect();
                self.insert_function(scope, ident, def, params, &ty)?;
            }
        }
        Ok(())
    }

    pub(crate) fn force_fmap_types(&mut self, modules: &[(ScopeRef, &Module)]) {
        for &(_, module) in modules {
            for expr in &module.ast.declarations {
                if let ast::Decl::Fmap(f) = &expr {
                    let ty = self.types.type_of(&f.ident);
                    let Type::Function(_, return_type) = &ty else {
                        ice!("filtermap should always have a function type")
                    };

                    let Type::Name(TypeName {
                        name: _,
                        args: arguments,
                    }) = &**return_type
                    else {
                        ice!("return type of a filtermap should always be a verdict")
                    };
                    let [a, r] = &arguments[..] else {
                        ice!("return type of a filtermap should always be a verdict")
                    };

                    if let Type::Var(x) = self.resolve_type(a) {
                        self.unify(&Type::Var(x), &Type::unit(), f.ident.id, None)
                            .unwrap();
                    }
                    if let Type::Var(x) = self.resolve_type(r) {
                        self.unify(&Type::Var(x), &Type::unit(), f.ident.id, None)
                            .unwrap();
                    }
                }
            }
        }
    }

    pub fn function(&mut self, scope: ScopeRef, decl: &ast::FuncDecl) -> TypeResult<()> {
        let result = decl.result.as_ref();
        let result = result.map_or(Ok(Type::unit()), |ty| self.eval_ty(scope, ty))?;
        self.typify_func(scope, &decl.ident, Some(&decl.params), &result, &decl.body)
    }

    /// Type check a filter map
    pub fn filter_map(&mut self, scope: ScopeRef, decl: &ast::FilterMap) -> TypeResult<()> {
        let Type::Function(_, result) = &self.types.type_of(&decl.ident) else {
            ice!()
        };
        self.typify_func(scope, &decl.ident, Some(&decl.params), result, &decl.body)
    }

    pub fn test(&mut self, scope: ScopeRef, test: &ast::Test) -> TypeResult<()> {
        let ast::Test { ident, body } = test;

        let ident = ident.clone().map(|ident| format!("test#{ident}").into());
        let result = Type::verdict(Type::unit(), Type::unit());

        let ty = Type::Function(Vec::new(), Box::new(result.clone()));
        let def = FunctionDefinition::Roto;
        self.insert_function(scope, &ident, def, Vec::new(), ty)?;
        self.typify_func(scope, &ident, None, &result, body)
    }

    /// Insert a function name into the given scope
    pub(crate) fn insert_function(
        &mut self,
        scope: ScopeRef,
        ident: &ast::Ident,
        def: FunctionDefinition,
        params: Vec<ast::ident::Ident>,
        ty: impl Borrow<Type>,
    ) -> TypeResult<()> {
        let ty = ty.borrow();
        let decl = {
            let ty = ty.clone();
            let kind = DeclKind::Function(Some(FunctionDeclaration { def, params, ty }));
            let doc = String::new();
            self.types.insert_decl(scope, ident, kind, doc, |kind| {
                matches!(kind, DeclKind::Function(None))
            })
        };

        match decl {
            Ok(decl) => {
                let name = decl.name;
                self.types.resolved_names.insert(ident.id, name);
                self.types.expr_types.insert(ident.id, ty.clone());
                Ok(())
            }
            Err(old) => Err(TypeError::declared_twice(ident, old)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn declare_runtime_function(
        &mut self,
        scope: ScopeRef,
        ident: ast::ident::Ident,
        id: RuntimeFunctionRef,
        params: Vec<ast::ident::Ident>,
        parameter_types: Vec<Type>,
        return_type: Type,
        doc: String,
        method: bool,
    ) -> Result<(), String> {
        let ty = Type::Function(parameter_types.clone(), Box::new(return_type.clone()));

        let ident = ast::Meta::intrinsic(ident);
        let def = FunctionDefinition::Runtime(id);

        let map = if method {
            DeclKind::Method
        } else {
            DeclKind::Function
        };

        let kind = map(Some(FunctionDeclaration { def, params, ty }));
        let name = self
            .types
            .insert_decl(scope, &ident, kind, doc, |kind| kind == &map(None))
            .map_err(|_| "Name declared twice")?
            .name;

        let signature = Signature {
            parameter_types,
            return_type,
        };

        self.functions.push(Function::new(
            name,
            &[],
            signature.clone(),
            FunctionDefinition::Runtime(id),
        ));
        self.types.rt_functions.insert(id, signature);

        Ok(())
    }

    fn typify_func(
        &mut self,
        scope: ScopeRef,
        ident: &ast::Ident,
        params: Option<&ast::Meta<ast::Params>>,
        result: &Type,
        body: &ast::Meta<ast::Block>,
    ) -> TypeResult<()> {
        let sty = ScopeType::Function(**ident);
        let scope = self.types.wrap(scope, sty);
        self.types.function_scopes.insert(ident.id, scope);

        if let Some(params) = params {
            for (ident, ty) in &params.0 {
                let ty = self.eval_ty(scope, ty)?;
                self.insert_var(scope, ident.clone(), ty)?;
            }
        }

        let ctx = ExprContext {
            expected_type: result.clone(),
            function_return_type: Some(result.clone()),
        };

        self.block(scope, &ctx, body)?;
        self.resolve_obligations()?;

        Ok(())
    }

    pub fn func_type(&mut self, scope: ScopeRef, dec: &ast::FuncDecl) -> TypeResult<Type> {
        let result = dec.result.as_ref();
        let result = Box::new(result.map_or(Ok(Type::unit()), |ty| self.eval_ty(scope, ty))?);
        let params = dec.params.0.iter();
        let params = params.map(|(_, ty)| self.eval_ty(scope, ty));
        Ok(Type::Function(params.collect::<TypeResult<_>>()?, result))
    }

    pub fn fmap_type(&mut self, scope: ScopeRef, dec: &ast::FilterMap) -> TypeResult<Type> {
        let result = Box::new(Type::verdict(self.fresh_var(), self.fresh_var()));
        let params = dec.params.0.iter();
        let params = params.map(|(_, ty)| self.eval_ty(scope, ty));
        Ok(Type::Function(params.collect::<TypeResult<_>>()?, result))
    }
}
