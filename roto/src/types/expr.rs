//! Type checking of expressions

use super::{
    DeclKind, Declaration, EnumVariant, Function, FunctionDefinition, MustBeSigned, ResolvedName,
    ScopeRef, ScopeType, Signature, Type, TypeDef, TypeError, TypeName, TypeOrStub, TypeResult,
    Typifier, ValueKind, typifier::Obligation,
};
use crate::{ast, ice};
use std::{borrow::Borrow, collections::HashSet};

/// The context for type checking expressions
///
/// This holds:
///  - the type that this expression is expected to have and
///  - the type that the current function should return.
#[derive(Clone)]
pub struct ExprContext {
    pub expected_type: Type,
    pub function_return_type: Option<Type>,
}

impl ExprContext {
    /// Create a new context with the expected type set to the passed type
    fn with_type(&self, t: impl Borrow<Type>) -> Self {
        Self {
            expected_type: t.borrow().clone(),
            ..self.clone()
        }
    }
}

#[derive(Clone)]
pub enum ResolvedPath {
    /// The path referenced a free function
    Function {
        name: ResolvedName,
        definition: FunctionDefinition,
        signature: Signature,
    },
    /// The path referenced a method
    Method {
        value: PathValue,
        name: ResolvedName,
        definition: FunctionDefinition,
        signature: Signature,
    },
    /// The path referenced a value
    ///
    /// A value can be a local variable, constant or context.
    Value(PathValue),
    /// The path referenced a static method
    StaticMethod {
        name: ResolvedName,
        definition: FunctionDefinition,
        signature: Signature,
    },
    /// The path referenced an enum constructor
    EnumConstructor { ty: TypeDef, variant: EnumVariant },
}

#[derive(Clone)]
pub struct PathValue {
    pub name: ResolvedName,
    pub kind: ValueKind,
    pub root_ty: Type,
    pub fields: Vec<(ast::ident::Ident, Type)>,
}

impl PathValue {
    pub fn final_type(&self) -> &Type {
        self.fields.last().map_or(&self.root_ty, |(_, ty)| ty)
    }
}

impl Typifier {
    /// Type check a block
    pub fn block(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        block: &ast::Meta<ast::Block>,
    ) -> TypeResult<bool> {
        let mut diverged = false;

        self.imports(scope, &block.imports.iter().collect::<Vec<_>>())?;

        for stmt in &block.body {
            // TODO: emit message for diverging statements
            diverged |= self.stmt(scope, ctx, stmt)?;
        }

        let ty = if let Some(expr) = &block.last {
            // TODO: emit message for diverging statements
            diverged |= self.expr(scope, ctx, expr)?;

            // Store the same type info on the block as on the expression
            self.types.expr_types[&expr.id].clone()
        } else {
            if !diverged {
                self.unify(&ctx.expected_type, &Type::unit(), block.id, None)?;
            }
            Type::unit()
        };

        self.types.expr_types.insert(block.id, ty);
        self.types.diverges.insert(block.id, diverged);
        Ok(diverged)
    }

    pub fn stmt(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        stmt: &ast::Meta<ast::Stmt>,
    ) -> TypeResult<bool> {
        match &stmt.node {
            ast::Stmt::Let(ident, ty, expr) => {
                let ty = if let Some(ty) = ty {
                    self.eval_ty(scope, ty)?
                } else {
                    self.fresh_var()
                };
                let ctx = ctx.with_type(&ty);
                let diverges = self.expr(scope, &ctx, expr)?;
                let ty = self.resolve_type(&ty);
                self.insert_var(scope, ident.clone(), ty)?;
                Ok(diverges)
            }
            ast::Stmt::Expr(expr) => {
                let var = self.fresh_var();
                let ctx = ctx.with_type(&var);
                self.expr(scope, &ctx, expr)
            }
        }
    }

