//! Machine code generation via cranelift
//!
//! This module takes LIR and translates that to cranelift IR.
//! Cranelift then does the rest.

use self::check::{FunctionRetrievalError, ReflectFunc, check_roto_type_reflect};
use crate::{
    Runtime,
    label::LabelStore,
    lir,
    runtime::{ConstantValue, RuntimeConstant, RuntimeFunctionRef, ty::Reflect},
    types,
    var::Var,
};
use cranelift::{
    codegen::{ir, isa::TargetIsa, settings, settings::Configurable as _},
    frontend::{FunctionBuilder, FunctionBuilderContext, Variable},
    jit::{JITBuilder, JITModule},
    module::{FuncId, Linkage, Module as _},
};
use std::{any::Any, collections::HashMap, marker::PhantomData, mem::ManuallyDrop, sync::Arc};

pub mod check;
mod emit;
pub mod func;
pub mod testing;

struct ModuleData {
    jit: ManuallyDrop<JITModule>,

    /// The functions in this module can reference constants. The values of
    /// these constants are stored in this `HashMap`. So, as long as the function
    /// are around, we have to keep these constants around. That is why they
    /// need to be stored in this struct, even though this field is unused.
    _constants: HashMap<types::ResolvedName, ConstantValue>,

    /// The functions in this module can reference registered function that
    /// might contain data (i.e. closures). We need to properly drop these.
    _registered_fns: Vec<Arc<Box<dyn Any>>>,
}

impl ModuleData {
    fn new(
        jit: JITModule,
        constants: HashMap<types::ResolvedName, ConstantValue>,
        registered_fns: Vec<Arc<Box<dyn Any>>>,
    ) -> Self {
        Self {
            jit: ManuallyDrop::new(jit),
            _constants: constants,
            _registered_fns: registered_fns,
        }
    }
}

impl Drop for ModuleData {
    fn drop(&mut self) {
        // SAFETY: We only give out functions that hold a SharedModuleData and
        // therefore an Arc to this module. This ensures that this drop method
        // is only called after all functions have been dropped. Therefore,
        // freeing this memory is ok.
        unsafe { ManuallyDrop::take(&mut self.jit).free_memory() }
    }
}

/// A wrapper around a cranelift [`JITModule`] that cleans up after itself
///
/// This is achieved by wrapping the module in an [`Arc`].
#[derive(Clone)]
pub struct SharedModuleData(Arc<ModuleData>);

impl SharedModuleData {
    fn new(
        jit: JITModule,
        constants: HashMap<types::ResolvedName, ConstantValue>,
        registered_fns: Vec<Arc<Box<dyn Any>>>,
    ) -> Self {
        Self(Arc::new(ModuleData::new(jit, constants, registered_fns)))
    }
}

// Just a simple debug to print _something_.
impl core::fmt::Debug for SharedModuleData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("SharedModuleData").finish()
    }
}

unsafe impl Send for ModuleData {}
unsafe impl Sync for ModuleData {}

/// A function extracted from Roto
///
/// A [`TypedFunc`] can be retrieved from a compiled script using
/// [`Package::get_function`](crate::Package::get_function).
///
/// The function can be called with one of the [`TypedFunc::call`] functions.
#[derive(Clone)]
pub struct TypedFunc<F> {
    func: *const u8,
    return_by_ref: bool,

    // The module holds the data for this function, that's why we need
    // to ensure that it doesn't get dropped. This field is ESSENTIAL
    // for the safety of calling this function. Without it, the data that
    // the `func` pointer points to might have been dropped.
    _module: SharedModuleData,

    marker: PhantomData<F>,
}

impl<F> core::fmt::Debug for TypedFunc<F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TypedFunc")
            .field("func", &self.func)
            .finish_non_exhaustive()
    }
}

/// SAFETY: These implementations are safe because we don't modify anything
/// the pointer points to and the pointer will stay valid as long as we hold
/// on to the `ModuleData`, which is stored in the `TypedFunc`.
unsafe impl<F> Send for TypedFunc<F> {}
unsafe impl<F> Sync for TypedFunc<F> {}

