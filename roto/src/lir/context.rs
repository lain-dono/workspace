use crate::{
    ice,
    label::LabelStore,
    lir::{self, Signature},
    runtime::{
        Runtime, RuntimeFunctionRef,
        layout::{Layout, LayoutBuilder},
    },
    types::{Num, Type, TypeDef, TypeInfo, TypeName},
};
use std::collections::HashMap;

pub struct LowerCtx<'c> {
    pub rt: &'c Runtime,
    pub types: &'c mut TypeInfo,
    pub labels: &'c mut LabelStore,
    pub rt_functions: &'c mut HashMap<RuntimeFunctionRef, Signature>,
}

impl LowerCtx<'_> {
    /// Whether or not the type is passed around by reference or by value
    ///
    /// Roto always has by-value semantics, but we still have types that we
    /// store in stack slots and then operate on by pointer. That is what
    /// we mean here with a reference type.
    ///
    /// Registered types, enums, records, ip addrs, prefixes and strings are all
    /// reference types. Integers, floats, booleans and AS numbers are not.
    ///
    /// This returns `None` if the type is uninhabited (e.g. `!`)
    pub fn is_reference_type(&mut self, ty: &Type) -> Option<bool> {
        let ty = self.types.resolve(ty);
        if self.layout_of(&ty)?.size() == 0 {
            return Some(false);
        }
        match ty {
            Type::Record(..) | Type::RecordVar(..) => Some(true),
            Type::Name(name) => Some(matches!(
                self.types.resolve_type_name(&name),
                TypeDef::Enum(..) | TypeDef::Struct(..) | TypeDef::Runtime(..) | TypeDef::String
            )),
            Type::Unit
            | Type::Never
            | Type::Var(_)
            | Type::ExplicitVar(_)
            | Type::IVar(_, _)
            | Type::FVar(_)
            | Type::Function(_, _) => Some(false),
        }
    }

    /// Compute the layout of a Roto type
    ///
    /// The layout of Roto types match the C representation of Rust types,
    /// because we cannot rely on the Rust representation.
    ///
    /// The C representation is described in the [Rust reference].
    ///
    /// The general rules are as follows:
    ///
    ///  - The minimum layout of any type is a size of 0 and an alignment of 1
    ///  - Each primitive has a size and alignment equal to itself.
    ///  - Each composite type has the alignment of the most-aligned field in it.
    ///  - Fields are layed out in order, each padded to their alignment.
    ///  - The size **must** be a multiple of the alignment.
    ///
    /// For enums we use the `#[repr(C, u8)]` representation, because other the
    /// other representations are platform-specific. This means that the tag for
    /// enums is a `u8` and therefore 1 byte.
    ///
    /// To implement these rules, we rely on the [`Layout`] struct from the Rust
    /// standard library. This also allows to get the layout of some Rust types
    /// we rely on.
    ///
    /// This function returns `None` if the type is uninhabited.
    ///
    /// [Rust reference]: https://doc.rust-lang.org/reference/type-layout.html
    pub fn layout_of(&mut self, ty: &Type) -> Option<Layout> {
        let ty = self.types.resolve(ty);
        let layout = match ty {
            Type::ExplicitVar(_) => ice!("Can't get the layout of an unconcrete type: {ty:?}"),
            Type::Function(_, _) => ice!("Can't get the layout of a function type"),
            Type::Unit => Layout::of::<()>(),
            Type::Var(_) | Type::Never => return None,
            Type::IVar(_, _) => Num::IVAR.layout(),
            Type::FVar(_) => Num::FVAR.layout(),
            Type::RecordVar(_, fields) | Type::Record(fields) => {
                let layouts = fields
                    .iter()
                    .map(|(_, f)| self.layout_of(f))
                    .collect::<Option<Vec<_>>>()?;
                Layout::concat(layouts)
            }
            Type::Name(type_name) => {
                match self.resolve_type_name(&type_name) {
                    TypeDef::Bool => Layout::of::<bool>(),
                    TypeDef::String => Layout::of::<std::sync::Arc<str>>(),
                    TypeDef::Scalar(num) => num.layout(),

                    TypeDef::Enum(type_constructor, variants) => {
                        let subs = type_constructor.args.iter().zip(&type_name.args);

                        let mut layout = None;
                        for variant in &variants {
                            let builder = LayoutBuilder::of::<u8>();
                            let builder = variant.fields.iter().try_fold(builder, |mut b, t| {
                                let t = t.substitute_iter(subs.clone());
                                b.add(self.layout_of(&t)?);
                                Some(b)
                            });

                            // If the variant contains uninhabited fields, the
                            // entire variant is uninhabited, so we don't need
                            // to consider it.
                            let Some(builder) = builder else {
                                continue;
                            };

                            let variant_layout = builder.finish();
                            layout = Some(
                                layout.map_or(variant_layout, |l: Layout| l.union(variant_layout)),
                            );
                        }

                        layout?
                    }
                    TypeDef::Struct(constructor, fields) => {
                        let subs = constructor.args.iter();
                        let subs = subs.zip(&type_name.args);

                        // If any of the fields of the record are uninhabited
                        // the entire record is uninhabited.
                        let mut builder = LayoutBuilder::new();
                        for (_, t) in fields {
                            let t = t.substitute_iter(subs.clone());
                            builder.add(self.layout_of(&t)?);
                        }
                        builder.finish()
                    }
                    TypeDef::Runtime(_, type_id) => self.rt.runtime_type(type_id).unwrap().layout(),
                }
            }
        };
        Some(layout)
    }

    pub(crate) fn copy_size(&mut self, ty: &Type) -> Option<usize> {
        Some(match ty {
            // These aren't copied, since they aren't leafs or zero sized
            Type::Never | Type::Unit => return None,
            Type::Record(_fields) | Type::RecordVar(_, _fields) => return None,
            Type::Function(_args, _) => return None,

            // These are invalid at this point
            Type::Var(_) | Type::ExplicitVar(_) => panic!(),
            Type::IVar(_, _) => Num::IVAR.layout().size(),
            Type::FVar(_) => Num::FVAR.layout().size(),
            Type::Name(ty) => match self.resolve_type_name(ty) {
                // For enums we will do most of the work in the traversal
                // but the discriminant should be done here.
                TypeDef::Bool | TypeDef::Enum(_, _) => 1,
                TypeDef::String => size_of::<std::sync::Arc<str>>(),
                TypeDef::Scalar(num) => num.layout().size(),
                TypeDef::Runtime(_, id) => self.rt.runtime_type(id).unwrap().layout().size(),
                TypeDef::Struct(..) => return None,
            },
        })
    }

    pub fn resolve_type_name(&mut self, ty: &TypeName) -> TypeDef {
        self.types.resolve_type_name(ty)
    }

    pub fn lower_type(&mut self, ty: &Type) -> Option<lir::Type> {
        let ty = self.types.resolve(ty);
        if self.layout_of(&ty).is_some_and(|l| l.size() == 0) {
            return None;
        }

        if let Type::Name(ty) = &ty {
            'pop: {
                return Some(match self.resolve_type_name(ty) {
                    TypeDef::Bool => lir::Type::Bool,
                    TypeDef::Runtime(_, _) => lir::Type::Ptr,

                    TypeDef::Scalar(Num::U8 | Num::I8) => lir::Type::I8,
                    TypeDef::Scalar(Num::U16 | Num::I16) => lir::Type::I16,
                    TypeDef::Scalar(Num::U32 | Num::I32) => lir::Type::I32,
                    TypeDef::Scalar(Num::U64 | Num::I64) => lir::Type::I64,

                    TypeDef::Scalar(Num::F32) => lir::Type::F32,
                    TypeDef::Scalar(Num::F64) => lir::Type::F64,

                    TypeDef::String | TypeDef::Enum(_, _) | TypeDef::Struct(_, _) => break 'pop,
                });
            }
        }

        Some(match ty {
            Type::IVar(_, _) => lir::Type::IVAR,
            Type::FVar(_) => lir::Type::FVAR,
            x if self.is_reference_type(&x)? => lir::Type::Ptr,
            _ => ice!("could not lower: {ty:?}"),
        })
    }
}
