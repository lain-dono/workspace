//! A 3D transformation Gizmo for the Bevy game engine.
//!
//! transform-gizmo-bevy provides a feature-rich and configurable 3D transformation
//! gizmo that can be used to manipulate entities' transforms (position, rotation, scale)
//! visually.
//!
//! # Usage
//!
//! Add `TransformGizmoPlugin` to your App.
//!
//! ```ignore
//! use bevy::prelude::*;
//! use transform_gizmo_bevy::prelude::*;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugins(TransformGizmoPlugin)
//!     .run();
//! ```
//!
//! Add [`GizmoCamera`] component to your Camera entity.
//!
//! Add [`GizmoTarget`] component to any of your entities that you would like to manipulate the [`Transform`] of.
//!
//! # Configuration
//!
//! You can configure the gizmo by modifying the [`GizmoOptions`] resource.
//!
//! You can either set it up with [`App::insert_resource`] when creating your App, or at any point in a system with [`ResMut<GizmoOptions>`].

pub mod picking;
pub mod plugin;
pub mod render;

pub mod config;
pub mod gizmo;
pub mod math;

pub use self::config::{GizmoConfig, GizmoDirection, GizmoMode, GizmoOrientation, GizmoVisuals};
pub use self::draw::DrawData;
pub use self::gizmo::{Gizmo, GizmoInteraction, GizmoResult};
pub use ecolor::Color32;
pub use emath::Rect;
pub use enumset::{enum_set, EnumSet};
pub use mint;
use prepared::GizmoTransform;

pub mod rotation;

pub mod draw;
pub mod prepared;

pub mod axis;
pub mod circle;
pub mod plane;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Transform {
    pub scale: mint::Vector3<f64>,
    pub rotation: mint::Quaternion<f64>,
    pub translation: mint::Vector3<f64>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: mint::Vector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            rotation: mint::Quaternion {
                v: mint::Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                s: 0.0,
            },
            translation: mint::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        }
    }
}

impl Transform {
    pub fn from_scale_rotation_translation(
        scale: impl Into<mint::Vector3<f64>>,
        rotation: impl Into<mint::Quaternion<f64>>,
        translation: impl Into<mint::Vector3<f64>>,
    ) -> Self {
        Self {
            scale: scale.into(),
            rotation: rotation.into(),
            translation: translation.into(),
        }
    }

    fn as_gizmo(&self) -> GizmoTransform {
        GizmoTransform {
            scale: self.scale.into(),
            rotation: self.rotation.into(),
            translation: self.translation.into(),
        }
    }
}
