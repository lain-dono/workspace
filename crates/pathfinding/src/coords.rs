pub use crate::landmass::CoordinateSystem as LandmassCoordinateSystem;
use crate::landmass::PointSampleDistance3d;
use bevy::math::{EulerRot, Quat, Vec2, Vec3, Vec3Swizzles};
use bevy::reflect::TypePath;

/// A [`CoordinateSystem`] compatible with `bevy_landmass`.
pub trait CoordinateSystem:
    LandmassCoordinateSystem<Coord: Default + Send + Sync + PartialEq, SampleDistance: Send + Sync>
    + TypePath
    + Send
    + Sync
{
    /// Converts a vertex from a mesh into this system's coordinate.
    fn from_mesh_vertex(v: [f32; 3]) -> Self::Coord;

    /// Converts a position in Bevy into this system's coordinate.
    fn from_bevy_position(v: Vec3) -> Self::Coord;

    /// Converts a [`Quat`] into the corresponding rotation (in
    /// radians) in this system.
    fn from_bevy_rotation(rotation: Quat) -> f32;

    /// Converts this system's coordinate into a world position.
    fn to_world_position(c: Self::Coord) -> Vec3;
}

/// A 3D coordinate system where X is right, Y is up, and -Z is forward.
#[derive(TypePath, Clone, Copy, Default)]
pub struct ThreeD;

impl LandmassCoordinateSystem for ThreeD {
    type Coord = Vec3;
    type SampleDistance = PointSampleDistance3d;

    fn to_landmass(v: Self::Coord) -> Vec3 {
        Vec3::new(v.x, -v.z, v.y)
    }

    fn from_landmass(v: Vec3) -> Self::Coord {
        Vec3::new(v.x, v.z, -v.y)
    }
}

impl CoordinateSystem for ThreeD {
    fn from_mesh_vertex([x, y, z]: [f32; 3]) -> Self::Coord {
        Vec3::new(x, y, z)
    }

    fn from_bevy_position(v: Vec3) -> Self::Coord {
        v
    }

    fn from_bevy_rotation(rotation: Quat) -> f32 {
        rotation.to_euler(EulerRot::YXZ).0
    }

    fn to_world_position(v: Self::Coord) -> Vec3 {
        v
    }
}

/// A 2D coordinate system, where XY form the 2D plane.
#[derive(TypePath, Clone, Copy, Default)]
pub struct TwoD;

impl LandmassCoordinateSystem for TwoD {
    type Coord = Vec2;
    type SampleDistance = f32;

    fn to_landmass(v: Self::Coord) -> Vec3 {
        Vec3::new(v.x, v.y, 0.0)
    }

    fn from_landmass(v: Vec3) -> Self::Coord {
        Vec2::new(v.x, v.y)
    }
}

impl CoordinateSystem for TwoD {
    fn from_mesh_vertex([x, y, _]: [f32; 3]) -> Self::Coord {
        Vec2::new(x, y)
    }

    fn from_bevy_position(v: Vec3) -> Self::Coord {
        v.xy()
    }

    fn from_bevy_rotation(rotation: Quat) -> f32 {
        rotation.to_euler(EulerRot::ZXY).0
    }

    fn to_world_position(c: Self::Coord) -> Vec3 {
        c.extend(0.0)
    }
}
