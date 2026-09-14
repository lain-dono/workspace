use crate::landmass::CoordinateSystem;
use bevy::math::{
    Quat, Vec3, Vec3A,
    bounding::{Aabb3d, BoundingVolume, IntersectsVolume},
};

/// A bounding box.
#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub struct BoundingBox(Option<Aabb3d>);

impl BoundingBox {
    #[cfg(test)]
    #[must_use]
    pub const fn empty() -> Self {
        Self(None)
    }

    /// Creates a box already with some data in it.
    ///
    /// `min` and `max` must already be valid - this is unchecked.
    pub fn new(min: impl Into<Vec3A>, max: impl Into<Vec3A>) -> Self {
        let (min, max) = (min.into(), max.into());
        Self(Some(Aabb3d { min, max }))
    }

    /// Creates a box already with some data in it.
    ///
    /// `min` and `max` must already be valid - this is unchecked.
    #[must_use]
    pub const fn from_aabb(aabb: Aabb3d) -> Self {
        Self(Some(aabb))
    }

    #[must_use]
    pub fn unwrap_inner(self) -> Aabb3d {
        self.0.unwrap()
    }

    /// Computes the smallest [`BoundingBox`] containing the given set of points.
    pub fn from_point_cloud(points: impl IntoIterator<Item = impl Into<Vec3A>>) -> Self {
        Self(Self::new_aabb(points))
    }

    pub fn new_aabb(points: impl IntoIterator<Item = impl Into<Vec3A>>) -> Option<Aabb3d> {
        let mut iter = points.into_iter().map(Into::into);
        iter.next().map(|first| {
            let (min, max) = iter.fold((first, first), |(min, max), point| {
                (point.min(min), point.max(max))
            });
            Aabb3d { min, max }
        })
    }

    /// Computes the smallest [`BoundingBox`] containing the given set of [`BoundingBox`].
    pub fn from_cloud(cloud: impl IntoIterator<Item = Self>) -> Self {
        let mut iter = cloud.into_iter();
        iter.next().map_or(Self(None), |first| {
            iter.fold(first, |acc, next| acc.union(&next))
        })
    }

    /// Returns whether the box is empty or not.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self(None))
    }

    /// Returns the bounds of the box, assuming it is non-empty.
    #[cfg(test)]
    #[must_use]
    pub fn to_min_max(self) -> (Vec3, Vec3) {
        if let Self(Some(Aabb3d { min, max })) = self {
            (min.into(), max.into())
        } else {
            panic!("BoundingBox is not a box.")
        }
    }

    #[must_use]
    pub fn center(&self) -> Option<Vec3> {
        self.0.map(|bbox| bbox.center().into())
    }

    /// Computes the size of the bounding box. Returns 0 if the bounds are empty.
    #[must_use]
    pub fn size(&self) -> Vec3 {
        self.0
            .map_or(Vec3::ZERO, |bbox| (bbox.max - bbox.min).into())
    }

    /// Expands the bounding box to contain the `other`.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        match (self.0, other.0) {
            (None, None) => Self(None),
            (this, None) => Self(this),
            (None, other) => Self(other),
            (Some(this), Some(other)) => Self(Some(this.merge(&other))),
        }
    }

    /// Expands the bounding box by `size`.
    ///
    /// An empty bounding box will still be empty after this.
    #[must_use]
    pub fn expand_by_size(&self, size: Vec3) -> Self {
        self.add_to_corners(-size, size)
    }

    /// Adds `delta_min` to the min corner, and `delta_max` to the max corner. An
    /// empty bounding box will still be empty after this.
    #[must_use]
    pub fn add_to_corners(&self, delta_min: impl Into<Vec3A>, delta_max: impl Into<Vec3A>) -> Self {
        let bbox = match self {
            Self(None) => return Self(None),
            &Self(Some(Aabb3d { min, max })) => Aabb3d {
                min: min + delta_min.into(),
                max: max + delta_max.into(),
            },
        };

        let size = bbox.max - bbox.min;
        let is_valid = size.x >= 0.0 && size.y >= 0.0 && size.z >= 0.0;

        Self(is_valid.then_some(bbox))
    }

    /// Determines if `point` is in `self`.
    #[must_use]
    pub fn contains_point(&self, point: Vec3) -> bool {
        self.0.is_some_and(|Aabb3d { min, max }| {
            let x = min.x <= point.x && point.x <= max.x;
            let y = min.y <= point.y && point.y <= max.y;
            let z = min.z <= point.z && point.z <= max.z;
            x && y && z
        })
    }

    /// Determines if `other` is fully contained by `self`.
    #[cfg(test)] // Used by tests.
    #[must_use]
    pub fn contains_bounds(&self, other: &Self) -> bool {
        match (self, other) {
            (Self(None), _) | (_, Self(None)) => false,
            (Self(Some(this)), Self(Some(other))) => this.contains(other),
        }
    }

    /// Detemrines if `other` intersects `self` at all.
    #[must_use]
    pub fn intersects_bounds(&self, other: &Self) -> bool {
        match (self, other) {
            (Self(None), _) | (_, Self(None)) => false,
            (Self(Some(this)), Self(Some(other))) => this.intersects(other),
        }
    }

    /// Creates a conservative bounding box around `self` after transforming it by `transform`.
    #[must_use]
    pub fn transform<T: CoordinateSystem>(&self, transform: &Transform<T>) -> Self {
        Self(self.0.map(|bbox| {
            let translation = T::to_landmass(transform.translation);
            let rotation = Quat::from_rotation_z(transform.rotation);
            bbox.transformed_by(translation, rotation)
        }))
    }
}