macro_rules! impl_typed_func_call {
    ($($T:ident),*) => {
        impl<$($T: Reflect,)* Return: Reflect> TypedFunc<fn($($T,)*) -> Return> {
            #[allow(non_snake_case)]
            #[allow(clippy::too_many_arguments)]
            pub fn call(&self, $($T: $T,)*) -> Return {
                let args = ($($T,)*);
                unsafe { <fn($($T,)*) -> Return>::invoke(args, self.func, self.return_by_ref) }
            }

            #[allow(non_snake_case)]
            pub fn into_func(self) -> impl Fn($($T,)*) -> Return {
                move |$($T,)*| self.call($($T,)*)
            }
        }
    }
}

variadics_please::all_tuples!(impl_typed_func_call, 0, 15, T);

pub struct FunctionInfo {
    id: FuncId,
    signature: types::Signature,
    return_by_ref: bool,
}

struct VariableMap {
    map: HashMap<Var, (Variable, lir::Type)>,
}

impl VariableMap {
    fn insert(&mut self, v: Var, var: Variable, ty: lir::Type) {
        self.map.insert(v, (var, ty));
    }

    fn find(&self, var: &Var) -> Variable {
        self.map.get(var).unwrap().0
    }

    fn get(&self, var: &Var) -> Variable {
        let v = self.map.get(var).copied().map(|v| v.0);
        v.unwrap_or_else(|| crate::ice!("did not find {var:?} in {:#?}", self.map))
    }
}

// We use `with_aligned` to make sure that we notice if anything is
// unaligned. It does add additional checks, so should be disabled at some
// point, or at least be configurable.
const MEMFLAGS: ir::MemFlags = ir::MemFlags::new().with_aligned();

/// The final compiled package of script.
///
/// Functions can be extracted from this package using [`Package::get_function`].
pub struct Package {
    /// The set of public functions and their signatures.
    functions: HashMap<String, FunctionInfo>,

    /// The inner cranelift module
    inner: SharedModuleData,

    /// Info from the typechecker for checking types against Rust types
    types: types::TypeInfo,
}

impl Package {
    pub fn func<F: ReflectFunc>(
        &mut self,
        name: &str,
    ) -> Result<TypedFunc<F>, FunctionRetrievalError> {
        let name = format!("pkg.{name}");
        let &FunctionInfo {
            id,
            ref signature,
            return_by_ref,
        } = self.functions.get(&name).ok_or_else(|| {
            let existing = self.functions.keys().cloned().collect();
            FunctionRetrievalError::DoesNotExist { name, existing }
        })?;

        F::check_args(&mut self.types, &signature.parameter_types)?;

        check_roto_type_reflect::<F::Return>(&mut self.types, &signature.return_type).map_err(
            |e| FunctionRetrievalError::TypeMismatch {
                ctx: "the return value".to_string(),
                roto_ty: e.roto_ty,
                rust_ty: e.rust_ty,
            },
        )?;

        let func_ptr = self.inner.0.jit.get_finalized_function(id);
        Ok(TypedFunc {
            func: func_ptr,
            return_by_ref,
            _module: self.inner.clone(),
            marker: PhantomData,
        })
    }

