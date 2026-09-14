use std::collections::HashMap;

#[derive(Default)]
pub struct MixState {
    anim_to_mix: HashMap<[String; 2], f32>,
    default_mix: f32,
}

impl MixState {
    pub fn new(default_mix: f32) -> Self {
        Self {
            anim_to_mix: HashMap::default(),
            default_mix,
        }
    }

    pub fn set(&mut self, from: impl Into<String>, to: impl Into<String>, duration: f32) {
        let key = [from.into(), to.into()];
        self.anim_to_mix.insert(key, duration);
    }

    pub fn get(&mut self, from: impl Into<String>, to: impl Into<String>) -> f32 {
        let key = [from.into(), to.into()];
        *self.anim_to_mix.get(&key).unwrap_or(&self.default_mix)
    }
}
