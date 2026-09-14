use std::marker::PhantomData;

use super::hierarchy::Selection;
use crate::ui::{EditorPanel, EditorStage, PanelRenderTarget};
use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*, render::camera::CameraProjection};
use egui_gizmo::{Gizmo, GizmoMode, GizmoOrientation};

#[derive(Clone, Copy)]
pub struct GizmoState {
    mode: GizmoMode,
    orientation: GizmoOrientation,
}

impl Default for GizmoState {
    fn default() -> Self {
        Self {
            mode: GizmoMode::Translate,
            orientation: GizmoOrientation::Global,
        }
    }
}

impl GizmoState {
    fn ui(self, world: &mut World, ui: &mut egui::Ui, entity: Entity) {
        let (camera_view, camera_proj) = world
            .query::<(&Transform, &Projection)>()
            .get(world, entity)
            .unwrap();

        let projection_matrix = camera_proj.get_projection_matrix().to_cols_array_2d();
        let view_matrix = camera_view.compute_matrix().inverse().to_cols_array_2d();

        world.resource_scope(|world, selection: Mut<Selection>| {
            for entity in selection.iter() {
                if world.get::<Transform>(entity).is_none() {
                    continue;
                }

                let Some(mut transform) = world.get_mut::<GlobalTransform>(entity) else {
                    continue;
                };

                let Some(result) = Gizmo::new(entity)
                    .model_matrix(transform.compute_matrix().to_cols_array_2d())
                    .view_matrix(view_matrix)
                    .projection_matrix(projection_matrix)
                    .orientation(self.orientation)
                    .mode(self.mode)
                    .interact(ui)
                else {
                    continue;
                };

                let result = Transform {
                    translation: Vec3::from(<[f32; 3]>::from(result.translation)),
                    rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
                    scale: Vec3::from(<[f32; 3]>::from(result.scale)),
                };

                *transform = GlobalTransform::from(result);

                let result = world
                    .get::<Parent>(entity)
                    .map(Parent::get)
                    .and_then(|parent| world.get::<GlobalTransform>(parent))
                    .map_or(result, |parent| {
                        GlobalTransform::from(result).reparented_to(parent)
                    });

                *world.get_mut::<Transform>(entity).unwrap() = result;
            }
        });
    }
}

struct Drag {
    delta: egui::Vec2,
    button: egui::PointerButton,
}

#[derive(Default)]
struct InputState {
    drag: Option<Drag>,
    hover_pos: Option<egui::Pos2>,
    modifiers: egui::Modifiers,

