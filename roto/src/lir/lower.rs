use crate::{
    ast, ice,
    lir::{self, BinOp, Compare, FlowBuilder, Lir, LowerCtx},
    mir,
    print::TypeDisplay,
    runtime::{
        RuntimeFunctionRef,
        layout::{Layout, LayoutBuilder},
    },
    types::{Num, ResolvedName, ScopeRef, Type, TypeDef},
    var::Var,
};

impl Lir {
    pub fn lower(ctx: &mut LowerCtx<'_>, mir::Mir { functions }: mir::Mir) -> Self {
        Self {
            functions: functions
                .into_iter()
                .filter_map(|function| Lowerer::function(ctx, function))
                .collect(),
        }
    }
}

pub struct Variables {
    function_scope: ScopeRef,
    tmp_idx: usize,
    storage: Vec<(Var, lir::ValueOrSlot)>,
}

impl Variables {
    pub(crate) fn tmp_val(&mut self, ty: lir::Type) -> lir::TypedVar {
        self.add_tmp(ty.into(), ty)
    }

    pub(crate) fn tmp_slot(&mut self, layout: Layout) -> lir::TypedVar {
        self.add_tmp(layout.into(), lir::Type::Ptr)
    }

    fn add_tmp(&mut self, val: lir::ValueOrSlot, ty: lir::Type) -> lir::TypedVar {
        let var = Var::Temp(self.function_scope, self.tmp_idx);
        self.tmp_idx += 1;
        self.storage.push((var, val));
        lir::TypedVar(var, ty)
    }
}

pub struct Lowerer<'c, 'r> {
    pub(crate) ctx: &'c mut LowerCtx<'r>,
    pub(crate) emit: FlowBuilder,
    pub(crate) vars: Variables,
    pub(crate) return_type: Type,
}

/// # Lower MIR constructs
impl Lowerer<'_, '_> {
    fn function(ctx: &mut LowerCtx<'_>, function: mir::Function) -> Option<lir::Function> {
        let mut lowerer = Lowerer {
            ctx,
            emit: FlowBuilder {
                storage: Vec::new(),
            },
            vars: Variables {
                function_scope: function.variables.scope,
                tmp_idx: function.variables.tmp_idx,
                storage: Vec::new(),
            },
            return_type: function.signature.return_type.clone(),
        };
        let name = function.name;
        let signature = function.signature;

        // All parameter must be inhabited.
        // If they aren't then we can skip lowering this entire function.
        signature
            .parameter_types
            .iter()
            .map(|ty| lowerer.ctx.layout_of(ty).map(|_| ()))
            .collect::<Option<()>>()?;

        lowerer.vars.storage = function
            .variables
            .variables
            .iter()
            .filter_map(|&mir::TypedVar(var, ref ty)| {
                // The MIR can emit unresolved types in places where the never
                // type is used. The instructions for those are eliminated by
                // dead code elimination.
                let ty = lowerer.ctx.types.resolve(ty);
                if matches!(&ty, Type::Var(_)) {
                    return None;
                }

                let var_type = if lowerer.ctx.is_reference_type(&ty)? {
                    // Parameters don't need a slot because they already live somewhere.
                    if function.parameters.contains(&var) {
                        lir::ValueOrSlot::from(lir::Type::Ptr)
                    } else {
                        lir::ValueOrSlot::from(lowerer.ctx.layout_of(&ty)?)
                    }
                } else {
                    lir::ValueOrSlot::from(lowerer.ctx.lower_type(&ty)?)
                };
                Some((var, var_type))
            })
            .collect();

        for block in function.blocks {
            lowerer.block(block);
        }

        let entry_block = lowerer.emit.storage[0].label;

        let (return_ir_type, return_ptr) = match lowerer.ctx.is_reference_type(&lowerer.return_type)
        {
            Some(true) => (None, true),
            Some(false) => (lowerer.ctx.lower_type(&lowerer.return_type), false),
            None => (None, false),
        };

        let ir_signature = lir::Signature {
            parameters: function
                .parameters
                .iter()
                .zip(&signature.parameter_types)
                .filter_map(|(def, ty)| {
                    def.label()
                        .and_then(|x| Some((x, lowerer.ctx.lower_type(ty)?)))
                })
                .collect(),
            return_ptr,
            return_type: return_ir_type,
        };

        Some(lir::Function {
            name,
            blocks: lowerer.emit.storage,
            variables: lowerer.vars.storage,
            signature,
            scope: function.variables.scope,
            ir_signature,
            entry_block,
            public: true,
        })
    }

