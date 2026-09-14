use super::{Local, Registration};
use bevy::reflect::{PartialReflect, Reflect, ReflectKind, ReflectRef, VariantType};
use std::borrow::Cow;

pub type OperationFn = fn(Context<'_, '_>, &'_ dyn PartialReflect) -> ScriptResult;

pub struct Script {
    instructions: Vec<(usize, Box<dyn PartialReflect>)>,
}

pub struct ScriptBuilder<'w> {
    registration: &'w Registration,
    instructions: Vec<(usize, Box<dyn PartialReflect>)>,
}

impl<'w> ScriptBuilder<'w> {
    pub fn new(registration: &'w Registration) -> Self {
        Self {
            registration,
            instructions: Vec::new(),
        }
    }

    pub fn set_command(&mut self, label: usize, command: &str, data: impl PartialReflect) {
        let command = self.registration.find(command).unwrap();
        self.instructions[label] = (command, Box::new(data));
    }

    pub fn emit_command(&mut self, command: &str, data: impl PartialReflect) -> usize {
        let index = self.instructions.len();
        let command = self.registration.find(command).unwrap();
        self.instructions.push((command, Box::new(data)));
        index
    }

    pub fn build(self) -> Script {
        let instructions = self.instructions;
        Script { instructions }
    }
}

pub struct Machine<'w> {
    registration: &'w Registration,
    script: &'w Script,
    current: usize,
    alloc: Vec<Box<dyn PartialReflect>>,
    stack: Vec<&'static dyn PartialReflect>,
}

impl<'w> Machine<'w> {
    pub fn new(registration: &'w Registration, script: &'w Script) -> Self {
        Self {
            registration,
            script,
            current: 0,
            alloc: Vec::new(),
            stack: Vec::new(),
        }
    }

    pub fn run(&mut self, local: &mut Local<'w>) -> Result<(), ScriptError> {
        while self.current < self.script.instructions.len() {
            let &(instruction, ref reflect) = &self.script.instructions[self.current];
            let reflect = reflect.as_partial_reflect();
            let instruction = self.registration.get(instruction);

            let mut jump = None;
            let ctx = Context {
                stack: &mut self.stack,
                alloc: &mut self.alloc,
                local,
                jump: &mut jump,
            };

            (instruction)(ctx, reflect)?;

            self.current = jump.unwrap_or(self.current + 1);
        }

        Ok(())
    }

    pub fn pop(&mut self) -> Option<&'_ dyn PartialReflect> {
        self.stack.pop()
    }
}

pub struct Context<'a, 'w> {
    alloc: &'a mut Vec<Box<dyn PartialReflect>>,
    stack: &'a mut Vec<&'static dyn PartialReflect>,
    local: &'a mut Local<'w>,
    jump: &'a mut Option<usize>,
}

impl<'a, 'w> Context<'a, 'w> {
    fn alloc(&mut self, value: impl PartialReflect) -> &'static mut dyn PartialReflect {
        self.alloc.push(Box::new(value));
        unsafe { &mut *(self.alloc.last_mut().unwrap().as_partial_reflect_mut() as *mut _) }
    }

    pub fn jump(&mut self, addr: usize) {
        *self.jump = Some(addr);
    }

    pub fn local(&mut self) -> &mut Local<'w> {
        self.local
    }

    pub unsafe fn local_ref<T>(&self, name: &str) -> Option<&T> {
        self.local.cast_ref(name)
    }

    pub unsafe fn local_mut<T>(&mut self, name: &str) -> Option<&mut T> {
        self.local.cast_mut(name)
    }

    // pub fn local_scope_ref<U>(&self, name: &str, f: impl FnOnce(Ptr<'_>) -> U) -> Option<U> {
    //     Some(f(self.local.get_ref(name)?))
    // }

    // pub fn local_scope_mut<U>(&mut self, name: &str, f: impl FnOnce(PtrMut<'_>) -> U) -> Option<U> {
    //     Some(f(self.local.get_mut(name)?))
    // }

    pub fn push_alloc(&mut self, value: impl PartialReflect) {
        let value = self.alloc(value);
        self.push(value);
    }

    pub fn push(&mut self, value: &'a dyn PartialReflect) {
        self.stack.push(unsafe { &*(value as *const _) });
    }

    pub fn pop(&mut self) -> Result<&'a dyn PartialReflect, ScriptError> {
        self.stack.pop().ok_or(ScriptError::StackEmpty)
    }

    pub fn peek(&mut self) -> Result<&'a dyn PartialReflect, ScriptError> {
        self.stack.last().copied().ok_or(ScriptError::StackEmpty)
    }

    pub fn peek_prev(&mut self, index: usize) -> Result<&'a dyn PartialReflect, ScriptError> {
        let index = self.stack.len() - index - 1;
        self.stack
            .get(index)
            .copied()
            .ok_or(ScriptError::StackEmpty)
    }
}
