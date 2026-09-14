use bevy::{prelude::*, render::render_asset::RenderAssetUsages};

#[derive(Default)]
pub struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
}

impl MeshBuilder {
    pub fn add(&mut self, position: [[f32; 3]; 3], normal: [[f32; 3]; 3], color: [[f32; 4]; 3]) {
        self.positions.extend_from_slice(&position);
        self.normals.extend_from_slice(&normal);
        self.colors.extend_from_slice(&color);
    }

    pub fn build(self) -> Mesh {
        Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
    }
}
