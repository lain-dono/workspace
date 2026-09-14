//! The machine that makes the magic happen.
//!
//! Pour all your ingredients into `Machine` and make it dance.

use super::{Code, InstructionTable};
use bevy::platform::collections::HashMap;
use std::fmt;

/// A call frame.
///
/// Used internally by the `Machine` to keep track of call scope.
///
/// Contains:
/// * A storage of local variables.
/// * A return address - the instruction pointer for the machine to return to when returning from this call.
#[derive(Debug)]
pub struct Frame<T> {
    locals_data: HashMap<String, T>,
    return_addr: usize,
}

impl<T> Frame<T> {
    /// Creates a new call frame with the specified return address.
    pub fn new(return_addr: usize) -> Self {
        Self {
            locals_data: HashMap::new(),
            return_addr,
        }
    }

    pub fn return_addr(&self) -> usize {
        self.return_addr
    }

    /// Return a reference to the specified local variable.
    pub fn local(&self, name: &str) -> Option<&T> {
        self.locals_data.get(name)
    }

    /// Set the value of a local variable.
    pub fn set_local(&mut self, name: impl Into<String>, value: T) {
        self.locals_data.insert(name.into(), value);
    }
}

/// `Machine` contains all the information needed to run your program.
///
/// * A `Code`, used describe the source instructions and data to execute.
/// * An instruction pointer, which points to the currently-executing instruciton.
/// * A `Table` of constants, which you can use in your instructions if needed.
/// * A `Stack` of `Frame` used to keep track of calls being executed.
/// * A `Stack` of `T` which is used as the main operand stack.
pub struct Machine<'a, T: fmt::Debug + 'a> {
    pub code: Code<T>,

    pub instruction_table: &'a InstructionTable<T>,
    pub constants: &'a HashMap<String, T>,

    pub ip: usize,
    pub call_stack: Vec<Frame<T>>,
    pub operand_stack: Vec<T>,
}

impl<'a, T: fmt::Debug + 'a> Machine<'a, T> {
    /// Returns a new `Machine` ready to execute instructions.
    ///
    /// The machine is initialised by passing in your [`Code`] which contains
    /// all the code and data of your program, and a `Table` of constants.
    pub fn new(
        code: Code<T>,
        constants: &'a HashMap<String, T>,
        instruction_table: &'a InstructionTable<T>,
    ) -> Self {
        Self {
            call_stack: vec![Frame::<T>::new(code.instructions.len())],
            operand_stack: vec![],

            code,
            instruction_table,
            constants,

            ip: 0,
        }
    }

    /// Run the machine.
    ///
    /// Kick off the process of running the program.
    ///
    /// Steps through the instructions in your program executing them one-by-one.
    /// Each instruction function is executed, much like a callback.
    ///
    /// Stops when either the last instruction is executed or when the
    /// last frame is removed from the call stack.
    pub fn run(&mut self) -> Result<(), crate::Error> {
        while self.ip < self.code.instructions.len() {
            let opcode = self.code.instructions[self.ip] & 0x0FFF_FFFF;
            let arity = (self.code.instructions[self.ip] >> 28) as usize;
            self.ip += 1;

            let instruction = self.instruction_table.by_op_code(opcode);
            let instruction = instruction.ok_or(crate::Error::InstructionNotFound(opcode))?;

            let args = &self.code.instructions[self.ip..self.ip + arity];
            let args = unsafe { std::slice::from_raw_parts(args.as_ptr(), args.len()) };
            self.ip += arity;

            (instruction.fun)(self, args);
        }

        Ok(())
    }

    /// Look up a local variable in the current call frame.
    ///
    /// Note that the variable may not be set in the current frame but it's up
    /// to your instruction to figure out how to deal with this situation.
    pub fn local(&self, name: &str) -> Option<&T> {
        self.call_stack.last()?.local(name)
    }

    /// Look for a local variable in all call frames.
    ///
    /// The machine will look in each frame in the call stack starting at the
    /// top and moving down until it locates the local variable in question
    /// or runs out of stack frames.
    pub fn local_find(&self, name: &str) -> Option<&T> {
        let mut iter = self.call_stack.iter().rev();
        iter.find_map(|frame| frame.local(name))
    }

    /// Set a local variable in the current call frame.
    ///
    /// Places a value in the frame's local variable table.
    pub fn set_local(&mut self, name: &str, value: T) {
        self.call_stack.last_mut().unwrap().set_local(name, value);
    }

    /// Push an operand onto the operand stack.
    pub fn push(&mut self, value: T) {
        self.operand_stack.push(value);
    }

    /// Pop an operand off the operand stack.
    pub fn pop(&mut self) -> Result<T, crate::Error> {
        self.operand_stack
            .pop()
            .ok_or(crate::Error::OperandStackIsEmpty)
    }

    /// Performs a call to a named label.
    ///
    /// This method is very similar to `jump` except that it records it's
    /// current instruction pointer and saves it in the call stack.
    ///
    /// This method performs the following actions:
    /// * Create a new frame with it's return address set to the current instruction pointer.
    /// * Jump to the named label using `jump`.
    ///
    /// This method specifically does not transfer operands to call arguments.
    pub fn call(&mut self, label: &str) {
        self.call_stack.push(Frame::new(self.ip));
        self.jump(label);
    }

    /// Perform a jump to a named label.
    ///
    /// This method performs the following actions:
    /// * Retrieve the instruction pointer for a given label from the Code.
    /// * Set the machine's instruction pointer to the new location.
    ///
    /// This method will panic the thread if the label does not exist.
    pub fn jump(&mut self, label: &str) {
        let mut iter = self.code.labels.iter();
        let next = iter.find_map(|&(index, ref name)| (name == label).then_some(index));
        self.ip = next.unwrap_or_else(|| panic!("Attempted to jump to unknown label {label}"));
    }

    /// Performs a return.
    ///
    /// This method pops the top frame off the call stack and moves the
    /// instruction pointer back to the frame's return address.
    /// It's up to you to push your return value onto the operand stack (if
    /// your language has such return semantics).
    ///
    /// The last call frame contains a return address at the end of the source
    /// code, so the machine will stop executing at the beginning of the next
    /// iteration.
    ///
    /// If you call `ret` too many times then the machine will panic when it
    /// attempts to pop the last frame off the stack.
    pub fn ret(&mut self) {
        self.ip = self.call_stack.pop().unwrap().return_addr;
    }
}

