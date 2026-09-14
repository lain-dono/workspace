use super::{EdgeKey, GraphData, KeyData, VertKey};

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FaceKey(KeyData);

impl From<KeyData> for FaceKey {
    fn from(value: KeyData) -> Self {
        Self(value)
    }
}

impl From<FaceKey> for KeyData {
    fn from(FaceKey(key): FaceKey) -> Self {
        key
    }
}

pub struct FaceSlot<T: GraphData> {
    pub loop_first: Option<LoopKey>,
    pub len: usize,
    pub data: T::Face,
}

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct LoopKey(KeyData);

impl From<KeyData> for LoopKey {
    fn from(value: KeyData) -> Self {
        Self(value)
    }
}

impl From<LoopKey> for KeyData {
    fn from(LoopKey(key): LoopKey) -> Self {
        key
    }
}

pub struct LoopSlot<T: GraphData> {
    pub vert: VertKey,
    pub edge: Option<EdgeKey>,
    pub face: FaceKey,

    pub radial: Option<RadialLoop>,

    pub next: LoopKey,
    pub prev: LoopKey,

    pub data: T::Loop,
}

#[derive(Clone)]
pub struct RadialLoop {
    pub next: LoopKey,
    pub prev: LoopKey,
}
