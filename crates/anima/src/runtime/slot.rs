use super::{util, Color};

pub struct Slot {
    pub name: String,
    pub bone: u32,
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SlotData {
    pub name: String,
    pub bone: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub attachment: String,

    #[serde(default, skip_serializing_if = "util::is_default")]
    pub blend: Blend,

    #[serde(default, skip_serializing_if = "util::is_default")]
    pub color: Color,

    #[serde(default, skip_serializing_if = "util::is_default")]
    pub dark: Option<Color>,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Blend {
    #[default]
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "additive")]
    Additive,
    #[serde(rename = "multiply")]
    Multiply,
    #[serde(rename = "screen")]
    Screen,
}
