use super::{FlowBuilder, Function, Mir, Place, Projection, TypedValue, TypedVar, Value};
use crate::{
    ast, ice,
    label::LabelStore,
    module::ModuleTree,
    print::{IrPrinter, Printable, TypeDisplay},
    runtime::{Runtime, RuntimeFunctionRef},
    types::{
        self, DeclKind, FunctionDefinition, PathValue, ResolvedName, ResolvedPath, Signature, Type,
        TypeDef, TypeInfo, ValueKind,
    },
    var::Var,
};
use std::{any::TypeId, sync::Arc};

/// All the stack slots that are allocated in each scope
///
/// The first element contains the variables allocated in the function body,
/// the last element is our current scope and between are the parent
/// scopes. At the end of a block we drop all variables in the last element
/// and then pop that element.
pub(crate) struct Stack {
    variables: super::Variables,
    slots: Vec<Vec<TypedVar>>,
}

impl Stack {
    pub fn push_frame(&mut self) {
        self.slots.push(Vec::new());
    }

    #[must_use]
    pub fn pop_frame(&mut self) -> Vec<TypedVar> {
        self.slots.pop().unwrap()
    }

    #[must_use]
    pub fn last_frame(&mut self) -> &mut Vec<TypedVar> {
        self.slots.last_mut().unwrap()
    }

    pub fn insert(&mut self, var: TypedVar) {
        self.slots.last_mut().unwrap().push(var);
    }

    pub fn declare(&mut self, var: TypedVar) {
        self.variables.variables.push(var);
    }

    pub fn add_live(&mut self, var: Var, ty: Type) {
        self.declare(TypedVar(var, ty.clone()));
        self.insert(TypedVar(var, ty));
    }

    #[must_use]
    pub fn remove_live(&mut self, var: &Var) -> Option<Type> {
        let slot = self.last_frame();
        let index = slot.iter().position(|TypedVar(v, _)| v == var);
        index.map(|index| slot.remove(index).1)
    }

    /// Create a new unique temporary variable that won't be dropped
    pub fn undropped_tmp(&mut self) -> Var {
        let scope = self.variables.scope;
        self.variables.tmp_idx += 1;
        Var::Temp(scope, self.variables.tmp_idx - 1)
    }

    pub fn undropped_tmp_ty(&mut self, ty: Type) -> TypedVar {
        TypedVar(self.undropped_tmp(), ty)
    }

    pub fn typed_undropped_tmp(&mut self, ty: Type) -> TypedVar {
        let var = self.undropped_tmp_ty(ty);
        self.declare(var.clone());
        var
    }

    /// Create a new temporary variable that will be dropped
    fn tmp_live(&mut self, ty: Type) -> TypedVar {
        let var = self.typed_undropped_tmp(ty.clone());
        self.insert(var.clone());
        var
    }
}

pub struct Lowerer<'r> {
    pub(crate) emit: FlowBuilder,
    pub(crate) rt: &'r Runtime,
    pub(crate) types: &'r mut TypeInfo,
    pub(crate) labels: &'r mut LabelStore,
    pub(crate) return_type: Type,
    pub(crate) vars: Stack,
}

impl<'r> Lowerer<'r> {
    fn new(
        rt: &'r Runtime,
        types: &'r mut TypeInfo,
        function_name: &ast::Ident,
        labels: &'r mut LabelStore,
    ) -> Self {
        let scope = types.function_scope(function_name);
        let ty = types.type_of(function_name);
        let Type::Function(_, return_type) = ty else {
            ice!()
        };
        Self {
            types,
            rt,
            return_type: (*return_type).clone(),
            emit: FlowBuilder {
                storage: Vec::new(),
            },
            vars: Stack {
                slots: Vec::new(),
                variables: super::Variables {
                    scope,
                    tmp_idx: 0,
                    variables: Vec::new(),
                },
            },
            labels,
        }
    }

