use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use workspace::Workspace;

mod app;
mod figma;
mod icon;
mod style;
mod widget_gallery;

mod vl;

fn main() {
    //  vl::_main();
    //  return;

    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .insert_resource(bevy::winit::WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Reactive {
                wait: std::time::Duration::from_secs_f64(1.0 / 60.0),
                react_to_device_events: true,
                react_to_user_events: true,
                react_to_window_events: true,
            },
            unfocused_mode: bevy::winit::UpdateMode::Reactive {
                wait: std::time::Duration::from_secs_f64(1.0),
                react_to_device_events: true,
                react_to_user_events: true,
                react_to_window_events: true,
            },
        });

    // app.add_plugins(editor_ui_plugin);
    // app.add_systems(Startup, setup)

    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    });

    app.add_systems(
        EguiPrimaryContextPass,
        system_add_fonts
            .run_if(run_once)
            .before(crate::figma::main_ui),
    );

    // app.add_systems(Startup, system_add_fonts);

    app.add_systems(Startup, startup_app);
    app.add_systems(EguiPrimaryContextPass, app::docking);
    //app.add_plugins(crate::figma::plugin);

    // app.add_systems(EguiPrimaryContextPass, app::ui_ws);

    app.insert_resource(ClearColor(Color::BLACK));

    app.insert_resource(WsState {
        state: Workspace::default(),
    });

    app.run();
}

fn system_add_fonts(mut contexts: Query<&mut bevy_egui::EguiContext>) -> Result {
    // let ctx = contexts.ctx_mut()?;
    for mut ctx in &mut contexts {
        add_font(ctx.get_mut());
    }
    Ok(())
}

fn add_font(ctx: &bevy_egui::egui::Context) {
    use bevy_egui::egui;
    use bevy_egui::egui::epaint::text::{FontInsert, InsertFontFamily};
    use egui::FontFamily::{self, Monospace, Name, Proportional};
    use egui::{FontId, TextStyle};

    let fonts: [(_, _, &[u8]); _] = [
        (
            "Inter-Regular",
            FontFamily::Name("regular".into()),
            include_bytes!("../../assets/fonts/Inter/extras/ttf/Inter-Regular.ttf"),
        ),
        (
            "Inter-Bold",
            FontFamily::Name("bold".into()),
            include_bytes!("../../assets/fonts/Inter/extras/ttf/Inter-Bold.ttf"),
        ),
        (
            "Inter-Medium",
            FontFamily::Name("medium".into()),
            include_bytes!("../../assets/fonts/Inter/extras/ttf/Inter-Medium.ttf"),
        ),
        (
            "Inter-SemiBold",
            FontFamily::Name("semi-bold".into()),
            include_bytes!("../../assets/fonts/Inter/extras/ttf/Inter-SemiBold.ttf"),
        ),
        (
            "Inter-Italic",
            FontFamily::Name("italic".into()),
            include_bytes!("../../assets/fonts/Inter/extras/ttf/Inter-Italic.ttf"),
        ),
        (
            "Inter-BoldItalic",
            FontFamily::Name("bold-italic".into()),
            include_bytes!("../../assets/fonts/Inter/extras/ttf/Inter-BoldItalic.ttf"),
        ),
    ];

    for (name, family, font) in fonts {
        let priority = egui::epaint::text::FontPriority::Highest;
        let families = vec![InsertFontFamily { family, priority }];
        let data = egui::FontData::from_static(font);
        ctx.add_font(FontInsert::new(name, data, families));
    }

    let text_styles: std::collections::BTreeMap<TextStyle, FontId> = [
        (TextStyle::Small, FontId::new(9.0, Proportional)),
        (TextStyle::Body, FontId::new(13.0, Proportional)),
        (TextStyle::Button, FontId::new(13.0, Proportional)),
        (TextStyle::Heading, FontId::new(18.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, Monospace)),
        (
            TextStyle::Name("regular".into()),
            FontId::new(11.0, FontFamily::Name("regular".into())),
        ),
        (
            TextStyle::Name("title".into()),
            FontId::new(11.0, FontFamily::Name("medium".into())),
        ),
    ]
    .into();

    ctx.all_styles_mut(move |style| style.text_styles = text_styles.clone());
}

fn startup_app(mut commands: Commands) {
    use dock::tree::{DockState, NodeIndex};

    let tab1 = app::TabState::new("TAB_-_1", Entity::PLACEHOLDER);
    let tab2 = app::TabState::new("TAB_-_2", Entity::PLACEHOLDER);
    let tab3 = app::TabState::new("TAB_-_3", Entity::PLACEHOLDER);
    let tab4 = app::TabState::new("TAB_-_4", Entity::PLACEHOLDER);
    let tab5 = app::TabState::new("TAB_-_5", Entity::PLACEHOLDER);

    let mut state = DockState::new(vec![tab1, tab2]);

    let tree = state.main_surface_mut();

    // You can modify the tree before constructing the dock
    let [a, b] = tree.split_left(NodeIndex::root(), 0.3, vec![tab3]);
    let [_, _] = tree.split_below(a, 0.7, vec![tab4]);
    let [_, _] = tree.split_below(b, 0.5, vec![tab5]);

    commands.insert_resource(app::DockApp { state, counter: 0 });
}

#[derive(Resource)]
pub struct WsState {
    pub state: Workspace,
}

pub fn ui_ws(mut contexts: EguiContexts, mut state: ResMut<WsState>) -> Result {
    let ctx = contexts.ctx_mut()?;

    let rect = ctx.content_rect();

    state.state.draw(ctx, rect);

    Ok(())
}
