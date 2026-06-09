use crate::state::InGame;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(InGame), setup_character)
        .add_systems(Update, movement_action);
}

pub fn setup_character(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh = meshes.add(crate::dev::character::character_mesh());
    let material = materials.add(StandardMaterial::default());

    let target = commands.spawn((
        Transform::from_xyz(-3.0, 0.1, -3.0),
        MoveTarget { radius: 0.5 },
    ));
    let target = target.id();

    commands.spawn((
        CharacterController {
            speed: 5.0,
            radius: 0.5,
        },
        Transform::from_xyz(3.0, 0.1, 3.0),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        MovementAction { target },
    ));
}

#[derive(Component)]
pub struct CharacterController {
    pub speed: f32,
    pub radius: f32,
}

#[derive(Component)]
pub struct MoveTarget {
    pub radius: f32,
}

#[derive(Component)]
pub struct MovementAction {
    pub target: Entity,
}

pub fn movement_action(
    time: Res<Time<Virtual>>,
    mut actors: Query<(&mut Transform, &CharacterController, &MovementAction), Without<MoveTarget>>,
    targets: Query<(&Transform, &MoveTarget), Without<CharacterController>>,
) -> Result {
    for (mut transform, ctrl, action) in &mut actors {
        let (goal_transform, move_to) = targets.get(action.target)?;
        let delta = goal_transform.translation - transform.translation;
        let distance = delta.length();

        if distance > move_to.radius + ctrl.radius {
            let step = (ctrl.speed * time.delta_secs()).min(distance);
            transform.translation += delta.normalize_or_zero() * step;
            transform.look_to(delta, Vec3::Y);
        } else {
            debug!("success");
        }
    }

    Ok(())
}

/*
#[derive(Component, Clone)]
pub struct FindAndMove<T: Component + Clone> {
    radius: f32,
    marker: std::marker::PhantomData<T>,
}

impl<T: Component + Clone> FindAndMove<T> {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            marker: std::marker::PhantomData,
        }
    }

    pub fn system(
        time: Res<Time<Virtual>>,
        query: Query<(Entity, &Transform), With<T>>,
        mut actors: Query<(&mut Transform, &mut CachedFinder, &CharacterController), Without<T>>,
        // mut actions: Query<(ActionQuery, &Self)>,
    ) {
        for (mut action, move_to) in actions.iter_mut() {
            let (mut transform, mut finder, ctrl) = actors.get_mut(action.actor()).unwrap();

            if action.is_executing() {
                let Some(goal) = finder.find(&query, transform.translation) else {
                    action.failure();
                    continue;
                };

                let delta = goal.translation - transform.translation;
                let distance = delta.length();

                trace!("Distance to {:?}: {}", std::any::type_name::<T>(), distance);

                if distance > move_to.radius {
                    let step = (ctrl.speed * time.delta_secs()).min(distance);
                    transform.translation += delta.normalize_or_zero() * step;
                    transform.look_to(delta, Vec3::Y);
                } else {
                    debug!("Reached {:?}", std::any::type_name::<T>());
                    action.success()
                }
            }

            if action.is_cancelled() {
                debug!("Movement to {:?} is cancelled", std::any::type_name::<T>());
                action.failure();
            }

            // cleanup just for sure
            if action.is_done() {
                let _ = finder.take_target();
            }
        }
    }
}

#[derive(Component, Default)]
pub struct CachedFinder {
    target: Option<Entity>,
}

impl CachedFinder {
    pub fn target(&self) -> Option<Entity> {
        self.target
    }

    #[must_use]
    pub fn take_target(&mut self) -> Option<Entity> {
        self.target.take()
    }

    pub fn find<T: Component>(
        &mut self,
        query: &Query<(Entity, &'_ Transform), With<T>>,
        translation: Vec3,
    ) -> Option<Transform> {
        let (entity, transform) = if let Some(entity) = self.target {
            query.get(entity).ok()
        } else {
            debug!("Try find {:?}", std::any::type_name::<T>());
            query.iter().min_by(|(_, a), (_, b)| {
                let a = (a.translation - translation).length_squared();
                let b = (b.translation - translation).length_squared();
                f32::total_cmp(&a, &b)
            })
        }?;

        self.target = Some(entity);

        Some(*transform)
    }
}

*/