    pub fn expr(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        expr: &ast::Meta<ast::Expr>,
    ) -> TypeResult<bool> {
        let id = expr.id;

        // Store the type for use in the lowering step
        self.types.expr_types.insert(id, ctx.expected_type.clone());

        match &expr.node {
            ast::Expr::Return(kind, e) => {
                let Some(ret) = &ctx.function_return_type else {
                    return Err(self.error_cannot_diverge_here(kind.str(), expr));
                };

                self.unify(&ctx.expected_type, &Type::Never, id, None)?;

                let expected_type = match kind {
                    ast::ReturnKind::Return => ret.clone(),
                    ast::ReturnKind::Accept => {
                        let a_ty = self.fresh_var();
                        let b_ty = self.fresh_var();
                        let ty = Type::verdict(&a_ty, b_ty);
                        self.unify(ret, &ty, id, None)?;
                        self.resolve_type(&a_ty)
                    }
                    ast::ReturnKind::Reject => {
                        let a_ty = self.fresh_var();
                        let r_ty = self.fresh_var();
                        let ty = Type::verdict(a_ty, &r_ty);
                        self.unify(ret, &ty, id, None)?;
                        self.resolve_type(&r_ty)
                    }
                };

                if let Some(e) = e {
                    self.expr(scope, &ctx.with_type(expected_type.clone()), e)?;
                } else {
                    self.unify(&expected_type, &Type::unit(), id, None)?;
                }

                let ty = ctx.function_return_type.clone().unwrap_or_else(Type::unit);
                self.types.return_types.insert(id, ty);

                Ok(true)
            }
            ast::Expr::Literal(l) => self.literal(ctx, l),
            ast::Expr::Match(m) => self.match_expr(scope, ctx, m),
            ast::Expr::FunctionCall(e, args) => match &e.node {
                ast::Expr::Path(p) => self.path_function_call(scope, ctx, id, p, args),
                ast::Expr::Access(e, name) => {
                    let ty = self.fresh_var();
                    let mut diverges = self.expr(scope, &ctx.with_type(&ty), e)?;
                    diverges |= self.method_call(scope, ctx, id, ty, name, args)?;
                    Ok(diverges)
                }
                _ => Err(TypeError::simple(
                    "arbitrary function expressions are not supported yet",
                    "cannot be called",
                    e.id,
                )),
            },
            ast::Expr::Access(e, field) => {
                let ty = self.fresh_var();
                let diverges = self.expr(scope, &ctx.with_type(&ty), e)?;
                let ty = self.resolve_type(&ty);
                let ty = self.access_field(&ty, field)?;
                self.unify(&ctx.expected_type, &ty, field.id, None)?;
                Ok(diverges)
            }
            ast::Expr::Assign(p, e) => {
                self.unify(&ctx.expected_type, &Type::unit(), id, None)?;

                let resolved_path = self.resolve_expression_path(scope, p)?;
                self.types.path_kinds.insert(p.id, resolved_path.clone());
                let ResolvedPath::Value(path_value) = resolved_path else {
                    todo!("cannot assign to this");
                };

                let ty = path_value.final_type();
                let ctx = ctx.with_type(ty);
                let diverges = self.expr(scope, &ctx, e)?;
                Ok(diverges)
            }
            ast::Expr::Path(p) => {
                let last_ident = p.idents.last().unwrap();
                let resolved_path = self.resolve_expression_path(scope, p)?;
                self.types.path_kinds.insert(p.id, resolved_path.clone());

                let ty = match resolved_path {
                    ResolvedPath::Value(PathValue {
                        name: _,
                        kind: _,
                        root_ty: ty,
                        fields,
                    }) => match fields.last() {
                        Some(f) => f.1.clone(),
                        None => ty,
                    },
                    ResolvedPath::EnumConstructor { ty, variant } => {
                        if !variant.fields.is_empty() {
                            return Err(TypeError::simple(
                                format!("enum variant {} requires arguments", variant.name),
                                "requires arguments",
                                last_ident.id,
                            ));
                        }
                        ty.instantiate(|| self.fresh_var())
                    }
                    _ => return Err(self.error_expected_value_path(last_ident, &resolved_path)),
                };

                self.types.expr_types.insert(p.id, ty.clone());
                self.unify(&ctx.expected_type, &ty, last_ident.id, None)?;
                Ok(false)
            }
            ast::Expr::Record(record) => {
                let field_types: Vec<_> = record
                    .fields
                    .iter()
                    .map(|(s, _)| (s.clone(), self.fresh_var()))
                    .collect();
                let rec = self.fresh_record(field_types.clone());
                self.unify(&ctx.expected_type, &rec, id, None)?;

                self.record_fields(scope, ctx, field_types, record, id)
            }
            ast::Expr::TypedRecord(path, record) => {
                let mut idents = path.idents.iter();
                let (ident, declaration) = self.resolve_module_part_of_path(scope, &mut idents)?;

                let DeclKind::Type(TypeOrStub::Type(type_def)) = &declaration.kind else {
                    return Err(TypeError::simple(
                        format!("Expected a record type, but found `{ident}`"),
                        "not a record type",
                        ident.id,
                    ));
                };

                let ty = type_def.instantiate(|| self.fresh_var());
                let Type::Name(type_name) = &ty else { ice!() };
                let Some(instantiated_fields) = type_def.record_fields(&type_name.args) else {
                    return Err(TypeError::simple(
                        format!("Expected a record type, but found `{ident}`"),
                        "not a record type",
                        ident.id,
                    ));
                };

                let diverges = self.record_fields(scope, ctx, instantiated_fields, record, id)?;

                self.unify(&ctx.expected_type, &ty, id, None)?;

                Ok(diverges)
            }
            ast::Expr::List(es) => {
                let var = self.fresh_var();
                let ty = Type::list(&var);
                self.unify(&ctx.expected_type, &ty, id, None)?;

                let mut diverges = false;
                let ctx = ctx.with_type(var);

                for e in es {
                    diverges |= self.expr(scope, &ctx, e)?;
                }

                Ok(diverges)
            }
            ast::Expr::Not(e) => {
                self.unify(&ctx.expected_type, &Type::bool(), id, None)?;
                self.expr(scope, &ctx.with_type(Type::bool()), e)
            }
            ast::Expr::Negate(e) => {
                let operand_ty = self.fresh_var();
                let new_ctx = ctx.with_type(operand_ty.clone());

                let mut diverges = false;
                diverges |= self.expr(scope, &new_ctx, e)?;

                let operand_ty = self.types.resolve(&operand_ty);
                if let Type::Name(name) = &operand_ty
                    && self.types.resolve_type_name(name).is_uint()
                {
                    return Err(TypeError::simple(
                        "cannot apply `-` to unsigned integer type",
                        "cannot apply `-`",
                        id,
                    ));
                }

                if self.types.is_numeric_type(&operand_ty) {
                    // If we get an int var, we need to store the fact that this
                    // var must be signed so that we can give an error if this
                    // is later unified with an unsigned integer.
                    if let &Type::IVar(index, MustBeSigned::No) = &operand_ty {
                        let ty = Type::IVar(index, MustBeSigned::Yes);
                        self.types.unionfind.set(index, ty);
                    }
                    self.unify(&ctx.expected_type, &operand_ty, id, None)?;
                    Ok(diverges)
                } else {
                    Err(self.error_expected_numeric_value(e, &operand_ty))
                }
            }
            ast::Expr::Binary(left, op, right) => self.binop(scope, ctx, *op, id, left, right),
            ast::Expr::Select(c, t, e) => {
                self.expr(scope, &ctx.with_type(Type::bool()), c)?;

                let idx = self.if_else_counter;
                self.if_else_counter += 1;

                if let Some(e) = e {
                    let mut diverges = false;
                    let then_scope = self.types.wrap(scope, ScopeType::Then(idx));
                    diverges |= self.block(then_scope, ctx, t)?;
                    let else_scope = self.types.wrap(scope, ScopeType::Else(idx));
                    diverges |= self.block(else_scope, ctx, e)?;

                    // Record divergence so that we can omit the
                    // block after the if-else while lowering
                    self.types.diverges.insert(id, diverges);
                    Ok(diverges)
                } else {
                    self.unify(&ctx.expected_type, &Type::unit(), id, None)?;

                    // An if without else does not always diverge, because
                    // the condition could be false
                    let then_scope = self.types.wrap(scope, ScopeType::Then(idx));
                    let _ = self.block(then_scope, &ctx.with_type(Type::unit()), t)?;
                    self.types.diverges.insert(id, false);
                    Ok(false)
                }
            }
            ast::Expr::Loop(c, b) => {
                let mut diverges = self.expr(scope, &ctx.with_type(Type::bool()), c)?;

                let idx = self.while_counter;
                self.while_counter += 1;

                let sty = ScopeType::WhileBody(idx);
                let body_scope = self.types.wrap(scope, sty);

                diverges |= self.block(body_scope, ctx, b)?;
                self.unify(&ctx.expected_type, &Type::unit(), id, None)?;

                Ok(diverges)
            }
            ast::Expr::QuestionMark(expr) => {
                let opt_ty = Type::option(ctx.expected_type.clone());
                let diverges = self.expr(scope, &ctx.with_type(opt_ty), expr)?;
                let Some(ret_ty) = &ctx.function_return_type else {
                    return Err(TypeError::simple(
                        "can only use `?` in function returning an optional value",
                        "cannot use `?` here",
                        id,
                    ));
                };
                let Type::Name(type_name) = self.types.resolve(ret_ty) else {
                    return Err(TypeError::simple(
                        "can only use `?` in function returning an optional value",
                        "cannot use `?` here",
                        id,
                    ));
                };
                if type_name.name != ResolvedName::global("Option") {
                    return Err(TypeError::simple(
                        "can only use `?` in function returning an optional value",
                        "cannot use `?` here",
                        id,
                    ));
                }
                Ok(diverges)
            }
            ast::Expr::FString(parts) => {
                // An f-string always has the type string
                self.unify(&ctx.expected_type, &Type::string(), id, None)?;

                let mut diverges = false;
                for part in parts {
                    match &part.node {
                        ast::FStringPart::String(_) => {
                            // always ok!
                        }
                        ast::FStringPart::Expr(expr) => {
                            // Each f-string part can be of any type
                            let ty = self.fresh_var();
                            let ctx = ctx.with_type(ty.clone());
                            diverges |= self.expr(scope, &ctx, expr)?;

                            // But that type needs a `to_string` method, which
                            // we are going to try to resolve later.
                            self.obligations.push(Obligation::ResolveMethod {
                                id: part.id,
                                receiver: ty.clone(),
                                ident: "to_string".into(),
                                parameter_types: vec![ty.clone()],
                                return_type: Type::string(),
                            });
                        }
                    }
                }

                Ok(diverges)
            }
        }
    }

