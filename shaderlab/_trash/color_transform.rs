pub struct ColorTransform {
    pub multiplier: [f32; 4],
    pub offset: [f32; 4],
}

impl ColorTransform {
    pub const IDENTITY: Self = Self {
        multiplier: [1.0; 4],
        offset: [0.0; 4],
    };
}

impl Default for ColorTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
