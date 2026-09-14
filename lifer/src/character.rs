use crate::loading::{AssetCache, ItemDatabase};
use crate::mechanics::*;
use crate::state::InGame;
use bevy::prelude::*;
use bevy::render::mesh::{SphereKind, SphereMeshBuilder};
use big_brain::*;

mod inventory;
mod movement;

use self::item::ContainerBundle;
pub use self::{
    inventory::Inventory,
    movement::{CachedFinder, FindAndMove},
};

pub const DEFAULT_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::BLACK);
pub const SLEEP_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::BLUE);
pub const FARM_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::YELLOW);

pub fn plugin(app: &mut App) {
    app.add_plugins((
        BigBrainPlugin::new(PreUpdate, PreUpdate, PostUpdate, Last),
        self::inventory::InventoryPlugin,
    ))
    .init_resource::<AssetCache>()
    .add_event::<SpawnCharacter>()
    .add_systems(PreUpdate, spawner_system.run_if(in_state(InGame)))
    .add_systems(Update, sync_character_color.run_if(in_state(InGame)));
}

#[derive(Component)]
pub struct CharacterController {
    pub speed: f32,
    pub color: Color,
    pub is_sleeping: bool,
}

pub fn sync_character_color(
    mut cache: ResMut<AssetCache>,
    query: Query<(&CharacterController, &Children)>,
    mut material_query: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (ctrl, children) in &query {
        let mut handle = children
            .first()
            .and_then(|&entity| material_query.get_mut(entity).ok());

        if let Some(dst) = handle.as_deref_mut() {
            dst.0 = cache.get_material(&mut materials, ctrl.color);
        }
    }
}

#[derive(Event, Default)]
pub struct SpawnCharacter {
    pub player: bool,
    pub transform: Transform,
    pub model: CharacterModel,
}

#[derive(Clone, Copy, PartialEq)]
pub struct CharacterModel {
    pub height: f32,
    pub radius: f32,
    pub face_height: f32,
}

impl std::cmp::Eq for CharacterModel {}

impl std::hash::Hash for CharacterModel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.height.to_bits().hash(state);
        self.radius.to_bits().hash(state);
        self.face_height.to_bits().hash(state);
    }
}

impl Default for CharacterModel {
    fn default() -> Self {
        Self {
            height: 1.75,
            radius: 0.5,
            face_height: 0.5,
        }
    }
}

#[derive(Component, Clone, ActionSpawn)]
pub struct Idle;

#[derive(Clone)]
pub struct ModelCacheEntry {
    pub capsule: Handle<Mesh>,
    pub cube: Handle<Mesh>,
}

#[allow(clippy::too_many_arguments)]
pub fn spawner_system(
    mut thinker_cache: Local<Option<Handle<ThinkerSpawner>>>,
    mut thinkers: ResMut<Assets<ThinkerSpawner>>,

    mut cache: ResMut<AssetCache>,
    mut commands: Commands,
    mut events: ResMut<Events<SpawnCharacter>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    items: Res<ItemDatabase>,
) {
    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    let mut rng = SmallRng::from_entropy();

    let thinker = thinker_cache
        .get_or_insert_with(|| thinkers.add(create_thinker()))
        .clone();

    for SpawnCharacter {
        player,
        transform,
        model,
    } in events.drain()
    {
        let mut container_entity = commands.spawn(ContainerBundle::default());
        let container = container_entity.id();

        if player {
            container_entity.with_children(|builder| {
                let sphere = SphereMeshBuilder::new(0.3, SphereKind::Ico { subdivisions: 3 });

                builder.spawn((
                    ItemHandle(items.potion.clone()),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    Mesh3d(meshes.add(Mesh::from(sphere))),
                    MeshMaterial3d(cache.get_material(
                        &mut materials,
                        Color::Srgba(bevy::color::palettes::basic::RED),
                    )),
                ));
            });
        }

        let mut entity = commands.spawn((
            StateScoped(InGame),
            transform,
            Visibility::Inherited,
            CharacterController {
                speed: 5.0,
                color: DEFAULT_COLOR,
                is_sleeping: false,
            },
            Fatigue {
                current: rng.gen_range(0.0..=100.0),
                change: 8.0,
            },
            Inventory { container },
            CachedFinder::default(),
            HandleThinkerSpawner(thinker.clone()),
        ));

        if player {
            entity.insert(crate::player::Player);
        }

        entity.with_children(|builder| {
            let ModelCacheEntry { capsule, cube } = cache.get_model(&mut meshes, model);

            builder.spawn((
                Mesh3d(capsule.clone()),
                MeshMaterial3d(cache.get_material(
                    &mut materials,
                    Color::Srgba(bevy::color::palettes::basic::YELLOW),
                )),
                Transform::from_translation(Vec3::new(0.0, model.height / 2.0 + model.radius, 0.0)),
            ));

            builder.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(cache.get_material(
                    &mut materials,
                    Color::Srgba(bevy::color::palettes::basic::RED),
                )),
                Transform::from_translation(Vec3::new(
                    0.0,
                    model.height - model.face_height / 2.0 + model.radius,
                    -model.radius,
                )),
            ));
        });

        entity.add_child(container);
    }
}

fn create_thinker() -> ThinkerSpawner {
    ThinkerSpawner::highest(0.0)
        .when(
            FatigueScorer::default(),
            Sequence::step((FindAndMove::<House>::new(0.1), Sleep::new(10.0, 30.0))),
        )
        .when(
            WorkNeedScorer,
            Sequence::step((FindAndMove::<Field>::new(0.1), Farm::new(30.0))),
        )
        .when(
            SellNeedScorer,
            Sequence::step((FindAndMove::<Market>::new(0.1), Sell)),
        )
        .when(FixedScorer::IDLE, Idle)
}
