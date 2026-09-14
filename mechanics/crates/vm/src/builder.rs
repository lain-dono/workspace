//! The instruction builder.
//!
//! Use this module to build code for your machine to execute.
//!
//! ## Examples
//!
//! ```
//! use vm::{Instruction, InstructionTable, Builder, Machine};
//!
//! fn push(machine: &mut Machine<f64>, args: &[u32]) {
//!     machine.push(machine.code.data[args[0] as usize])
//! }
//!
//! let mut instruction_table: InstructionTable<f64> = InstructionTable::new();
//! instruction_table.add(0, "push", 1, push);
//!
//! let mut builder: Builder<f64> = Builder::new(&instruction_table);
//! builder.push("push", vec![1.23]);
//! ```

use super::{Code, InstructionTable, Opcode};
use bevy::platform::collections::HashMap;
use std::fmt;

/// The builder struct.
///
/// Contains:
/// * an `InstructionTable`.
/// * a list of instructions that have been pushed into this builder.
/// * a `Table` of labels used for jumping.
/// * a list of `T` to be stored in the builder's data section.
pub struct Builder<'a, T: 'a + fmt::Debug + PartialEq> {
    instruction_table: &'a InstructionTable<T>,
    instructions: Vec<Opcode>,
    labels: HashMap<String, usize>,
    data: Vec<T>,
}

impl<T: fmt::Debug + PartialEq> Builder<'_, T> {
    /// Convert a `Builder` into `Code`.
    ///
    /// This function consumes the builder and returns a `Code`.
    pub fn build(self) -> Code<T> {
        let Self {
            instruction_table,
            instructions,
            labels,
            data,
        } = self;

        let symbols = {
            let mut result: Vec<_> = instruction_table.symbols_iter().collect();
            result.sort_by_key(|&(index, _)| index);
            result
        };

        let labels = labels.iter();
        let labels = labels.map(|(key, index)| (*index, key.clone()));

        let mut labels = labels.collect::<Vec<_>>();
        labels.sort_by_key(|&(index, _)| index);

        Code {
            instructions,
            data,
            symbols,
            labels,
        }
    }
}

impl<'a, T: fmt::Debug + PartialEq> Builder<'a, T> {
    /// Create a new `Builder` from an `InstructionTable`.
    pub fn new(instruction_table: &'a InstructionTable<T>) -> Self {
        let mut labels = HashMap::new();

        labels.insert(String::from("main"), 0);

        Self {
            instruction_table,
            instructions: vec![],
            labels,
            data: vec![],
        }
    }

    /// Insert a label at this point in the code.
    ///
    /// Labels are used as targets for jumps.  When you call this method a
    /// label is stored which points to the position of the next instruction.
    pub fn label(&mut self, name: impl Into<String>) {
        let name = name.into();
        assert!(!self.labels.contains_key(&name), "labels must be unique");
        self.labels.insert(name, self.instructions.len());
    }

    /// Push an instruction into the code.
    ///
    /// * `name` should match that of an instruction in the `InstructionTable`.
    /// * `args` a vector of operands to be pushed into the builder's data section.
    pub fn push(&mut self, name: &str, args: impl IntoIterator<Item = T>) {
        let instruction = self.instruction_table.by_name(name);
        let instruction =
            instruction.unwrap_or_else(|| panic!("Unable to find instruction with name {name:?}"));

        self.instructions
            .push(instruction.opcode | (instruction.arity << 28));

        let mut len = 0;
        for arg in args {
            let position = self.data.iter().position(|d| d == &arg);
            let position = position.unwrap_or_else(|| {
                self.data.push(arg);
                self.data.len() - 1
            });

            self.instructions.push(position as u32);
            len += 1;
        }

        assert!(
            len == instruction.arity,
            "Instruction {} has arity of {}, but you provided {len} arguments.",
            instruction.name,
            instruction.arity
        );
    }
}

impl<'a, T: 'a + fmt::Debug + PartialEq> fmt::Debug for Builder<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.data.len() {
            writeln!(f, "@{} = {:?}", i, self.data[i])?;
        }

        let mut ip = 0;
        let len = self.instructions.len();

        loop {
            for (label, &index) in &self.labels {
                if index == ip {
                    writeln!(f, "\n.{label}:")?;
                }
            }

            if ip == len {
                break;
            }

            let op_code = self.instructions[ip] & 0x0FFF_FFFF;
            let arity = (self.instructions[ip] >> 28) as usize;
            ip += 1;

            let instruction = self.instruction_table.by_op_code(op_code);
            let instruction = instruction
                .unwrap_or_else(|| panic!("Unable to find instruction with op code {op_code}"));

            write!(f, "\t{}", &instruction.name)?;

            for &const_idx in &self.instructions[ip..ip + arity] {
                write!(f, " @{const_idx}")?;
            }
            ip += arity;

            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::{Builder, InstructionTable, Machine};

    fn noop(_machine: &mut Machine<usize>, _args: &[u32]) {}

    fn instruction_table() -> InstructionTable<usize> {
        let mut it = InstructionTable::new();
        it.add(0, "noop", 0, noop);
        it.add(1, "push", 1, noop);
        it.add(2, "pop", 0, noop);
        it
    }

    #[test]
    fn new() {
        let it = instruction_table();
        let builder = Builder::new(&it);
        assert!(builder.instructions.is_empty());
    }

    #[test]
    fn push() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("noop", []);
        assert!(!builder.instructions.is_empty());
    }

    #[test]
    #[should_panic(expected = "has arity of")]
    fn push_with_incorrect_arity() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("noop", [1]);
    }

    #[test]
    fn label() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("noop", []);
        builder.label("wow");
        assert_eq!(*builder.labels.get("wow").unwrap(), 1);
    }

    #[test]
    fn data_is_deduped() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("push", [123]);
        builder.push("push", [123]);
        builder.push("push", [123]);
        assert_eq!(builder.data.len(), 1);
    }

    #[test]
    fn debug_format() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("noop", []);
        builder.push("push", [123]);
        builder.push("push", [456]);
        builder.label("some_function");
        builder.push("pop", []);

        let actual = format!("{builder:?}");
        let expected = "@0 = 123
@1 = 456

.main:
\tnoop
\tpush @0
\tpush @1

.some_function:
\tpop
";

        println!("{actual}");
        println!("{expected}");

        assert_eq!(actual, expected);
    }
}
