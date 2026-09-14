//! Evaluate IR programs
//!
//! This is mostly used for testing purposes, since this evaluation is
//! fairly slow. This is because all variables at this point are identified
//! by strings and therefore stored as a hashmap.

use crate::{
    ast, lir,
    runtime::{ConstantValue, Runtime, RuntimeFunctionRef},
    types::ResolvedName,
    var::Var,
};
use log::trace;
use std::{collections::HashMap, sync::Arc};

macro_rules! num_op {
    (@i ( $($x:ident),+ ), $f:expr) => {
        match ($($x,)+) {
            ($(lir::Value::I8($x),)+) => lir::Value::I8($f as i8),
            ($(lir::Value::I16($x),)+) => lir::Value::I16($f as i16),
            ($(lir::Value::I32($x),)+) => lir::Value::I32($f as i32),
            ($(lir::Value::I64($x),)+) => lir::Value::I64($f as i64),
            _ => panic!(),
        }
    };

    (@f ( $($x:ident),+ ), $f:expr) => {
        match ($($x,)+) {
            ($(lir::Value::F32($x),)+) => lir::Value::F32($f as f32),
            ($(lir::Value::F64($x),)+) => lir::Value::F64($f as f64),
            _ => panic!(),
        }
    };
}

/// Memory for the IR evaluation
///
/// This matches the
///
/// The IR evaluation is meant to be close to the native execution, but with
/// additional safety guarantees, not just to check programs, but mostly to
/// check correctness of the compiler. To achieve this, we store more
/// metadata for each allocation.
///
/// A pointer consists of:
///
///  - a stack frame index,
///  - a stack frame id,
///  - an allocation index, and
///  - an offset
///
/// When we read or write from a pointer, we do the following steps:
///
///  1. Get the stack frame at the given index.
///  2. Check whether the id matches.
///  3. Read the allocation at the allocation index.
///  4. Check that the access is in bounds (i.e. `offset + size < len`).
///  5. Check that the access is aligned (i.e. `offset % size == 0`)
///
/// This guarantees most of the properties that we would like to check:
///
///  - No use after free.
///  - No unaligned accesses.
///  - No out of bounds accesses.
///
/// Of course these are only checked at runtime, not statically enforced.
/// But it provides a good mechanism for testing the compiler.
#[derive(Debug)]
pub struct Memory {
    /// The current stack
    stack: Vec<StackFrame>,

    /// All pointers that have been given out
    pointers: Vec<Pointer>,
}

#[derive(Clone, Debug)]
enum Pointer {
    Global(GlobalPointer),
    Local(LocalPointer),
}

#[derive(Clone, Debug)]
pub struct GlobalPointer {
    ptr: *mut (),
}

#[derive(Clone, Debug)]
pub struct LocalPointer {
    /// Where in the stack the pointee lives
    stack_index: usize,

    /// ID of the stack frame where the pointee lives
    stack_id: usize,

    /// Allocation within the stack frame this pointer was created with
    allocation_index: usize,

    /// Offset within the allocation this pointer refers to
    allocation_offset: usize,
}

#[derive(Debug)]
struct StackFrame {
    id: usize,
    return_address: usize,
    return_place: Option<Var>,
    allocations: Vec<Allocation>,
}

#[derive(Debug)]
struct Allocation {
    inner: Box<[u8]>,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            pointers: Vec::new(),
            stack: vec![StackFrame {
                id: 0,
                return_address: 0,
                return_place: None,
                allocations: Vec::new(),
            }],
        }
    }
}

impl Memory {
    pub fn write(&mut self, p: usize, val: &[u8]) {
        match &self.pointers[p] {
            Pointer::Local(p) => {
                let frame = &mut self.stack[p.stack_index];
                assert_eq!(frame.id, p.stack_id);
                frame.write(p, val);
            }
            Pointer::Global(_p) => panic!("Don't write to globals!"),
        }
    }

    #[must_use]
    pub fn read_array<const N: usize>(&self, p: usize) -> [u8; N] {
        let slice = self.read_slice(p, N);
        slice.try_into().unwrap()
    }

    #[must_use]
    pub fn read_slice(&self, p: usize, size: usize) -> &[u8] {
        match &self.pointers[p] {
            Pointer::Local(p) => {
                let frame = &self.stack[p.stack_index];
                assert_eq!(frame.id, p.stack_id);
                frame.read(p, size)
            }
            Pointer::Global(p) => unsafe { std::slice::from_raw_parts(p.ptr.cast::<u8>(), size) },
        }
    }

    fn push_frame(&mut self, return_address: usize, return_place: Option<Var>) {
        self.stack.push(StackFrame {
            id: self.stack.len(),
            return_address,
            return_place,
            allocations: Vec::new(),
        });
    }