    fn block(&mut self, block: mir::Block) {
        self.emit.start_block(block.label);

        for instruction in block.body {
            match instruction {
                mir::Instruction::Assign(to, ty, value) => self.assign(to, ty, value),
                mir::Instruction::Jump(label) => self.emit.jump(label),

                mir::Instruction::Switch {
                    examinee,
                    mut branches,
                    fallback,
                } => {
                    let fallback = fallback.unwrap_or_else(|| branches.pop().unwrap().1);
                    self.emit.switch(examinee, branches, fallback);
                }
                mir::Instruction::Branch {
                    cond,
                    accept,
                    reject,
                } => self.emit.branch(cond, accept, reject),

                mir::Instruction::SetDiscriminant { to, ty, variant } => {
                    let Type::Name(ty) = ty else { ice!() };
                    let TypeDef::Enum(_, variants) = self.ctx.resolve_type_name(&ty) else {
                        ice!()
                    };
                    let mut variants = variants.iter();
                    let index = variants.position(|v| v.name == variant.name).unwrap();
                    self.emit.write(to, lir::Value::I8(index as i8));
                }
                mir::Instruction::Return(from) => {
                    let from = from.into();
                    let layout = self.ctx.layout_of(&self.return_type);
                    let result = if layout.as_ref().is_none_or(|l| l.size() == 0) {
                        None
                    } else if self.ctx.is_reference_type(&self.return_type).unwrap() {
                        let to = Var::Return(self.vars.function_scope);
                        self.emit.emit_copy(to, from, layout.unwrap().size());
                        None
                    } else {
                        Some(from)
                    };
                    self.emit.ret(result);
                }
                mir::Instruction::Drop(val, ty) => {
                    // Any reference type (and therefore any type that needs drop)
                    // has a pointer location, we can ignore the rest.
                    if let Some(lir::Location::Ptr(ptr)) = self.location(val, ty.clone()) {
                        self.drop_type(ptr, &ty);
                    }
                }
            }
        }
    }

    fn assign(&mut self, to: mir::Place, base_ty: Type, value: mir::Value) {
        let target = self.location(to, base_ty.clone());
        match value {
            mir::Value::Const(lit, ty) => {
                if let Some(op) = self.literal(&lit, &ty) {
                    self.move_val(target, op, &base_ty);
                }
            }
            mir::Value::Constant(name, ty) => {
                let base = self.vars.tmp_val(lir::Type::Ptr);
                let to = base.clone();
                self.emit.emit(lir::Instruction::ConstAddr { to, name });
                if let Some(to) = target {
                    self.clone_place(to, lir::Location::ptr(base, 0), &ty);
                }
            }
            mir::Value::Discriminant(from) => {
                let tmp = self.read_ty(lir::Type::I8, from);
                self.move_val(target, tmp, &base_ty);
            }
            mir::Value::Unary(op, mir::TypedVar(var, ty)) => {
                let val = var.into();
                let from = self.vars.tmp_val(self.ctx.lower_type(&ty).unwrap());
                let kind = self.ctx.types.get_num(&ty);

                let op = match (kind, op) {
                    (None, mir::UnOp::Not) if ty == Type::bool() => lir::UnOp::Eqz,
                    (Some(num), mir::UnOp::Not) if num.is_int() => lir::UnOp::BNot,
                    (Some(num), mir::UnOp::Neg) if num.is_int() => lir::UnOp::INeg,
                    (Some(num), mir::UnOp::Neg) if num.is_float() => lir::UnOp::FNeg,
                    _ => ice!(),
                };
                let to = from.clone();
                self.emit.emit(lir::Instruction::Unary { op, to, val });
                self.move_val(target, from, &base_ty);
            }
            mir::Value::Move(var) => self.move_val(target, var, &base_ty),
            mir::Value::Clone(place) => {
                let from = self.location(place, base_ty.clone());
                if let (Some(to), Some(from)) = (target, from) {
                    self.clone_place(to, from, &base_ty);
                }
            }
            mir::Value::BinOp { lhs, op, ty, rhs } => {
                let op = self.binop(lhs, op, ty, rhs);
                self.move_val(target, op, &base_ty);
            }
            mir::Value::Call { func, args } => {
                if let Some(op) = self.call(func, args, &base_ty) {
                    self.move_val(target, op, &base_ty);
                }
            }
            mir::Value::CallRuntime { func, args } => {
                if let Some(op) = self.call_runtime(func, args, &base_ty) {
                    self.move_val(target, op, &base_ty);
                }
            }
        }
    }

