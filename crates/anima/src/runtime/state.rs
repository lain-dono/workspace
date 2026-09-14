/*
impl ArmatureData {

    pub fn add_bone(&mut self, bone: BoneData) {
        let index = self
            .bones
            .iter()
            .position(|item| item.name == bone.parent)
            .map(|i| i + 1)
            .filter(|&i| i < self.bones.len());

        if let Some(index) = index {
            self.bones.insert(index, bone)
        } else {
            self.bones.push(bone)
        }
    }
}
*/

use super::{AnimaData, Bone, BoneData, ClipData, Matrix, SkinData, Slot, SlotData};
use bevy::prelude::Resource;
use std::collections::HashMap;

pub enum PlayControl {
    First,
    Prev,
    PlayReverse,
    Pause,
    Play,
    Next,
    Last,
}

#[derive(Default, Clone, Copy)]
pub enum PlayState {
    #[default]
    Stop,
    Play,
    PlayReverse,
}

#[derive(Default, Resource)]
pub struct AnimaState {
    pub bones: Vec<Bone>,
    pub slots: Vec<Slot>,

    pub index: DataIndex,

    pub current_time: u32,
    pub current_clip: u32,
    pub state: PlayState,

    pub local_to_world: Vec<Matrix>,
    pub local_to_screen: Vec<Matrix>,
}

impl AnimaState {
    pub fn is_playing(&self) -> bool {
        matches!(self.state, PlayState::Play | PlayState::PlayReverse)
    }

    pub fn action(&mut self, control: PlayControl, max_time: u32) {
        match control {
            PlayControl::First => self.current_time = 0,
            PlayControl::Prev if self.current_time > 0 => self.current_time -= 1,
            PlayControl::Prev => self.current_time = max_time,

            PlayControl::PlayReverse => self.state = PlayState::PlayReverse,
            PlayControl::Play => self.state = PlayState::Play,
            PlayControl::Pause => self.state = PlayState::Stop,

            PlayControl::Next if self.current_time < max_time => self.current_time += 1,
            PlayControl::Next => self.current_time = 0,
            PlayControl::Last => self.current_time = max_time,
        }
    }

    pub fn world_to_screen(&mut self, screen: Matrix) {
        let iter = self
            .local_to_world
            .iter()
            .map(|&world| screen.prepend(world));

        self.local_to_screen.clear();
        self.local_to_screen.extend(iter);
    }

    pub fn sync_index(&mut self, data: &AnimaData) {
        self.index.sync(data);
    }

    pub fn sync_data(&mut self, data: &AnimaData) {
        self.bones.clear();
        for bone in &data.bones {
            self.bones.push(Bone {
                parent: self.index.bone_u32(&bone.parent),
                color: bone.color,
                length: bone.length,
                transform: bone.transform(),
            })
        }
    }

    pub fn sync(&mut self, data: &AnimaData) {
        self.sync_index(data);
        self.sync_data(data);

        self.local_to_world.clear();

        let time = self.current_time as f32;
        let clip = data.clips.get(self.current_clip as usize);

        for (index, bone) in self.bones.iter().enumerate() {
            let clip = clip.and_then(|clip| clip.bone_transform(index, time));

            let transform = clip.map_or(bone.transform, |clip| bone.transform.mul_transform(clip));
            let transform = transform.to_matrix();

            let parent = bone.parent as usize;
            let parent = *self.local_to_world.get(parent).unwrap_or(&Matrix::IDENTITY);

            self.local_to_world.push(parent.prepend(transform));
        }
    }
}

#[derive(Default, Debug)]
pub struct DataIndex {
    pub bones: HashMap<String, usize>,
    pub slots: HashMap<String, usize>,
    pub skins: HashMap<String, usize>,
    pub clips: HashMap<String, usize>,
}

macro_rules! impl_finder {
    ($idx_fn:ident, $idx_fn_or_max:ident, $ref_fn:ident, $mut_fn:ident => $field:ident : $ty:ident) => {
        pub fn $idx_fn(&self, name: &str) -> Option<usize> {
            self.$field.get(name).copied()
        }

        pub fn $idx_fn_or_max(&self, name: &str) -> u32 {
            self.$field.get(name).map_or(u32::MAX, |&i| i as u32)
        }

        pub fn $ref_fn<'a>(&self, data: &'a AnimaData, name: &str) -> Option<&'a $ty> {
            self.$idx_fn(name).and_then(|i| data.$field.get(i))
        }

        pub fn $mut_fn<'a>(&self, data: &'a mut AnimaData, name: &str) -> Option<&'a mut $ty> {
            self.$idx_fn(name).and_then(|i| data.$field.get_mut(i))
        }
    };
}

impl DataIndex {
    impl_finder!(bone, bone_u32, bone_ref, bone_mut => bones: BoneData);
    impl_finder!(slot, slot_u32, slot_ref, slot_mut => slots: SlotData);
    impl_finder!(skin, skin_u32, skin_ref, skin_mut => skins: SkinData);
    impl_finder!(clip, clip_u32, clip_ref, clip_mut => clips: ClipData);

    pub fn sync(&mut self, data: &AnimaData) {
        fn names<'a, T: 'a>(
            iter: impl IntoIterator<Item = &'a T> + 'a,
            map: impl Fn(&'a T) -> &str + 'a,
        ) -> impl Iterator<Item = (String, usize)> + 'a {
            let iter = iter.into_iter().map(map).enumerate();
            iter.map(|(index, name)| (name.to_string(), index))
        }

        self.bones.clear();
        self.slots.clear();
        self.skins.clear();
        self.clips.clear();

        self.bones.extend(names(&data.bones, |v| &v.name));
        self.slots.extend(names(&data.slots, |v| &v.name));
        self.skins.extend(names(&data.skins, |v| &v.name));
        self.clips.extend(names(&data.clips, |v| &v.name));
    }
}

/*
impl AnimaState {
    pub fn insert_bone(&mut self, index: usize, bone: Bone) {
        for bone in &mut self.bones {
            if bone.parent >= index as u32 {
                bone.parent = bone.parent.saturating_add(1);
            }
        }
        self.bones.insert(index, bone);
    }

    pub fn add_bone(&mut self, bone: Bone) {
        self.bones.push(bone);
    }
}
*/
