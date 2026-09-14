#![allow(clippy::type_complexity)]
// #![deny(missing_docs)] -- This would be great! But we are far away.
//! An integration to render SVG and Lottie assets in Bevy with Vello.

use crate::prelude::*;

mod plugin;
pub use plugin::VelloPlugin;

#[cfg(feature = "picking")]
mod picking;

pub mod render;
pub mod scene;
pub mod text;

mod error;
pub use self::error::VectorLoaderError;

// Re-exports
pub use ::parley;
pub use ::vello;

pub mod prelude {
    pub use crate::{
        render::{VelloRenderSettings, VelloView},
        scene::{UiVelloScene, VelloScene2d},
        text::{
            UiVelloText, VelloFont, VelloText2d, VelloTextAlign, VelloTextAnchor, VelloTextStyle,
        },
    };
    // Vendor re-exports
    pub use ::vello::{self, kurbo, peniko};
}
