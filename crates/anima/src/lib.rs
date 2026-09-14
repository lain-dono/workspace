pub mod armature;
pub mod editor;
pub mod example;
pub mod gizmo;
pub mod grid;
pub mod runtime;
pub mod timeline;
pub mod viewport;

mod util;

pub use self::editor::AnimaEditorContext;
pub use self::grid::{Grid, GridViewport};
pub use self::runtime::{AnimaData, AnimaState, Matrix, PlayControl, PlayState, Transform};
pub use self::timeline::TimelinePanel;
pub use self::viewport::Animation2d;

use crate::ui::{AddEditorTab, EditorPanel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use reui::plugin::Recorder;

#[derive(Default)]
pub struct AnimaPlugin;

impl bevy::app::Plugin for AnimaPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(reui::plugin::ReuiPlugin)
            .insert_resource(self::example::armature())
            .insert_resource(self::example::data())
            .init_resource::<self::editor::Record>()
            .add_systems(Update, sync_frame)
            .add_systems(PostUpdate, draw)
            .add_editor_tab::<TimelinePanel>()
            .add_editor_tab::<Animation2d>();
    }
}

fn sync_frame(mut state: ResMut<AnimaState>, data: Res<AnimaData>) {
    state.sync(&data);
}

fn draw(
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut query: Query<(&mut Recorder, &Animation2d, &EditorPanel)>,
) {
    let time = time.elapsed_seconds();
    let ppi = windows.single().scale_factor() as f32;
    for (mut recorder, anima, panel) in query.iter_mut() {
        recorder.clear();

        if let Some(frame) = panel.viewport {
            let zoom = anima.grid.zoom;
            let viewport = {
                let center = frame.center() - frame.min;
                let offset = center + anima.grid.offset * egui::vec2(-zoom, zoom);

                reui::Transform {
                    sx: ppi,
                    shy: 0.0,
                    shx: 0.0,
                    sy: -ppi,
                    tx: offset.x * ppi,
                    ty: offset.y * ppi,
                }
            };

            let mut canvas = self::gizmo::Gizmos::new(&mut recorder, viewport, zoom);

            let x_color = reui::Color::bgra(0xFFFF0000);
            let y_color = reui::Color::bgra(0xFF00FF00);

            use std::f32::consts::{FRAC_PI_2, PI};
            let rotation = time;

            canvas.set_origin(0.0, 0.0, rotation);
            canvas.rotate(x_color);
            canvas.set_origin(45.0, 0.0, rotation);
            canvas.length(x_color);

            canvas.set_origin(45.0, 20.0, rotation);
            canvas.pose(x_color);

            canvas.set_origin(70.0, 0.0, rotation);
            canvas.arrow_translate(x_color);
            canvas.set_origin(70.0, 0.0, rotation - FRAC_PI_2);
            canvas.arrow_translate(y_color);

            canvas.set_origin(140.0, 0.0, rotation);
            canvas.arrow_scale(x_color);
            canvas.set_origin(140.0, 0.0, rotation - FRAC_PI_2);
            canvas.arrow_scale(y_color);

            canvas.set_origin(200.0, 0.0, 0.0);
            canvas.shear(x_color, time);
            canvas.set_origin(200.0, 0.0, PI);
            canvas.shear(x_color, time);

            canvas.set_origin(200.0, 0.0, FRAC_PI_2);
            canvas.shear(y_color, -time);
            canvas.set_origin(200.0, 0.0, -FRAC_PI_2);
            canvas.shear(y_color, -time);
        }
    }
}
