use bevy::ecs::query::{QueryFilter, ROQueryItem, ReadOnlyQueryData};
use bevy::prelude::*;

macro_rules! marker {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
            #[derive(Component, Default, Reflect, serde::Serialize, serde::Deserialize)]
            #[reflect(Component, Default, FromReflect, Serialize, Deserialize)]
            pub struct $name;
        )+
    };
}

marker!(FlatSurface, ServingSurface, PrepareSurface, EatingSurface);
marker!(WashDish, WashHands, Clean, Dispose, Eat, Sit, Stand);
marker!(Repair, Broken);

pub fn find_best_for<T: ReadOnlyQueryData, F: QueryFilter>(
    query: Query<(Entity, T), F>,
    scorer: impl Fn(ROQueryItem<'_, '_, T>) -> Option<f32>,
) -> Option<Entity> {
    query
        .into_iter()
        .filter_map(|(entity, data)| Some((entity, scorer(data)?)))
        .max_by(|(_, a), (_, b)| f32::total_cmp(a, b))
        .map(|(entity, _)| entity)
}