    fn literal(&mut self, ctx: &ExprContext, lit: &ast::Meta<ast::Literal>) -> TypeResult<bool> {
        let t = match lit.node {
            ast::Literal::String(_) => Type::string(),
            ast::Literal::Bool(_) => Type::bool(),
            ast::Literal::Integer(_) => self
                .types
                .unionfind
                .fresh(|n| Type::IVar(n, MustBeSigned::No)),
            ast::Literal::Float(_) => self.types.unionfind.fresh(Type::FVar),
            ast::Literal::Unit => Type::unit(),
        };

        self.unify(&ctx.expected_type, &t, lit.id, None)?;
        Ok(false)
    }

    fn match_expr(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        mat: &ast::Meta<ast::Match>,
    ) -> TypeResult<bool> {
        let span = mat.id;
        let ast::Match { expr, arms } = &mat.node;

        let diverges;

        let t_expr = {
            let examinee_type = self.fresh_var();
            let ctx = ctx.with_type(&examinee_type);
            diverges = self.expr(scope, &ctx, expr)?;
            self.resolve_type(&examinee_type)
        };

        if diverges {
            todo!("make a pretty error")
        }

        let Type::Name(type_name) = &t_expr else {
            return Err(self.error_can_only_match_on_enum(&t_expr, expr.id));
        };

        let type_def = self.types.resolve_type_name(type_name);
        let Some(variants) = type_def.match_patterns(&type_name.args) else {
            return Err(self.error_can_only_match_on_enum(&t_expr, expr.id));
        };

        // We'll keep track of used variants to do some basic
        // exhaustiveness checking.
        let mut used_variants = Vec::new();

        // Match diverges if all its branches diverge
        let mut arms_diverge = true;

        // Whether there is a default arm present (with '_')
        let mut default_arm = false;

        let match_id = self.match_counter;
        self.match_counter += 1;

        for ast::MatchArm {
            pattern,
            guard,
            body,
        } in arms
        {
            // Anything after default is unreachable
            if default_arm {
                return Err(self.error_unreachable_expression(body));
            }

            match &pattern.node {
                ast::Pattern::Underscore => {
                    let sty = ScopeType::MatchArm(match_id, None);
                    let arm_scope = self.types.wrap(scope, sty);

                    if let Some(guard) = guard {
                        let ctx = ctx.with_type(Type::bool());
                        self.expr(arm_scope, &ctx, guard)?;
                    } else {
                        default_arm = true;
                    }

                    arms_diverge &= self.block(arm_scope, ctx, body)?;
                }
                ast::Pattern::EnumVariant {
                    variant,
                    fields: data_field,
                } => {
                    let Some(idx) = variants.iter().position(|v| v.name == variant.node) else {
                        return Err(self.error_variant_does_not_exist(variant, &t_expr));
                    };

                    let variant_already_used = used_variants.contains(&variant.node);
                    if variant_already_used {
                        println!(
                            "WARNING: Variant occurs multiple times in match! This arm is unreachable"
                        );
                    }

                    let field_types = &variants[idx].fields;
                    let sty = ScopeType::MatchArm(match_id, Some(idx));
                    let arm_scope = self.types.wrap(scope, sty);

                    match (field_types.as_slice(), data_field) {
                        ([], None) => {} // ok!
                        ([], Some(_)) => {
                            return Err(self.error_variant_does_not_have_fields(variant, &t_expr));
                        }
                        (field_types, Some(fields)) => {
                            if field_types.len() != fields.len() {
                                return Err(TypeError::number_of_arguments_dont_match(
                                    "pattern",
                                    variant,
                                    field_types.len(),
                                    fields.len(),
                                ));
                            }
                            for (ty, field) in field_types.iter().zip(&fields.node) {
                                self.insert_var(arm_scope, field.clone(), ty)?;
                            }
                        }
                        (_field_types, None) => {
                            return Err(self.error_need_arguments_on_pattern(variant, &t_expr));
                        }
                    }

                    if let Some(guard) = guard {
                        self.expr(arm_scope, &ctx.with_type(Type::bool()), guard)?;
                    } else if !variant_already_used {
                        // If there is a guard we don't mark the variant as used,
                        // because the guard could evaluate to false and hence the
                        // variant _should_ actually be used again.
                        used_variants.push(variant.node);
                    }

                    arms_diverge &= self.block(arm_scope, ctx, body)?;
                }
            }
        }

        if !default_arm && used_variants.len() < variants.len() {
            let mut missing_variants = Vec::new();
            for v in variants {
                let v = v.name;
                if !used_variants.contains(&v) {
                    missing_variants.push(v);
                }
            }
            return Err(self.error_nonexhaustive_match(span, &missing_variants));
        }

        Ok(arms_diverge)
    }

