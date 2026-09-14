//! Dead code elimination on the MIR

use crate::{ice, label::LabelRef, mir};

impl mir::Mir {
    pub fn eliminate_dead_code(&mut self) {
        let mut state = Vec::new();
        for function in &mut self.functions {
            process_function(&mut state, &mut function.blocks);
            state.clear();
        }
    }
}

#[inline]
fn add(dst: &mut Vec<LabelRef>, label: LabelRef) {
    if !dst.contains(&label) {
        dst.push(label);
    }
}

fn process_function(state: &mut Vec<LabelRef>, blocks: &mut Vec<mir::Block>) {
    add(state, blocks[0].label);

    // We can't turn this into a for loop because the length of the state increases over time.
    let mut index = 0;
    while index < state.len() {
        let label = state[index];
        let block = blocks.iter_mut().find(|b| b.label == label).unwrap();

        // Truncate each block to its reachable instructions and add reachable blocks to the state.
        if let Some(index) = truncate_block(state, &block.body) {
            block.body.truncate(index + 1);
        } else {
            ice!("Malformed MIR: block ends without a terminating instruction")
        }

        index += 1;
    }

    blocks.retain(|b| state.contains(&b.label));
}

fn truncate_block(state: &mut Vec<LabelRef>, body: &[mir::Instruction]) -> Option<usize> {
    body.iter().position(|inst| {
        match inst {
            &mir::Instruction::Jump(label) => {
                add(state, label);
                true
            }
            mir::Instruction::Switch {
                branches, fallback, ..
            } => {
                // We consider each of the branches reachable
                for &(_, label) in branches {
                    add(state, label);
                }
                if let &Some(default) = fallback {
                    add(state, default);
                }
                true
            }
            &mir::Instruction::Branch { accept, reject, .. } => {
                add(state, accept);
                add(state, reject);
                true
            }
            mir::Instruction::Return(_) => true,

            mir::Instruction::Assign { .. }
            | mir::Instruction::SetDiscriminant { .. }
            | mir::Instruction::Drop { .. } => false,
        }
    })
}
