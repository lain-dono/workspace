use crate::{
    ast::ident::Ident,
    ice,
    lir::{self, Operand, Pointer},
    runtime::layout::LayoutBuilder,
    runtime::{CloneDrop, Movability},
    types::{Type, TypeDef},
};
use std::{any::TypeId, sync::Arc};

impl super::Lowerer<'_, '_> {
    pub fn drop_type(&mut self, ptr: Pointer, ty: &Type) {
        let var = self.ptr_offset(ptr);
        let new_var = var.clone();
        let f = move |this: &mut Self, offset, ty: &Type| {
            if let Some(&CloneDrop { drop, .. }) = this.leaf_clone_drop(ty) {
                let var = this.ptr_offset(Pointer::new(new_var.clone(), offset as u32));
                this.emit.drop_var(var, Some(drop));
            }
        };
        self.traverse_type(&var.clone(), 0, ty, "drop".into(), &f);
    }

    pub fn clone_type(&mut self, from: Operand, to: Operand, ty: &Type) {
        let (new_from, new_to) = (from.clone(), to.clone());
        let f = move |this: &mut Self, offset, ty: &Type| {
            let from = this.ptr_offset(Pointer::new(new_from.clone(), offset as u32));
            let to = this.ptr_offset(Pointer::new(new_to.clone(), offset as u32));
            if let Some(&CloneDrop { clone, .. }) = this.leaf_clone_drop(ty) {
                this.emit.emit_clone(to, from, clone);
            } else if let Some(size) = this.ctx.copy_size(ty) {
                this.emit.emit_copy(to, from, size);
            }
        };
        self.traverse_type(&from, 0, ty, "clone".into(), &f);
    }

    fn leaf_clone_drop(&mut self, ty: &Type) -> Option<&CloneDrop> {
        let Type::Name(ty) = ty else {
            return None;
        };
        let id = match self.ctx.resolve_type_name(ty) {
            TypeDef::Runtime(_, id) => id,
            TypeDef::String => TypeId::of::<Arc<str>>(),
            _ => return None,
        };
        let ty = self.ctx.rt.runtime_type(id).unwrap();
        if let Movability::CloneDrop(clone_drop) = ty.movability() {
            Some(clone_drop)
        } else {
            None
        }
    }

    pub(crate) fn traverse_type(
        &mut self,
        var: &Operand,
        offset: usize,
        ty: &Type,
        doc_label: Ident,
        callback: &(impl Fn(&mut Self, usize, &Type) + Clone),
    ) {
        let ty = self.ctx.types.resolve(ty);

        callback(self, offset, &ty);

        match ty {
            Type::Never | Type::Unit | Type::IVar(_, _) | Type::FVar(_) => {}
            Type::Var(_) | Type::Function(_, _) | Type::ExplicitVar(_) => {
                panic!("Can't traverse: {ty:?}")
            }
            Type::RecordVar(_, fields) | Type::Record(fields) => {
                let mut builder = LayoutBuilder::new();
                for (_, ty) in fields {
                    let Some(layout) = self.ctx.layout_of(&ty) else {
                        ice!("Need an inhabited type");
                    };
                    let new_offset = builder.add(layout);
                    self.traverse_type(var, offset + new_offset, &ty, doc_label, callback);
                }
            }
            Type::Name(ty) => match self.ctx.resolve_type_name(&ty) {
                TypeDef::Bool | TypeDef::Runtime(_, _) | TypeDef::Scalar(_) | TypeDef::String => {}

                TypeDef::Enum(constructor, variants) => {
                    let subs = constructor.args.iter().zip(&ty.args);

                    let current_label = self.emit.current_label();
                    let prefix = self.ctx.labels.label(current_label, doc_label);
                    let fallback = self.ctx.labels.next(current_label);

                    let branches = (0..variants.len())
                        .map(|i| (i, self.ctx.labels.label(prefix, format!("variant_{i}"))))
                        .collect::<Vec<_>>();

                    let offset_var = self.ptr_offset(Pointer::new(var.clone(), offset as u32));
                    let discriminant = self.read_ty(lir::Type::I8, offset_var);
                    self.emit.switch(discriminant, branches.clone(), fallback);

                    'outer: for (idx, label) in branches {
                        self.emit.start_block(label);
                        let variant = &variants[idx];

                        let mut layouts = Vec::new();
                        for ty in &variant.fields {
                            let ty = ty.substitute_iter(subs.clone());
                            let Some(layout) = self.ctx.layout_of(&ty) else {
                                self.emit.jump(fallback);
                                continue 'outer;
                            };
                            layouts.push((ty, layout));
                        }

                        let mut builder = LayoutBuilder::of::<u8>();
                        for (ty, layout) in layouts {
                            let offset = offset + builder.add(layout);
                            self.traverse_type(var, offset, &ty, doc_label, callback);
                        }
                        self.emit.jump(fallback);
                    }

                    self.emit.start_block(fallback);
                }
                TypeDef::Struct(type_constructor, fields) => {
                    let subs = type_constructor.args.iter().zip(&ty.args);

                    let mut builder = LayoutBuilder::new();
                    for (_, ty) in &fields {
                        let ty = ty.substitute_iter(subs.clone());
                        let offset = offset + builder.add(self.ctx.layout_of(&ty).unwrap());
                        self.traverse_type(var, offset, &ty, doc_label, callback);
                    }
                }
            },
        }
    }
}