    /// Lower a syntax tree
    pub(crate) fn tree(
        rt: &'r Runtime,
        types: &'r mut TypeInfo,
        tree: &ModuleTree,
        labels: &mut LabelStore,
    ) -> Mir {
        let mut functions = Vec::new();

        for m in &tree.modules {
            for d in &m.ast.declarations {
                match d {
                    ast::Decl::Fmap(x) => {
                        functions.push(Lowerer::new(rt, types, &x.ident, labels).filter_map(x));
                    }
                    ast::Decl::Func(x) => {
                        functions.push(Lowerer::new(rt, types, &x.ident, labels).function(x));
                    }
                    // We give tests special names, so that they can't be referenced from Roto.
                    // It's a bit of a hack, but works well enough.
                    ast::Decl::Test(x) => {
                        let name = x.ident.clone().map(|ident| format!("test#{ident}").into());
                        functions.push(Lowerer::new(rt, types, &name, labels).test(x));
                    }
                    // Ignore the rest
                    _ => {}
                }
            }
        }
        Mir { functions }
    }

    /// Lower a filtermap
    fn filter_map(self, fm: &ast::FilterMap) -> Function {
        let ast::FilterMap {
            ident,
            body,
            params,
            ..
        } = fm;

        let Type::Function(_, ret) = self.types.type_of(ident) else {
            ice!("The type of a filter(map) must be a function");
        };
        self.function_like(ident, params, &ret, body)
    }

    fn function(self, function: &ast::FuncDecl) -> Function {
        let name = self.types.resolved_name(&function.ident);
        let dec = self.types.get_declaration(name);

        let DeclKind::Function(Some(func_dec)) = dec.kind else {
            ice!();
        };

        let Type::Function(_, ret) = self.types.resolve(&func_dec.ty) else {
            ice!("A function must have a function type");
        };

        self.function_like(&function.ident, &function.params, &ret, &function.body)
    }

    fn test(self, test: &ast::Test) -> Function {
        let ident = test.ident.clone();
        let ident = ident.map(|ident| format!("test#{ident}").into());
        let return_type = Type::verdict(Type::unit(), Type::unit());
        let params = ast::Params(Vec::new());
        self.function_like(&ident, &params, &return_type, &test.body)
    }

    /// Lower a function-like construct (i.e. a function, filtermap or test)
    fn function_like(
        mut self,
        ident: &ast::Ident,
        params: &ast::Params,
        return_type: &Type,
        body: &ast::Meta<ast::Block>,
    ) -> Function {
        let scope = self.types.function_scope(ident);
        let label = self.labels.label(None, "entry");
        self.emit.new_block(label);

        let mut parameter_types = Vec::new();

        for (x, _) in &params.0 {
            let ty = self.types.type_of(x);
            parameter_types.push((self.types.resolved_name(x), ty));
        }

        self.vars.push_frame();

        let mut parameters = Vec::new();
        for (name, ty) in &parameter_types {
            let var = Var::Ident(name.scope, name.ident);
            parameters.push(var);
            self.vars.add_live(var, ty.clone());
        }

        let signature = Signature {
            parameter_types: parameter_types.iter().map(|x| &x.1).cloned().collect(),
            return_type: return_type.clone(),
        };

        let last = self.block(body);

        let to_drop = self.vars.pop_frame();
        self.emit.drop_frame(to_drop);

        let tmp = self.assign_to_var(TypedValue(last, return_type.clone()));
        self.emit.emit_return(tmp.0);

        let name = self.types.resolved_name(ident);
        let name = self.types.full_name(&name);

        Function {
            name,
            variables: super::Variables {
                scope,
                tmp_idx: self.vars.variables.tmp_idx,
                variables: self.vars.variables.variables,
            },
            parameters,
            blocks: self.emit.storage,
            signature,
        }
    }

