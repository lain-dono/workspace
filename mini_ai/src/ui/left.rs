use bevy::{ecs::component::Components, prelude::*};
use bevy_egui::{EguiContexts, egui};

pub fn panel(
    mut contexts: EguiContexts,
    actors: Single<(Entity, &mut ai::Actor, Option<&ai::CurrentAction>)>,
    targets: Query<(Entity, &Name, &ai::Interactive)>,
    mut interactions: Query<(Entity, &Name, &mut ai::Interaction, &ai::Action)>,
    components: &Components,
) -> Result {
    let (actor_entity, mut actor, current_action) = actors.into_inner();

    let panel = egui::SidePanel::left("left_panel").min_width(300.0);
    panel.show(contexts.ctx_mut()?, |ui| {
        let top_spacing_amount = ui.spacing().window_margin.top;
        ui.add_space(top_spacing_amount.into());

        ui.horizontal(|ui| {
            ui.heading(format!("Actor {actor_entity}"));
            let name = current_action
                .and_then(|current| components.get_name(current.action.0))
                .unwrap_or("None".into());

            ui.label(format!("{name}"));
        });

        for (index, motive) in actor.motives.iter_mut().enumerate() {
            let name = crate::MOTIVE_NAMES[index];
            ui.add(egui::Slider::new(&mut motive.current, 0.0..=100.0).text(name));
        }

        ui.separator();
        ui.heading("Objects:");

        for (entity, name, entities) in targets {
            let heading = format!("{entity} {name}");
            let ch = egui::CollapsingHeader::new(heading).default_open(true);
            ch.show(ui, |ui| {
                // let mut interactions = interactions.reborrow();
                // let mut iter = interactions.iter_many_mut(entities);
                for &entity in entities {
                    let (_, name, mut its, aciton) = interactions.get_mut(entity).unwrap();
                    let action_name = components.get_name(aciton.0).unwrap_or("".into());
                    ui.label(format!("{entity} {name} ({action_name})"));
                    ui.indent(entity, |ui| {
                        for ai::Ad { idx, min, max, .. } in &mut its.advertising {
                            ui.horizontal(|ui| {
                                let min = egui::DragValue::new(min).prefix("min ");
                                let max = egui::DragValue::new(max).prefix("max ");

                                ui.label(crate::MOTIVE_NAMES[*idx]);
                                ui.add(min.speed(1.0).range(0.0..=100.0));
                                ui.add(max.speed(1.0).range(0.0..=100.0));
                            });
                        }
                    });
                }
            });
        }

        ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    });

    Ok(())
}
