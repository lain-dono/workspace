use crate::{
    inspector::guess_entity_name,
    reflect_editor::{errors, get_component_reflect, Arg, ReflectEditor},
    ui::{icon, EditorStage},
};
use bevy::ecs::{
    component::ComponentId, system::CommandQueue, world::unsafe_world_cell::UnsafeEntityCell,
};
use bevy::prelude::*;

const WRAP_WIDTH: f32 = 235.0;
pub const PERCENT: f32 = 0.35;
const ERR_COLOR: egui::Color32 = egui::Color32::GRAY;

#[derive(Default, Component)]
pub struct OutlinerTab;

impl OutlinerTab {
    pub fn add_panel(app: &mut App) {
        app.add_systems(Update, Self::panel_system.in_set(EditorStage::Tabs));
    }

    pub fn panel_system(world: &mut World) {
        super::run_panel_scrollbar::<With<Self>>(world, Self::run_panel);
    }

    fn run_panel(_entity: Entity, world: &mut World, ui: &mut egui::Ui) {
        let style = {
            let style = world.resource::<crate::ui::Style>();
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(rect, 0.0, style.panel_fill);
            style.clone()
        };

        let registry = world.resource::<AppTypeRegistry>().0.clone();
        let registry = registry.read();

        let selection = world.resource::<super::hierarchy::Selection>();
        let Some(entity) = selection.as_slice().first().copied() else {
            return;
        };

        let entity_name = guess_entity_name(world.as_unsafe_world_cell(), entity);
        let id = egui::Id::new((entity, "#outliner"));

        let frame = egui::Frame {
            inner_margin: egui::Margin {
                left: 2.0,
                right: 2.0,
                top: 4.0,
                bottom: 6.0,
            },
            ..default()
        };

        let mut queue = CommandQueue::default();

        let skip_components = [
            world.component_id::<Name>().unwrap(),
            // hierarhcy
            world.component_id::<Parent>().unwrap(),
            world.component_id::<Children>().unwrap(),
            // visibility
            world.component_id::<Visibility>().unwrap(),
            world.component_id::<InheritedVisibility>().unwrap(),
            world.component_id::<ViewVisibility>().unwrap(),
            // transform
            //world.component_id::<Transform>().unwrap(),
            //world.component_id::<GlobalTransform>().unwrap(),
        ];

        frame.show(ui, |ui| {
            //ui.heading(entity_name);

            let world = world.as_unsafe_world_cell();

            let Some(entity_ref) = world.get_entity(entity) else {
                return errors::entity_does_not_exist(ui, entity);
            };

            let archetype = entity_ref.archetype();

            show_meta(ui, &style, entity_ref, &mut queue, entity_name);

            let filter = |id: &ComponentId| !skip_components.contains(id);

            for component_id in archetype.components().filter(filter) {
                let info = world.components().get_info(component_id).unwrap();
                let name = pretty_type_name::pretty_type_name_str(info.name());

                let Some(type_id) = info.type_id() else {
                    errors::no_type_id(ui, &name);
                    continue;
                };

                let id = id.with(component_id);
                let mut state = State::load(ui.ctx(), id);

                let frame = egui::Frame {
                    fill: egui::Color32::from_gray(0x32),
                    rounding: egui::Rounding::same(2.0),
                    ..default()
                };

                frame.show(ui, |ui| {
                    let state = (info.layout().size() != 0).then_some(&mut state);
                    if component_header(state, ui, &style, &name, extra_dots) {
                        let margin = egui::Margin {
                            left: 9.0,
                            right: 9.0,
                            top: 0.0,
                            bottom: 6.0,
                        };
                        let frame = egui::Frame::none();
                        frame.inner_margin(margin).show(ui, |ui| {
                            // create a context with access to the world except for the currently viewed component

                            let view =
                                unsafe { get_component_reflect(&registry, world, entity, type_id) };

                            let mut value = match view {
                                Ok(value) => value,
                                Err(error) => return error.show(ui, &name),
                            };

                            /*
                            let max_rect = ui.max_rect();
                            let layout = egui::Layout::top_down_justified(egui::Align::Max);
                            let mut ui =
                                ui.child_ui_with_id_source(max_rect, layout, id.with("#ui"));
                            */
                            //ui.expand_to_include_x(dbg!(ui.available_size_before_wrap().x));

                            let bypass = value.bypass_change_detection();
                            let mut editor = ReflectEditor::for_bevy(&registry, world, &mut queue);
                            if editor.reflect_mut(Arg::id(id, ui), bypass) {
                                value.set_changed();
                            }
                        });
                    }
                });

                state.store(ui.ctx(), id);
            }
        });

        queue.apply(world);
    }
}