    pub(crate) fn block(&mut self, block: &ast::Meta<ast::Block>) -> Value {
        self.vars.push_frame();

        // Resulting operand is ignored
        for stmt in &block.body {
            match &**stmt {
                ast::Stmt::Let(ident, _, expr) => {
                    let name = self.types.resolved_name(ident);
                    let value = self.expr_ty_val(expr);
                    let to = TypedVar(Var::Ident(name.scope, **ident), value.1.clone());
                    self.vars.add_live(to.0, to.1.clone());
                    self.do_assign(to, value);
                }
                ast::Stmt::Expr(expr) => {
                    let value = self.expr_ty_val(expr);
                    let var = self.assign_to_var(value).0;
                    let ty = self.vars_remove_live(&var);
                    self.emit.emit_drop(Place::new(var, ty.clone()), ty);
                }
            }
        }

        let op = match &block.last {
            Some(expr) => self.expr(expr),
            None => Value::Const(ast::Literal::Unit, Type::unit()),
        };

        let ty = self.types.type_of(block);
        let final_var = self.assign_to_var(TypedValue(op, ty));
        self.vars_remove_live(&final_var.0);

        let to_drop = self.vars.pop_frame();

        // If the block diverges, which happens for instance with an explicit
        // return, then we don't need to drop anything. That will only generate
        // noise in the MIR.
        if !self.types.diverges(block) {
            self.emit.drop_frame(to_drop);
        }

        Value::Move(final_var)
    }

    pub(crate) fn expr(&mut self, expr: &ast::Meta<ast::Expr>) -> Value {
        let id = expr.id;
        match &**expr {
            ast::Expr::Return(kind, expr) => self.r#return(kind, expr.as_deref()),
            ast::Expr::Literal(literal) => {
                let ty = self.types.type_of(literal);
                Value::Const((**literal).clone(), ty)
            }
            ast::Expr::Match(r#match) => self.r#match(id, r#match),
            ast::Expr::FunctionCall(function, args) => self.function_call(id, function, args),
            ast::Expr::Access(expr, field) => {
                let val = self.expr_ty_val(expr);
                let var = self.assign_to_var(val.clone()).0;
                Value::Clone(Place::new(var, val.1).with_field(**field))
            }
            ast::Expr::Path(path) => match self.types.path_kind(path).clone() {
                ResolvedPath::Value(value) => self.path_value(&value),
                ResolvedPath::EnumConstructor { ty: _, variant } => {
                    let to = self.vars.tmp_live(self.types.type_of(id));
                    self.emit.set_discriminant(to.clone(), variant);
                    Value::Move(to)
                }
                _ => ice!("should be rejected by the type checker"),
            },
            ast::Expr::Record(record) | ast::Expr::TypedRecord(_, record) => {
                let to = self.vars.tmp_live(self.types.type_of(id));
                for (s, expr) in &record.fields {
                    let field = self.expr_ty_val(expr);
                    let to = Place::from(to.clone()).with_field(**s);
                    self.do_assign(to, field);
                }
                Value::Move(to)
            }
            ast::Expr::List(_list) => todo!(),
            ast::Expr::Not(expr) => {
                let val = self.expr_ty_val(expr);
                assert_eq!(val.1, Type::bool());
                let var = self.assign_to_var(val.clone());
                Value::Unary(super::UnOp::Not, var)
            }
            ast::Expr::Negate(expr) => {
                let val = self.expr_ty_val(expr);
                let var = self.assign_to_var(val.clone());
                Value::Unary(super::UnOp::Neg, var)
            }
            ast::Expr::Assign(expr, field) => self.assign(expr, field),
            ast::Expr::Binary(left, op, right) => self.binop(left, *op, right),
            ast::Expr::Select(cond, accept, reject) => {
                self.select(id, cond, accept, reject.as_ref())
            }
            ast::Expr::Loop(cond, body) => self.r#while(cond, body),
            ast::Expr::QuestionMark(expr) => self.question_mark(expr),
            ast::Expr::FString(parts) => self.f_string(parts),
        }
    }

    fn r#return(&mut self, kind: &ast::ReturnKind, expr: Option<&ast::Meta<ast::Expr>>) -> Value {
        let val = match expr {
            Some(expr) => self.expr_ty_val(expr),
            None => TypedValue(Value::Const(ast::Literal::Unit, Type::unit()), Type::unit()),
        };

        match kind {
            ast::ReturnKind::Return => self.return_value(val.0),
            ast::ReturnKind::Accept => {
                let ty = self.return_type.clone();
                let val = self.make_enum(ty, "Accept".into(), &[val]);
                self.return_value(val)
            }
            ast::ReturnKind::Reject => {
                let ty = self.return_type.clone();
                let val = self.make_enum(ty, "Reject".into(), &[val]);
                self.return_value(val)
            }
        }
    }