    #[must_use]
    pub fn codegen(
        runtime: &Runtime,
        ir: &[lir::Function],
        rt_functions: &HashMap<RuntimeFunctionRef, lir::Signature>,
        labels: LabelStore,
        types: types::TypeInfo,
    ) -> Self {
        fn make_sign(
            jit: &JITModule,
            params: impl IntoIterator<Item = ir::AbiParam>,
        ) -> ir::Signature {
            let mut signature = jit.make_signature();
            signature.params.extend(params);
            signature
        }

        // The ISA is the Instruction Set Architecture. We always compile for
        // the system we run on, so we use `cranelift_native` to get the ISA
        // for the current system. We enable building for speed only. Size is
        // not super important at the moment.
        let mut settings = settings::builder();
        settings.set("opt_level", "speed").unwrap();
        let flags = settings::Flags::new(settings);
        let isa = cranelift::native::builder().unwrap().finish(flags).unwrap();

        let mut builder =
            JITBuilder::with_isa(isa.clone(), cranelift::module::default_libcall_names());

        // This is a fix for cranelift not finding the memcpy libcall when it is
        // compiled with static linking (e.g. with musl).
        //
        // We might need to add more symbols in the future, but for now, this passes
        // the tests.
        builder.symbol("memcpy", libc::memcpy as *const u8);

        for &func in rt_functions.keys() {
            let f = runtime.get_function(func);
            builder.symbol(
                format!("runtime_function_trampoline_{}", f.id),
                f.func.trampoline(),
            );
        }

        let jit = JITModule::new(builder);
        let abi_ptr = ir::AbiParam::new(isa.pointer_type());

        let mut module = ModuleBuilder {
            constants: HashMap::new(),
            functions: HashMap::new(),
            registered_fns: Vec::new(),
            rt_functions: HashMap::new(),
            vars: VariableMap {
                map: HashMap::new(),
            },
            labels,
            types,

            drop_signature: make_sign(&jit, [abi_ptr]),
            clone_signature: make_sign(&jit, [abi_ptr, abi_ptr]),
            init_string_signature: make_sign(&jit, [abi_ptr, abi_ptr, abi_ptr]),

            jit,
            isa,
        };

        for constant in runtime.constants().values() {
            module.declare_constant(constant);
        }

        for (func_ref, ir_sig) in rt_functions {
            let mut signature = module.jit.make_signature();

            // This function is the trampoline,
            // so we need to pass the pointer to the actual function.
            let vt = module.isa.pointer_type();
            signature.params.push(ir::AbiParam::new(vt));

            for &(_, ty) in &ir_sig.parameters {
                signature.params.push(ir::AbiParam::new(ty.into()));
            }
            if let Some(ty) = ir_sig.return_type {
                signature.returns.push(ir::AbiParam::new(ty.into()));
            }

            let f = runtime.get_function(*func_ref);
            let name = format!("runtime_function_trampoline_{}", f.id);
            let linkage = Linkage::Import;
            let Ok(func_id) = module.jit.declare_function(&name, linkage, &signature) else {
                panic!()
            };

            let arc_box = f.func.pointer();
            let ptr = (&raw const **arc_box).cast::<u8>();
            module.registered_fns.push(arc_box);
            module.rt_functions.insert(*func_ref, (ptr, func_id));
        }

        // Our functions might call each other, so we declare them before we define them.
        // This is also when we start building the function hashmap.
        for func in ir {
            module.declare_function(func);
        }

        let mut builder_context = FunctionBuilderContext::new();
        for func in ir {
            module.define_function(func, &mut builder_context);
        }

        module.finalize()
    }
}

struct ModuleBuilder {
    constants: HashMap<types::ResolvedName, ConstantValue>,

    registered_fns: Vec<Arc<Box<dyn Any>>>,

    /// The set of public functions and their signatures.
    functions: HashMap<String, FunctionInfo>,

    /// External functions
    rt_functions: HashMap<RuntimeFunctionRef, (*const u8, FuncId)>,

    /// The inner cranelift module
    jit: JITModule,

    /// Instruction set architecture
    isa: Arc<dyn TargetIsa>,

    /// Map of cranelift variables and their types
    ///
    /// This is necessary because cranelift does not seem to allow us to
    /// query it.
    vars: VariableMap,

    /// To print labels for debugging.
    #[allow(unused)]
    labels: LabelStore,

    /// The information generated by the type checker
    types: types::TypeInfo,

    /// Signature to use for calls to `clone`
    clone_signature: ir::Signature,

    /// Signature to use for calls to `drop`
    drop_signature: ir::Signature,

    /// Signature to use for calls to `init_string`
    init_string_signature: ir::Signature,
}

impl ModuleBuilder {
    fn declare_constant(&mut self, constant: &RuntimeConstant) {
        // Every constant needs to live as long as the functions and therefore
        // module that references them. However, the runtime might be dropped
        // before we call a Roto function. Therefore we clone the constants into
        // this hashmap which we pass to the ModuleData so that they will be
        // kept around.
        self.constants.insert(constant.name, constant.value.clone());
    }