    fn binop(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        op: ast::BinOp,
        span: ast::MetaId,
        left: &ast::Meta<ast::Expr>,
        right: &ast::Meta<ast::Expr>,
    ) -> TypeResult<bool> {
        if let ast::BinOp::Add = op {
            let var = self.fresh_var();
            let ctx_new = ctx.with_type(var.clone());

            let mut diverges = false;
            diverges |= self.expr(scope, &ctx_new, left)?;

            let resolved = self.resolve_type(&var);

            if Type::string() == resolved {
                diverges |= self.expr(scope, &ctx_new, right)?;

                self.unify(&ctx.expected_type, &Type::string(), span, None)?;

                let ty = ResolvedName::global("String");

                let function = self.get_function_in_type(ty, "append".into()).unwrap();
                self.types.function_calls.insert(span, function);
                return Ok(diverges);
            }
        }

        match op {
            ast::BinOp::And | ast::BinOp::Or => {
                self.unify(&ctx.expected_type, &Type::bool(), span, None)?;

                let ctx = ctx.with_type(Type::bool());

                let mut diverges = false;
                diverges |= self.expr(scope, &ctx, left)?;
                diverges |= self.expr(scope, &ctx, right)?;
                Ok(diverges)
            }
            ast::BinOp::Lt | ast::BinOp::Le | ast::BinOp::Gt | ast::BinOp::Ge => {
                self.unify(&ctx.expected_type, &Type::bool(), span, None)?;

                let ty = self.fresh_var();
                let ctx = ctx.with_type(ty.clone());

                let mut diverges = false;
                diverges |= self.expr(scope, &ctx, left)?;
                if self.types.is_numeric_type(&ty) {
                    diverges |= self.expr(scope, &ctx, right)?;
                    Ok(diverges)
                } else {
                    Err(self.error_expected_numeric_value(left, &ty))
                }
            }
            ast::BinOp::Eq | ast::BinOp::Ne => {
                self.unify(&ctx.expected_type, &Type::bool(), span, None)?;
                let ctx = ctx.with_type(self.fresh_var());

                let mut diverges = false;
                diverges |= self.expr(scope, &ctx, left)?;
                diverges |= self.expr(scope, &ctx, right)?;

                let ty = self.resolve_type(&ctx.expected_type);
                let comparable = match ty {
                    Type::IVar(_, _) | Type::FVar(_) | Type::Never => true,
                    Type::Name(type_name) => {
                        // This could be relaxed in the future but we only
                        // support comparing primitives now.
                        matches!(
                            self.types.resolve_type_name(&type_name),
                            TypeDef::Scalar(_) | TypeDef::Bool | TypeDef::String
                        )
                    }
                    Type::Record(..)
                    | Type::RecordVar(..)
                    | Type::Unit
                    | Type::Var(_)
                    | Type::ExplicitVar(_)
                    | Type::Function(_, _) => false,
                };

                if !comparable {
                    return Err(TypeError::simple(
                        "type cannot be compared",
                        "cannot be compared",
                        span,
                    ));
                }

                Ok(diverges)
            }
            ast::BinOp::Add | ast::BinOp::Sub | ast::BinOp::Mul | ast::BinOp::Div => {
                let operand_ty = self.fresh_var();
                let new_ctx = ctx.with_type(operand_ty.clone());

                let mut diverges = false;
                diverges |= self.expr(scope, &new_ctx, left)?;

                if self.types.is_numeric_type(&operand_ty) {
                    diverges |= self.expr(scope, &new_ctx, right)?;
                    self.unify(&ctx.expected_type, &operand_ty, span, None)?;
                    Ok(diverges)
                } else {
                    Err(self.error_expected_numeric_value(left, &operand_ty))
                }
            }
            ast::BinOp::In | ast::BinOp::NotIn => {
                self.unify(&ctx.expected_type, &Type::bool(), span, None)?;

                let ty = self.fresh_var();

                let mut diverges = false;
                diverges |= self.expr(scope, &ctx.with_type(&ty), left)?;
                diverges |= self.expr(scope, &ctx.with_type(Type::list(ty)), right)?;

                Ok(diverges)
            }
        }
    }

