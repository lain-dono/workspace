use super::{Curve, Lerp, Offset, Transform};
use std::collections::BTreeMap;

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClipData {
    pub name: String,
    pub bones: Vec<BoneClipData>,
}

impl ClipData {
    pub fn max_time(&self) -> u32 {
        self.bones
            .iter()
            .map(BoneClipData::max_time)
            .max()
            .unwrap_or(0)
    }

    pub fn bone_transform(&self, bone: usize, time: f32) -> Option<Transform> {
        self.bones.get(bone).map(|clip| clip.resolve(time))
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Keyframe<T> {
    pub curve: Curve,
    pub p0: T,
    pub p1: T,
    pub p2: T,
}

impl<T: Lerp> Keyframe<T> {
    fn start(&self) -> T {
        self.p0
    }
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct BoneClipData {
    pub bone: String,
    pub rotate: Timeline<f32>,
    pub translate: Timeline<Offset>,
    pub scale: Timeline<Offset>,
    pub shear: Timeline<Offset>,
}

impl BoneClipData {
    pub fn min_time(&self) -> u32 {
        let a = self.rotate.min_time().unwrap_or(0);
        let b = self.translate.min_time().unwrap_or(0);
        let c = self.scale.min_time().unwrap_or(0);
        let d = self.shear.min_time().unwrap_or(0);
        a.min(b).min(c).min(d)
    }

    pub fn max_time(&self) -> u32 {
        let a = self.rotate.max_time().unwrap_or(0);
        let b = self.translate.max_time().unwrap_or(0);
        let c = self.scale.max_time().unwrap_or(0);
        let d = self.shear.max_time().unwrap_or(0);
        a.max(b).max(c).max(d)
    }

    pub fn resolve(&self, time: f32) -> Transform {
        let rotate = self.rotate.resolve(time).unwrap_or(0.0);
        let translate = self.translate.resolve(time).unwrap_or(Offset::zero());
        let scale = self.scale.resolve(time).unwrap_or(Offset::new(1.0, 1.0));
        let shear = self.shear.resolve(time).unwrap_or(Offset::zero());

        Transform {
            rotate,
            translate: translate.into(),
            scale: scale.into(),
            shear: shear.into(),
        }
    }
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct Timeline<T> {
    frames: BTreeMap<u32, Keyframe<T>>,
}

impl<T: Lerp> Timeline<T> {
    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, u32, Keyframe<T>> {
        self.frames.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<'_, u32, Keyframe<T>> {
        self.frames.iter_mut()
    }

    pub fn time_iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.frames.keys().copied()
    }

    pub fn add(&mut self, time: u32, curve: Curve, p0: T, p1: T, p2: T) {
        self.frames.insert(time, Keyframe { curve, p0, p1, p2 });
    }

    pub fn add_step(&mut self, time: u32, p0: T) {
        self.add(time, Curve::Step, p0, T::default(), T::default());
    }

    pub fn add_linear(&mut self, time: u32, p0: T) {
        self.add(time, Curve::Linear, p0, T::default(), T::default());
    }

    pub fn add_cubic(&mut self, time: u32, p0: T, p1: T, p2: T) {
        self.add(time, Curve::Cubic, p0, p1, p2);
    }

    pub fn remove(&mut self, time: u32) -> Option<Keyframe<T>> {
        self.frames.remove(&time)
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn get(&self, time: u32) -> Option<&Keyframe<T>> {
        self.frames.get(&time)
    }

    pub fn min_time(&self) -> Option<u32> {
        self.frames.keys().next().copied()
    }

    pub fn max_time(&self) -> Option<u32> {
        self.frames.keys().last().copied()
    }

    pub fn time_min_max(&self) -> Option<(u32, u32)> {
        let mut iter = self.frames.keys();
        let min = iter.next().copied()?;
        Some((min, iter.last().copied().unwrap_or(min)))
    }

    pub fn last_value(&self) -> Option<T> {
        self.frames.values().last().map(Keyframe::start)
    }

    pub fn resolve(&self, time: f32) -> Option<T> {
        self.resolve_impl(time).or_else(|| self.last_value())
    }

    #[inline]
    fn resolve_impl(&self, time: f32) -> Option<T> {
        use std::ops::Bound::*;

        let key = time.floor() as u32;
        let (&t0, k0) = self.frames.range((Unbounded, Included(key))).last()?;
        let (&t1, k1) = self.frames.range((Excluded(key), Unbounded)).next()?;

        let (t0, t1) = (t0 as f32, t1 as f32);
        let t = (time - t0) / (t1 - t0);

        Some(k0.curve.lerp_to(t, k0.p0, k0.p1, k0.p2, k1.p0))
    }
}
