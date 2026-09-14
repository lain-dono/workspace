//! # vm
//!
//! This crate implements a generic stack machine for which you provide the
//! operands and the instructions and this crate provides the rest of the
//! infrastructure required to run it.
//!
//! It also provides a simple instruction builder which you can use to generate
//! your program.
//!
//! Stack machines are computers which use an operand stack to perform the
//! evaluation of postfix expressions.  Every computer architecture has it's
//! own instruction set which is the basic set of operations that the computer
//! can perform.
//!
//! Instructions usually describe basic arithmetic operations, I/O, jumps, etc.
//!
//! ## Computer Architecture
//!
//! The architecture of a computer system is how the various logic components
//! are connected together in order to execute the instructions and produce
//! side-effects (useful outcomes).
//!
//! There are two main ways to organise a computer architecture:
//! * *Von Neumann* - in which the program data and instructions are stored in
//!   the same memory.
//! * *Harvard* - in which the program data and instructions are stored in
//!   separate memory sections.
//!
//! The bulk of modern processors are Von Neumann type machines.
//!
//! We can also classify our machines by the way that they store intermediate
//! values:
//! * *Accumulator* - the most basic form of processor where only a single
//!   register is used to store the results of computation.
//! * *Stack* - stack machines use an operand stack to push and pop results
//!   off the top of.
//! * *Register* - register machines use a number of named (or numbered)
//!   registers to store values or pass arguments.
//!
//! Most modern processors are register machines, although interestingly both
//! register and stack machines can be used to emulate their cousin.
//!
//! ## Instruction Sets
//!
//! The instruction set is the definition of the machine.  Without instructions
//! your machine can't *do* anything.  They are the fundamental building blocks
//! of your computer, so you need to think this through before building it.
//!
//! This virtual machine uses Rust functions as instructions rather than
//! transistors and logic gates, but the effect is the same.
//!
//! In order to generate your instructions you need to create a bunch of Rust
//! functions which conform to the `vm::InstructionFn` signature.
//!
//! For example:
//!
//! ```
//! use vm::Machine;
//! type Operand = i64;
//!
//! fn push(machine: &mut Machine<Operand>, args: &[u32]) {
//!     machine.push(machine.code.data[args[0] as usize]);
//! }
//! ```
//!
//! Once you have finished defining your instructions you can use them to build
//! a `vm::InstructionTable`, where every instruction is identified by
//! it's `op_code`, `name` and `arity`.
//!
//! * `op_code` a positive integer which uniquely identifies this instruction.
//!   This is manually entered rather than auto-generated from insert order
//!   so that you can maintain as much compatibility between versions of your
//!   VM as possible.
//!
//! * `name` a string used to identify this instruction; mainly for debugging.
//!
//! * `arity` the number of arguments your instruction expects *from program
//!   data*.  This is not the number of operands your function needs off the
//!   operand stack.  This is used so that you can place constant data into
//!   the program at compile time.
//!
//! ```
//! use vm::{Instruction, InstructionTable, Machine};
//! type Operand = i64;
//!
//! fn push(machine: &mut Machine<Operand>, args: &[u32]) {
//!     machine.push(machine.code.data[args[0] as usize]);
//! }
//!
//! fn add(machine: &mut Machine<Operand>, _args: &[u32]) {
//!     let rhs = machine.pop().unwrap().clone();
//!     let lhs = machine.pop().unwrap().clone();
//!     machine.push(lhs + rhs);
//! }
//!
//! let mut instruction_table = InstructionTable::new();
//! instruction_table.add(0, "push", 1, push);
//! instruction_table.add(1, "add",  0, add);
//! ```
//!
//! ## Code generation
//!
//! One your instruction set is defined then you can use the
//! `vm::Builder` object to build a representation that the VM can
//! execute.
//!
//! For example, to push two integers on the stack and add them:
//!
//! ```
//! use vm::{Instruction, InstructionTable, Machine, Builder};
//! type Operand = i64;
//!
//! fn push(machine: &mut Machine<Operand>, args: &[u32]) {
//!     machine.push(machine.code.data[args[0] as usize]);
//! }
//!
//! fn add(machine: &mut Machine<Operand>, _args: &[u32]) {
//!     let rhs = machine.pop().unwrap().clone();
//!     let lhs = machine.pop().unwrap().clone();
//!     machine.push(lhs + rhs);
//! }
//!
//! let mut instruction_table = InstructionTable::new();
//! instruction_table.add(0, "push", 1, push);
//! instruction_table.add(1, "add",  0, add);
//!
//! let mut builder: Builder<Operand> = Builder::new(&instruction_table);
//! builder.push("push", vec![3 as Operand]);
//! builder.push("push", vec![4 as Operand]);
//! builder.push("add", vec![]);
//! ```
//!
//! This will result in the following code:
//!
//! ```text
//! @0 = 3
//! @1 = 4
//!
//! .main:
//!   push @0
//!   push @1
//!   add
//! ```
//!
//! ## Running your program
//!
//! Once you have the instructions and code generated then you can put them
//! together with the `vm::Machine` to execute it.
//!
//! ```
//! use vm::{Instruction, InstructionTable, Machine, Builder};
//! use bevy::platform::collections::HashMap;
//!
//! type Operand = i64;
//!
//! fn push(machine: &mut Machine<Operand>, args: &[u32]) {
//!     machine.push(machine.code.data[args[0] as usize]);
//! }
//!
//! fn add(machine: &mut Machine<Operand>, _args: &[u32]) {
//!     let rhs = machine.pop().unwrap().clone();
//!     let lhs = machine.pop().unwrap().clone();
//!     machine.push(lhs + rhs);
//! }
//!
//! let mut instruction_table = InstructionTable::new();
//! instruction_table.add(0, "push", 1, push);
//! instruction_table.add(1, "add",  0, add);
//!
//! let mut builder: Builder<Operand> = Builder::new(&instruction_table);
//! builder.push("push", vec![3 as Operand]);
//! builder.push("push", vec![4 as Operand]);
//! builder.push("add", vec![]);
//!
//! let constants = HashMap::new();
//! let mut machine = Machine::new(builder.build(), &constants, &instruction_table);
//! machine.run();
//! assert_eq!(machine.pop().unwrap(), 7);
//! ```
//!
//! ## Calling functions:
//!
//! Functions are executed by having the machine jump to another label within
//! the code and continue executing from there.
//!
//! Every time the machine jumps it creates a new call frame, which allows it
//! to store and retrieve local variables without clobbering their parent
//! call context.  It also contains the return address, meaning that when you
//! ask the machine to return it will know which address in the code to go back
//! to after removing the frame.
//!
//! You can find an example of function calling in this package's acceptance
//! tests.

