use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use egui_plot::{Bar, BarChart, Legend, Line, PlotItem, PlotPoints, PlotUi, Points};

mod bottom;
mod left;

#[derive(Resource, Clone)]
pub struct Context {
    current_motive: Option<usize>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            current_motive: Some(0),
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_plugins(EguiPlugin::default())
        .init_resource::<Context>()
        .add_systems(
            EguiPrimaryContextPass,
            (bottom::panel, left::panel, center_panel).chain(),
        );
}

fn select_motive(ui: &mut egui::Ui, context: &mut Context, actor: &mut ai::Actor) {
    for index in 0..actor.motives.len() {
        let checked = context.current_motive == Some(index);
        let text = format!("curve {index}");
        if ui.selectable_label(checked, text).clicked() {
            context.current_motive = if checked { None } else { Some(index) };
        }
    }
}

fn mark(value: f32, step_size: f32) -> egui_plot::GridMark {
    egui_plot::GridMark {
        value: f64::from(value),
        step_size: f64::from(step_size),
    }
}

fn ui_threshold(ui: &mut PlotUi, threshold: f32) {
    let color = egui::Color32::from_white_alpha(0x04);
    let bar = Bar::new(50.0, f64::from(threshold));
    let bar = BarChart::new("threshold", vec![bar.stroke((0.0, color)).fill(color)]);
    ui.bar_chart(bar.width(100.0).color(color).allow_hover(false));
}

pub fn center_panel(
    mut context: ResMut<Context>,
    mut contexts: EguiContexts,
    actors: Single<&mut ai::Actor>,
    mut interactions: Query<(Entity, &Name, &mut ai::Interaction, &ai::Action)>,
) -> Result {
    let mut actor = actors.into_inner();

    let add_contents = |ui: &mut egui::Ui| {
        let (_, _, mut interaction, _) = interactions.iter_mut().next().unwrap();
        let Context { current_motive, .. } = *context;

        let reward = &mut interaction.advertising[0].max;
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::Slider::new(reward, 0.0..=100.0).text("reward"));
            ui.separator();
            select_motive(ui, &mut context, &mut actor);
        });
        let reward = f64::from(*reward);

        if let Some(current_motive) = current_motive {
            curve_ui(&mut actor.motives[current_motive], ui);
        } else {
            curve_ui_space(ui);
        }

        let plot = egui_plot::Plot::new("plot")
            .allow_zoom(false)
            .allow_drag(false)
            .allow_scroll(false)
            .show_axes([false; 2])
            .show_background(false)
            .legend(Legend::default().follow_insertion_order(true))
            .default_x_bounds(-2., 102.0)
            .default_y_bounds(-1.1, 1.5);

        let plot = if let Some(current_motive) = current_motive {
            let keys = actor.motives[current_motive].curve.x;
            plot.x_grid_spacer(move |_input| {
                keys.map(|value| mark(f32::from(value) * 100.0 / 255.0, 10.0))
                    .into_iter()
                    .chain([mark(0.0, 100.0), mark(100.0, 100.0)])
                    .collect()
            })
        } else {
            plot
        };

        plot.show(ui, |ui| {
            ui_threshold(ui, actor.threshold);

            draw_guides(ui, &actor);
            edit_curves(ui, &mut actor, current_motive);
            draw_curves(ui, &actor);
            draw_scorers(ui, &actor, current_motive, reward);
            draw_scorers_value(ui, &actor, reward);
        })
    };

    egui::Window::new("center").show(contexts.ctx_mut()?, add_contents);
    // egui::CentralPanel::default().show(contexts.ctx_mut(), add_contents);

    Ok(())
}

fn draw_guides(ui: &mut PlotUi<'_>, actor: &ai::Actor) {
    for (index, &motive) in actor.motives.iter().enumerate() {
        for reward in (0..=100).step_by(5) {
            let color = acolor(index + 4).gamma_multiply_u8(0x5F);
            let name = format!("guides {index}");
            let scorer = move |need: f64| f64::from(motive.scorer(need as f32, reward as f32));
            let series = PlotPoints::from_explicit_callback(scorer, 0.0..100.0, 100);
            ui.line(Line::new(name, series).width(0.5).color(color));
        }
    }
}

