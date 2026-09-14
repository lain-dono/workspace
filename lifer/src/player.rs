use crate::{character::FindAndMove, raycast::PlaneRaycast, settings::SettingsMenu, state::InGame};
use bevy::{input::mouse::MouseWheel, prelude::*};
use big_brain::{ActionState, BigBrainSet, HasThinker, Thinker};

pub mod inventory;
pub mod time;

pub fn plugin(app: &mut App) {
    app.init_resource::<CurrentlySelected>()
        .init_resource::<InputConfig>()
        .add_systems(
            PreUpdate,
            FindAndMove::<PlayerTarget>::system
                .in_set(BigBrainSet::Actions)
                .run_if(in_state(InGame)),
        )
        .add_systems(
            Update,
            (
                (update_player_camera, object_select, target_select)
                    .before(crate::raycast::plane_raycast),
                self::inventory::inventory_ui,
                self::time::time_ui,
                self::time::pause_on_esc.run_if(not(resource_exists::<SettingsMenu>)),
                self::time::pause_menu.run_if(not(resource_exists::<SettingsMenu>)),
            )
                .run_if(in_state(InGame)),
        )
        .add_systems(OnEnter(InGame), spawn_player_target)
        .add_systems(Update, target_select.run_if(in_state(InGame)));
}

fn spawn_player_target(mut commands: Commands) {
    commands.spawn((
        StateScoped(InGame),
        PlayerTarget,
        Transform::default(),
        Visibility::Inherited,
    ));
}

#[derive(Component, Clone, Copy)]
pub struct PlayerTarget;

pub fn target_select(
    mut camera: Query<&PlaneRaycast, With<Camera>>,
    player: Query<&HasThinker, With<Player>>,
    mut thinkers: Query<&mut Thinker>,
    mut states: Query<&mut ActionState>,
    mut target: Query<&mut Transform, With<PlayerTarget>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
) {
    if mouse_btn.just_released(MouseButton::Left) {
        let raycast = camera.single_mut();
        let player = player.single().entity();
        let thinker = thinkers.get_mut(player).ok();
        if let Some((position, mut thinker)) = raycast.result.zip(thinker) {
            if !thinker.has_scheduled() {
                thinker.schedule(FindAndMove::<PlayerTarget>::new(0.1));

                // cancel current action
                if let Some(action) = thinker.current() {
                    bevy::log::warn!("cancel current action");
                    let mut state = states.get_mut(action.entity()).unwrap();
                    if state.is_executing() {
                        state.cancel()
                    }
                }
            }

            target.single_mut().translation = position;
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct CameraController;

#[derive(Component)]
pub struct Selectable;

#[derive(Resource)]
pub struct CurrentlySelected {
    pub selected: Entity,
}

impl Default for CurrentlySelected {
    fn default() -> Self {
        Self {
            selected: Entity::PLACEHOLDER,
        }
    }
}

pub fn object_select(
    mut query: Query<&PlaneRaycast, With<Camera>>,
    selectables: Query<(&GlobalTransform, &Selectable, Entity)>,
    mbtn: Res<ButtonInput<MouseButton>>,
    mut sel: ResMut<CurrentlySelected>,
) -> Result {
    if !mbtn.just_pressed(MouseButton::Left) {
        return Ok(());
    }

    let raycast = query.single_mut()?;
    let Some(pos) = raycast.result else {
        return Ok(());
    };

    let control_distance = 2.0;

    for p in selectables.iter() {
        if (p.0.translation() - pos).length() < control_distance {
            sel.selected = p.2;
            println!("Selected entity with id = {}", p.2.index());
        }
    }

    Ok(())
}

#[derive(Resource)]
pub struct InputConfig {
    pub movement_speed: f32,
    pub rotation_speed: f32,

    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,

    pub rotate_left: KeyCode,
    pub rotate_right: KeyCode,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            movement_speed: 20.0,
            rotation_speed: 3.0,

            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,

            rotate_left: KeyCode::KeyQ,
            rotate_right: KeyCode::KeyE,
        }
    }
}

pub fn update_player_camera(
    input: Res<InputConfig>,
    mut controller: Query<(&mut Transform, &Children), (With<CameraController>, Without<Camera>)>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<CameraController>)>,

    mut taa: Query<&mut bevy::core_pipeline::experimental::taa::TemporalAntiAliasing>,

    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    time: Res<Time<Real>>,
) -> Result {
    let wheel = mouse_wheel.read().fold(0.0, |acc, event| acc + event.y);

    let (mut transform, children) = controller.single_mut()?;

    let camera_entity = children.first().copied();
    let camera_transform = camera_entity.and_then(|entity| camera.get_mut(entity).ok());
    let camera_taa = camera_entity.and_then(|entity| taa.get_mut(entity).ok());

    let mut changed = false;

    if wheel != 0.0 {
        changed |= true;

        if let Some(mut transform) = camera_transform {
            let old_len = transform.translation.length();
            let new_len = (old_len - wheel * 2.0).clamp(20.0, 200.0);
            transform.translation = transform.translation.normalize() * new_len;
        }
    }

    let mut diff = Vec3::new(0.0, 0.0, 0.0);

    if keyboard.pressed(input.move_left) {
        diff.x += 1.0;
        changed |= true;
    }
    if keyboard.pressed(input.move_right) {
        diff.x -= 1.0;
        changed |= true;
    }
    if keyboard.pressed(input.move_forward) {
        diff.z += 1.0;
        changed |= true;
    }
    if keyboard.pressed(input.move_backward) {
        diff.z -= 1.0;
        changed |= true;
    }

    let base_forward = transform.forward();

    let forward = Vec3::new(base_forward.x, 0.0, base_forward.z);
    let right = Vec3::new(base_forward.z, 0.0, -base_forward.x);

    let movement = (right * diff.x + forward * diff.z).normalize_or_zero();

    transform.translation += movement * time.delta_secs() * input.movement_speed;

    let mut angle = 0.0;
    if keyboard.pressed(input.rotate_right) {
        angle += 1.0;
        changed |= true;
    }
    if keyboard.pressed(input.rotate_left) {
        angle -= 1.0;
        changed |= true;
    }
    transform.rotate_y(angle * time.delta_secs() * input.rotation_speed);

    if changed {
        if let Some(mut taa) = camera_taa {
            taa.reset = true;
        }
    }

    Ok(())
}
