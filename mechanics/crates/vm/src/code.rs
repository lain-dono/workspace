//! Code
//!
//! This module represents a hunk of code ready to be executed by the VM.
//! Code can be created in one of two ways:
//!
//! * From an instance of a `Builder`.
//! * From dumped bytecode.
//!
//! ## Creating code from a Builder:
//!
//! ```
//! # use vm::{Machine, Instruction, InstructionTable, Code, Builder};
//!
//! #[derive(Debug, PartialEq)]
//! struct Operand(i64);
//!
//! fn example_noop(_machine: &mut Machine<Operand>, _args: &[u32]) {}
//!
//! # fn main() {
//! let mut instruction_table = InstructionTable::new();
//! instruction_table.add(1, "push", 1, example_noop);
//!
//! let mut builder = Builder::new(&instruction_table);
//! builder.push("push", vec![Operand(13)]);
//! builder.push("push", vec![Operand(14)]);
//!
//! let code: Code<Operand> = builder.build();
//! # }
//! ```

use super::instruction::Opcode;
use std::fmt;

/// A structure containing runnable or dumpable code.
///
/// See the module-level docs for more details.
pub struct Code<T> {
    /// List of instructions.
    ///
    /// This is the executable source program of the code.
    /// It is a simple format based around the following:
    ///
    /// ```text
    /// | Op Code | No of args | Args ...         |
    /// | 0x01    | 0x03       | 0x01, 0x02, 0x03 |
    /// ```
    pub instructions: Vec<Opcode>,

    /// The constant data compiled into the code.
    pub data: Vec<T>,

    /// List of tuples containing op codes and instruction names.
    pub symbols: Vec<(Opcode, String)>,

    /// List of tuples containing the IP of the label and the name of the label.
    pub labels: Vec<(usize, String)>,
}

impl<T: fmt::Debug> fmt::Debug for Code<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write out constant data into the header.
        for i in 0..self.data.len() {
            writeln!(f, "@{} = {:?}", i, self.data[i])?;
        }

        // Loop through the code and print out useful stuff.
        let mut ip = 0;
        let len = self.instructions.len();

        loop {
            // If this IP has a label, then print it out.
            for &(index, ref label) in &self.labels {
                if ip == index {
                    writeln!(f, "\n.{label}:")?;
                    break;
                }
            }

            if ip == len {
                break;
            }

            let op_code = self.instructions[ip] & 0x0FFF_FFFF;
            let arity = (self.instructions[ip] >> 28) as usize;
            ip += 1;

            // Print this instruction's name
            for &(op, ref symbol) in &self.symbols {
                if op == op_code {
                    write!(f, "\t{symbol}")?;
                    break;
                }
            }

            for &const_idx in &self.instructions[ip..ip + arity] {
                write!(f, " @{const_idx}")?;
            }
            ip += arity;

            writeln!(f)?;
        }

        Ok(())
    }
}

impl<T: fmt::Debug> Code<T> {
    /// Returns the IP for a given label.
    ///
    /// This function is used within the `Machine` to perform jumps.
    pub fn find_label(&self, name: &str) -> Option<usize> {
        self.labels
            .iter()
            .find_map(|&(index, ref label)| (label == name).then_some(index))
    }

    /// Returns the IP for a given label.
    ///
    /// This function is used within the `Machine` to perform jumps.
    pub fn find_label_rev(&self, name: &str) -> Option<usize> {
        self.labels
            .iter()
            .rev()
            .find_map(|&(index, ref label)| (label == name).then_some(index))
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
    fn from_builder() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("push", [13]);
        builder.push("push", [14]);

        let code = builder.build();

        assert_eq!(code.symbols.len(), 3);
        assert_eq!(code.symbols[0], (0, "noop".to_string()));
        assert_eq!(code.symbols[1], (1, "push".to_string()));
        assert_eq!(code.symbols[2], (2, "pop".to_string()));

        assert_eq!(code.instructions, [0x1000_0001, 0, 0x1000_0001, 1]);
        assert_eq!(code.data, [13, 14]);
        assert_eq!(code.labels.len(), 1);
        assert_eq!(code.labels[0], (0, "main".to_string()));
    }

    #[test]
    fn main_addr() {
        let it = instruction_table();
        let builder = Builder::new(&it);

        let code = builder.build();
        assert_eq!(code.find_label("main").unwrap(), 0);
    }

    #[test]
    fn debug_formatter() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("noop", []);
        builder.push("push", [123]);
        builder.push("push", [456]);
        builder.label("some_function");
        builder.push("pop", []);

        let code = builder.build();

        let actual = format!("{code:?}");
        let expected = "@0 = 123
@1 = 456

.main:
\tnoop
\tpush @0
\tpush @1

.some_function:
\tpop
";
        assert_eq!(actual, expected);
    }
}
