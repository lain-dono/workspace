use super::runtime::{AnimaData, AnimaState, BoneData, SkinData, SlotData};
use bevy::{ecs::system::SystemParam, prelude::ResMut};

mod bone;
mod record;
mod skin;
mod slot;

//pub use self::record::{Record, RecordAction};

pub type Record = self::record::Record<Action>;

#[derive(SystemParam)]
pub struct AnimaEditorContext<'w, 's> {
    pub record: ResMut<'w, Record>,
    pub data: ResMut<'w, AnimaData>,
    pub state: ResMut<'w, AnimaState>,

    #[system_param(ignore)]
    marker: std::marker::PhantomData<&'s usize>,
}

impl<'w, 's> AnimaEditorContext<'w, 's> {
    /*
    pub fn sync(&mut self, data: &Data) {
        self.index.sync(data);
    }

    pub fn serialize(&self, data: &Data) -> ron::Result<String> {
        let config = ron::ser::PrettyConfig::new().compact_arrays(true);
        ron::Options::default().to_string_pretty(data, config)
    }
    */
}

/// undo/redo
impl<'w, 's> AnimaEditorContext<'w, 's> {
    pub fn apply(&mut self, action: Action) {
        self.record.apply(&mut self.data, action);
        self.state.sync_index(&self.data);
    }

    pub fn undo(&mut self) {
        self.record.undo(&mut self.data);
        self.state.sync_index(&self.data);
    }

    pub fn redo(&mut self) {
        self.record.redo(&mut self.data);
        self.state.sync_index(&self.data);
    }
}

pub enum Action {
    CreateBone(BoneData),
    SwapBone(usize, BoneData),

    CreateSlot(SlotData),
    SwapSlot(usize, SlotData),

    CreateSkin(SkinData),
    SwapSkin(usize, SkinData),

    TransformBone(usize, bone::BoneTransform),
}

impl self::record::Action for Action {
    type Target = AnimaData;

    fn apply(&mut self, target: &mut Self::Target) {
        match self {
            Self::CreateBone(data) => action_push(&mut target.bones, data),
            Self::CreateSlot(data) => action_push(&mut target.slots, data),
            Self::CreateSkin(data) => action_push(&mut target.skins, data),

            Self::SwapBone(index, data) => action_swap(&mut target.bones[*index], data),
            Self::SwapSlot(index, data) => action_swap(&mut target.slots[*index], data),
            Self::SwapSkin(index, data) => action_swap(&mut target.skins[*index], data),

            Self::TransformBone(bone, t) => t.swap_bone(&mut target.bones[*bone]),
        }
    }

    fn undo(&mut self, target: &mut Self::Target) {
        match self {
            Self::CreateBone(_data) => action_pop(&mut target.bones),
            Self::CreateSlot(_data) => action_pop(&mut target.slots),
            Self::CreateSkin(_data) => action_pop(&mut target.skins),

            Self::SwapBone(index, data) => action_swap(&mut target.bones[*index], data),
            Self::SwapSlot(index, data) => action_swap(&mut target.slots[*index], data),
            Self::SwapSkin(index, data) => action_swap(&mut target.skins[*index], data),

            Self::TransformBone(bone, t) => t.swap_bone(&mut target.bones[*bone]),
        }
    }
}

fn action_swap<T>(x: &mut T, y: &mut T) {
    std::mem::swap(x, y)
}

fn action_push<T: Clone>(v: &mut Vec<T>, value: &T) {
    v.push(value.clone());
}

fn action_pop<T>(v: &mut Vec<T>) {
    v.pop().unwrap();
}