    fn pop_frame(&mut self) -> Option<StackFrame> {
        // Keep the root, because it's values provided by the runtime
        if self.stack.len() == 1 {
            return None;
        }
        self.stack.pop()
    }

    pub fn allocate(&mut self, bytes: usize) -> usize {
        let stack_index = self.stack.len() - 1;
        let frame = &mut self.stack[stack_index];
        let stack_id = frame.id;
        let allocation_index = frame.allocations.len();
        frame.allocations.push(Allocation {
            inner: vec![0; bytes].into_boxed_slice(),
        });
        self.pointers.push(Pointer::Local(LocalPointer {
            stack_index,
            stack_id,
            allocation_index,
            allocation_offset: 0,
        }));
        self.pointers.len() - 1
    }

    #[must_use]
    pub fn get(&self, p: usize) -> *mut () {
        match &self.pointers[p] {
            Pointer::Local(p) => self.stack[p.stack_index].get(p),
            Pointer::Global(p) => p.ptr,
        }
    }
}

impl StackFrame {
    fn write(&mut self, p: &LocalPointer, val: &[u8]) {
        let alloc = &mut self.allocations[p.allocation_index];
        alloc.write(p.allocation_offset, val);
    }

    fn read(&self, p: &LocalPointer, size: usize) -> &[u8] {
        let alloc = &self.allocations[p.allocation_index];
        alloc.read(p.allocation_offset, size)
    }

    fn get(&self, p: &LocalPointer) -> *mut () {
        let alloc = &self.allocations[p.allocation_index];
        alloc.get(p.allocation_offset)
    }
}

impl Allocation {
    fn write(&mut self, offset: usize, val: &[u8]) {
        assert!(
            offset + val.len() <= self.inner.len(),
            "memory access out of bounds"
        );
        assert!(
            offset.is_multiple_of(val.len()),
            "memory access is unaligned"
        );

        self.inner[offset..offset + val.len()].copy_from_slice(val);
    }

    fn read(&self, offset: usize, size: usize) -> &[u8] {
        assert!(
            offset + size <= self.inner.len(),
            "memory access out of bounds"
        );
        assert!(offset.is_multiple_of(size), "memory access is unaligned");

        &self.inner[offset..offset + size]
    }

    fn get(&self, offset: usize) -> *mut () {
        &raw const self.inner[offset] as *mut _
    }
}

