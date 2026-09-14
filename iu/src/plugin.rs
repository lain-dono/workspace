use crate::{
    VelloRenderSettings,
    render::{VelloCanvasSettings, VelloRenderPlugin, extract::VelloExtractStep},
    scene::render,
};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use vello::AaConfig;

#[derive(Clone)]
pub struct VelloPlugin {
    /// The render layers that will be used for the Vello canvas mesh.
    pub canvas_render_layers: RenderLayers,

    /// Use CPU instead of GPU
    pub use_cpu: bool,

    /// Which antialiasing strategy to use
    pub antialiasing: AaConfig,
}

impl Default for VelloPlugin {
    fn default() -> Self {
        let default_canvas_settings = VelloCanvasSettings::default();
        let default_render_settings = VelloRenderSettings::default();
        Self {
            canvas_render_layers: default_canvas_settings.render_layers,
            use_cpu: default_render_settings.use_cpu,
            antialiasing: default_render_settings.antialiasing,
        }
    }
}

impl Plugin for VelloPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VelloRenderPlugin {
            canvas_settings: VelloCanvasSettings {
                render_layers: self.canvas_render_layers.clone(),
            },
            render_settings: VelloRenderSettings {
                use_cpu: self.use_cpu,
                antialiasing: self.antialiasing,
            },
        });

        // scene
        {
            #[cfg(feature = "picking")]
            app.add_plugins(crate::picking::WorldPickingPlugin::<super::VelloScene2d>::default());

            let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
                return;
            };

            render_app
                .add_systems(
                    ExtractSchedule,
                    (render::extract_world_scenes, render::extract_ui_scenes)
                        .in_set(VelloExtractStep::ExtractAssets),
                )
                .add_systems(
                    Render,
                    render::prepare_scene_affines.in_set(RenderSystems::Prepare),
                );
        }

        app.add_plugins(crate::text::VelloTextIntegrationPlugin);
    }
}
