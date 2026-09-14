use super::runtime::{Lerp, Offset, Timeline, Transform};
use super::{armature::paint_bones, AnimaEditorContext, Grid};
use crate::ui::{icon, EditorPanel, EditorTab, PanelRenderTarget, Style};
use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::{Assets, Commands, Image};
use egui::*;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SetupMode {
    #[default]
    Pose,
    Edit,
    Weight,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Setup,
    Animate,
}

#[derive(Default, bevy::prelude::Component)]
pub struct Animation2d {
    pub setup_mode: SetupMode,
    pub mode: Mode,
    pub grid: Grid,
    pub selected: Option<usize>,
}

impl Animation2d {
    pub fn spawn(commands: &mut Commands, images: &mut Assets<Image>) -> crate::ui::Tab {
        use bevy::prelude::*;

        let target = PanelRenderTarget::create_render_target(images);

        let clear_color = bevy::prelude::Color::rgba(0.0, 0.0, 0.0, 0.0);
        let clear_color = ClearColorConfig::Custom(clear_color);

        let entity = commands
            .spawn((
                Camera2dBundle {
                    camera_2d: Camera2d { clear_color },
                    camera: Camera {
                        target,
                        order: 1,
                        ..default()
                    },
                    ..default()
                },
                Self::default(),
                EditorPanel::default(),
                reui::plugin::Recorder::default(),
                PanelRenderTarget::default(),
            ))
            .id();

        crate::ui::Tab::new(crate::ui::icon::VIEW_ORTHO, "Animate 2d", entity)
    }
}

impl EditorTab for Animation2d {
    type Param = (
        SQuery<Read<PanelRenderTarget>>,
        SRes<Style>,
        AnimaEditorContext<'static, 'static>,
    );

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        (target, style, editor): &mut SystemParamItem<'_, '_, Self::Param>,
    ) {
        let frame = ui.available_rect_before_wrap();
        ui.painter().rect_filled(frame, 0.0, style.panel_fill);

        let inner_frame = frame.shrink(16.0);

        let pointer = ui.input(|i| i.pointer.hover_pos().filter(|&p| frame.contains(p)));

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = 1.0;
            ui.spacing_mut().item_spacing.y = 2.0;
            style.set_theme_visuals(ui);

            ui.set_clip_rect(frame);

            let ppi = ui.ctx().pixels_per_point();
            let px = ppi.recip();

            self.grid.update(ui, frame);
            let viewport = self.grid.viewport(pointer, frame);

            let hovered = {
                editor.state.world_to_screen(viewport.matrix());

                let mut shapes = Vec::new();
                self.grid.paint(ui, frame, pointer, &mut shapes);

                if let Some(texture_id) = target.get(entity).ok().and_then(|t| t.texture_id) {
                    let rect = frame;
                    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
                    let mut mesh = Mesh::with_texture(texture_id);
                    mesh.add_rect_with_uv(rect, uv, Color32::WHITE);
                    ui.painter().add(Shape::mesh(mesh));
                }

                let hovered = paint_bones(ui, viewport, &editor.state, &mut shapes, self.selected);
                ui.painter().extend(shapes);

                hovered
            };

            let frame = egui::Frame::none()
                .inner_margin(2.0)
                .rounding(2.0)
                .fill(style.panel_fill)
                .stroke(Stroke {
                    width: px,
                    color: Color32::from_gray(0x19),
                });

            let size = vec2(220.0, 25.0);
            let mut mode_ui = ui.child_ui_with_id_source(
                Align2::LEFT_TOP.align_size_within_rect(size, inner_frame),
                Layout::left_to_right(Align::Min),
                Id::new("animation2d::mode_select"),
            );

            let size = vec2(220.0, 25.0);
            let mut setup_mode = ui.child_ui_with_id_source(
                Align2::LEFT_BOTTOM.align_size_within_rect(size, inner_frame),
                Layout::left_to_right(Align::Min),
                Id::new("animation2d::mode_select"),
            );

            let size = vec2(180.0, 80.0);
            let mut controls_ui = ui.child_ui_with_id_source(
                Align2::CENTER_BOTTOM.align_size_within_rect(size, inner_frame),
                Layout::top_down(Align::Center),
                Id::new("animation2d::bottom_control"),
            );

            let InnerResponse { response, .. } = frame.show(&mut mode_ui, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                    ui.columns(2, |ui| {
                        let text = "Setup";
                        ui[0].selectable_value(&mut self.mode, Mode::Setup, text);
                        let text = "Animate";
                        ui[1].selectable_value(&mut self.mode, Mode::Animate, text);
                    });
                });
            });

            let cursor_in_mode = response.hover_pos().is_some();

            let InnerResponse { response, .. } = frame.show(&mut setup_mode, |ui| {
                ui.scope(|ui| {
                    ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
                    ui.columns(3, |ui| {
                        let text = format!("{} Pose", icon::POSE_HLT);
                        ui[0].selectable_value(&mut self.setup_mode, SetupMode::Pose, text);
                        let text = format!("{} Edit", icon::EDITMODE_HLT);
                        ui[1].selectable_value(&mut self.setup_mode, SetupMode::Edit, text);
                        let text = format!("{} Weight", icon::WPAINT_HLT);
                        ui[2].selectable_value(&mut self.setup_mode, SetupMode::Weight, text);
                    });
                });
            });

            let cursor_in_setup_mode = response.hover_pos().is_some();

            let cursor_in_controls = if let Some(bone) = self.selected {
                frame
                    .show(&mut controls_ui, |ui| {
                        editor.transform_widget(ui, bone, self.mode)
                    })
                    .response
                    .hover_pos()
                    .is_some()
            } else {
                false
            };

            let cursor_in_ui = cursor_in_mode || cursor_in_setup_mode || cursor_in_controls;

            if pointer.is_some() && !cursor_in_ui {
                ui.ctx()
                    .output_mut(|o| o.cursor_icon = CursorIcon::Crosshair);
                if ui.input(|i| i.pointer.any_click()) {
                    self.selected = hovered;
                }
            }
        });
    }
}