    pub(crate) fn read_ty(
        &mut self,
        ty: lir::Type,
        from: impl Into<lir::Operand>,
    ) -> lir::TypedVar {
        let tmp = self.vars.tmp_val(ty);
        self.emit.read(tmp.clone(), from);
        tmp
    }

    fn move_val(
        &mut self,
        to: impl Into<Option<lir::Location>>,
        from: impl Into<lir::Operand>,
        base_ty: &Type,
    ) {
        self.move_val_impl(to.into(), from.into(), base_ty);
    }

    // There are valid assignments in MIR that have the never type. For
    // example, a function call that returns the never type. So we cannot
    // unwrap `to` here, but we will simply only assign the value if it is
    // inhabited.
    fn move_val_impl(&mut self, to: Option<lir::Location>, from: lir::Operand, base_ty: &Type) {
        let Some(to) = to else { return };
        let Some(ty) = self.ctx.lower_type(base_ty) else {
            return;
        };

        match to {
            lir::Location::Var(to) => self.emit.assign(lir::TypedVar(to, ty), from),
            lir::Location::Ptr(ptr) => {
                let to = self.ptr_offset(ptr);
                match self.ctx.is_reference_type(base_ty) {
                    Some(true) => {
                        let size = self.ctx.layout_of(base_ty).unwrap().size();
                        self.emit.emit_copy(to, from, size);
                    }
                    Some(false) => self.emit.write(to, from),
                    None => {}
                }
            }
        }
    }

    fn call(
        &mut self,
        func: ResolvedName,
        args: Vec<Var>,
        return_type: &Type,
    ) -> Option<lir::Operand> {
        let (to, out_ptr) = match self.ctx.is_reference_type(return_type) {
            Some(true) => {
                let layout = self.ctx.layout_of(return_type).unwrap();
                (None, Some(self.vars.tmp_slot(layout)))
            }
            Some(false) => {
                let to = self.ctx.lower_type(return_type);
                (to.map(|ty| self.vars.tmp_val(ty)), None)
            }
            None => (None, None),
        };

        self.emit.emit(lir::Instruction::Call {
            to: to.clone(),
            func: self.ctx.types.full_name(&func),
            args: args.into_iter().map(Var::into).collect(),
            out_ptr: out_ptr.clone(),
        });

        out_ptr.or(to).map(Into::into)
    }