    scroll: f32,

    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

#[derive(Component)]
pub struct SceneTab<F> {
    pub yew: f32,
    pub pitch: f32,
    pub zsorting: Vec<(Entity, f32, egui::Pos2, GlobalTransform)>,
    pub gizmo: GizmoState,
    marker: PhantomData<fn() -> F>,
}

impl<F: ReadOnlyWorldQuery> Default for SceneTab<F> {
    fn default() -> Self {
        Self {
            yew: 0.0,
            pitch: 0.0,
            zsorting: Vec::new(),
            gizmo: GizmoState::default(),
            marker: PhantomData::<fn() -> F>,
        }
    }
}

impl<F: ReadOnlyWorldQuery + 'static> SceneTab<F> {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel.in_set(EditorStage::Tabs));
    }

    pub fn spawn(commands: &mut Commands, images: &mut Assets<Image>) -> crate::ui::Tab {
        use bevy::core_pipeline::clear_color::ClearColorConfig;
        use bevy::prelude::*;

        let target = PanelRenderTarget::create_render_target(images);

        let gray = 0x2B as f32 / 255.0;
        let clear_color = Color::rgba(gray, gray, gray, 1.0);
        //let clear_color = Color::rgba(gray, 0.0, 0.0, 1.0);

        let box_size = 2.0;
        let box_thickness = 0.15;
        let box_offset = (box_size + box_thickness) / 2.0;

        let entity = commands
            .spawn(Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(clear_color),

                    ..default()
                },
                camera: Camera {
                    target,
                    //order: -1,
                    ..default()
                },

                //transform: Transform::from_translation(Vec3::new(0.0, 5.0, 15.0)),
                transform: Transform::from_xyz(0.0, box_offset, 4.0)
                    .looking_at(Vec3::new(0.0, box_offset, 0.0), Vec3::Y),

                ..default()
            })
            .insert(Self::default())
            .insert(EditorPanel::default())
            .insert(PanelRenderTarget::default())
            .id();

        bevy::log::info!("spawn scene panel {entity:?}");

        crate::ui::Tab::new(crate::ui::icon::VIEW3D, "Scene", entity)
    }

    pub fn panel(world: &mut World) {
        let Some((entity, mut ui)) = super::start_panel::<With<Self>>(world) else {
            return;
        };

        {
            let style = world.resource::<crate::ui::Style>();
            style.set_theme_visuals(&mut ui);
        }

        let scale = ui.ctx().pixels_per_point();
        let size_ui = ui.available_size_before_wrap();
        let size_px = size_ui * scale;

        let target = world
            .query::<&PanelRenderTarget>()
            .get_mut(world, entity)
            .unwrap();
        let Some(texture_id) = target.texture_id else {
            return;
        };

        let scene_texture_widget = egui::widgets::Image::new((texture_id, size_ui));
        let response = ui.add(scene_texture_widget);

        let gizmo_state = world.get::<Self>(entity).unwrap().gizmo;
        gizmo_state.ui(world, &mut ui, entity);

        let (mut scene, mut camera_view, mut camera_proj) = world
            .query::<(&mut Self, &mut Transform, &mut Projection)>()
            .get_mut(world, entity)
            .unwrap();

        {
            let inner_frame = response.rect.shrink(16.0);

            let frame = egui::Frame::none()
                .inner_margin(2.0)
                .rounding(2.0)
                .fill(egui::Color32::from_gray(0x2D))
                .stroke(egui::Stroke {
                    width: scale.recip(),
                    color: egui::Color32::from_gray(0x19),
                });

            let size = egui::vec2(220.0, 25.0);

            let mut mode = ui.child_ui_with_id_source(
                egui::Align2::LEFT_TOP.align_size_within_rect(size, inner_frame),
                egui::Layout::left_to_right(egui::Align::Min),
                egui::Id::new((entity, "Scene::mode_select")),
            );

            let mut orientation = ui.child_ui_with_id_source(
                egui::Align2::CENTER_TOP.align_size_within_rect(size, inner_frame),
                egui::Layout::left_to_right(egui::Align::Min),
                egui::Id::new((entity, "Scene::orientation_select")),
            );

            let egui::InnerResponse { .. } = frame.show(&mut mode, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
                    ui.columns(3, |ui| {
                        let mode = &mut scene.gizmo.mode;
                        ui[0].selectable_value(mode, GizmoMode::Translate, "Translate");
                        ui[1].selectable_value(mode, GizmoMode::Rotate, "Rotate");
                        ui[2].selectable_value(mode, GizmoMode::Scale, "Scale");
                    });
                });
            });

            let egui::InnerResponse { .. } = frame.show(&mut orientation, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 0.0);
                    ui.columns(2, |ui| {
                        let mode = &mut scene.gizmo.orientation;
                        let global = format!("{} Global", crate::ui::icon::ORIENTATION_GLOBAL);
                        ui[0].selectable_value(mode, GizmoOrientation::Global, global);
                        let local = format!("{} Local", crate::ui::icon::ORIENTATION_LOCAL);
                        ui[1].selectable_value(mode, GizmoOrientation::Local, local);
                    });
                });
            });
        }

        let response = response.interact(egui::Sense::click_and_drag());

        let screen_rect = response.rect;
        {
            let mut state = InputState::default();
            if response.dragged_by(egui::PointerButton::Primary) {
                state.drag = Some(Drag {
                    delta: response.drag_delta(),
                    button: egui::PointerButton::Primary,
                });
            }
            if response.dragged_by(egui::PointerButton::Secondary) {
                state.drag = Some(Drag {
                    delta: response.drag_delta(),
                    button: egui::PointerButton::Secondary,
                });
            }
            if response.dragged_by(egui::PointerButton::Middle) {
                state.drag = Some(Drag {
                    delta: response.drag_delta(),
                    button: egui::PointerButton::Middle,
                });
            }

            state.hover_pos = response.hover_pos();

            ui.input(|input| {
                state.modifiers = input.modifiers;
                state.scroll = input.scroll_delta.y;
                state.forward = input.key_down(egui::Key::W);
                state.backward = input.key_down(egui::Key::S);
                state.left = input.key_down(egui::Key::A);
                state.right = input.key_down(egui::Key::D);
            });

            let mov_speed = 8.0;
            let pan_speed = 25.0;
            let rot_speed = 2.0;
            let scroll_speed = 0.2;

            camera_proj.update(size_px.x, size_px.y);

            if let Projection::Perspective(proj) = camera_proj.as_mut() {
                if let Some(drag) = state.drag.take() {
                    let delta = drag.delta / size_px;
                    let ratio = proj.aspect_ratio;
                    let fov = proj.fov;
                    match drag.button {
                        egui::PointerButton::Middle => {
                            let pan = delta * egui::Vec2::new(fov * ratio, fov);
                            let right = camera_view.rotation * Vec3::X * -pan.x;
                            let up = camera_view.rotation * Vec3::Y * pan.y;
                            camera_view.translation += (right + up) * pan_speed;
                        }
                        egui::PointerButton::Secondary => {
                            scene.yew += delta.x * fov * ratio * rot_speed;
                            scene.pitch += delta.y * fov * rot_speed;

                            camera_view.rotation =
                                Quat::from_euler(EulerRot::YXZ, scene.yew, scene.pitch, 0.0);
                        }
                        _ => (),
                    }
                }
            }

            let mut movement = Vec3::ZERO;

            movement -= Vec3::Z * if state.forward { 1.0 } else { 0.0 };
            movement += Vec3::Z * if state.backward { 1.0 } else { 0.0 };
            movement -= Vec3::X * if state.left { 1.0 } else { 0.0 };
            movement += Vec3::X * if state.right { 1.0 } else { 0.0 };

            movement = movement.normalize_or_zero();
            if state.hover_pos.is_some() {
                movement -= Vec3::Z * state.scroll * scroll_speed;
            }

            let movement = camera_view.rotation * movement;
            camera_view.translation += movement * ui.input(|input| input.predicted_dt) * mov_speed;
        }

        let proj = camera_proj.get_projection_matrix();
        let view = camera_view.compute_matrix().inverse();
        let world_to_ndc = proj * view;
        let ndc_to_world = view.inverse() * proj.inverse();

        let world_to_screen_and_z = |world: Vec3| {
            let ndc = world_to_ndc.project_point3(world);

            // NDC z-values outside of 0 < z < 1 are outside the camera frustum and are thus not in screen space
            if ndc.is_nan() || ndc.z < 0.0 || ndc.z > 1.0 {
                None
            } else {
                // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
                let pos = screen_rect.min + egui::vec2(ndc.x + 1.0, 1.0 - ndc.y) / 2.0 * size_ui;
                Some((pos, 1.0 - ndc.z))
            }
        };

        let mut zsorting = std::mem::take(&mut scene.zsorting);
        zsorting.clear();

        let mut entities = world.query_filtered::<(Entity, &GlobalTransform), F>();
        zsorting.extend(entities.iter(world).filter_map(|(entity, &transform)| {
            let (pos, z) = world_to_screen_and_z(transform.affine().translation.into())?;
            Some((entity, z, pos, transform))
        }));

        // reverse z-sorting
        zsorting.sort_by(|(_, az, _, _), (_, bz, _, _)| f32::total_cmp(bz, az));

        let _world_to_screen = |world: Vec3| {
            let ndc = world_to_ndc.project_point3(world);

            // NDC z-values outside of 0 < z < 1 are outside the camera frustum and are thus not in screen space
            if ndc.is_nan() || ndc.z < 0.0 || ndc.z > 1.0 {
                None
            } else {
                // Once in NDC space, we can discard the z element and rescale x/y to fit the screen
                Some(screen_rect.min + egui::vec2(ndc.x + 1.0, 1.0 - ndc.y) / 2.0 * size_ui)
                // XXX: 1.0 - z
            }
        };

        let _screen_to_world = |screen: egui::Pos2, z: f32| {
            let local = ((screen - screen_rect.min) * 2.0) / size_ui;
            ndc_to_world.project_point3(Vec3::new(local.x - 1.0, 1.0 - local.y, 1.0 - z))
        };

        ui.set_clip_rect(screen_rect);

        let click_pos = response.clicked().then(|| response.hover_pos()).flatten();

        //let mouse = ui.input(|p| p.pointer.hover_pos());
        //let selected = selection.lock_or(None);

        let mut next_select = None;
        let mut selection = world.resource_mut::<Selection>();

        for &(entity, _z, center, _transform) in &zsorting {
            let size = egui::vec2(16.0, 16.0);
            let rect = egui::Rect::from_center_size(center, size);
            let inner = ui.allocate_rect(rect, egui::Sense::hover());

            let mut color = egui::Color32::WHITE;
            if inner.hovered() {
                color = from_raw(clrs::YELLOW);
                if click_pos.is_some() {
                    next_select = Some(entity);
                }
            }

            fn from_raw([r, g, b, _]: [u8; 4]) -> egui::Color32 {
                egui::Color32::from_rgb(r, g, b)
            }

            let painter = ui.painter();

            /*
            let is_selected = selection.contains(entity);

            let get_pos = |dir, color| {
                world_to_screen(Vec3::from(transform.affine().translation) + dir)
                    .map(|p| (p, (1.0, from_raw(color))))
            };
            */

            // if entity.has::<ProxyPointLight>() {
            //     painter.text(
            //         center,
            //         egui::Align2::CENTER_CENTER,
            //         icon::LIGHT_POINT,
            //         egui::FontId::proportional(20.0),
            //         color,
            //     );
            // } else {
            let fill = color;
            let stroke = (1.0, egui::Color32::BLACK);
            painter.circle(center, 4.0, fill, stroke);
            //}
        }

        if let Some(entity) = next_select {
            selection.select_replace(entity);
        }

        let mut scene = world.query::<&mut Self>().get_mut(world, entity).unwrap();
        scene.zsorting = zsorting;
    }
}