    fn question_mark(&mut self, expr: &ast::Meta<ast::Expr>) -> Value {
        let current_label = self.emit.current_label();
        let is_none = self.labels.label(current_label, "return-none");
        let is_some = self.labels.next(current_label);

        let examinee = self.expr_ty_val(expr);
        let examinee = self.assign_to_var(examinee);
        let cond = self.vars.undropped_tmp_ty(Type::discriminant());
        let to = Place::from(cond.clone());
        self.emit.assign(
            to,
            Type::discriminant(),
            Value::Discriminant(examinee.clone()),
        );
        self.emit.branch(cond, is_none, is_some);

        self.emit.new_block(is_none);
        let ty = self.return_type.clone();
        let val = self.make_enum(ty, "None".into(), &[]);
        let _ = self.return_value(val);

        self.emit.new_block(is_some);
        let ty = self.types.type_of(expr);
        Value::Clone(Place::new(examinee.0, ty).with_variant("Some", 0))
    }

    fn return_value(&mut self, val: Value) -> Value {
        let var = self.assign_to_var(TypedValue(val, self.return_type.clone()));
        let _ty = self.vars_remove_live(&var.0);
        for frame in self.vars.slots.clone().iter().rev() {
            self.emit.drop_frame(frame.iter().cloned());
        }
        self.emit.emit_return(var.0);
        Value::Const(ast::Literal::Unit, Type::unit())
    }

    fn function_call(
        &mut self,
        id: ast::MetaId,
        function: &ast::Meta<ast::Expr>,
        arguments: &ast::Meta<Vec<ast::Meta<ast::Expr>>>,
    ) -> Value {
        match &**function {
            ast::Expr::Path(p) => match self.types.path_kind(p).clone() {
                ResolvedPath::Method {
                    value, signature, ..
                } => {
                    let op = self.path_value(&value.clone());
                    let ty = signature.parameter_types[0].clone();
                    let func = self.types.function(id).clone();
                    self.normalized_function_call(&func, Some(TypedValue(op, ty)), arguments)
                }
                ResolvedPath::Function { .. } | ResolvedPath::StaticMethod { .. } => {
                    let func = self.types.function(id).clone();
                    self.normalized_function_call(&func, None, arguments)
                }
                ResolvedPath::EnumConstructor { ty: _, variant } => {
                    let ty = self.types.type_of(id);
                    self.enum_constructor(&ty, variant.name, arguments)
                }
                ResolvedPath::Value { .. } => ice!(),
            },
            ast::Expr::Access(expr, _) => {
                let val = self.expr_ty_val(expr);
                let func = self.types.function(id).clone();
                self.normalized_function_call(&func, Some(val), arguments)
            }
            _ => ice!(),
        }
    }