/*
fn run_meta(ui: &mut egui::Ui, style: &crate::ui::Style, entity_ref: UnsafeEntityCell<'_>) {
    unsafe {
        if let Some(mut name) = entity_ref.get_mut::<Name>() {
            let mut name_text = name.bypass_change_detection().to_string();
            if show_meta(ui, &style, &mut name_text) {
                name.set(name_text);
            }
        } else {
            let mut name = entity_name.clone();
            if show_meta(ui, &style, &mut name) {
                queue.push(bevy::ecs::system::Insert {
                    entity: entity_ref.id(),
                    bundle: Name::new(name),
                });
            }
        }
    }
}
*/

fn show_meta(
    ui: &mut egui::Ui,
    style: &crate::ui::Style,
    entity: UnsafeEntityCell,
    queue: &mut CommandQueue,
    guess_name: String,
) {
    let mut name = if let Some(mut name) = unsafe { entity.get_mut::<Name>() } {
        name.bypass_change_detection().to_string()
    } else {
        guess_name
    };

    ui.horizontal(|ui| {
        let frame = egui::Frame::none().inner_margin(egui::style::Margin::symmetric(2.0, 3.0));

        frame.fill(style.tab_base).show(ui, |ui| {
            let mut icon = crate::ui::icon::MESH_CUBE as u32;

            //let icon_field = data.get_field_mut::<u32>("icon").unwrap();
            let icon_field = &mut icon;

            let icon_char = char::from_u32(*icon_field).unwrap();
            let icon = egui::text::LayoutJob::simple_singleline(
                icon_char.into(),
                egui::FontId::proportional(13.0),
                style.input_text,
            );

            let egui::InnerResponse { inner, response } = ui.menu_button(icon, |ui| {
                style.for_scrollbar(ui);
                let scroll = egui::ScrollArea::vertical().auto_shrink([false; 2]);
                scroll.id_source("inspector icons").show(ui, |ui| {
                    style.set_theme_visuals(ui);
                    style.scrollarea(ui);
                    ui.set_width(300.0);
                    ui.horizontal_wrapped(|ui| {
                        for c in 0xE900..=0xEB99 {
                            let c = char::from_u32(c).unwrap();
                            if ui.button(String::from(c)).clicked() {
                                ui.close_menu();
                                return Some(c);
                            }
                        }

                        None
                    })
                })
            });
            response.on_hover_cursor(egui::CursorIcon::PointingHand);

            if let Some(icon) = inner.and_then(|r| r.inner.inner) {
                *icon_field = icon as u32;
            }

            {
                let visibility = unsafe { entity.get::<Visibility>().cloned() };
                let mut current = visibility.unwrap_or_default();

                egui::ComboBox::from_id_source((entity.id(), "#visibility"))
                    .width(70.0)
                    .selected_text(format!("{:?}", current))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut current, Visibility::Inherited, "Inherited");
                        ui.selectable_value(&mut current, Visibility::Hidden, "Hidden");
                        ui.selectable_value(&mut current, Visibility::Visible, "Visible");
                    });

                if visibility != Some(current) {
                    queue.push(bevy::ecs::system::Insert {
                        entity: entity.id(),
                        bundle: current,
                    });
                }
            }

            // ui.add_space(4.0);

            let text = egui::TextEdit::singleline(&mut name);

            if ui.add(text.desired_width(f32::INFINITY)).changed() {
                queue.push(bevy::ecs::system::Insert {
                    entity: entity.id(),
                    bundle: Name::new(name),
                });
            }

            //
        });
    });

    /*
    if false {
        let icon = if self.lock.is_some() {
            crate::blender::LOCKED
        } else {
            crate::blender::UNLOCKED
        };
        let widget = Button::new(icon.to_string()).frame(false);
        if ui.add(widget).clicked() {
            if self.lock.is_some() {
                self.lock.take();
            } else {
                self.lock = Some(entity.index);
            }
        }
        ui.add_space(4.0);
    }
    */
}

