use bevy::asset::AssetMut;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;

pub struct MeshBuilder<'a> {
    pub mesh: AssetMut<'a, Mesh>,
}

impl<'a> MeshBuilder<'a> {
    #[inline]
    pub fn new(mesh: AssetMut<'a, Mesh>) -> Self {
        Self { mesh }
    }
}

impl MeshBuilder<'_> {
    #[inline]
    pub fn clear(&mut self) {
        Self::position(&mut self.mesh).clear();
        Self::indices(&mut self.mesh).clear();
        Self::normal(&mut self.mesh).clear();
        Self::color(&mut self.mesh).clear();
    }

    #[inline]
    pub fn quad(&mut self, vertices: [Vec3; 4], normal: Vec3, color: [f32; 4]) {
        self.extend(vertices, [0, 1, 2, 0, 2, 3], normal, color);
    }

    #[inline]
    pub fn triangle(
        &mut self,
        vertices: [Vec3; 3],
        indices: [u32; 3],
        normal: Vec3,
        color: [f32; 4],
    ) {
        let base = Self::position(&mut self.mesh).len() as u32;
        Self::position(&mut self.mesh).extend(vertices.map(|p| p.to_array()));
        Self::indices(&mut self.mesh).extend(indices.map(|i| base + i));
        Self::normal(&mut self.mesh).extend([normal.to_array(); 3]);
        Self::color(&mut self.mesh).extend([color; 3]);
    }

    #[inline]
    pub fn extend<V, I>(&mut self, vertices: V, indices: I, normal: Vec3, color: [f32; 4])
    where
        V: IntoIterator<Item = Vec3>,
        I: IntoIterator<Item = u32>,
        V::IntoIter: std::iter::ExactSizeIterator,
    {
        let vertices = vertices.into_iter();
        let len = std::iter::ExactSizeIterator::len(&vertices);
        let base = Self::position(&mut self.mesh).len() as u32;
        Self::position(&mut self.mesh).extend(vertices.map(|p| p.to_array()));
        Self::indices(&mut self.mesh).extend(indices.into_iter().map(|i| base + i));
        Self::normal(&mut self.mesh).extend((0..len).map(|_| normal.to_array()));
        Self::color(&mut self.mesh).extend((0..len).map(|_| color));
    }

    #[inline]
    fn position(mesh: &mut Mesh) -> &mut Vec<[f32; 3]> {
        match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn normal(mesh: &mut Mesh) -> &mut Vec<[f32; 3]> {
        match mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
            Some(VertexAttributeValues::Float32x3(v)) => v,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn color(mesh: &mut Mesh) -> &mut Vec<[f32; 4]> {
        match mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(v)) => v,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn indices(mesh: &mut Mesh) -> &mut Vec<u32> {
        match mesh.indices_mut() {
            Some(Indices::U32(v)) => v,
            _ => unreachable!(),
        }
    }
}

/*

    // pub poly: bmesh::Polygon<u32>,
    // pub vertices: Vec<(u32, [f32; 3])>,
    // pub index: Vec<u32>,
            // poly: default(),
            // vertices: default(),
            // index: default(),

#[inline]
pub fn polygon_normal(&mut self, points: &[Vec3], normal: Vec3, color: [f32; 4]) {
    let iter = points.iter().enumerate();
    let iter = iter.map(|(i, &p)| (i as u32, p.into()));
    self.vertices.clear();
    self.vertices.extend(iter);

    let normal = normal.to_array();
    self.poly.project_to(&self.vertices, normal);

    self.index.clear();
    self.poly.triangulate(&mut self.index);
    self.polygon_impl(normal, color);
}

#[inline]
pub fn polygon(&mut self, points: &[Vec3], color: [f32; 4]) {
    let iter = points.iter().enumerate();
    let iter = iter.map(|(i, &p)| (i as u32, p.into()));
    self.vertices.clear();
    self.vertices.extend(iter);

    if let Some(normal) = self.poly.project(&self.vertices, self.vertices.len()) {
        self.index.clear();
        self.poly.triangulate(&mut self.index);
        self.polygon_impl(normal, color);
    }
}

#[inline]
fn polygon_impl(&mut self, normal: [f32; 3], color: [f32; 4]) {
    let base = Self::position(self.mesh).len() as u32;
    Self::position(self.mesh).extend(self.vertices.iter().map(|&(_, p)| p));
    Self::indices(self.mesh).extend(self.index.iter().map(|i| base + i));
    Self::normal(self.mesh).extend(self.vertices.iter().map(|_| normal));
    Self::color(self.mesh).extend(self.vertices.iter().map(|_| color));
}
*/
