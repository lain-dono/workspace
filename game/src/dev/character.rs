use bevy::mesh::{Indices, PrimitiveTopology, triangle_normal};
use bevy::prelude::*;

pub fn character_mesh() -> Mesh {
    let height = 1.7;
    let dim_x = 0.2;
    let dim_z = 0.2;

    Builder::default()
        .cuboid(
            Vec3::new(-dim_x, 0.0, -dim_z),
            Vec3::new(dim_x, height, dim_z),
            Srgba::BLACK,
        )
        .cuboid(
            Vec3::new(-dim_x + 0.10, height - 0.25, dim_z),
            Vec3::new(dim_x - 0.10, height - 0.05, dim_z + 0.10),
            Srgba::RED,
        )
        .build()
}

type Vertex = ([f32; 3], [f32; 3], [f32; 2], Srgba);

#[derive(Default)]
struct Builder {
    vtx: Vec<Vertex>,
    idx: Vec<u32>,
}

impl Builder {
    fn cuboid(mut self, min: Vec3, max: Vec3, color: Srgba) -> Self {
        // Suppose Y-up right hand, and camera look from +Z to -Z

        let front = [
            ([min.x, min.y, max.z], [0.0, 0.0]),
            ([max.x, min.y, max.z], [1.0, 0.0]),
            ([max.x, max.y, max.z], [1.0, 1.0]),
            ([min.x, max.y, max.z], [0.0, 1.0]),
        ];
        let back = [
            ([min.x, max.y, min.z], [1.0, 0.0]),
            ([max.x, max.y, min.z], [0.0, 0.0]),
            ([max.x, min.y, min.z], [0.0, 1.0]),
            ([min.x, min.y, min.z], [1.0, 1.0]),
        ];
        let right = [
            ([max.x, min.y, min.z], [0.0, 0.0]),
            ([max.x, max.y, min.z], [1.0, 0.0]),
            ([max.x, max.y, max.z], [1.0, 1.0]),
            ([max.x, min.y, max.z], [0.0, 1.0]),
        ];
        let left = [
            ([min.x, min.y, max.z], [1.0, 0.0]),
            ([min.x, max.y, max.z], [0.0, 0.0]),
            ([min.x, max.y, min.z], [0.0, 1.0]),
            ([min.x, min.y, min.z], [1.0, 1.0]),
        ];
        let top = [
            ([max.x, max.y, min.z], [1.0, 0.0]),
            ([min.x, max.y, min.z], [0.0, 0.0]),
            ([min.x, max.y, max.z], [0.0, 1.0]),
            ([max.x, max.y, max.z], [1.0, 1.0]),
        ];
        let bottom = [
            ([max.x, min.y, max.z], [0.0, 0.0]),
            ([min.x, min.y, max.z], [1.0, 0.0]),
            ([min.x, min.y, min.z], [1.0, 1.0]),
            ([max.x, min.y, min.z], [0.0, 1.0]),
        ];

        self.quad(front, color);
        self.quad(back, color);
        self.quad(right, color);
        self.quad(left, color);
        self.quad(top, color);
        self.quad(bottom, color);

        self
    }

    fn quad(&mut self, vtx: [([f32; 3], [f32; 2]); 4], color: Srgba) {
        let na = triangle_normal(vtx[0].0, vtx[1].0, vtx[2].0);
        let nb = triangle_normal(vtx[2].0, vtx[3].0, vtx[0].0);

        let normal = Vec3::midpoint(Vec3::from(na), Vec3::from(nb)).to_array();
        let vtx = vtx.map(|(p, uv)| (p, normal, uv, color));

        let base = self.vtx.len() as u32;
        let idx = [0, 1, 2, 2, 3, 0].map(|index| base + index);

        self.vtx.extend_from_slice(&vtx);
        self.idx.extend_from_slice(&idx);
    }

    fn build(self) -> Mesh {
        let positions: Vec<_> = self.vtx.iter().map(|v| v.0).collect();
        let normals: Vec<_> = self.vtx.iter().map(|v| v.1).collect();
        let uvs: Vec<_> = self.vtx.iter().map(|v| v.2).collect();
        let color: Vec<_> = self.vtx.iter().map(|v| v.3.to_f32_array()).collect();

        Mesh::new(PrimitiveTopology::TriangleList, default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, color)
            .with_inserted_indices(Indices::U32(self.idx))
    }
}
