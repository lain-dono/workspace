use crate::{
    print::TypeDisplay,
    runtime::ty::{Reflect, TypeDescription, TypeRegistry},
    types::{ResolvedName, Type, TypeDef, TypeInfo},
};
use std::{any::TypeId, mem::MaybeUninit, sync::Arc};

#[derive(Debug)]
pub struct TypeMismatch {
    pub rust_ty: String,
    pub roto_ty: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FunctionRetrievalError {
    #[error(
        "The function `{name}` does not exist.\nHint: the following functions are defined:\n{}",
        print_existing(existing)
    )]
    DoesNotExist { name: String, existing: Vec<String> },
    #[error(
        "The number of arguments do not match\nThe Roto function has {expected} arguments, but the Rust function has {got}."
    )]
    IncorrectNumberOfArguments { expected: usize, got: usize },
    #[error(
        "The types for {ctx} of the function do not match\nExpected `{roto_ty}` got `{rust_ty}`."
    )]
    TypeMismatch {
        ctx: String,
        rust_ty: String,
        roto_ty: String,
    },
}

fn print_existing(existing: &Vec<String>) -> String {
    let mut dst = String::new();
    for n in existing {
        dst.push_str(" - ");
        dst.push_str(n);
        dst.push('\n');
    }
    dst
}

pub fn check_roto_type_reflect<T: Reflect>(
    types: &mut TypeInfo,
    roto_ty: &Type,
) -> Result<(), TypeMismatch> {
    check_roto_type(types, TypeRegistry::resolve::<T>().type_id, roto_ty)
}

fn check_roto_type(
    types: &mut TypeInfo,
    rust_ty: TypeId,
    roto_ty: &Type,
) -> Result<(), TypeMismatch> {
    let Some(rust_ty) = TypeRegistry::get(rust_ty) else {
        return Err(TypeMismatch {
            rust_ty: "unknown".into(),
            roto_ty: roto_ty.display(types).to_string(),
        });
    };

    let error_message = TypeMismatch {
        rust_ty: rust_ty.rust_name.to_string(),
        roto_ty: roto_ty.display(types).to_string(),
    };

    let mut roto_ty = types.resolve(roto_ty);

    if let Type::IVar(_, _) = roto_ty {
        roto_ty = Type::ivar();
    }

    if let Type::FVar(_) = roto_ty {
        roto_ty = Type::fvar();
    }

    match rust_ty.description {
        TypeDescription::Leaf => {
            let expected_name = match rust_ty.type_id {
                x if x == TypeId::of::<()>() => {
                    return if roto_ty == Type::Unit {
                        Ok(())
                    } else {
                        Err(error_message)
                    };
                }

                x if x == TypeId::of::<bool>() => "bool",

                x if x == TypeId::of::<u8>() => "u8",
                x if x == TypeId::of::<u16>() => "u16",
                x if x == TypeId::of::<u32>() => "u32",
                x if x == TypeId::of::<u64>() => "u64",

                x if x == TypeId::of::<i8>() => "i8",
                x if x == TypeId::of::<i16>() => "i16",
                x if x == TypeId::of::<i32>() => "i32",
                x if x == TypeId::of::<i64>() => "i64",

                x if x == TypeId::of::<f32>() => "f32",
                x if x == TypeId::of::<f64>() => "f64",

                x if x == TypeId::of::<Arc<str>>() => "String",

                x => panic!("{x:?}"),
            };

            if roto_ty == Type::named(expected_name, Vec::new()) {
                Ok(())
            } else {
                Err(error_message)
            }
        }
        TypeDescription::Val(_) => {
            let Type::Name(type_name) = roto_ty else {
                return Err(error_message);
            };
            let TypeDef::Runtime(_, id) = types.resolve_type_name(&type_name) else {
                return Err(error_message);
            };
            if rust_ty.type_id == id {
                Ok(())
            } else {
                Err(error_message)
            }
        }
        TypeDescription::Verdict(rust_accept, rust_reject) => {
            let Type::Name(type_name) = &roto_ty else {
                return Err(error_message);
            };
            if type_name.name != ResolvedName::global("Verdict") {
                return Err(error_message);
            }
            let [roto_accept, roto_reject] = &type_name.args[..] else {
                return Err(error_message);
            };
            check_roto_type(types, rust_accept, roto_accept)?;
            check_roto_type(types, rust_reject, roto_reject)?;
            Ok(())
        }
        TypeDescription::Option(rust_ty) => {
            let Type::Name(type_name) = &roto_ty else {
                return Err(error_message);
            };
            if type_name.name != ResolvedName::global("Option") {
                return Err(error_message);
            }
            let [roto_ty] = &type_name.args[..] else {
                return Err(error_message);
            };
            check_roto_type(types, rust_ty, roto_ty)
        }
    }
}

