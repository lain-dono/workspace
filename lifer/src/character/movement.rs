use super::CharacterController;
use bevy::prelude::*;
use big_brain::*;

// TODO: radius maybe readed from world

#[derive(Component, Clone, ActionSpawn)]
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
        mut actions: Query<(ActionQuery, &Self)>,
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
