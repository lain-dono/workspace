use super::super::runtime::SkinData;
use super::{Action, AnimaEditorContext};

/// Skin actions
impl<'w, 's> AnimaEditorContext<'w, 's> {
    pub fn apply_create_skin(&mut self, name: impl ToString) {
        let skin = SkinData {
            name: name.to_string(),
            ..Default::default()
        };
        self.apply(Action::CreateSkin(skin));
    }
}
