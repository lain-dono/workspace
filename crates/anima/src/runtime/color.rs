#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Color(pub u32);

impl Default for Color {
    fn default() -> Self {
        Self(0xFFFFFFFF)
    }
}

impl Color {
    pub fn from_str_rgba(src: &str) -> Result<Self, std::num::ParseIntError> {
        u32::from_str_radix(src, 16).map(Self)
    }
}