#[cfg(test)]
mod test {
    use super::super::{Builder, InstructionTable, Machine};
    use bevy::platform::collections::HashMap;

    type Operand = u16;

    fn push(machine: &mut Machine<Operand>, args: &[u32]) {
        let arg = machine.code.data[args[0] as usize];
        machine.operand_stack.push(arg);
    }

    fn add(machine: &mut Machine<Operand>, _args: &[u32]) {
        let rhs = machine.pop().unwrap();
        let lhs = machine.pop().unwrap();
        machine.operand_stack.push(lhs + rhs);
    }

    fn instruction_table() -> InstructionTable<Operand> {
        let mut it = InstructionTable::new();
        it.add(1, "push", 1, push);
        it.add(2, "add", 0, add);
        it
    }

    #[test]
    fn new() {
        let it = instruction_table();
        let builder = Builder::new(&it);
        let constants = HashMap::new();

        let machine = Machine::new(builder.build(), &constants, &it);
        assert_eq!(machine.ip, 0);
        assert!(!machine.call_stack.is_empty());
        assert!(machine.operand_stack.is_empty());
    }

    #[test]
    fn run() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.push("push", [2]);
        builder.push("push", [3]);
        builder.push("add", []);

        let constants = HashMap::new();
        let mut machine = Machine::new(builder.build(), &constants, &it);

        machine.run().unwrap();

        let result = machine.operand_stack.pop().unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn local() {
        let it = instruction_table();
        let builder = Builder::new(&it);
        let constants = HashMap::new();

        let mut machine = Machine::new(builder.build(), &constants, &it);
        assert!(machine.local("example").is_none());

        machine.set_local("example", 13);
        assert_eq!(*machine.local("example").unwrap(), 13);
    }

    #[test]
    fn local_deep() {
        let it = instruction_table();
        let mut builder = Builder::new(&it);
        builder.label("next");

        let constants = HashMap::new();
        let mut machine = Machine::new(builder.build(), &constants, &it);

        machine.set_local("outer", 13);
        assert_eq!(*machine.local_find("outer").unwrap(), 13);

        machine.call("next");
        machine.set_local("outer", 14);
        machine.set_local("inner", 15);
        assert_eq!(*machine.local_find("outer").unwrap(), 14);
        assert_eq!(*machine.local_find("inner").unwrap(), 15);

        machine.ret();
        assert_eq!(*machine.local_find("outer").unwrap(), 13);
        assert!(machine.local_find("inner").is_none());
    }
}
