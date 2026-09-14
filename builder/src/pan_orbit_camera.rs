use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        pan_orbit_camera.run_if(any_with_component::<PanOrbitState>),
    );
}
// Bundle to spawn our custom camera easily
#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub state: PanOrbitState,
    pub settings: PanOrbitSettings,
}

// The internal state of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitState {
    pub center: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub pitch: f32,
    pub yaw: f32,
}

/// The configuration of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitSettings {
    /// World units per pixel of mouse motion
    pub pan_sensitivity: f32,
    /// Radians per pixel of mouse motion
    pub orbit_sensitivity: f32,
    /// Exponent per pixel of mouse motion
    pub zoom_sensitivity: f32,
    /// What action is bound to the scroll wheel?
    pub scroll_action: Option<PanOrbitAction>,
    /// For devices with a notched scroll wheel, like desktop mice
    pub scroll_line_sensitivity: f32,
    /// For devices with smooth scrolling, like touchpads
    pub scroll_pixel_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanOrbitAction {
    Pan,
    Orbit,
    Zoom,
}

impl Default for PanOrbitState {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 1.0,
            upside_down: false,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

impl Default for PanOrbitSettings {
    fn default() -> Self {
        Self {
            pan_sensitivity: 0.001,                  // 2000 pixels per world unit
            orbit_sensitivity: f32::to_radians(0.1), // 0.1 degree per pixel
            zoom_sensitivity: 0.01,
            scroll_action: Some(PanOrbitAction::Zoom),
            scroll_line_sensitivity: 16.0, // 1 "line" == 16 "pixels of motion"
            scroll_pixel_sensitivity: 1.0,
        }
    }
}

/*
fn spawn_camera(mut commands: Commands) {
    let mut camera = PanOrbitCameraBundle::default();
    // Position our camera using our component,
    // not Transform (it would get overwritten)
    camera.state.center = Vec3::new(1.0, 2.0, 3.0);
    camera.state.radius = 50.0;
    camera.state.pitch = f32::to_radians(15.0);
    camera.state.yaw = f32::to_radians(30.0);
    commands.spawn(camera);
}
*/

fn pan_orbit_camera(
    kbd: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    wheel: Res<AccumulatedMouseScroll>,
    mut q_camera: Query<(&PanOrbitSettings, &mut PanOrbitState, &mut Transform)>,
) {
    // First, accumulate the total amount of
    // mouse motion and scroll, from all pending events:
    // let motion = mouse_motion.read().map(|ev| ev.delta).sum::<Vec2>() * Vec2::new(1.0, -1.0);
    let motion = motion.delta * Vec2::new(1.0, -1.0);

    let (lines, pixels) = match wheel.unit {
        MouseScrollUnit::Line => (wheel.delta * Vec2::new(1.0, -1.0), Vec2::ZERO),
        MouseScrollUnit::Pixel => (Vec2::ZERO, wheel.delta * Vec2::new(1.0, -1.0)),
    };

    let mid_mouse = mouse.pressed(MouseButton::Middle);
    let just_mid_mouse = mouse.just_pressed(MouseButton::Middle);

    for (settings, mut state, mut transform) in &mut q_camera {
        // Check how much of each thing we need to apply.
        // Accumulate values from motion and scroll,
        // based on our configuration settings.

        let mut is_pan = false;
        let mut is_orbit = false;
        let mut is_orbit_started = false;
        let mut is_zoom = false;

        if mid_mouse {
            if kbd.pressed(KeyCode::ShiftLeft) {
                is_pan = true;
            } else if kbd.pressed(KeyCode::ControlLeft) {
                is_zoom = true;
            } else {
                is_orbit = true;
                is_orbit_started = just_mid_mouse;
            }
        }

        let mut pan = Vec2::ZERO;
        if is_pan {
            pan -= motion * settings.pan_sensitivity;
        }
        if matches!(settings.scroll_action, Some(PanOrbitAction::Pan)) {
            pan -= lines * settings.scroll_line_sensitivity * settings.pan_sensitivity;
            pan -= pixels * settings.scroll_pixel_sensitivity * settings.pan_sensitivity;
        }

        let mut orbit = Vec2::ZERO;
        if is_orbit {
            orbit -= motion * settings.orbit_sensitivity;
        }
        if matches!(settings.scroll_action, Some(PanOrbitAction::Orbit)) {
            orbit -= lines * settings.scroll_line_sensitivity * settings.orbit_sensitivity;
            orbit -= pixels * settings.scroll_pixel_sensitivity * settings.orbit_sensitivity;
        }

        let mut zoom = Vec2::ZERO;
        if is_zoom {
            zoom -= -motion * settings.zoom_sensitivity;
        }
        if matches!(settings.scroll_action, Some(PanOrbitAction::Zoom)) {
            zoom -= lines * settings.scroll_line_sensitivity * settings.zoom_sensitivity;
            zoom -= pixels * settings.scroll_pixel_sensitivity * settings.zoom_sensitivity;
        }

        // Upon starting a new orbit maneuver (key is just pressed),
        // check if we are starting it upside-down
        if is_orbit_started {
            state.upside_down = state.pitch < -FRAC_PI_2 || state.pitch > FRAC_PI_2;
        }

        // If we are upside down, reverse the X orbiting
        if state.upside_down {
            orbit.x = -orbit.x;
        }

        // Now we can actually do the things!

        let mut any_changed = false;

        // To ZOOM, we need to multiply our radius.
        if !zoom.abs_diff_eq(Vec2::ZERO, 0.0001) {
            any_changed = true;

            // in order for zoom to feel intuitive,
            // everything needs to be exponential
            // (done via multiplication)
            // not linear
            // (done via addition)

            // so we compute the exponential of our
            // accumulated value and multiply by that
            state.radius *= (-zoom.y).exp();
        }

        // To ORBIT, we change our pitch and yaw values
        if !orbit.abs_diff_eq(Vec2::ZERO, 0.0001) {
            any_changed = true;

            state.yaw += orbit.x;
            state.pitch -= orbit.y;

            // wrap around, to stay between +- 180 degrees
            state.yaw = (state.yaw + PI).rem_euclid(TAU) - PI;
            state.pitch = (state.pitch + PI).rem_euclid(TAU) - PI;

            // state.yaw = state.yaw.rem_euclid(TAU);
            // state.pitch = state.pitch.rem_euclid(TAU);
        }

        // To PAN, we can get the UP and RIGHT direction
        // vectors from the camera's transform, and use
        // them to move the center point. Multiply by the
        // radius to make the pan adapt to the current zoom.
        if !pan.abs_diff_eq(Vec2::ZERO, 0.0001) {
            any_changed = true;

            let radius = state.radius;
            state.center += transform.right() * pan.x * radius;
            state.center += transform.up() * pan.y * radius;
        }

        // Finally, compute the new camera transform.
        // (if we changed anything, or if the pan-orbit
        // controller was just added and thus we are running
        // for the first time and need to initialize)
        if any_changed || state.is_added() {
            // YXZ Euler Rotation performs yaw/pitch/roll.
            transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
            // To position the camera, get the backward direction vector
            // and place the camera at the desired radius from the center.
            transform.translation = state.center + transform.back() * state.radius;
        }
    }
}