/*
fn dist(a: egui::Pos2, b: egui::Pos2, p: egui::Pos2) -> f32 {
    let ab = a - b;
    let ba = b - a;
    let pb = p - b;
    let pa = p - a;

    if ab.x * pb.x + ab.y * pb.y <= 0.0 {
        return (pb.x * pb.x + pb.y * pb.y).sqrt();
    }

    if ba.x * pa.x + ba.y * pa.y <= 0.0 {
        return (pa.x * pa.x + pa.y * pa.y).sqrt();
    }

    (ba.y * p.x - ba.x * p.y + b.x * a.y - b.y * a.x).abs() / (ab.y * ab.y + ab.x * ab.x).sqrt()
}
*/

// see http://clrs.cc
pub mod clrs {
    #![allow(clippy::unreadable_literal, dead_code)]

    const fn rgb(c: u32) -> [u8; 4] {
        let [b, g, r, _] = c.to_le_bytes();
        [r, g, b, 0xFF]
    }

    pub const X_AXIS: [u8; 4] = RED;
    pub const Y_AXIS: [u8; 4] = GREEN;
    pub const Z_AXIS: [u8; 4] = BLUE;

    pub const XY_PLANE: [u8; 4] = Z_AXIS;
    pub const XZ_PLANE: [u8; 4] = Y_AXIS;
    pub const YZ_PLANE: [u8; 4] = X_AXIS;