/// Evaluate IR
///
/// This is mostly used for testing purposes, since this evaluation is
/// fairly slow. This is because all variables at this point are identified
/// by strings and therefore stored as a hashmap.
pub fn eval(
    rt: &Runtime,
    p: &[lir::Function],
    name: &str,
    mem: &mut Memory,
    args: Vec<lir::Value>,
) -> Option<lir::Value> {
    let name = ast::ident::Ident::from(format!("pkg.{name}"));
    let f = p
        .iter()
        .find(|f| f.name == name)
        .expect("Need a main function!");

    let parameters = f.ir_signature.parameters.clone();

    // Make the program easier to work with by collecting all instructions
    // and constructing a map from labels to indices.
    let mut block_map = HashMap::new();
    let mut instructions = Vec::new();

    for block in p.iter().flat_map(|f| &f.blocks) {
        block_map.insert(block.label, instructions.len());
        instructions.extend(block.body.clone());
    }

    let constants: HashMap<ResolvedName, ConstantValue> = rt
        .constants()
        .values()
        .map(|g| (g.name, g.value.clone()))
        .collect();

    // This is our working memory for the interpreter
    let mut vars = HashMap::<Var, lir::Value>::new();

    assert_eq!(
        parameters.len(),
        args.len() - f.ir_signature.return_ptr as usize,
        "incorrect number of arguments"
    );

    let mut values = args.into_iter();
    if f.ir_signature.return_ptr {
        vars.insert(Var::Return(f.scope), values.next().unwrap());
    }

    for ((x, _), v) in parameters.iter().zip(values) {
        vars.insert(Var::Ident(f.scope, *x), v);
    }

    for &(var, ref val_or_slot) in &f.variables {
        if let lir::ValueOrSlot::Slot(layout) = val_or_slot {
            let ptr = mem.allocate(layout.size());
            vars.insert(var, lir::Value::Ptr(ptr));
        }
    }

    let mut program_counter = block_map[&f.entry_block];

    loop {
        let instruction = &instructions[program_counter];
        trace!("{instruction:?}");
        match instruction {
            lir::Instruction::Jump(b) => {
                program_counter = block_map[b];
                continue;
            }
            lir::Instruction::Switch {
                examinee,
                branches,
                fallback,
            } => {
                let val = examinee.eval(&vars);
                let x = val.switch_on();
                let mut branches = branches.iter();
                let label = branches.find_map(|(i, branch)| (*i == x).then_some(branch));
                program_counter = block_map[label.unwrap_or(fallback)];
                continue;
            }
            lir::Instruction::Branch {
                cond,
                accept,
                reject,
            } => {
                let val = cond.eval(&vars);
                let x = val.switch_on();
                program_counter = block_map[if x != 0 { accept } else { reject }];
                continue;
            }
            lir::Instruction::Assign { to, from: val, .. } => {
                let val = val.eval(&vars);
                vars.insert(to.0, val);
            }
            lir::Instruction::ConstAddr { to, name } => {
                let x = constants.get(name).unwrap();
                let x = x.ptr();
                mem.pointers
                    .push(Pointer::Global(GlobalPointer { ptr: x.cast_mut() }));
                vars.insert(to.0, lir::Value::Ptr(mem.pointers.len() - 1));
            }
            lir::Instruction::Call {
                to,
                func,
                args,
                out_ptr,
            } => {
                let f = p.iter().find(|f| f.name == *func).unwrap();

                mem.push_frame(program_counter, to.clone().map(|to| to.0));

                for &(var, ref val_or_slot) in &f.variables {
                    if let lir::ValueOrSlot::Slot(layout) = val_or_slot {
                        let ptr = mem.allocate(layout.size());
                        vars.insert(var, lir::Value::Ptr(ptr));
                    }
                }

                if let Some(out_ptr) = out_ptr {
                    let val = lir::Operand::from(out_ptr.clone()).eval(&vars);
                    vars.insert(Var::Return(f.scope), val);
                }

                let names = f.ir_signature.parameters.iter().map(|p| p.0);

                for (name, arg) in names.zip(args) {
                    let val = arg.eval(&vars);
                    vars.insert(Var::Ident(f.scope, name), val);
                }
                program_counter = block_map[&f.entry_block];
                continue;
            }
            lir::Instruction::CallRuntime { func, args } => {
                let args: Vec<_> = args.iter().map(|a| a.eval(&vars)).collect();
                call_runtime_function(rt, mem, *func, args);
            }
            lir::Instruction::Return(ret) => {
                let val = ret.as_ref().map(|r| r.eval(&vars));
                if let Some(frame) = mem.pop_frame() {
                    if let Some(val) = val {
                        vars.insert(frame.return_place.unwrap(), val);
                    }
                    program_counter = frame.return_address + 1;
                    continue;
                }
                return val;
            }
            lir::Instruction::Cmp { to, cmp, lhs, rhs } => {
                let lhs = lhs.eval(&vars);
                let rhs = rhs.eval(&vars);
                vars.insert(to.0, cmp.eval(lhs, rhs));
            }
            lir::Instruction::Unary { op, to, val } => {
                let x = val.eval(&vars);
                vars.insert(to.0, op.eval(x));
            }
            lir::Instruction::Op { op, to, lhs, rhs } => {
                let lhs = lhs.eval(&vars);
                let rhs = rhs.eval(&vars);
                vars.insert(to.0, op.eval(lhs, rhs));
            }
            lir::Instruction::Offset { to, from, offset } => {
                let lir::Value::Ptr(from) = from.eval(&vars) else {
                    panic!()
                };
                let Pointer::Local(ptr) = &mem.pointers[from] else {
                    panic!("Don't offset global pointer")
                };
                mem.pointers.push(Pointer::Local(LocalPointer {
                    allocation_offset: ptr.allocation_offset + *offset as usize,
                    ..ptr.clone()
                }));
                let new = mem.pointers.len() - 1;
                vars.insert(to.0, lir::Value::Ptr(new));
            }
            lir::Instruction::Initialize { to, bytes, layout } => {
                // There are many cases where we only want to initialize the start of an allocation,
                // but it needs to be in bounds of the allocation.
                assert!(bytes.len() <= layout.size());
                let pointer = mem.allocate(layout.size());
                mem.write(pointer, bytes);
                vars.insert(to.0, lir::Value::Ptr(pointer));
            }
            lir::Instruction::Write { to, val } => {
                let lir::Value::Ptr(to) = to.eval(&vars) else {
                    panic!()
                };
                mem.write(to, val.eval(&vars).as_slice());
            }
            lir::Instruction::Read { to, from } => {
                let lir::Value::Ptr(from) = from.eval(&vars) else {
                    panic!()
                };
                let res = mem.read_slice(from, to.1.size());
                vars.insert(to.0, lir::Value::from_slice(to.1, res));
            }
            lir::Instruction::Copy { to, from, size } => {
                let to = to.eval(&vars);
                let from = from.eval(&vars);
                let (lir::Value::Ptr(to), lir::Value::Ptr(from)) = (to, from) else {
                    panic!()
                };
                let data: Vec<_> = mem.read_slice(from, *size).into();
                mem.write(to, &data);
            }
            lir::Instruction::Clone { to, from, clone } => {
                let to = to.eval(&vars);
                let from = from.eval(&vars);
                let (lir::Value::Ptr(to), lir::Value::Ptr(from)) = (to, from) else {
                    panic!()
                };
                let to = mem.get(to);
                let from = mem.get(from);
                unsafe { (clone)(from, to) }
            }
            lir::Instruction::Drop { var, drop } => {
                if let Some(drop) = drop {
                    let lir::Value::Ptr(val) = var.eval(&vars) else {
                        panic!()
                    };
                    let p = mem.get(val);
                    unsafe { (drop)(p) }
                }
            }
            lir::Instruction::InitString { to, string, .. } => {
                let ptr = mem.allocate(size_of::<Arc<str>>());
                let ptr_value = mem.get(ptr).cast::<Arc<str>>();
                unsafe { std::ptr::write(ptr_value, Arc::from(string.as_ref())) };
                vars.insert(to.0, lir::Value::Ptr(ptr));
            }
        }

        program_counter += 1;
    }
}