fn extra_dots(ui: &mut egui::Ui, style: &crate::ui::Style, rect: egui::Rect) -> egui::Rect {
    let height = rect.height();
    let font_id = egui::FontId::proportional(14.0);
    //let dots_pos = rect.right_top() - egui::vec2(12.0, 0.0);
    let pos = rect.right_top() + egui::vec2(-10.0, (height - font_id.size) / 2.0);

    let anchor = egui::Align2::CENTER_TOP;
    //icon::GRIP,
    let color = style.input_text;
    let icon = icon::THREE_DOTS;
    let rect = ui.painter().text(pos, anchor, icon, font_id, color);

    rect
}

pub fn component_header(
    state: Option<&mut State>,
    ui: &mut egui::Ui,
    style: &crate::ui::Style,
    name: &str,
    extra: impl FnOnce(&mut egui::Ui, &crate::ui::Style, egui::Rect) -> egui::Rect,
) -> bool {
    use egui::*;

    let icon_size = FontId::proportional(14.0);
    let label_size = FontId::proportional(10.0);
    let dots_size = FontId::proportional(16.0);
    let tri_size = FontId::proportional(16.0);

    let height = 25.0;

    let tri_color = style.input_text;
    let text_color = style.input_text;

    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    let tri_pos = rect.left_top() + vec2(11.0, 0.0);
    //let icon_pos = rect.left_center() + vec2(24.0, 0.0);
    //let label_pos = rect.left_center() + vec2(38.0, 0.0);
    let label_pos = rect.left_center() + vec2(21.0, 0.0);
    let dots_pos = rect.right_top() - vec2(12.0, 0.0);

    /*
    ui.painter().text(icon_pos, Align2::CENTER_CENTER, icon, icon_size, text_color);
    */

    ui.painter()
        .text(label_pos, Align2::LEFT_CENTER, name, label_size, text_color);

    let dots_rect = extra(ui, style, rect);

    if false {
        /*
        let left_x = egui::lerp(rect.min.x..=rect.max.x, PERCENT);
        let rect = rect.intersect(Rect::everything_right_of(left_x + 7.0));
        let rect = rect.intersect(Rect::everything_left_of(dots_rect.min.x));
        let rect = rect.shrink2(vec2(0.0, 1.0));

        let layout = Layout::top_down(Align::Min);
        let mut ui = ui.child_ui(rect, layout);
        extra(&mut ui)
        */
    }

    if let Some(state) = state {
        if response.clicked() {
            state.open = !state.open;
            ui.ctx().request_repaint();
        }
        let tri_icon = if state.open {
            icon::DISCLOSURE_TRI_DOWN
        } else {
            icon::DISCLOSURE_TRI_RIGHT
        };

        let tri_icon = if state.open {
            icon::DOWNARROW_HLT
        } else {
            icon::RIGHTARROW_THIN
        };

        ui.painter().text(
            tri_pos + egui::vec2(0.0, height - tri_size.size) / 2.0,
            Align2::CENTER_TOP,
            tri_icon,
            tri_size,
            tri_color,
        );
        state.open
    } else {
        false
    }
}

#[derive(Clone, Debug, Default)]
pub struct State {
    open: bool,
}

impl State {
    pub fn load(ctx: &egui::Context, id: egui::Id) -> Self {
        ctx.data(|data| data.get_temp(id).unwrap_or(Self { open: true }))
    }

    pub fn store(self, ctx: &egui::Context, id: egui::Id) {
        ctx.data_mut(|data| data.insert_temp(id, self));
    }
}
