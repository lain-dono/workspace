//! A virtual instruction.
//!
//! Instructions consist of an op code, a name, an arity and a function.
//!
//! ## Examples
//!
//! A simple `push` instruction, which takes a piece of data from the builder's
//! data space and places it onto the operand stack.
//!
//! ```
//! use vm::{Instruction, Machine};
//!
//! fn push(machine: &mut Machine<u64>, args: &[u32]) {
//!     machine.push(machine.code.data[args[0] as usize]);
//! }
//!
//! Instruction::new(0, "push", 1, push);
//! ```
//!
//! A `noop` instruction which does nothing.
//!
//! ```
//! use vm::{Instruction, Machine};
//!
//! fn noop(_machine: &mut Machine<u64>, _args: &[u32]) {
//!     println!("noop");
//! }
//! ```
//!
//! A `jump` instruction, which takes the name of a label from the builder's data
//! and then jumps to it.
//!
//! Note that operand types have to implement `std::fmt::Debug`.
//!
//! ```
//! use std::fmt;
//! use vm::{Instruction, Machine};
//!
//! #[derive(Debug)]
//! enum Operand { I(i64), S(String) }
//!
//! fn jump(machine: &mut Machine<Operand>, args: &[u32]) {
//!     let label = match &machine.code.data[args[0] as usize] {
//!         &Operand::S(ref str) => str.clone(),
//!         _ => panic!("Cannot jump to non-string label.")
//!     };
//!     machine.jump(&label);
//! }
//!
//! Instruction::new(1, "jump", 1, jump);
//! ```

use super::machine::Machine;
use std::collections::HashMap;
use std::fmt;

pub type Opcode = u32;

/// Describes a single instruction which can be used to execute programs.
///
/// Contains:
/// * An op code - a unique integer to identify this instruction.
/// * A name for serialisation and debugging reasons.
/// * An arity - the number of arguments this instruction expects to receive.
/// * A function which is used to execute the instruction.
pub struct Instruction<T: fmt::Debug> {
    pub opcode: Opcode,
    pub name: String,
    pub arity: u32,
    pub fun: InstructionFn<T>,
}

/// The instruction function signature.
///
/// Each instruction is defined in terms of a function which takes a mutable
/// reference to a `Machine` and an array of `usize`.
///
/// Your instruction is able to manipulate the state of the machine as
/// required (by pushing operands to the stack, for example).
///
/// The `args` array contains indexes into the `Builder`'s data section. It's
/// up to your instruction to retrieve said data.
pub type InstructionFn<T> = fn(machine: &mut Machine<T>, args: &[u32]);

impl<T: fmt::Debug> fmt::Debug for Instruction<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Instruction {{ op_code: {}, name: {}, arity: {} }}",
            self.opcode, self.name, self.arity
        )
    }
}

impl<T: fmt::Debug> Instruction<T> {
    /// Create a new instruction.
    pub fn new(
        op_code: Opcode,
        name: impl Into<String>,
        arity: u32,
        fun: InstructionFn<T>,
    ) -> Self {
        Self {
            opcode: op_code,
            name: name.into(),
            arity,
            fun,
        }
    }
}

/// The instruction table.
///
/// Stores the instructions of your machine and allows them to be retrieved
/// by name or op code.
///
/// Implemented as a `HashMap` behind the scenes.
#[derive(Debug, Default)]
pub struct InstructionTable<T: fmt::Debug> {
    by_opcode: HashMap<Opcode, Instruction<T>>,
}

impl<T: fmt::Debug> InstructionTable<T> {
    /// Create a new empty instruction table.
    pub fn new() -> Self {
        Self {
            by_opcode: HashMap::new(),
        }
    }

    /// Retrieve an instruction by looking up it's op code.
    pub fn by_op_code(&self, op_code: Opcode) -> Option<&Instruction<T>> {
        self.by_opcode.get(&op_code)
    }

    /// Retrieve an instruction by looking up it's name.
    pub fn by_name(&self, name: &str) -> Option<&Instruction<T>> {
        self.by_opcode.values().find(|instr| instr.name == name)
    }

    /// Insert an instruction into the table.
    pub fn add(
        &mut self,
        op_code: Opcode,
        name: impl Into<String>,
        arity: u32,
        fun: InstructionFn<T>,
    ) {
        let instruction = Instruction::new(op_code, name, arity, fun);
        self.by_opcode.insert(op_code, instruction);
    }

    /// Returns `true` if the instruction table is empty.
    pub fn is_empty(&self) -> bool {
        self.by_opcode.is_empty()
    }

    /// Returns a list of symbols for use in the `Code` struct.
    ///
    /// Generates an iterator of tuples containing the op code and the name of each instruction.
    pub fn symbols_iter(&self) -> impl Iterator<Item = (Opcode, String)> + '_ {
        self.by_opcode
            .values()
            .map(|instr| (instr.opcode, instr.name.clone()))
    }
}

#[cfg(test)]
mod creation {
    use super::super::{Instruction, InstructionTable, Machine};

    fn noop(_machine: &mut Machine<()>, _args: &[u32]) {}

    #[test]
    fn instruction() {
        let operand = Instruction::new(13, "noop", 7, noop);
        assert_eq!(operand.opcode, 13);
        assert_eq!(operand.name, "noop".to_string());
        assert_eq!(operand.arity, 7);
    }

    #[test]
    fn table() {
        let mut table = InstructionTable::new();
        assert!(table.is_empty());

        table.add(0, "NOOP", 0, noop);
        assert!(!table.is_empty());

        assert_eq!(table.by_op_code(0).unwrap().name, "NOOP");
        assert_eq!(table.by_name("NOOP").unwrap().opcode, 0);
    }
}