    /// Find a function in a type (i.e. a static method)
    fn get_function_in_type(
        &mut self,
        ty: ResolvedName,
        method: ast::ident::Ident,
    ) -> Option<Function> {
        let type_dec = self.types.get_declaration(ty);
        let scope = type_dec.scope.unwrap();
        self.get_function(ResolvedName::new(scope, method))
    }

    pub(super) fn get_method(&mut self, ty: &Type, method: &ast::Ident) -> Option<Function> {
        let ty = self.types.resolve(ty);
        let Type::Name(type_name) = ty else {
            return None;
        };
        let type_dec = self.types.get_declaration(type_name.name);
        let type_scope = type_dec.scope.unwrap();

        let dec = self.types.resolve_name(type_scope, method, false)?;

        let DeclKind::Method(Some(func_dec)) = dec.kind else {
            return None;
        };

        let Type::Function(parameter_types, return_type) = func_dec.ty else {
            ice!("Function must have function type");
        };

        Some(Function {
            signature: Signature {
                parameter_types: parameter_types.clone(),
                return_type: (*return_type).clone(),
            },
            name: dec.name,
            vars: Vec::new(),
            definition: func_dec.def,
        })
    }

    /// Resolve a function name to a function
    fn get_function(&mut self, name: ResolvedName) -> Option<Function> {
        let dec = self.types.get_declaration(name);

        let (DeclKind::Function(Some(func_dec)) | DeclKind::Method(Some(func_dec))) = dec.kind
        else {
            return None;
        };

        let Type::Function(parameter_types, return_type) = func_dec.ty else {
            ice!("Function must have function type");
        };

        Some(Function {
            signature: Signature {
                parameter_types: parameter_types.clone(),
                return_type: (*return_type).clone(),
            },
            name,
            vars: Vec::new(),
            definition: func_dec.def,
        })
    }