    fn normalized_function_call(
        &mut self,
        func: &types::Function,
        receiver: Option<TypedValue>,
        arguments: &[ast::Meta<ast::Expr>],
    ) -> Value {
        let name = func.name;

        let mut args = Vec::new();
        if let Some(receiver) = receiver {
            args.push(self.drop_by_callee(receiver));
        }
        args.extend(arguments.iter().map(|expr| {
            let val = self.expr_ty_val(expr);
            self.drop_by_callee(val)
        }));

        match func.definition {
            FunctionDefinition::Runtime(func_ref) => Value::CallRuntime {
                func: func_ref,
                args,
            },
            FunctionDefinition::Roto => Value::Call { func: name, args },
        }
    }

    // These values will be dropped by the callee
    fn drop_by_callee(&mut self, TypedValue(val, ty): TypedValue) -> Var {
        let tmp = self.vars.typed_undropped_tmp(ty.clone());
        self.do_assign(tmp.clone(), TypedValue(val, ty));
        tmp.0
    }

    fn enum_constructor(
        &mut self,
        ty: &Type,
        variant: ast::ident::Ident,
        arguments: &[ast::Meta<ast::Expr>],
    ) -> Value {
        let ty = ty.clone();
        let arguments: Vec<_> = arguments.iter().map(|a| self.expr_ty_val(a)).collect();
        self.make_enum(ty, variant, &arguments)
    }

    fn make_enum(
        &mut self,
        ty: Type,
        variant: ast::ident::Ident,
        arguments: &[TypedValue],
    ) -> Value {
        let to = self.vars.tmp_live(ty.clone());

        let Type::Name(name) = self.types.resolve(&ty) else {
            ice!()
        };

        let TypeDef::Enum(_, variants) = self.types.resolve_type_name(&name) else {
            ice!("Not an enum: {}", ty.display(self.types))
        };

        let Some(variant) = variants.iter().find(|v| v.name == variant) else {
            ice!()
        };

        self.emit.set_discriminant(to.clone(), variant.clone());
        for (i, field) in arguments.iter().enumerate() {
            let to = Place::from(to.clone()).with_variant(variant.name, i);
            self.do_assign(to, field.clone());
        }
        Value::Move(to)
    }

    pub(crate) fn expr_ty_val(&mut self, expr: &ast::Meta<ast::Expr>) -> TypedValue {
        TypedValue(self.expr(expr), self.types.type_of(expr))
    }

    fn assign(&mut self, path: &ast::Meta<ast::Path>, expr: &ast::Meta<ast::Expr>) -> Value {
        let path = self.types.path_kind(path);
        let ResolvedPath::Value(path) = path.clone() else {
            ice!("should be rejected by type checker");
        };

        let ty = self.types.type_of(expr);
        let to = Var::Ident(path.name.scope, path.name.ident);
        let fields = path.fields.iter();

        let place = Place {
            var: TypedVar(to, path.root_ty),
            proj: fields.map(|(s, _)| Projection::Field(*s)).collect(),
        };

        let val = self.expr(expr);
        let tmp = self.vars.tmp_live(ty.clone());
        self.do_assign(tmp.clone(), TypedValue(val, ty.clone()));

        self.emit.emit_drop(place.clone(), ty.clone());
        self.do_assign(place, TypedValue(Value::Move(tmp), ty));

        Value::Const(ast::Literal::Unit, Type::unit())
    }