    fn call_runtime(
        &mut self,
        func: RuntimeFunctionRef,
        args: Vec<Var>,
        return_type: &Type,
    ) -> Option<lir::Operand> {
        let layout = self.ctx.layout_of(return_type);
        let layout = layout.unwrap_or_else(Layout::of::<()>);

        let out_ptr = self.vars.tmp_slot(layout);

        let mut parameters = Vec::new();
        parameters.push(("ret".into(), lir::Type::Ptr));

        let sig = self.ctx.types.runtime_function_signature(func);
        parameters.extend(
            sig.parameter_types
                .iter()
                .enumerate()
                .filter_map(|(i, ty)| Some((i.to_string().into(), self.ctx.lower_type(ty)?))),
        );

        let ir_signature = lir::Signature {
            parameters,
            return_ptr: true,
            return_type: None,
        };

        let args = core::iter::once(lir::Operand::from(out_ptr.clone()))
            .chain(args.iter().copied().map(Var::into))
            .collect();

        self.ctx.rt_functions.insert(func, ir_signature);

        self.emit.emit(lir::Instruction::CallRuntime { func, args });

        if self.ctx.is_reference_type(return_type)? {
            Some(out_ptr.into())
        } else {
            let ty = self.ctx.lower_type(return_type)?;
            Some(self.read_ty(ty, out_ptr).into())
        }
    }

    fn clone_place(
        &mut self,
        to: impl Into<lir::Location>,
        from: impl Into<lir::Location>,
        ty: &Type,
    ) {
        match (to.into(), from.into()) {
            // This is a not-by-reference type so we'll just assign it.
            (lir::Location::Var(to), lir::Location::Var(from)) => {
                if let Some(ty) = self.ctx.lower_type(ty) {
                    self.emit.assign(lir::TypedVar(to, ty), from);
                }
            }
            // We read a not-by-reference type from a field
            (lir::Location::Var(to), lir::Location::Ptr(from)) => {
                if let Some(ty) = self.ctx.lower_type(ty) {
                    let from = self.ptr_offset(from);
                    self.emit.read(lir::TypedVar(to, ty), from);
                }
            }
            // We write a not-by-reference type to a field
            (lir::Location::Ptr(to), lir::Location::Var(from)) => {
                if let Some(_ty) = self.ctx.lower_type(ty) {
                    let to = self.ptr_offset(to);
                    self.emit.write(to, from);
                }
            }
            (lir::Location::Ptr(to), lir::Location::Ptr(from)) => {
                let from = self.ptr_offset(from);
                let to = self.ptr_offset(to);
                self.clone_type(from, to, ty);
            }
        }
    }

    /// Returns `None` if the type uninhabited
    fn location(&mut self, place: mir::Place, ty: Type) -> Option<lir::Location> {
        if place.proj.is_empty() {
            if self.ctx.is_reference_type(&ty)? {
                Some(lir::Location::ptr(place.var.0, 0))
            } else {
                Some(lir::Location::Var(place.var.0))
            }
        } else {
            let mir::TypedVar(base, ty) = place.var;
            let offset = self.project(place.proj, ty)?;
            Some(lir::Location::ptr(base, offset as u32))
        }
    }