/// A transform that can be applied to Vec3's.
#[derive(Clone, Copy)]
pub struct Transform<T: CoordinateSystem> {
    /// The translation to apply.
    pub translation: T::Coord,
    /// The rotation to apply around the "up" direction.
    ///
    /// Specifically, the up direction is perpendicular to the plane of movement.
    pub rotation: f32,
}

impl<T: CoordinateSystem> Transform<T> {
    pub const fn new(translation: T::Coord, rotation: f32) -> Self {
        Self {
            translation,
            rotation,
        }
    }
}

// Manual Debug impl to avoid `T` having a Debug bound itself.
impl<T: CoordinateSystem<Coord: std::fmt::Debug>> std::fmt::Debug for Transform<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Transform")
            .field("translation", &self.translation)
            .field("rotation", &self.rotation)
            .finish()
    }
}

// Manual Default impl to avoid `T` having a Default bound itself.
impl<T: CoordinateSystem<Coord: Default>> Default for Transform<T> {
    fn default() -> Self {
        Self {
            translation: T::Coord::default(),
            rotation: 0.0,
        }
    }
}

// Manual PartialEq impl to avoid `T` having a PartialEq bound itself.
impl<T: CoordinateSystem<Coord: PartialEq>> PartialEq for Transform<T> {
    fn eq(&self, other: &Self) -> bool {
        self.translation == other.translation && self.rotation == other.rotation
    }
}

impl<T: CoordinateSystem> Transform<T> {
    /// Applies the transformation.
    pub(crate) fn apply(&self, point: Vec3) -> Vec3 {
        Quat::from_rotation_z(self.rotation) * point + T::to_landmass(self.translation)
    }

    /// Inverses the transformation.
    pub(crate) fn apply_inverse(&self, point: Vec3) -> Vec3 {
        Quat::from_rotation_z(-self.rotation) * (point - T::to_landmass(self.translation))
    }
}

pub struct BoundingBoxHierarchy<T> {
    nodes: Vec<Option<Node<T>>>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Node<T> {
    Leaf(BoundingBox, T),
    Branch(BoundingBox),
}

impl<T: Copy> BoundingBoxHierarchy<T> {
    /// Creates a hierarchy from values and their bounding boxes.
    ///
    /// The values are all expected to be Some, and the values will be moved into the hierarchy
    /// (leaving behind None).
    #[must_use]
    pub fn new(values: &mut [(BoundingBox, T)]) -> Self {
        let len = values.len().next_power_of_two() * 2 - 1;
        let tree = vec![None; len];
        let mut tree = Self { nodes: tree };
        tree.build_recursive(0, values);
        tree
    }

    fn build_recursive(&mut self, index: usize, values: &mut [(BoundingBox, T)]) {
        assert!(!values.is_empty());

        if values.len() == 1 {
            let (bounds, value) = values[0];
            self.nodes[index] = Some(Node::Leaf(bounds, value));
            return;
        }

        let bounds = BoundingBox::from_cloud(values.iter().map(|&(bbox, _)| bbox));

        let size = bounds.size();
        if size.x > size.y && size.x > size.z {
            values.sort_by(|(a, _), (b, _)| {
                f32::total_cmp(&a.center().unwrap().x, &b.center().unwrap().x)
            });
        } else if size.y > size.z {
            values.sort_by(|(a, _), (b, _)| {
                f32::total_cmp(&a.center().unwrap().y, &b.center().unwrap().y)
            });
        } else {
            values.sort_by(|(a, _), (b, _)| {
                f32::total_cmp(&a.center().unwrap().z, &b.center().unwrap().z)
            });
        }

        let (prev, next) = values.split_at_mut(values.len() / 2);

        self.nodes[index] = Some(Node::Branch(bounds));
        self.build_recursive(index * 2 + 1, prev);
        self.build_recursive(index * 2 + 2, next);
    }

    #[must_use]
    pub fn depth(&self) -> u32 {
        usize::BITS - (self.nodes.len()).leading_zeros()
    }

    #[must_use]
    pub fn query_box(&self, min: Vec3, max: Vec3) -> Vec<T> {
        self.query(BoundingBox::new(min, max))
    }

    #[must_use]
    pub fn query(&self, query: BoundingBox) -> Vec<T> {
        let mut result = Vec::new();
        if !query.is_empty() {
            self.query_recursive(&query, 0, &mut result);
        }
        result
    }

    fn query_recursive(&self, query: &BoundingBox, index: usize, result: &mut Vec<T>) {
        match self.nodes[index].unwrap() {
            Node::Leaf(bounds, value) => {
                if query.intersects_bounds(&bounds) {
                    result.push(value);
                }
            }
            Node::Branch(bounds) => {
                if query.intersects_bounds(&bounds) {
                    self.query_recursive(query, index * 2 + 1, result);
                    self.query_recursive(query, index * 2 + 2, result);
                }
            }
        }
    }
}