    fn binop(
        &mut self,
        lhs: &ast::Meta<ast::Expr>,
        op: ast::BinOp,
        rhs: &ast::Meta<ast::Expr>,
    ) -> Value {
        let lhs_ty = self.types.type_of(lhs);
        let rhs_ty = self.types.type_of(rhs);

        if lhs_ty == Type::string() {
            let (name, return_type) = match op {
                ast::BinOp::Eq => ("eq", Type::bool()),
                ast::BinOp::Ne => ("ne", Type::bool()),
                ast::BinOp::Add => ("append", Type::string()),
                _ => ice!("Operator {op} is not implemented for String"),
            };

            let func_ref = self.find_method(TypeId::of::<Arc<str>>(), name);

            let lhs = self.expr(lhs);
            let lhs = self.assign_to_var(TypedValue(lhs, Type::string())).0;

            let rhs = self.expr(rhs);
            let rhs = self.assign_to_var(TypedValue(rhs, Type::string())).0;

            let tmp = self.vars.tmp_live(return_type.clone());
            let val = self.call_runtime(func_ref, vec![lhs, rhs]);
            self.do_assign(tmp.clone(), TypedValue(val, return_type));

            return Value::Move(tmp);
        }

        match op {
            ast::BinOp::Or => return self.logical(lhs, rhs, false),
            ast::BinOp::And => return self.logical(lhs, rhs, true),
            _ => (),
        }

        let ty = lhs_ty.clone();

        let lhs = TypedValue(self.expr(lhs), lhs_ty);
        let lhs = self.assign_to_var(lhs);

        let rhs = TypedValue(self.expr(rhs), rhs_ty);
        let rhs = self.assign_to_var(rhs);

        Value::BinOp { lhs, op, ty, rhs }
    }

    fn logical(
        &mut self,
        lhs: &ast::Meta<ast::Expr>,
        rhs: &ast::Meta<ast::Expr>,
        branch: bool,
    ) -> Value {
        let current_label = self.emit.current_label();
        let label_cont = self.labels.next(current_label);
        let label_other = self.labels.label(current_label, "logic_other");

        let val = self.expr(lhs);
        let tmp = self.assign_to_var(TypedValue(val, Type::bool()));

        if branch {
            self.emit.branch(tmp.clone(), label_other, label_cont);
        } else {
            self.emit.branch(tmp.clone(), label_cont, label_other);
        }

        self.emit.new_block(label_other);
        let val = TypedValue(self.expr(rhs), Type::bool());
        self.do_assign(tmp.clone(), val);
        self.emit.jump(label_cont);

        self.emit.new_block(label_cont);
        Value::Move(tmp)
    }

    fn call_runtime(&mut self, func: RuntimeFunctionRef, args: Vec<Var>) -> Value {
        for var in &args {
            self.vars_remove_live(var);
        }
        Value::CallRuntime { func, args }
    }

    fn select(
        &mut self,
        id: ast::MetaId,
        cond_ast: &ast::Meta<ast::Expr>,
        accept_ast: &ast::Meta<ast::Block>,
        reject_ast: Option<&ast::Meta<ast::Block>>,
    ) -> Value {
        let examinee = TypedValue(self.expr(cond_ast), Type::bool());
        let examinee = self.assign_to_var(examinee);

        let current_label = self.emit.current_label();
        let merge = self.labels.next(current_label);
        let accept = self.labels.label(current_label, "accept");
        let reject = self.labels.label(current_label, "reject");

        let default = if reject_ast.is_some() { reject } else { merge };
        self.emit.branch(examinee, accept, default);

        self.emit.new_block(accept);
        let op = self.block(accept_ast);
        let ty = self.types.type_of(id);
        let res = self.vars.undropped_tmp_ty(self.types.type_of(id));
        self.emit
            .assign(Place::from(res.clone()), res.1.clone(), op);
        self.emit.jump(merge);

        if let Some(reject_ast) = reject_ast {
            self.emit.new_block(reject);
            let op = self.block(reject_ast);
            self.emit
                .assign(Place::from(res.clone()), res.1.clone(), op);
            self.emit.jump(merge);
        }
        self.emit.new_block(merge);
        self.vars.add_live(res.0, ty);
        Value::Move(res)
    }

