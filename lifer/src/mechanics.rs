pub mod field;
pub mod house;
pub mod item;
pub mod market;

pub use self::{
    field::{Farm, Field, WorkNeedScorer},
    house::{Fatigue, FatigueScorer, House, Sleep},
    item::{Item, ItemAsset, ItemAssetLoader, ItemAssetLoaderError, ItemHandle, ItemSpawnError},
    market::{Market, Sell, SellNeedScorer},
};

use crate::state::InGame;
use bevy::prelude::*;

pub const FIELD_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::YELLOW);
pub const HOUSE_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::BLUE);
pub const MARKET_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::RED);

pub fn plugin(app: &mut App) {
    app.add_plugins((
        self::field::FieldPlugin,
        self::house::HousePlugin,
        self::item::ItemPlugin,
        self::market::MarketPlugin,
    ))
    .add_systems(OnEnter(InGame), spawn_scene);
}

pub fn spawn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    // let rotation = Transform::from_rotation(rotation);
    // let model = meshes.add(shape::Circle::new(0.5).into());

    let bed = asset_server.load("models/nature/bed_floor.glb#Scene0");
    let sign = asset_server.load("models/nature/sign.glb#Scene0");
    let plant = asset_server.load("models/nature/plant_bushDetailed.glb#Scene0");

    // let sample = [
    //     // asset_server.load("models/nature/flower_purpleA.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_purpleB.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_purpleC.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_redA.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_redB.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_redC.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_yellowA.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_yellowB.glb#Scene0"),
    //     // asset_server.load("models/nature/flower_yellowC.glb#Scene0"),
    //     asset_server.load("models/nature/plant_bush.glb#Scene0"),
    //     asset_server.load("models/nature/plant_bushDetailed.glb#Scene0"),
    //     asset_server.load("models/nature/plant_bushLarge.glb#Scene0"),
    //     asset_server.load("models/nature/plant_bushLargeTriangle.glb#Scene0"),
    //     asset_server.load("models/nature/plant_bushSmall.glb#Scene0"),
    //     asset_server.load("models/nature/plant_bushTriangle.glb#Scene0"),
    //     asset_server.load("models/nature/plant_flatShort.glb#Scene0"),
    //     asset_server.load("models/nature/plant_flatTall.glb#Scene0"),
    // ];
    // let bed_model = asset_server.load("models/nature/bed.glb#Mesh0/Primitive0");

    // some characters

    use rand::rngs::SmallRng;
    use rand::{Rng, SeedableRng};
    use std::hash::Hasher;
    let mut rng = SmallRng::from_entropy();

    fn rng_from_point(x: f64, z: f64) -> SmallRng {
        let mut hasher = std::hash::DefaultHasher::new();
        hasher.write_u64(x.to_bits());
        hasher.write_u64(z.to_bits());
        SmallRng::seed_from_u64(hasher.finish())
    }

    let positions = fast_poisson::Poisson2D::new()
        .with_dimensions([100.0; 2], 10.0)
        .with_seed(rng.gen());

    for [x, z] in positions {
        let mut rng = rng_from_point(x, z);
        // farm field
        commands.spawn((
            Field,
            StateScoped(InGame),
            Transform {
                translation: Vec3::new((x - 100.0) as f32, 0.25, (z - 100.0) as f32),
                rotation: Quat::from_rotation_y(rng.gen_range(0.0..=std::f32::consts::TAU)),
                scale: Vec3::splat(5.0),
            },
            SceneRoot(plant.clone()),
        ));
    }

    let positions = fast_poisson::Poisson2D::new()
        .with_dimensions([100.0; 2], 10.0)
        .with_seed(rng.gen());

    for [x, z] in positions {
        let mut rng = rng_from_point(x, z);
        // sleeping house
        commands.spawn((
            House,
            StateScoped(InGame),
            Transform {
                translation: Vec3::new((x - 100.0) as f32, 0.25, (z - 100.0) as f32),
                rotation: Quat::from_rotation_y(rng.gen_range(0.0..=std::f32::consts::TAU)),
                scale: Vec3::splat(5.0),
            },
            SceneRoot(bed.clone()),
        ));
    }

    let positions = fast_poisson::Poisson2D::new()
        .with_dimensions([100.0; 2], 10.0)
        .with_seed(rng.gen());

    for [x, z] in positions {
        let mut rng = rng_from_point(x, z);

        // marketplace
        commands.spawn((
            Market,
            StateScoped(InGame),
            Transform {
                translation: Vec3::new((x - 100.0) as f32, 0.25, (z - 100.0) as f32),
                rotation: Quat::from_rotation_y(rng.gen_range(0.0..=std::f32::consts::TAU)),
                scale: Vec3::splat(5.0),
            },
            SceneRoot(sign.clone()),
        ));
    }
}
