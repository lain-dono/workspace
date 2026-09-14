use crate::inspector::{guess_entity_name, integration::ui_for_entity_components};
use crate::reflect_editor::{Arg, OptionsType, ReflectEditor};
use bevy::ecs::{entity::Entity, system::CommandQueue};
use std::any::Any;

#[derive(Clone)]
#[non_exhaustive]
pub struct EntityOptions {
    pub display: EntityDisplay,
    pub despawnable: bool,
}

impl Default for EntityOptions {
    fn default() -> Self {
        Self {
            display: EntityDisplay::default(),
            despawnable: true,
        }
    }
}

#[derive(Copy, Clone, Default)]
#[non_exhaustive]
pub enum EntityDisplay {
    #[default]
    Id,
    Components,
}

impl OptionsType for Entity {
    type Derive = EntityOptions;
    type Options = EntityOptions;

    fn options_from_derive(derive: Self::Derive) -> Self::Options {
        derive
    }
}

pub fn entity_ref(_: ReflectEditor, Arg { ui, .. }: Arg, value: &dyn Any) {
    let entity = value.downcast_ref::<Entity>().unwrap();
    ui.label(format!("{entity:?}"));
}

pub fn entity_mut(env: ReflectEditor, Arg { ui, opt, id }: Arg, value: &mut dyn Any) -> bool {
    let entity = *value.downcast_ref::<Entity>().unwrap();
    let opt = opt.downcast_or_default::<EntityOptions>();

    match opt.display {
        EntityDisplay::Id => {
            ui.label(format!("{entity:?}"));
        }
        EntityDisplay::Components => {
            let entity_name = guess_entity_name(env.world, entity);
            egui::CollapsingHeader::new(entity_name)
                .id_source(id)
                .show(ui, |ui| {
                    let _queue = CommandQueue::default();
                    ui_for_entity_components(env.world, env.queue, entity, ui, id, env.registry);
                    if opt.despawnable && env.world.get_entity(entity).is_some() {
                        let text = egui::RichText::new("✖ Despawn").color(egui::Color32::RED);
                        if ui.add(egui::Button::new(text)).clicked() {
                            env.queue.push(bevy::ecs::system::Despawn { entity });
                        }
                    }
                });
        }
    }

    false
}