    fn fn_signature(mut sig: ir::Signature, func: &lir::Signature) -> ir::Signature {
        let abi = |ty: lir::Type| ir::AbiParam::new(ty.into());

        let ret = func.return_ptr.then_some(lir::Type::Ptr);
        let ret = ret.map(|ty| ir::AbiParam::special(ty.into(), ir::ArgumentPurpose::StructReturn));
        let params = func.parameters.iter().map(|p| p.1);

        sig.params.extend(ret.into_iter().chain(params.map(abi)));
        sig.returns = func.return_type.map(abi).into_iter().collect();
        sig
    }

    /// Declare a function and its signature (without the body)
    fn declare_function(&mut self, func: &lir::Function) {
        let sig = Self::fn_signature(self.jit.make_signature(), &func.ir_signature);

        let linkage = if func.public {
            Linkage::Export
        } else {
            Linkage::Local
        };

        let name = func.name.as_str();
        let info = FunctionInfo {
            id: self.jit.declare_function(name, linkage, &sig).unwrap(),
            signature: func.signature.clone(),
            return_by_ref: func.ir_signature.return_ptr,
        };
        self.functions.insert(name.to_string(), info);
    }

    /// Define a function body
    ///
    /// The function must be declared first.
    fn define_function(&mut self, func: &lir::Function, func_ctx: &mut FunctionBuilderContext) {
        let func_id = self.functions[func.name.as_str()].id;

        let mut ctx = self.jit.make_context();
        ctx.func.signature = Self::fn_signature(self.jit.make_signature(), &func.ir_signature);

        #[cfg(feature = "disas")]
        ctx.set_disasm(true);

        let mut builder = FunctionBuilder::new(&mut ctx.func, func_ctx);

        let mut stack_slots = Vec::new();
        for &(v, ref t) in &func.variables {
            let ty = match t {
                lir::ValueOrSlot::Value(ty) => *ty,
                lir::ValueOrSlot::Slot(layout) => {
                    let kind = ir::StackSlotKind::ExplicitSlot;
                    let size = layout.size() as u32;
                    let align_shift = layout.align().ilog2() as u8;
                    let data = ir::StackSlotData::new(kind, size, align_shift);
                    stack_slots.push((v, builder.create_sized_stack_slot(data)));
                    lir::Type::Ptr
                }
            };

            let var = builder.declare_var(ty.into());
            self.vars.insert(v, var, ty);
        }

        let mut func_gen = self::func::FuncGen {
            drop_signature: builder.import_signature(self.drop_signature.clone()),
            clone_signature: builder.import_signature(self.clone_signature.clone()),
            init_string_signature: builder.import_signature(self.init_string_signature.clone()),
            module: self,
            ctx: builder,
            scope: func.scope,
            blocks: HashMap::new(),
        };

        func_gen.entry_block(
            &func.blocks[0],
            &func.ir_signature.parameters,
            stack_slots,
            func.ir_signature.return_ptr,
        );

        // Translate an IR block to a Cranelift block
        for block in &func.blocks[1..] {
            let b = func_gen.block(block.label);
            func_gen.ctx.switch_to_block(b);
            for instruction in &block.body {
                func_gen.instruction(instruction);
            }
        }

        func_gen.finalize();

        self.jit.define_function(func_id, &mut ctx).unwrap();

        #[cfg(feature = "disas")]
        {
            let capstone = self.isa.to_capstone().unwrap();
            let code = ctx.compiled_code().unwrap();
            let code = code.disassemble(None, &capstone).unwrap();
            log::info!("\n{code}");
        }
        self.jit.clear_context(&mut ctx);
    }

    fn finalize(mut self) -> Package {
        self.jit.finalize_definitions().unwrap();
        Package {
            functions: self.functions,
            inner: SharedModuleData::new(self.jit, self.constants, self.registered_fns),
            types: self.types,
        }
    }
}
