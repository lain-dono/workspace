use super::{util, Color};
use std::collections::HashMap;

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SkinData {
    pub name: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bones: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ik: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transform: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attachments: HashMap<String, HashMap<String, Attachment>>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Attachment {
    Alias(String),

    Point {
        #[serde(default, skip_serializing_if = "util::is_zero_f32")]
        x: f32,
        #[serde(default, skip_serializing_if = "util::is_zero_f32")]
        y: f32,
        #[serde(default, skip_serializing_if = "util::is_zero_f32")]
        rot: f32,
        #[serde(default, skip_serializing_if = "util::is_default")]
        color: Color,
    },

    Region {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        path: String,

        #[serde(default, skip_serializing_if = "util::is_zero_f32x2")]
        min: [f32; 2],
        #[serde(default, skip_serializing_if = "util::is_zero_f32x2")]
        max: [f32; 2],
        #[serde(default, skip_serializing_if = "util::is_one_f32x2")]
        scale: [f32; 2],
        #[serde(default, skip_serializing_if = "util::is_zero_f32")]
        rotation: f32,
        #[serde(default, skip_serializing_if = "util::is_default")]
        tint: Color,
    },

    Mesh {
        #[serde(default, skip_serializing_if = "String::is_empty")]
        path: String,

        vtx: (Vec<SimpleVertex>, Vec<Vec<WeightedVertex>>),

        idx: Vec<u16>,
        edges: Vec<[u16; 2]>,
        uvs: Vec<[f32; 2]>,
        #[serde(default, skip_serializing_if = "util::is_default")]
        hull: usize,

        #[serde(default, skip_serializing_if = "util::is_default")]
        size: [f32; 2],
        #[serde(default, skip_serializing_if = "util::is_default")]
        color: Color,
    },

    Polygon {
        vtx: (Vec<SimpleVertex>, Vec<Vec<WeightedVertex>>),

        #[serde(default, skip_serializing_if = "Option::is_none")]
        clipping_end: Option<String>,
        #[serde(default, skip_serializing_if = "util::is_default")]
        color: Color,
    },
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, from = "[f32; 2]", into = "[f32; 2]")]
pub struct SimpleVertex {
    pub x: f32,
    pub y: f32,
}

impl From<[f32; 2]> for SimpleVertex {
    fn from([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }
}

impl From<SimpleVertex> for [f32; 2] {
    fn from(v: SimpleVertex) -> Self {
        [v.x, v.y]
    }
}

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, from = "(u32, f32, f32, f32)", into = "(u32, f32, f32, f32)")]
pub struct WeightedVertex {
    pub bone: u32,
    pub weight: f32,
    pub x: f32,
    pub y: f32,
}

impl From<(u32, f32, f32, f32)> for WeightedVertex {
    fn from((bone, weight, x, y): (u32, f32, f32, f32)) -> Self {
        Self { bone, weight, x, y }
    }
}

impl From<WeightedVertex> for (u32, f32, f32, f32) {
    fn from(v: WeightedVertex) -> Self {
        (v.bone, v.weight, v.x, v.y)
    }
}
