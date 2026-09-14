use crate::{label::LabelRef, mir, var::Var};

#[derive(Clone, Debug)]
pub struct Block {
    pub label: LabelRef,
    pub body: Vec<mir::Instruction>,
}

pub struct FlowBuilder {
    pub(crate) storage: Vec<Block>,
}

impl FlowBuilder {
    /// Add a new block to the blocks in the program.
    pub fn new_block(&mut self, label: LabelRef) {
        self.storage.push(Block {
            label,
            body: Vec::new(),
        });
    }

    /// Get the label of the block we are currently building
    #[must_use]
    pub fn current_label(&self) -> LabelRef {
        self.storage.last().unwrap().label
    }

    pub fn emit(&mut self, instruction: mir::Instruction) {
        self.storage.last_mut().unwrap().body.push(instruction);
    }

    pub fn jump(&mut self, label: LabelRef) {
        self.emit(mir::Instruction::Jump(label));
    }

    pub fn switch(
        &mut self,
        examinee: mir::TypedVar,
        branches: Vec<(usize, LabelRef)>,
        fallback: Option<LabelRef>,
    ) {
        self.emit(mir::Instruction::Switch {
            examinee,
            branches,
            fallback,
        });
    }

    pub fn branch(&mut self, cond: mir::TypedVar, accept: LabelRef, reject: LabelRef) {
        self.emit(mir::Instruction::Branch {
            cond: cond.0,
            accept,
            reject,
        });
    }

    pub fn assign(&mut self, to: mir::Place, ty: mir::Type, value: mir::Value) {
        self.emit(mir::Instruction::Assign(to, ty, value));
    }

    pub fn set_discriminant(
        &mut self,
        mir::TypedVar(to, ty): mir::TypedVar,
        variant: mir::EnumVariant,
    ) {
        self.emit(mir::Instruction::SetDiscriminant { to, ty, variant });
    }

    pub fn emit_drop(&mut self, val: mir::Place, ty: mir::Type) {
        self.emit(mir::Instruction::Drop(val, ty));
    }

    pub fn drop_frame<I: DoubleEndedIterator<Item = mir::TypedVar>>(
        &mut self,
        frame: impl IntoIterator<Item = mir::TypedVar, IntoIter = I>,
    ) {
        // Drop order is reversed
        for mir::TypedVar(var, ty) in frame.into_iter().rev() {
            self.emit(mir::Instruction::Drop(mir::Place::new(var, ty.clone()), ty));
        }
    }

    pub fn emit_return(&mut self, var: Var) {
        self.emit(mir::Instruction::Return(var));
    }
}