/// Parameters of a Roto function
///
/// This trait allows for checking the types against Roto types and converting
/// the values into values appropriate for Roto.
///
/// The `invoke` method can (unsafely) invoke a pointer as if it were a function
/// with these parameters.
///
/// This trait is implemented on function pointers with several numbers of parameters.
///
/// This trait is _sealed_, meaning that it cannot be implemented by downstream
/// crates.
pub trait ReflectFunc {
    /// Argument types of this function
    type Args;

    /// Return type of this function
    type Return: Reflect;

    /// Type of a Roto function with this type using a return pointer
    type WithReturnPointer;

    /// Type of a Roto function with this type returning directly
    type WithoutReturnPointer;

    /// Check whether these parameters match a parameter list from Roto.
    fn check_args(types: &mut TypeInfo, ty: &[Type]) -> Result<(), FunctionRetrievalError>;

    /// Call a function pointer as if it were a function with these parameters.
    ///
    /// This is _extremely_ unsafe, do not pass this arbitrary pointers and
    /// always call `RotoParams::check` before calling this function. Don't
    /// forget to also check the return type.
    ///
    /// A [`TypedFunc`](super::TypedFunc) is a safe abstraction around this
    /// function.
    unsafe fn invoke(args: Self::Args, func_ptr: *const u8, return_by_ref: bool) -> Self::Return;
}

/// Implement the [`RotoParams`] trait for a tuple with some type parameters.
macro_rules! impl_reflect_func {
    // Little helper macro to create a unit
    (@unit $t:tt) => { () };

    ($($Arg:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        impl<$($Arg: Reflect,)* Return: Reflect> ReflectFunc for fn($($Arg,)*) -> Return {
            type Args = ($($Arg,)*);
            type Return = Return;

            type WithReturnPointer = extern "C" fn(*mut Return::Transformed, $($Arg::AsParam),*) -> ();
            type WithoutReturnPointer = extern "C" fn($($Arg::AsParam,)*) -> Return::Transformed;

            fn check_args(
                types: &mut TypeInfo,
                ty: &[Type]
            ) -> Result<(), FunctionRetrievalError> {
                let [$($Arg),*] = ty else {
                    let x: &[()] = &[$(impl_reflect_func!(@unit $Arg)),*];
                    return Err(FunctionRetrievalError::IncorrectNumberOfArguments {
                        expected: ty.len(),
                        got: x.len(),
                    });
                };

                let mut i = 0;
                $(
                    i += 1;
                    check_roto_type_reflect::<$Arg>(types, $Arg)
                        .map_err(|e| FunctionRetrievalError::TypeMismatch {
                            ctx: format!("argument {i}"),
                            roto_ty: e.roto_ty,
                            rust_ty: e.rust_ty
                        })?;
                )*
                Ok(())
            }

            unsafe fn invoke(args: Self::Args, func_ptr: *const u8, return_by_ref: bool) -> Self::Return {
                let ($($Arg,)*) = args;
                let mut transformed = ($(<$Arg as Reflect>::transform($Arg),)*);
                let ($($Arg,)*) = &mut transformed;
                let ($($Arg,)*) = ($(<$Arg as Reflect>::as_param($Arg),)*);

                // We forget values that we pass into Roto.
                // The script is responsible for cleaning them op.
                // Forgetting copy types does nothing, but that's fine.
                #[allow(forgetting_copy_types)]
                core::mem::forget(transformed);

                unsafe {
                    if return_by_ref {
                        let func_ptr = core::mem::transmute::<*const u8, Self::WithReturnPointer>(func_ptr);
                        let mut ret = MaybeUninit::<<Self::Return as Reflect>::Transformed>::uninit();
                        func_ptr(ret.as_mut_ptr(), $($Arg),*);
                        Self::Return::untransform(ret.assume_init())
                    } else {
                        let func_ptr = core::mem::transmute::<*const u8, Self::WithoutReturnPointer>(func_ptr);
                        <Self::Return as Reflect>::untransform(func_ptr($($Arg),*))
                    }
                }
            }
        }
    };
}

variadics_please::all_tuples!(impl_reflect_func, 0, 15, A);
