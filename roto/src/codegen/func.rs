use super::{MEMFLAGS, ModuleBuilder, emit::ImmOp};
use crate::{ast::ident::Ident, label::LabelRef, lir, types, var::Var};
use cranelift::{
    codegen::ir,
    frontend::{FunctionBuilder, Switch, Variable},
    module::{DataDescription, FuncId, Module as _},
    prelude::InstBuilder,
};
use std::{collections::HashMap, sync::Arc};

pub(super) struct FuncGen<'c> {
    pub module: &'c mut ModuleBuilder,

    /// The cranelift function builder
    pub ctx: FunctionBuilder<'c>,

    /// Scope of the function
    pub scope: types::ScopeRef,

    /// Blocks of the function
    pub blocks: HashMap<LabelRef, ir::Block>,

    /// Signature to use for calls to `clone`
    pub clone_signature: ir::SigRef,

    /// Signature to use for calls to `drop`
    pub drop_signature: ir::SigRef,

    /// Signature to use for calls to `init_string`
    pub init_string_signature: ir::SigRef,
}

impl FuncGen<'_> {
    pub fn finalize(mut self) {
        self.ctx.seal_all_blocks();
        self.ctx.finalize();
    }

    /// Set up the entry block for the function
    pub fn entry_block(
        &mut self,
        block: &lir::Block,
        parameters: &[(Ident, lir::Type)],
        stack_slots: Vec<(Var, ir::StackSlot)>,
        return_ptr: bool,
    ) {
        let entry = self.block(block.label);
        self.ctx.switch_to_block(entry);

        {
            let scope = self.scope;
            let ret = return_ptr.then(|| lir::TypedVar::ret(scope, lir::Type::Ptr));
            let args = parameters.iter();
            let args = args.map(|&(x, ty)| lir::TypedVar::explicit(scope, x, ty));
            for var in ret.into_iter().chain(args) {
                let _ = self.decl_var(var);
            }
        }

        self.ctx.append_block_params_for_function_params(entry);

        let args = self.ctx.block_params(entry).to_owned();
        let mut args = args.into_iter();

        if return_ptr {
            let var = self.module.vars.find(&Var::Return(self.scope));
            self.ctx.def_var(var, args.next().unwrap());
        }

        for (&(ident, _), val) in parameters.iter().zip(args) {
            let var = self.module.vars.find(&Var::Ident(self.scope, ident));
            self.ctx.def_var(var, val);
        }

        for (var, slot) in stack_slots {
            let var = self.module.vars.find(&var);
            let value = self.stack_addr(slot, 0);
            self.ctx.def_var(var, value);
        }

        for instruction in &block.body {
            self.instruction(instruction);
        }
    }

    /// Translate an IR instruction to cranelift instructions which are
    /// added to the current block
    pub fn instruction(&mut self, instruction: &lir::Instruction) {
        match instruction {
            lir::Instruction::Jump(label) => {
                let block = self.block(*label);
                self.ctx.ins().jump(block, &[]);
            }
            lir::Instruction::Switch {
                examinee,
                branches,
                fallback,
            } => {
                let mut switch = Switch::new();
                for &(index, label) in branches {
                    switch.set_entry(index as u128, self.block(label));
                }
                let otherwise = self.block(*fallback);
                let examinee = self.operand(examinee);
                switch.emit(&mut self.ctx, examinee, otherwise);
            }
            lir::Instruction::Branch {
                cond,
                accept,
                reject,
            } => {
                let c = self.operand(cond);
                let accept = self.block(*accept);
                let reject = self.block(*reject);
                self.ctx.ins().brif(c, accept, &[], reject, &[]);
            }

            lir::Instruction::Assign { to, from } => {
                let val = self.operand(from);
                self.def_var(to.clone(), val);
            }
            lir::Instruction::Call {
                to,
                func,
                args,
                out_ptr,
            } => {
                let func = self.module.functions[func.as_str()].id;
                self.call(to.clone(), func, out_ptr.clone(), args);
            }
            lir::Instruction::CallRuntime { func, args } => {
                let (ptr, func) = self.module.rt_functions[func];
                let mut new_args = vec![lir::Operand::Value(lir::Value::Ptr(ptr as usize))];
                new_args.extend_from_slice(args);
                self.call(None, func, None, &new_args);
            }
            lir::Instruction::Return(Some(val)) => {
                let val = self.operand(val);
                self.ctx.ins().return_(&[val]);
            }
            lir::Instruction::Return(None) => {
                self.ctx.ins().return_(&[]);
            }
            lir::Instruction::Cmp { to, cmp, lhs, rhs } => {
                let lhs = self.operand(lhs);
                let rhs = self.operand(rhs);
                let value = cmp.emit(self.ctx.ins(), lhs, rhs);
                self.def_var(to.clone(), value);
            }
            lir::Instruction::Unary { op, to, val } => {
                let val = self.operand(val);
                let value = op.emit(self.ctx.ins(), val);
                self.def_var(to.clone(), value);
            }

            lir::Instruction::Op { op, to, lhs, rhs } => {
                let lhs = self.operand(lhs);
                let rhs = self.operand(rhs);
                let value = op.emit(self.ctx.ins(), lhs, rhs);
                self.def_var(to.clone(), value);
            }
            lir::Instruction::Initialize { to, bytes, layout } => {
                let src = self.anon_data(bytes.clone().into());
                let (dst, _) = self.calloc(*layout);
                self.copy_nonoverlapping(src, dst, bytes.len());
                self.def_var(to.clone(), dst);
            }
            lir::Instruction::Write { to, val } => {
                let x = self.operand(val);
                let to = self.operand(to);
                self.ctx.ins().store(MEMFLAGS, x, to, 0);
            }
            lir::Instruction::Read { to, from } => {
                let from = self.operand(from);
                let from = self.ctx.ins().load(to.1.into(), MEMFLAGS, from, 0);
                self.def_var(to.clone(), from);
            }
            lir::Instruction::Offset { to, from, offset } => {
                let from = self.operand(from);
                let value = ImmOp::Iadd.emit(self.ctx.ins(), from, *offset as i64);
                self.def_var(to.clone(), value);
            }
            lir::Instruction::Copy { to, from, size } => {
                let (src, dst) = (self.operand(from), self.operand(to));
                self.copy_nonoverlapping(src, dst, *size);
            }
            lir::Instruction::Clone { to, from, clone } => {
                let (src, dst) = (self.operand(from), self.operand(to));
                self.call_indirect(self.clone_signature, *clone as usize, &[src, dst]);
            }
            lir::Instruction::Drop { var, drop } => {
                if let Some(drop) = drop {
                    let var = self.operand(var);
                    self.call_indirect(self.drop_signature, *drop as usize, &[var]);
                }
            }
            lir::Instruction::ConstAddr { to, name } => {
                let ptr = self.module.constants.get(name).unwrap().ptr();
                let val = self.emit_const(lir::Value::Ptr(ptr as usize));
                self.def_var(to.clone(), val);
            }
            lir::Instruction::InitString { to, string } => {
                unsafe extern "C" fn init(s: *mut Arc<str>, data: *mut u8, len: usize) {
                    unsafe {
                        let slice = core::slice::from_raw_parts(data, len);
                        let str = core::str::from_utf8_unchecked(slice);
                        s.write(Arc::<str>::from(str));
                    }
                }

                let to = self.use_var(&to.0);
                let len = self.emit_const(lir::Value::Ptr(string.len()));
                let data = self.anon_data(string.clone().into_bytes().into());

                let args = [to, data, len];
                self.call_indirect(self.init_string_signature, init as usize, &args);
            }
        }
    }

    fn anon_data(&mut self, contents: Box<[u8]>) -> ir::Value {
        let jit = &mut self.module.jit;

        let data_id = jit.declare_anonymous_data(false, false).unwrap();
        let mut data = DataDescription::new();
        data.define(contents);
        jit.define_data(data_id, &data).unwrap();

        let data = jit.declare_data_in_func(data_id, self.ctx.func);
        let ptr_ty = self.module.isa.pointer_type();
        self.ctx.ins().global_value(ptr_ty, data)
    }

    fn copy_nonoverlapping(&mut self, src: ir::Value, dst: ir::Value, size: usize) {
        let config = self.module.isa.frontend_config();
        let size = size as u64;
        self.ctx
            .emit_small_memory_copy(config, dst, src, size, 0, 0, true, MEMFLAGS);
    }

    fn calloc(&mut self, layout: crate::runtime::layout::Layout) -> (ir::Value, ir::StackSlot) {
        let size = layout.size() as u32;
        let align_shift = layout.align().ilog2() as u8;
        let data = ir::StackSlotData::new(ir::StackSlotKind::ExplicitSlot, size, align_shift);
        let slot = self.ctx.create_sized_stack_slot(data);
        (self.stack_addr(slot, 0), slot)
    }

    fn stack_addr(&mut self, slot: ir::StackSlot, offset: i32) -> ir::Value {
        let ty = lir::Type::Ptr.into();
        self.ctx.ins().stack_addr(ty, slot, offset)
    }

    fn call(
        &mut self,
        to: Option<lir::TypedVar>,
        func: FuncId,
        out_ptr: Option<lir::TypedVar>,
        args: &[lir::Operand],
    ) {
        let func = self.module.jit.declare_func_in_func(func, self.ctx.func);

        let ret_arg = out_ptr.map(lir::Operand::from);
        let ret_arg = ret_arg.as_ref().into_iter();

        let args = ret_arg.chain(args.iter()).map(|val| self.operand(val));
        let args = args.collect::<Vec<_>>();

        let inst = self.ctx.ins().call(func, &args);

        if let Some(to) = to {
            self.def_var(to, self.ctx.inst_results(inst)[0]);
        }
    }

    fn call_indirect(&mut self, sig: ir::SigRef, addr: usize, args: &[ir::Value]) {
        let init = self.emit_const(lir::Value::Ptr(addr));
        self.ctx.ins().call_indirect(sig, init, args);
    }

    /// Get the block for the given label or create it if it doesn't exist
    pub fn block(&mut self, label: LabelRef) -> ir::Block {
        let entry = self.blocks.entry(label);
        *entry.or_insert_with(|| self.ctx.create_block())
    }

    /// Define a variable with a value
    fn def_var(&mut self, var: lir::TypedVar, value: ir::Value) {
        let var = self.decl_var(var);
        self.ctx.def_var(var, value);
    }

    fn decl_var(&mut self, lir::TypedVar(var, ty): lir::TypedVar) -> Variable {
        let entry = self.module.vars.map.entry(var);
        let &mut (var, _) = entry.or_insert_with(|| (self.ctx.declare_var(ty.into()), ty));
        var
    }

    fn operand(&mut self, val: &lir::Operand) -> ir::Value {
        match val {
            lir::Operand::Place(p) => self.use_var(p),
            lir::Operand::Value(val) => self.emit_const(*val),
        }
    }

    fn use_var(&mut self, var: &Var) -> ir::Value {
        self.ctx.use_var(self.module.vars.get(var))
    }

    fn emit_const(&mut self, val: lir::Value) -> ir::Value {
        val.emit(self.ctx.ins())
    }
}