    fn check_arguments(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        call_type: &str,
        name: &ast::Ident,
        params: &[Type],
        args: &[ast::Meta<ast::Expr>],
    ) -> TypeResult<bool> {
        if args.len() != params.len() {
            return Err(TypeError::number_of_arguments_dont_match(
                call_type,
                name,
                params.len(),
                args.len(),
            ));
        }

        let mut diverges = false;
        for (arg, ty) in args.iter().zip(params) {
            diverges |= self.expr(scope, &ctx.with_type(ty), arg)?;
        }

        Ok(diverges)
    }

    fn record_fields(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        field_types: Vec<(ast::Ident, Type)>,
        record: &ast::Record,
        span: ast::MetaId,
    ) -> TypeResult<bool> {
        let mut used_fields = HashSet::new();
        let mut missing_fields: Vec<_> = field_types.iter().map(|x| x.0.node).collect();
        let mut invalid_fields = Vec::new();
        let mut duplicate_fields = Vec::new();

        for (ident, _) in &record.fields {
            if used_fields.contains(&ident.node) {
                duplicate_fields.push(ident);
            } else if let Some(idx) = missing_fields.iter().position(|f| f == &ident.node) {
                missing_fields.remove(idx);
                used_fields.insert(ident.node);
            } else {
                invalid_fields.push(ident);
            }
        }

        if !invalid_fields.is_empty() || !duplicate_fields.is_empty() || !missing_fields.is_empty()
        {
            return Err(TypeError::field_mismatch(
                span,
                invalid_fields,
                duplicate_fields,
                missing_fields,
            ));
        }

        let mut diverges = false;
        for (ident, expr) in &record.fields {
            let expected_type = field_types
                .iter()
                .find_map(|(s, t)| (s.node == ident.node).then_some(t))
                .unwrap()
                .clone();

            diverges |= self.expr(scope, &ctx.with_type(expected_type), expr)?;
        }
        Ok(diverges)
    }