#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]

mod builder;
mod code;
mod instruction;
mod machine;

pub mod al;

pub use self::builder::Builder;
pub use self::code::Code;
pub use self::instruction::{Instruction, InstructionTable, Opcode};
pub use self::machine::Machine;
// pub use self::table::{Table, WriteManyTable};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("operand stack is empty")]
    OperandStackIsEmpty,

    #[error("Unable to find instruction with op code {0}")]
    InstructionNotFound(Opcode),
}

#[test]
fn example() {
    use bevy::platform::collections::HashMap;

    type Operand = i64;

    fn push(machine: &mut Machine<Operand>, args: &[u32]) {
        machine.push(machine.code.data[args[0] as usize]);
    }

    fn add(machine: &mut Machine<Operand>, _args: &[u32]) {
        let rhs = machine.pop().unwrap();
        let lhs = machine.pop().unwrap();
        machine.push(lhs + rhs);
    }

    let mut table = InstructionTable::new();
    table.add(0, "push", 1, push);
    table.add(1, "add", 0, add);

    let mut builder = Builder::new(&table);
    builder.push("push", [3]);
    builder.push("push", [4]);
    builder.push("add", []);

    let constants = HashMap::new();
    let mut machine = Machine::new(builder.build(), &constants, &table);

    machine.run().unwrap();

    assert_eq!(machine.pop().unwrap(), 7);
}