    pub const GRID_COLOR: [u8; 4] = GRAY;

    pub const NAVY: [u8; 4] = rgb(0x001F3F);
    pub const BLUE: [u8; 4] = rgb(0x0074D9);
    pub const AQUA: [u8; 4] = rgb(0x7FDBFF);
    pub const TEAL: [u8; 4] = rgb(0x39CCCC);
    pub const OLIVE: [u8; 4] = rgb(0x3D9970);
    pub const GREEN: [u8; 4] = rgb(0x2ECC40);
    pub const LIME: [u8; 4] = rgb(0x01FF70);
    pub const YELLOW: [u8; 4] = rgb(0xFFDC00);
    pub const ORANGE: [u8; 4] = rgb(0xFF851B);
    pub const RED: [u8; 4] = rgb(0xFF4136);
    pub const MAROON: [u8; 4] = rgb(0x85144B);
    pub const FUCHSIA: [u8; 4] = rgb(0xF012BE);
    pub const PURPLE: [u8; 4] = rgb(0xB10DC9);
    pub const BLACK: [u8; 4] = rgb(0x111111);
    pub const GRAY: [u8; 4] = rgb(0xAAAAAA);
    pub const SILVER: [u8; 4] = rgb(0xDDDDDD);
    pub const WHITE: [u8; 4] = rgb(0xFFFFFF);
}
