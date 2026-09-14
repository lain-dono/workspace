use std::any::{Any, TypeId};
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::sync::Arc;

use crate::Reflect;
use crate::lir::{self, Memory};

pub trait RegisterableFn<A, R>: Send + 'static {
    /// The type of a Rust function wrapping a function of this type
    type RustWrapper;

    const TRAMPOLINE: Self::RustWrapper;

    fn ptr(self) -> Arc<Box<dyn Any>>;
    fn parameter_types(&self) -> Vec<TypeId>;
    fn return_type() -> TypeId;

    fn ir_function(&self) -> RustIrFunction;
}

#[derive(Clone)]
pub struct FunctionDescription {
    parameter_types: Vec<TypeId>,
    return_type: TypeId,
    pointer: Arc<Box<dyn Any>>,
    trampoline: *const u8,
    ir_function: RustIrFunction,
}

// SAFETY: FunctionDescription is only not Send and Sync because of the function
// pointer, but that's fine because those are static. The only constructors for
// FunctionDescription are in this module, so we know that it's only instantiated
// with these pointers that are ok to send and sync.
unsafe impl Send for FunctionDescription {}
unsafe impl Sync for FunctionDescription {}

impl FunctionDescription {
    pub fn of<A, R, F: RegisterableFn<A, R>>(func: F) -> Self {
        let parameter_types = func.parameter_types();
        let return_type = F::return_type();
        let trampoline = unsafe { *std::ptr::from_ref(&F::TRAMPOLINE).cast::<*const u8>() };
        let ir_function = func.ir_function();
        let pointer = func.ptr();

        Self {
            parameter_types,
            return_type,
            pointer,
            trampoline,
            ir_function,
        }
    }

    pub fn parameter_types(&self) -> &[TypeId] {
        &self.parameter_types
    }

    pub fn return_type(&self) -> TypeId {
        self.return_type
    }

    pub fn pointer(&self) -> Arc<Box<dyn Any>> {
        self.pointer.clone()
    }

    pub fn ir_function(&self) -> RustIrFunction {
        self.ir_function.clone()
    }

    pub fn trampoline(&self) -> *const u8 {
        self.trampoline
    }
}

impl PartialEq for FunctionDescription {
    fn eq(&self, other: &Self) -> bool {
        self.parameter_types == other.parameter_types
            && self.return_type == other.return_type
            && Arc::ptr_eq(&self.pointer, &other.pointer)
    }
}

impl Eq for FunctionDescription {}

impl core::fmt::Debug for FunctionDescription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FunctionDescription")
            .field("parameter_types", &self.parameter_types)
            .field("return_type", &self.return_type)
            .field("pointer", &self.pointer)
            .field("wrapped", &"<function>")
            .finish_non_exhaustive()
    }
}

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct RustIrFunction(Arc<dyn Fn(&mut Memory, Vec<lir::Value>)>);

impl Deref for RustIrFunction {
    type Target = Arc<dyn Fn(&mut Memory, Vec<lir::Value>)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! registerable_fn {
    ($($a:ident),*) => {
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[allow(unused_mut)]
        impl<$($a,)* Return, F> RegisterableFn<($($a,)*), Return> for F
        where
            $($a: Reflect,)*
            Return: Reflect,
            F: Fn($($a,)*) -> Return + Send + 'static,
        {
            type RustWrapper = extern "C" fn (*const Self, *mut Return::Transformed, $($a::AsParam),*) -> ();

            const TRAMPOLINE: Self::RustWrapper = {
                extern "C" fn foo<$($a: Reflect,)* Return: Reflect>(x: *const impl Fn($($a,)*) -> Return, out: *mut Return::Transformed, $($a: $a::AsParam),*) -> () {
                    let res = (unsafe { &*x })(
                        $(<$a as Reflect>::untransform(<$a as Reflect>::to_value($a)),)*
                    );
                    unsafe { std::ptr::write(out, <Return as Reflect>::transform(res)) };
                }
                foo
            };

            fn ptr(self) -> Arc<Box<dyn Any>> {
                Arc::new(Box::new(self))
            }

            fn parameter_types(&self) -> Vec<TypeId> {
                vec![$($a::resolve().type_id),*]
            }

            fn return_type() -> TypeId {
                Return::resolve().type_id
            }

            fn ir_function(&self) -> RustIrFunction {
                let f = self as *const _;
                let f = move |mem: &mut Memory, args: Vec<lir::Value>| {
                    let [Return, $($a,)*]: &[lir::Value] = &args else {
                        panic!("Number of arguments is not correct")
                    };

                    let &lir::Value::Ptr(ret) = Return else {
                        panic!("Out pointer is not a pointer")
                    };
                    let ret = mem.get(ret);

                    $(
                        let Ok($a) = <$a as Reflect>::from_ir_value(mem, $a.clone()) else {
                            panic!("Type of argument is not correct: {}", $a)
                        };
                    )*
                    let mut uninit_ret = MaybeUninit::<<Return as Reflect>::Transformed>::uninit();
                    Self::TRAMPOLINE(f, ret.cast(), $($a,)*);
                };
                RustIrFunction(Arc::new(f))
            }
        }
    }
}

variadics_please::all_tuples!(registerable_fn, 0, 15, A);
