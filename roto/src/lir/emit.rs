use crate::{label::LabelRef, lir};

type DropFn = unsafe extern "C" fn(*mut ());
type CloneFn = unsafe extern "C" fn(*const (), *mut ());

#[derive(Debug)]
pub struct Block {
    pub label: LabelRef,
    pub body: Vec<lir::Instruction>,
}

pub struct FlowBuilder {
    pub(crate) storage: Vec<Block>,
}

impl FlowBuilder {
    pub fn start_block(&mut self, label: LabelRef) {
        self.storage.push(Block {
            label,
            body: Vec::new(),
        });
    }

    #[must_use]
    pub fn current_label(&self) -> LabelRef {
        self.storage.last().unwrap().label
    }

    pub fn emit(&mut self, instruction: lir::Instruction) {
        self.storage.last_mut().unwrap().body.push(instruction);
    }

    pub fn jump(&mut self, label: LabelRef) {
        self.emit(lir::Instruction::Jump(label));
    }

    pub fn ret(&mut self, result: Option<lir::Operand>) {
        self.emit(lir::Instruction::Return(result));
    }

    pub fn switch(
        &mut self,
        examinee: impl Into<lir::Operand>,
        branches: Vec<(usize, LabelRef)>,
        fallback: LabelRef,
    ) {
        self.emit(lir::Instruction::Switch {
            examinee: examinee.into(),
            branches,
            fallback,
        });
    }

    pub fn branch(&mut self, cond: impl Into<lir::Operand>, accept: LabelRef, reject: LabelRef) {
        self.emit(lir::Instruction::Branch {
            cond: cond.into(),
            accept,
            reject,
        });
    }

    pub fn read(&mut self, to: lir::TypedVar, from: impl Into<lir::Operand>) {
        let from = from.into();
        self.emit(lir::Instruction::Read { to, from });
    }

    pub fn write(&mut self, to: impl Into<lir::Operand>, val: impl Into<lir::Operand>) {
        let (to, val) = (to.into(), val.into());
        self.emit(lir::Instruction::Write { to, val });
    }

    pub fn drop_var(&mut self, var: impl Into<lir::Operand>, drop: Option<DropFn>) {
        let var = var.into();
        self.emit(lir::Instruction::Drop { var, drop });
    }

    pub fn emit_clone(
        &mut self,
        to: impl Into<lir::Operand>,
        from: impl Into<lir::Operand>,
        clone: CloneFn,
    ) {
        let (to, from) = (to.into(), from.into());
        self.emit(lir::Instruction::Clone { to, from, clone });
    }

    pub fn emit_copy(
        &mut self,
        to: impl Into<lir::Operand>,
        from: impl Into<lir::Operand>,
        size: usize,
    ) {
        let (to, from) = (to.into(), from.into());
        self.emit(lir::Instruction::Copy { to, from, size });
    }

    pub fn assign(&mut self, to: lir::TypedVar, from: impl Into<lir::Operand>) {
        let from = from.into();
        self.emit(lir::Instruction::Assign { to, from });
    }
}