fn edit_curves(ui: &mut PlotUi<'_>, actor: &mut ai::Actor, current_motive: Option<usize>) {
    #[derive(Clone, Copy)]
    struct DragPayload(usize);

    if let Some(index) = current_motive {
        let motive = actor.motives[index];
        let (name, color) = (format!("curve {index}"), acolor(index + 4));

        let p = std::array::from_fn::<_, 6, _>(|i| motive.curve.point(i));
        let p = p.map(|[x, y]| [f64::from(x), f64::from(y)]);
        let series = p.to_vec();
        let points = Points::new(&name, series).radius(2.5).color(color);

        let coord = ui.pointer_coordinate();
        let closest =
            coord.and_then(|point| points.find_closest(ui.screen_from_plot(point), ui.transform()));
        ui.points(points);

        let closest = closest.filter(|closest| closest.dist_sq < 16.0 * 16.0);
        let closest = closest.map(|elem| DragPayload(elem.index));

        if let Some(started) = closest.filter(|_| ui.response().drag_started()) {
            ui.response().dnd_set_drag_payload(started);
        }
        if let Some(dragged) = ui.response().dnd_hover_payload::<DragPayload>()
            && let Some(next) = ui.pointer_coordinate()
        {
            let [x, y] = [next.x, next.y].map(|v| v as f32);
            actor.motives[index].curve.set_point(dragged.0, x, y);
        }
        if ui.response().drag_stopped() {
            ui.response().dnd_release_payload::<DragPayload>();
        }

        if let Some(hovered) = closest.map(|payload| payload.0) {
            let points = Points::new(name, vec![p[hovered]]).radius(4.5).color(color);
            ui.points(points);
        }
    }
}

fn draw_curves(ui: &mut PlotUi<'_>, actor: &ai::Actor) {
    for (index, &motive) in actor.motives.iter().enumerate() {
        let (name, color) = (format!("curve {index}"), acolor(index + 4));
        let function = move |t: f64| f64::from(motive.curve.get(t as f32));
        let series = PlotPoints::from_explicit_callback(function, 0.0..100.0, 100);
        ui.line(Line::new(name, series).width(0.5).color(color));
    }
}

fn draw_scorers<'a>(
    ui: &mut PlotUi<'a>,
    actor: &'a ai::Actor,
    current_motive: Option<usize>,
    reward: f64,
) {
    for (index, motive) in actor.motives.iter().enumerate() {
        let (name, color) = (format!("scorer {index}"), acolor(index + 4));
        let scorer = move |need: f64| f64::from(motive.scorer(need as f32, reward as f32));
        let is_current = Some(index) == current_motive;
        let width = if is_current { 1.5 } else { 0.5 };
        let scorer = PlotPoints::from_explicit_callback(scorer, 0.0..100.0, 100);
        ui.line(Line::new(name, scorer).width(width).color(color));
    }
}

fn draw_scorers_value(ui: &mut PlotUi<'_>, actor: &ai::Actor, reward: f64) {
    for (index, motive) in actor.motives.iter().enumerate() {
        let need = f64::from(motive.current);
        let (name, color) = (format!("scorer {index}"), acolor(index + 4));
        let scorer = move |need: f64| f64::from(motive.scorer(need as f32, reward as f32));
        let series = vec![[need, scorer(need)]];
        ui.points(Points::new(name, series).radius(2.5).color(color));
    }
}

fn curve_ui_space(ui: &mut egui::Ui) {
    ui.add_space(ui.spacing().interact_size.y);
    ui.add_space(ui.spacing().item_spacing.y);
    ui.add_space(ui.spacing().interact_size.y);
    ui.add_space(ui.spacing().item_spacing.y);
}

fn curve_ui(motive: &mut ai::Motive, ui: &mut egui::Ui) {
    ui.columns_const::<6, _>(|ui| {
        let cx = motive.curve.x;
        for (i, (value, ui)) in motive.curve.x.iter_mut().zip(ui).enumerate() {
            let min = if i == 0 { 0x00 } else { cx[i - 1] + 1 };
            let max = if i == 5 { 0xFF } else { cx[i + 1] - 1 };
            ui.add(egui::DragValue::new(value).range(min..=max).speed(1.00));
        }
    });

    ui.columns_const::<6, _>(|ui| {
        for (value, ui) in motive.curve.y.iter_mut().zip(ui) {
            ui.add(egui::DragValue::new(value).speed(1.00));
        }
    });
}

fn acolor(i: usize) -> egui::Color32 {
    let golden_ratio = (5.0_f32.sqrt() - 1.0) / 2.0; // 0.61803398875
    let h = i as f32 * golden_ratio;
    egui::epaint::Hsva::new(h, 0.85, 0.5, 1.0).into() // TODO(emilk): OkLab or some other perspective color space
}