fn call_runtime_function(
    rt: &Runtime,
    mem: &mut Memory,
    func: RuntimeFunctionRef,
    args: Vec<lir::Value>,
) {
    let func = rt.get_function(func);

    // The number of passed arguments should be the number of arguments the
    // function takes plus 1 for the out pointer.
    assert_eq!(func.func.parameter_types().len() + 1, args.len());

    (func.func.ir_function())(mem, args);
}

impl lir::Operand {
    fn eval(&self, mem: &HashMap<Var, lir::Value>) -> lir::Value {
        match self {
            Self::Place(p) => *mem.get(p).unwrap_or_else(|| {
                panic!("No value was found for place {p:?} in memory: {mem:#?}")
            }),
            Self::Value(v) => *v,
        }
    }
}

impl lir::Compare {
    fn eval(self, lhs: lir::Value, rhs: lir::Value) -> lir::Value {
        lir::Value::Bool(match self {
            lir::Compare::IEq | lir::Compare::FEq => lhs == rhs,
            lir::Compare::INe | lir::Compare::FNe => lhs != rhs,

            lir::Compare::ULt => lhs.as_u64() < rhs.as_u64(),
            lir::Compare::ULe => lhs.as_u64() <= rhs.as_u64(),
            lir::Compare::UGt => lhs.as_u64() > rhs.as_u64(),
            lir::Compare::UGe => lhs.as_u64() >= rhs.as_u64(),

            lir::Compare::SLt => lhs.as_i64() < rhs.as_i64(),
            lir::Compare::SLe => lhs.as_i64() <= rhs.as_i64(),
            lir::Compare::SGt => lhs.as_i64() > rhs.as_i64(),
            lir::Compare::SGe => lhs.as_i64() >= rhs.as_i64(),

            lir::Compare::FLt => lhs.as_f64() < rhs.as_f64(),
            lir::Compare::FLe => lhs.as_f64() <= rhs.as_f64(),
            lir::Compare::FGt => lhs.as_f64() > rhs.as_f64(),
            lir::Compare::FGe => lhs.as_f64() >= rhs.as_f64(),
        })
    }
}

impl lir::UnOp {
    fn eval(self, x: lir::Value) -> lir::Value {
        match self {
            lir::UnOp::Eqz => match x {
                lir::Value::Bool(x) => lir::Value::Bool(!x),
                _ => panic!(),
            },

            lir::UnOp::Clz => num_op!(@i(x), x.cast_unsigned().leading_zeros()),
            lir::UnOp::Ctz => num_op!(@i(x), x.cast_unsigned().trailing_zeros()),
            lir::UnOp::Pop => num_op!(@i(x), x.cast_unsigned().count_ones()),
            lir::UnOp::BNot => num_op!(@i(x), !x),

            lir::UnOp::INeg => num_op!(@i(x), -x),
            lir::UnOp::FNeg => num_op!(@f(x), -x),
        }
    }
}

impl lir::BinOp {
    fn eval(self, l: lir::Value, r: lir::Value) -> lir::Value {
        match self {
            Self::IAdd => num_op!(@i(l, r), l + r),
            Self::FAdd => num_op!(@f(l, r), l + r),
            Self::ISub => num_op!(@i(l, r), l - r),
            Self::FSub => num_op!(@f(l, r), l - r),
            Self::IMul => num_op!(@i(l, r), l * r),
            Self::FMul => num_op!(@f(l, r), l * r),
            Self::SDiv => num_op!(@i(l, r), l / r),
            Self::UDiv => num_op!(@i(l, r), l.cast_unsigned()  / r .cast_unsigned()),
            Self::FDiv => num_op!(@f(l, r), l / r),
            Self::SRem => num_op!(@i(l, r), l % r),
            Self::URem => num_op!(@i(l, r), l.cast_unsigned() % r.cast_unsigned()),
        }
    }
}
