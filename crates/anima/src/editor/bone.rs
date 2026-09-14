use super::{Action, AnimaEditorContext};
use crate::runtime::BoneData;

// TODO: name, len, parent, inherit, color

/// Bone actions
impl<'w, 's> AnimaEditorContext<'w, 's> {
    pub fn apply_create_bone(&mut self, name: impl ToString, parent: impl ToString) {
        let bone = BoneData {
            name: name.to_string(),
            parent: parent.to_string(),
            ..Default::default()
        };
        self.apply(Action::CreateBone(bone));
    }

    pub fn apply_bone_transform(&mut self, index: usize, transform: BoneTransform) {
        self.apply(Action::TransformBone(index, transform));
    }

    pub fn apply_bone_length(&mut self, bone: &str, len: f32) {
        let index = self.state.index.bone(bone).unwrap();
        let curr = BoneTransform::from_bone(&self.data.bones[index]);
        self.apply_bone_transform(index, BoneTransform { len, ..curr });
    }

    pub fn apply_bone_translate(&mut self, bone: &str, tx: f32, ty: f32) {
        let index = self.state.index.bone(bone).unwrap();
        let curr = BoneTransform::from_bone(&self.data.bones[index]);
        self.apply_bone_transform(index, BoneTransform { tx, ty, ..curr });
    }

    pub fn apply_bone_rotate(&mut self, bone: &str, rot: f32) {
        let index = self.state.index.bone(bone).unwrap();
        let curr = BoneTransform::from_bone(&self.data.bones[index]);
        self.apply_bone_transform(index, BoneTransform { rot, ..curr });
    }

    pub fn apply_bone_scale(&mut self, bone: &str, sx: f32, sy: f32) {
        let index = self.state.index.bone(bone).unwrap();
        let curr = BoneTransform::from_bone(&self.data.bones[index]);
        self.apply_bone_transform(index, BoneTransform { sx, sy, ..curr });
    }

    pub fn apply_bone_shear(&mut self, bone: &str, shx: f32, shy: f32) {
        let index = self.state.index.bone(bone).unwrap();
        let curr = BoneTransform::from_bone(&self.data.bones[index]);
        self.apply_bone_transform(index, BoneTransform { shx, shy, ..curr });
    }
}

pub struct BoneTransform {
    pub len: f32,
    pub rot: f32,

    pub tx: f32,
    pub ty: f32,

    pub sx: f32,
    pub sy: f32,

    pub shx: f32,
    pub shy: f32,
}

impl BoneTransform {
    pub fn from_bone(bone: &BoneData) -> Self {
        Self {
            len: bone.length,
            rot: bone.rotate,

            tx: bone.translate[0],
            ty: bone.translate[1],

            sx: bone.scale[0],
            sy: bone.scale[1],

            shx: bone.shear[0],
            shy: bone.shear[1],
        }
    }

    pub fn swap_bone(&mut self, bone: &mut BoneData) {
        std::mem::swap(&mut self.len, &mut bone.length);
        std::mem::swap(&mut self.rot, &mut bone.rotate);

        std::mem::swap(&mut self.tx, &mut bone.translate[0]);
        std::mem::swap(&mut self.ty, &mut bone.translate[1]);

        std::mem::swap(&mut self.sx, &mut bone.scale[0]);
        std::mem::swap(&mut self.sy, &mut bone.scale[1]);

        std::mem::swap(&mut self.shx, &mut bone.shear[0]);
        std::mem::swap(&mut self.shy, &mut bone.shear[1]);
    }
}
