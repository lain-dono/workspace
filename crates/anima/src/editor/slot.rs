use super::super::runtime::SlotData;
use super::{Action, AnimaEditorContext};

/// Slot actions
impl<'w, 's> AnimaEditorContext<'w, 's> {
    pub fn apply_create_slot(&mut self, name: impl ToString, bone: impl ToString) {
        let slot = SlotData {
            name: name.to_string(),
            bone: bone.to_string(),
            ..Default::default()
        };
        self.apply(Action::CreateSlot(slot));
    }
}
