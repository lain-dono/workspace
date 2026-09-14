use bevy::ecs::component::Component;

#[derive(Component, Clone, Copy, Default)]
pub struct Score {
    value: f32,
}

impl From<Score> for f32 {
    fn from(Score { value }: Score) -> Self {
        value
    }
}

impl From<&Score> for f32 {
    fn from(&Score { value }: &Score) -> Self {
        value
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), std::cmp::Ordering::Equal)
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.total_cmp(&other.value)
    }
}

impl std::fmt::Debug for Score {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Score").field(&self.value).finish()
    }
}

impl Score {
    pub const MIN: Self = Self { value: 0.0 };
    pub const MAX: Self = Self { value: 1.0 };

    #[must_use]
    pub const fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
        }
    }

    #[must_use]
    pub fn get(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        self.value = value.clamp(0.0, 1.0);
    }
}