    fn r#while(
        &mut self,
        cond_ast: &ast::Meta<ast::Expr>,
        body_ast: &ast::Meta<ast::Block>,
    ) -> Value {
        let current_label = self.emit.current_label();
        let merge = self.labels.next(current_label);
        let cond = self.labels.label(current_label, "loop-cond");
        let body = self.labels.label(current_label, "loop-body");

        self.emit.jump(cond);

        self.emit.new_block(cond);
        let examinee = TypedValue(self.expr(cond_ast), Type::bool());
        let examinee = self.assign_to_var(examinee);
        self.emit.branch(examinee, body, merge);

        self.emit.new_block(body);
        let val = self.block(body_ast);
        let _ = self.assign_to_var(TypedValue(val, Type::unit()));
        self.emit.jump(cond);

        self.emit.new_block(merge);
        Value::Const(ast::Literal::Unit, Type::unit())
    }

    fn path_value(&mut self, path_value: &PathValue) -> Value {
        let &PathValue {
            name,
            ref kind,
            ref root_ty,
            ref fields,
        } = path_value;

        match *kind {
            ValueKind::Local => Value::Clone(Place {
                var: TypedVar(Var::Ident(name.scope, name.ident), root_ty.clone()),
                proj: fields.iter().map(|f| Projection::Field(f.0)).collect(),
            }),
            ValueKind::Constant => {
                assert!(
                    fields.is_empty(),
                    "Getting fields of constants not supported yet"
                );
                Value::Constant(name, root_ty.clone())
            }
        }
    }

    fn find_method(&self, type_id: TypeId, ident: &str) -> RuntimeFunctionRef {
        let ty = self
            .rt
            .types()
            .iter()
            .find(|t| t.type_id() == type_id)
            .unwrap();

        let scope = self.types.get_declaration(ty.name()).scope.unwrap();
        let dec = self.types.get_declaration(ResolvedName::new(scope, ident));
        let DeclKind::Method(Some(f)) = dec.kind else {
            ice!();
        };
        let FunctionDefinition::Runtime(r) = f.def else {
            ice!();
        };

        r
    }

    fn f_string(&mut self, parts: &[ast::Meta<ast::FStringPart>]) -> Value {
        fn make_string(s: String) -> Value {
            Value::Const(ast::Literal::String(s), Type::string())
        }

        let string = self.assign_to_var(TypedValue(make_string(String::new()), Type::string()));

        let type_id = TypeId::of::<Arc<str>>();
        let func_ref = self.find_method(type_id, "append");

        for part in parts {
            let new_string = match &**part {
                ast::FStringPart::String(s) => make_string(s.clone()),
                ast::FStringPart::Expr(expr) => {
                    let val = self.expr(expr);
                    let ty = self.types.type_of(expr);

                    // Get the function that the type checker determined we
                    // should call, this will be the `to_string` method on
                    // the type.
                    let func = self.types.function(part.id).clone();
                    self.normalized_function_call(&func, Some(TypedValue(val, ty)), &[])
                }
            };

            let new_string = self.assign_to_var(TypedValue(new_string, Type::string()));
            let val = self.call_runtime(func_ref, vec![string.0, new_string.0]);

            self.vars.insert(string.clone());
            self.do_assign(string.clone(), TypedValue(val, Type::string()));
        }

        Value::Move(string)
    }
}

impl Lowerer<'_> {
    pub(crate) fn assign_to_var(&mut self, TypedValue(value, ty): TypedValue) -> TypedVar {
        if let Value::Move(x) = value {
            x
        } else {
            let to = self.vars.tmp_live(ty.clone());
            self.do_assign(to.clone(), TypedValue(value, ty));
            to
        }
    }

    pub(crate) fn do_assign(&mut self, to: impl Into<Place>, TypedValue(value, ty): TypedValue) {
        if let Value::Move(var) = &value {
            self.vars_remove_live(&var.0);
        }
        self.emit.assign(to.into(), ty, value);
    }

    fn vars_remove_live(&mut self, var: &Var) -> Type {
        self.vars.remove_live(var).unwrap_or_else(|| {
            let printer = IrPrinter {
                types: self.types,
                labels: self.labels,
                scope: None,
                rt: self.rt,
            };
            let var = var.print(&printer);
            ice!("Variable wasn't live: {var:?}")
        })
    }
}