    /// Resolve the module part of a path
    ///
    /// In a path, we might start with a path of modules and at some point, we
    /// transition into other items. This function resolves that first part.
    pub fn resolve_module_part_of_path<'a>(
        &self,
        mut scope: ScopeRef,
        mut idents: impl Iterator<Item = &'a ast::Ident>,
    ) -> TypeResult<(&'a ast::Ident, Declaration)> {
        let mut ident = idents.next().unwrap();

        while ident.node == "super".into() {
            let Some(dec) = self.types.parent_module(scope) else {
                return Err(TypeError::simple(
                    "could not resolve name: too many leading `super` keywords",
                    "too many leading `super` keywords",
                    ident.id,
                ));
            };

            let Some(s) = dec.scope else {
                unreachable!();
            };

            scope = s;

            let Some(tmp_ident) = idents.next() else {
                return Ok((ident, dec));
            };

            ident = tmp_ident;
        }

        // Keep checking modules until we find something that isn't a module
        // The current implementation is a bit strange because it uses
        // resolve_name, but after the first identifier, it should actually
        // not really traverse the scope graph.
        let mut recurse = true;
        loop {
            if ident.node == "super".into() {
                return Err(TypeError::simple(
                    "could not resolve name: too many leading `super` keywords",
                    "too many leading `super` keywords",
                    ident.id,
                ));
            }
            let Some(stub) = self.types.resolve_name(scope, ident, recurse) else {
                return Err(TypeError::not_defined(ident));
            };

            let Some(s) = stub.scope else {
                return Ok((ident, stub));
            };
            let Some(i) = idents.next() else {
                return Ok((ident, stub));
            };

            scope = s;
            ident = i;
            recurse = false;
        }
    }

    /// Resolve a path of identifiers into a value
    fn resolve_expression_path(
        &mut self,
        scope: ScopeRef,
        ast::Path { idents }: &ast::Path,
    ) -> TypeResult<ResolvedPath> {
        let mut idents = idents.iter();
        let (ident, dec) = self.resolve_module_part_of_path(scope, &mut idents)?;

        #[allow(clippy::match_same_arms)]
        match &dec.kind {
            // We have reached the end of the iterator, but are still a module even though we
            // should be an expression. Time to error!
            DeclKind::Module => Err(self.error_expected_value(ident, &dec)),
            // We ended on a type which is not a valid expression, so we yield an error.
            DeclKind::Type(_) => Err(self.error_expected_value(ident, &dec)),
            // A type param is not a valid value
            DeclKind::TypeParam(_) => Err(self.error_expected_value(ident, &dec)),
            // We ended on a function, which means there can be no identifiers left
            DeclKind::Function(Some(func_dec)) | DeclKind::Method(Some(func_dec)) => {
                if let Some(field) = idents.next() {
                    return Err(self.error_no_field_on_type(&func_dec.ty, field));
                }
                let Type::Function(parameter_types, return_type) = &func_dec.ty else {
                    panic!()
                };
                Ok(ResolvedPath::Function {
                    name: dec.name,
                    definition: func_dec.def.clone(),
                    signature: Signature {
                        parameter_types: parameter_types.clone(),
                        return_type: (**return_type).clone(),
                    },
                })
            }
            // We have a value, so the rest of the idents should be field accesses
            // optionally ending with a method.
            DeclKind::Value(kind, root_ty) => {
                let mut fields = Vec::new();
                let mut ty = root_ty.clone();

                // We loop until we either find the last field or a method.
                // The method must be the last identifier.
                while let Some(field) = idents.next() {
                    if let Some(function) = self.get_method(&ty, field) {
                        if let Some(field) = idents.next() {
                            return Err(self.error_no_field_on_type(&ty, field));
                        }
                        return Ok(ResolvedPath::Method {
                            value: PathValue {
                                name: dec.name,
                                kind: kind.clone(),
                                root_ty: root_ty.clone(),
                                fields,
                            },
                            name: function.name,
                            definition: function.definition,
                            signature: function.signature,
                        });
                    }

                    // The field is not a method, so we try a field access.
                    let Ok(new_ty) = self.access_field(&ty, field) else {
                        return Err(self.error_no_field_or_method_on_type(&ty, field));
                    };
                    ty = new_ty;
                    fields.push((field.node, ty.clone()));
                }

                Ok(ResolvedPath::Value(PathValue {
                    name: dec.name,
                    kind: kind.clone(),
                    root_ty: root_ty.clone(),
                    fields,
                }))
            }
            DeclKind::Variant(Some((ty, variant))) => {
                if let Some(_field) = idents.next() {
                    todo!("make a nice error for variant cannot have field")
                }
                Ok(ResolvedPath::EnumConstructor {
                    ty: ty.clone(),
                    variant: variant.clone(),
                })
            }
            DeclKind::Variant(None) | DeclKind::Function(None) | DeclKind::Method(None) => {
                ice!("These should be declared at this point")
            }
        }
    }

    pub fn resolve_type_path(
        &self,
        scope: ScopeRef,
        path: &ast::Meta<ast::Path>,
        params: &[ast::Meta<ast::TypeExpr>],
    ) -> TypeResult<Type> {
        let mut idents = path.idents.iter();
        let (ident, declaration) = self.resolve_module_part_of_path(scope, &mut idents)?;

        match declaration.kind {
            DeclKind::Value(..)
            | DeclKind::Function(..)
            | DeclKind::Method(..)
            | DeclKind::Variant(..)
            | DeclKind::Module => Err(TypeError::expected_type(ident, declaration)),
            DeclKind::TypeParam(ident) => {
                if !params.is_empty() {
                    return Err(TypeError::simple(
                        format!("expected 0 type parameters, got {}", params.len()),
                        "expected 0 type parameters",
                        path.id,
                    ));
                }
                Ok(Type::ExplicitVar(ident))
            }
            DeclKind::Type(type_or_stub) => {
                let num_params = match type_or_stub {
                    TypeOrStub::Type(type_def) => type_def.type_name().args.len(),
                    TypeOrStub::Stub { num_params } => num_params,
                };

                if num_params != params.len() {
                    return Err(TypeError::simple(
                        format!(
                            "expected {num_params} type parameters, got {}",
                            params.len()
                        ),
                        format!("expected {num_params} type parameters"),
                        path.id,
                    ));
                }

                let params = params
                    .iter()
                    .map(|t| self.eval_ty(scope, t))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Type::Name(TypeName {
                    name: declaration.name,
                    args: params,
                }))
            }
        }
    }

    fn access_field(&mut self, ty: &Type, field: &ast::Ident) -> TypeResult<Type> {
        let ty = self.types.resolve(ty);

        let fields_vec;
        let fields = match &ty {
            Type::Record(fields) | Type::RecordVar(_, fields) => Some(fields),
            Type::Name(name) => {
                let type_def = self.types.resolve_type_name(name);
                fields_vec = type_def.record_fields(&name.args);
                fields_vec.as_ref()
            }
            _ => None,
        };

        if let Some(fields) = fields
            && let Some((_, t)) = fields.iter().find(|(s, _)| s.node == field.node)
        {
            return Ok(t.clone());
        }

        Err(self.error_no_field_on_type(&ty, field))
    }

    fn path_function_call(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        id: ast::MetaId,
        p: &ast::Meta<ast::Path>,
        args: &ast::Meta<Vec<ast::Meta<ast::Expr>>>,
    ) -> TypeResult<bool> {
        let last_ident = p.idents.last().unwrap();
        let resolved_path = self.resolve_expression_path(scope, p)?;

        self.types.path_kinds.insert(p.id, resolved_path.clone());

        let (name, definition, signature, description) = match &resolved_path {
            ResolvedPath::Function {
                name,
                definition,
                signature,
            } => (name, definition, signature, "function"),
            ResolvedPath::StaticMethod {
                name,
                definition,
                signature,
            } => (name, definition, signature, "static method"),
            ResolvedPath::Method {
                value: _,
                definition,
                name,
                signature,
            } => (name, definition, signature, "method"),
            ResolvedPath::EnumConstructor {
                ty: type_def,
                variant,
            } => {
                let ty = type_def.instantiate(|| self.fresh_var());
                let Type::Name(type_name) = &ty else { ice!() };

                let original_name = type_def.type_name();
                let subs = original_name.args.iter().zip(&type_name.args);
                let variant = variant.substitute_iter(subs);

                let diverges = self.check_arguments(
                    scope,
                    ctx,
                    "enum constructor",
                    last_ident,
                    &variant.fields,
                    args,
                )?;

                self.unify(&ctx.expected_type, &ty, id, None)?;
                return Ok(diverges);
            }
            ResolvedPath::Value(_) => {
                return Err(self.error_expected_function(last_ident, &resolved_path));
            }
        };

        // Tell the lower stage about the kind of function this is.
        self.types.function_calls.insert(
            id,
            Function {
                signature: signature.clone(),
                name: *name,
                vars: Vec::new(),
                definition: definition.clone(),
            },
        );

        let last_ident = p.idents.last().unwrap();

        // Skip the first parameter if we are checking a method
        let params = if let ResolvedPath::Method { .. } = resolved_path {
            &signature.parameter_types[1..]
        } else {
            &signature.parameter_types
        };

        let diverges = self.check_arguments(scope, ctx, description, last_ident, params, args)?;

        self.unify(&ctx.expected_type, &signature.return_type, id, None)?;
        Ok(diverges)
    }

    fn method_call(
        &mut self,
        scope: ScopeRef,
        ctx: &ExprContext,
        id: ast::MetaId,
        ty: Type,
        field: &ast::Ident,
        args: &ast::Meta<Vec<ast::Meta<ast::Expr>>>,
    ) -> TypeResult<bool> {
        let Some(function) = self.get_method(&ty, field) else {
            return Err(self.error_no_method_on_type(&ty, field));
        };

        let params = &function.signature.parameter_types[1..];
        let diverges = self.check_arguments(scope, ctx, "method", field, params, args)?;
        self.unify(
            &ctx.expected_type,
            &function.signature.return_type,
            id,
            None,
        )?;

        self.types.function_calls.insert(id, function);
        Ok(diverges)
    }
}
