use bevy::prelude::*;

pub mod prefab;

use self::prefab::*;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PrefabPlugin)
        .register_type::<Foo>()
        .register_type::<Bar>()
        .register_type::<smallvec::SmallVec<[Entity; 8]>>()
        .add_system(update_system)
        .add_startup_system(setup_system_sample)
        .run();
}

fn update_system(
    query: Query<(&Handle<Prefab>, &PrefabInstance)>,
    prefabs: Res<Assets<Prefab>>,
    type_registry: Res<AppTypeRegistry>,
) {
    bevy::log::info!("--- step ---");
    for (prefab, instance) in query.iter() {
        bevy::log::info!("{:?}", instance);
        if let Some(prefab) = prefabs.get(prefab) {
            let data = prefab.serialize_ron(&type_registry).unwrap();
            bevy::log::info!("{}", data);
        }
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct Foo {
    a: usize,
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
struct Bar(usize);

fn setup_system_load(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(PrefabBundle {
        prefab: assets.load("test.prefab.ron"),
        ..default()
    });
}

fn setup_system_sample(
    mut commands: Commands,
    mut prefabs: ResMut<Assets<Prefab>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let prefab = sample_prefab(&type_registry);
    commands.spawn(PrefabBundle {
        prefab: prefabs.add(prefab),
        ..default()
    });
}

fn sample_prefab(registry: &AppTypeRegistry) -> Prefab {
    let mut world = World::default();

    world.spawn(Foo { a: 5 }).with_children(|builder| {
        builder.spawn((Foo { a: 7 }, Bar(9)));
    });

    world.spawn(Bar(11));

    Prefab::from_world(&world, registry)
}
