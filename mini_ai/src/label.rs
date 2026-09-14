use bevy::{ecs::query::QueryEntityError, prelude::*};

#[derive(Component)]
pub struct EntityLabel {
    pub entity: Entity,
    pub offset: Vec3,
}

impl EntityLabel {
    #[must_use]
    pub fn bundle(
        entity: Entity,
        color: impl Into<Color>,
        font: Handle<Font>,
        label: &str,
    ) -> impl Bundle {
        let offset = Vec3::Y;
        (
            EntityLabel { entity, offset },
            Text::new(label),
            TextLayout::new(Justify::Left, LineBreak::NoWrap),
            TextFont { font, ..default() },
            TextColor(color.into()),
            // BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
        )
    }
}

pub fn sync_labels(
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut labels: Query<(
        Entity,
        &mut Node,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        &EntityLabel,
    )>,
    labeled: Query<&GlobalTransform>,
    mut commands: Commands,
) {
    let (camera, camera_transform) = camera.into_inner();

    for (node_entity, mut node, computed, target, label) in &mut labels {
        match labeled.get(label.entity) {
            Ok(transform) => {
                let world_position = transform.translation() + label.offset;
                let viewport_position = camera.world_to_viewport(camera_transform, world_position);
                if let Ok(viewport_position) = viewport_position {
                    let size = computed.size / target.scale_factor();
                    node.left = Val::Px(viewport_position.x - size.x * 0.5);
                    node.top = Val::Px(viewport_position.y - size.y);
                }
            }
            Err(QueryEntityError::EntityDoesNotExist(_)) => commands.entity(node_entity).despawn(),
            Err(err) => panic!("{err}"),
        }
    }
}