    fn project(&mut self, proj: Vec<mir::Projection>, mut ty: Type) -> Option<usize> {
        let mut offset = 0;
        for p in proj {
            match p {
                mir::Projection::Field(ident) => {
                    let (new_offset, new_ty) = 'output: {
                        let res_ty = self.ctx.types.resolve(&ty);

                        let fields = match res_ty {
                            Type::Name(name) => {
                                let TypeDef::Struct(ty_name, fields) =
                                    self.ctx.resolve_type_name(&name)
                                else {
                                    ice!()
                                };

                                let subs = ty_name.args.iter().zip(&name.args);

                                fields
                                    .into_iter()
                                    .map(|(ident, ty)| (ident, ty.substitute_iter(subs.clone())))
                                    .collect()
                            }
                            Type::Record(fields) | Type::RecordVar(_, fields) => fields,
                            _ => ice!(
                                "Cannot get field {ident} of type {}",
                                ty.display(self.ctx.types)
                            ),
                        };

                        let mut builder = LayoutBuilder::new();
                        for (field, new_ty) in fields {
                            let layout = self.ctx.layout_of(&new_ty);
                            let offset = builder.add(layout.unwrap());
                            if *field == ident {
                                break 'output (offset, new_ty);
                            }
                        }

                        ice!("Field not found: {ident}!")
                    };

                    ty = new_ty;
                    offset += new_offset;
                }
                mir::Projection::VariantField(variant_name, n) => {
                    let Type::Name(name) = &ty else { ice!() };
                    let TypeDef::Enum(ty_name, variants) = self.ctx.resolve_type_name(name) else {
                        ice!()
                    };

                    let subs = ty_name.args.iter().zip(&name.args);

                    let mut builder = LayoutBuilder::of::<u8>(); // discriminant
                    let variant = variants.iter().find(|v| v.name == variant_name).unwrap();

                    let mut last_ty = None;
                    let mut new_offset = 0;
                    for field_ty in variant.fields.iter().take(n + 1) {
                        let field_ty = field_ty.substitute_iter(subs.clone());
                        new_offset = builder.add(self.ctx.layout_of(&field_ty)?);
                        last_ty = Some(field_ty);
                    }

                    ty = last_ty.unwrap();
                    offset += new_offset;
                }
            }
        }
        Some(offset)
    }

    /// Lower a literal
    fn literal(&mut self, lit: &ast::Literal, ty: &Type) -> Option<lir::Operand> {
        Some(match &lit {
            ast::Literal::Unit => return None,
            ast::Literal::Bool(x) => lir::Value::Bool(*x).into(),
            ast::Literal::String(s) => {
                let layout = Layout::of::<std::sync::Arc<str>>();
                let to = self.vars.tmp_slot(layout);
                self.emit.emit(lir::Instruction::InitString {
                    to: to.clone(),
                    string: s.clone(),
                });
                to.into()
            }
            ast::Literal::Integer(x) => match ty {
                Type::IVar(_, _) => return Some(lir::Value::I32(*x as i32).into()),
                Type::Name(ty) => {
                    if let TypeDef::Scalar(num) = self.ctx.resolve_type_name(ty) {
                        match num {
                            Num::U8 | Num::I8 => lir::Value::I8(*x as _),
                            Num::U16 | Num::I16 => lir::Value::I16(*x as _),
                            Num::U32 | Num::I32 => lir::Value::I32(*x as _),
                            Num::U64 | Num::I64 => lir::Value::I64(*x as _),
                            Num::F32 | Num::F64 => ice!("float literal?"),
                        }
                        .into()
                    } else {
                        ice!("should be a type error")
                    }
                }
                _ => ice!("should be a type error"),
            },
            ast::Literal::Float(x) => match ty {
                Type::FVar(_) => return Some(lir::Value::F64(*x).into()),
                Type::Name(ty) => {
                    if let TypeDef::Scalar(num) = self.ctx.resolve_type_name(ty) {
                        match num {
                            Num::F32 => lir::Value::F32(*x as f32),
                            Num::F64 => lir::Value::F64(*x),
                            _ => ice!("wrong float {num:?}"),
                        }
                        .into()
                    } else {
                        ice!("should be a type error")
                    }
                }
                _ => ice!("should be a type error"),
            },
        })
    }

    fn binop(
        &mut self,
        lhs: mir::TypedVar,
        op: ast::BinOp,
        ty: Type,
        rhs: mir::TypedVar,
    ) -> lir::Operand {
        let lhs = lhs.into();
        let rhs = rhs.into();

        if ty == Type::bool() {
            match op {
                ast::BinOp::Eq => return self.cmp(Compare::IEq, lhs, rhs).into(),
                ast::BinOp::Ne => return self.cmp(Compare::INe, lhs, rhs).into(),
                _ => (),
            }
        }

        if let Some(kind) = self.ctx.types.get_num(&ty) {
            match op {
                ast::BinOp::Eq if kind.is_int() => self.cmp(Compare::IEq, lhs, rhs),
                ast::BinOp::Ne if kind.is_int() => self.cmp(Compare::INe, lhs, rhs),

                ast::BinOp::Lt if kind.is_sint() => self.cmp(Compare::SLt, lhs, rhs),
                ast::BinOp::Le if kind.is_sint() => self.cmp(Compare::SLe, lhs, rhs),
                ast::BinOp::Gt if kind.is_sint() => self.cmp(Compare::SGt, lhs, rhs),
                ast::BinOp::Ge if kind.is_sint() => self.cmp(Compare::SGe, lhs, rhs),

                ast::BinOp::Lt if kind.is_uint() => self.cmp(Compare::ULt, lhs, rhs),
                ast::BinOp::Le if kind.is_uint() => self.cmp(Compare::ULe, lhs, rhs),
                ast::BinOp::Gt if kind.is_uint() => self.cmp(Compare::UGt, lhs, rhs),
                ast::BinOp::Ge if kind.is_uint() => self.cmp(Compare::UGe, lhs, rhs),

                ast::BinOp::Eq if kind.is_float() => self.cmp(Compare::FEq, lhs, rhs),
                ast::BinOp::Ne if kind.is_float() => self.cmp(Compare::FNe, lhs, rhs),
                ast::BinOp::Lt if kind.is_float() => self.cmp(Compare::FLt, lhs, rhs),
                ast::BinOp::Le if kind.is_float() => self.cmp(Compare::FLe, lhs, rhs),
                ast::BinOp::Gt if kind.is_float() => self.cmp(Compare::FGt, lhs, rhs),
                ast::BinOp::Ge if kind.is_float() => self.cmp(Compare::FGe, lhs, rhs),

                // + - *
                ast::BinOp::Add if kind.is_int() => self.op(BinOp::IAdd, &ty, lhs, rhs),
                ast::BinOp::Sub if kind.is_int() => self.op(BinOp::ISub, &ty, lhs, rhs),
                ast::BinOp::Mul if kind.is_int() => self.op(BinOp::IMul, &ty, lhs, rhs),

                ast::BinOp::Add if kind.is_float() => self.op(BinOp::FAdd, &ty, lhs, rhs),
                ast::BinOp::Sub if kind.is_float() => self.op(BinOp::FSub, &ty, lhs, rhs),
                ast::BinOp::Mul if kind.is_float() => self.op(BinOp::FMul, &ty, lhs, rhs),

                // division
                ast::BinOp::Div if kind.is_sint() => self.op(BinOp::SDiv, &ty, lhs, rhs),
                ast::BinOp::Div if kind.is_uint() => self.op(BinOp::UDiv, &ty, lhs, rhs),
                ast::BinOp::Div if kind.is_float() => self.op(BinOp::FDiv, &ty, lhs, rhs),

                _ => ice!(),
            }
            .into()
        } else {
            ice!("Could not lower binop")
        }
    }
}

/// # Emit instructions
impl Lowerer<'_, '_> {
    #[must_use]
    fn op(&mut self, op: BinOp, ty: &Type, lhs: lir::Operand, rhs: lir::Operand) -> lir::TypedVar {
        let to = self.vars.tmp_val(self.ctx.lower_type(ty).unwrap());
        let ret = to.clone();
        self.emit.emit(lir::Instruction::Op { op, to, lhs, rhs });
        ret
    }

    #[must_use]
    fn cmp(&mut self, cmp: Compare, lhs: lir::Operand, rhs: lir::Operand) -> lir::TypedVar {
        let to = self.vars.tmp_val(lir::Type::Bool);
        let ret = to.clone();
        self.emit.emit(lir::Instruction::Cmp { to, cmp, lhs, rhs });
        ret
    }

    pub(crate) fn ptr_offset(&mut self, ptr: lir::Pointer) -> lir::Operand {
        let lir::Pointer { base, offset } = ptr;

        if offset == 0 {
            base
        } else {
            let new = self.vars.tmp_val(lir::Type::Ptr);
            let (to, from) = (new.clone(), base);
            self.emit
                .emit(lir::Instruction::Offset { to, from, offset });
            new.into()
        }
    }
}
