// character::{CharacterModel, ModelCacheEntry},
// mechanics::ItemAsset,
// use bevy::{
//     platform::collections::hash_map::HashMap,
// render::mesh::Capsule3dMeshBuilder,
// };
use crate::state::AppState;
use crate::ui::style;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use iyes_progress::{Progress, ProgressReturningSystem, ProgressTracker};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (track_fake_long_task::<4>.track_progress::<AppState>(),)
            .chain()
            .run_if(in_state(AppState::Loading))
            .after(LoadingStateSet(AppState::Loading)),
    )
    .add_systems(
        EguiPrimaryContextPass,
        print_progress.run_if(in_state(AppState::Loading)),
    );
}

fn track_fake_long_task<const TOTAL: u32>(time: Res<Time>) -> Progress {
    let progress = Progress {
        done: time.elapsed_secs() as u32,
        total: TOTAL,
    };
    if progress.done >= progress.total {
        info!("Long task is completed");
    }
    progress
}

fn print_progress(mut contexts: EguiContexts, counter: Res<ProgressTracker<AppState>>) -> Result {
    egui::Area::new(egui::Id::new("#LOADING_SCREEN"))
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -100.0])
        .show(contexts.ctx_mut()?, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 8.0);
                ui.spacing_mut().button_padding = egui::vec2(0.0, 32.0);
                ui.set_max_width(800.0);

                ui.vertical_centered_justified(|ui| {
                    ui.label(style::rich("loading", "loading..."));

                    let progress = counter.get_global_progress();
                    let bar = egui::ProgressBar::new(progress.done as f32 / progress.total as f32);
                    ui.add(bar.corner_radius(0.0));
                });
            });
        });

    Ok(())
}

/*
#[derive(AssetCollection, Resource)]
pub struct ItemDatabase {
    #[asset(path = "items/money.item.ron")]
    pub money: Handle<ItemAsset>,

    #[asset(path = "items/raw_food.item.ron")]
    pub raw_food: Handle<ItemAsset>,

    #[asset(path = "items/potion.item.ron")]
    pub potion: Handle<ItemAsset>,
}
*/

/*
#[derive(Clone, Copy, PartialEq)]
struct HashedColor(Color);

impl std::cmp::Eq for HashedColor {}

impl std::hash::Hash for HashedColor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Srgba {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        } = self.0.to_srgba();
        r.to_bits().hash(state);
        g.to_bits().hash(state);
        b.to_bits().hash(state);
        a.to_bits().hash(state);
    }
}

#[derive(Resource, Default)]
pub struct AssetCache {
    model: HashMap<CharacterModel, ModelCacheEntry>,
    material: HashMap<HashedColor, Handle<StandardMaterial>>,
}

impl AssetCache {
    pub fn get_model(
        &mut self,
        meshes: &mut Assets<Mesh>,
        model: CharacterModel,
    ) -> ModelCacheEntry {
        self.model
            .entry(model)
            .or_insert_with(|| {
                let capsule = Capsule3dMeshBuilder::new(model.radius, model.height, 6, 6);
                let capsule = meshes.add(Mesh::from(capsule));
                let cube = meshes.add(Mesh::from(Cuboid::new(
                    model.radius * 2.0,
                    model.face_height,
                    model.radius,
                )));

                ModelCacheEntry { capsule, cube }
            })
            .clone()
    }

    pub fn get_material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        color: Color,
    ) -> Handle<StandardMaterial> {
        self.material
            .entry(HashedColor(color))
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 1.0,
                    ..default()
                })
            })
            .clone()
    }
}
*/