impl<'w, 's> AnimaEditorContext<'w, 's> {
    fn transform_widget(
        &mut self,
        ui: &mut Ui,
        bone_index: usize,
        mode: Mode,
    ) -> InnerResponse<()> {
        let data: &mut super::AnimaData = &mut self.data;
        let state: &mut super::AnimaState = &mut self.state;

        let current_time = state.current_time;
        let current_clip = state.current_clip;

        let clip = &mut data.clips[current_clip as usize];

        let bone = BoneEditor {
            clip: &mut clip.bones[bone_index],
            state: &mut state.bones[bone_index],
            data: &mut data.bones[bone_index],
        };

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

            ui.horizontal(|ui| {
                ui.add_space(2.0);

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, 5.0);
                    ui.add_space(2.0);
                    ui.label("rotate");
                    ui.label("translate");
                    ui.label("scale");
                    ui.label("shear");
                });

                ui.add_space(4.0);

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = vec2(1.0, 1.0);

                    match mode {
                        Mode::Setup => {
                            if let Some(transform) = bone.data.transform().ui(ui) {
                                bone.data.set_transform(transform);
                            }
                        }
                        Mode::Animate => {
                            let clip = bone.clip.resolve(current_time as f32);
                            let transform = bone.state.transform.mul_transform(clip);
                            if let Some(transform) = transform.ui(ui) {
                                bone.state.transform = transform.mul_transform(clip.inverse());
                            }
                        }
                    };
                });

                ui.vertical(|ui| {
                    fn toggle_key<T: Lerp>(
                        ui: &mut Ui,
                        time: u32,
                        timeline: &mut Timeline<T>,
                        default: T,
                        pose: T,
                    ) {
                        let cond = timeline.get(time).is_some();
                        let icon = if cond { icon::KEYFRAME_HLT } else { icon::DOT };
                        let btn = Button::new(icon.to_string()).frame(false);
                        if ui.add(btn).clicked() {
                            if cond {
                                timeline.remove(time);
                            } else {
                                let value = timeline.resolve(time as f32).unwrap_or(default);
                                timeline.add_linear(time, value);
                            }
                        }
                    }

                    ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
                    ui.add_space(2.0);

                    let pose = bone.state.transform.rotate;
                    toggle_key(ui, current_time, &mut bone.clip.rotate, 0.0, pose);
                    let pose: Offset = bone.state.transform.translate.into();
                    toggle_key(
                        ui,
                        current_time,
                        &mut bone.clip.translate,
                        Offset::new(0.0, 0.0),
                        pose,
                    );
                    let pose: Offset = bone.state.transform.scale.into();
                    toggle_key(
                        ui,
                        current_time,
                        &mut bone.clip.scale,
                        Offset::new(1.0, 1.0),
                        pose,
                    );
                    let pose: Offset = bone.state.transform.shear.into();
                    toggle_key(
                        ui,
                        current_time,
                        &mut bone.clip.shear,
                        Offset::new(0.0, 0.0),
                        pose,
                    );
                });
            });
        })
    }
}

struct BoneEditor<'a> {
    state: &'a mut super::runtime::Bone,
    clip: &'a mut super::runtime::BoneClipData,
    data: &'a mut super::runtime::BoneData,
}

impl Transform {
    fn ui(mut self, ui: &mut Ui) -> Option<Transform> {
        let rotate = ui.columns(1, |ui| drag_angle(&mut ui[0], &mut self.rotate));

        let translate = ui.columns(2, |ui| {
            let x = drag_value(&mut ui[0], &mut self.translate[0], 1.0);
            let y = drag_value(&mut ui[1], &mut self.translate[1], 1.0);
            x | y
        });

        let scale = ui.columns(2, |ui| {
            let x = drag_value(&mut ui[0], &mut self.scale[0], 0.1);
            let y = drag_value(&mut ui[1], &mut self.scale[1], 0.1);
            x | y
        });

        let shear = ui.columns(2, |ui| {
            let x = drag_angle(&mut ui[0], &mut self.shear[0]);
            let y = drag_angle(&mut ui[1], &mut self.shear[1]);
            x | y
        });

        (rotate | translate | scale | shear).then_some(self)
    }
}

pub fn drag_value(ui: &mut Ui, value: &mut f32, speed: f32) -> bool {
    ui.add(DragValue::new(value).speed(speed)).changed()
}

pub fn drag_angle(ui: &mut Ui, radians: &mut f32) -> bool {
    let mut degrees = radians.to_degrees();
    let mut response = ui.add(DragValue::new(&mut degrees).speed(1.0).suffix("°"));

    // only touch `*radians` if we actually changed the degree value
    if degrees != radians.to_degrees() {
        *radians = degrees.to_radians();
        response.changed = true;
    }

    response.changed
}
