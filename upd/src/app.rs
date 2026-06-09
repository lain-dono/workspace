use crate::widget_gallery::WidgetGallery;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use dock::{
    dock_area::{DockArea, DockAreaConfig},
    style::Style,
    tab_viewer::{OnCloseResponse, TabViewer},
    tree::{DockState, NodeIndex, SurfaceIndex},
};

#[derive(Resource)]
pub struct DockApp {
    pub state: DockState<TabState>,
    pub counter: usize,
}

pub fn docking(
    mut style: Local<Option<Style>>,
    mut added: Local<Vec<(SurfaceIndex, NodeIndex)>>,
    mut contexts: EguiContexts,
    mut app: ResMut<DockApp>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let style = style.get_or_insert_with(|| crate::style::init(ctx.style()));

    let config = DockAreaConfig {
        show_add_buttons: true,
        // show_add_popup: true,
        ..Default::default()
    };

    let area = DockArea::new(&mut app.state, style.clone(), config);

    let mut viewer = Viewer { added: &mut added };

    area.show(ctx, &mut viewer);
    viewer.spawn_tabs(&mut app);

    Ok(())
}

pub struct TabState {
    pub entity: Entity,
    pub label: String,
    pub wg: WidgetGallery,
}

impl TabState {
    pub fn new(label: impl Into<String>, entity: Entity) -> Self {
        Self {
            entity,
            label: label.into(),
            wg: crate::widget_gallery::WidgetGallery::default(),
        }
    }
}

struct Viewer<'a> {
    added: &'a mut Vec<(SurfaceIndex, NodeIndex)>,
}

impl Viewer<'_> {
    fn spawn_tabs(&mut self, app: &mut DockApp) {
        self.added.drain(..).for_each(|(surface, node)| {
            let tab = TabState::new(format!("{}", app.counter), Entity::PLACEHOLDER);
            app.state.set_focused_node_and_surface((surface, node));
            app.state.push_to_focused_leaf(tab);
            app.counter += 1;
        });
    }
}

impl TabViewer for Viewer<'_> {
    type Tab = TabState;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.label.clone().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {}", tab.label));
        tab.wg.ui(ui);
    }

    fn context_menu(
        &mut self,
        _ui: &mut egui::Ui,
        _tab: &mut Self::Tab,
        _surface: SurfaceIndex,
        _node: NodeIndex,
    ) {
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(self.title(tab).text())
    }

    fn on_tab_button(&mut self, _tab: &mut Self::Tab, _response: &egui::Response) {}

    fn on_close(&mut self, tab: &mut Self::Tab) -> OnCloseResponse {
        println!("Closed tab: {}", tab.label);
        OnCloseResponse::Close
    }

    fn is_closeable(&self, _tab: &Self::Tab) -> bool {
        true
    }

    fn force_close(&mut self, _tab: &mut Self::Tab) -> bool {
        false
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        self.added.push((surface, node));
    }

    fn on_rect_changed(&mut self, _tab: &mut Self::Tab) {}

    fn add_popup(&mut self, _ui: &mut egui::Ui, _surface: SurfaceIndex, _node: NodeIndex) {}

    fn tab_style_override(
        &self,
        _tab: &Self::Tab,
        _global_style: &dock::style::TabStyle,
    ) -> Option<dock::style::TabStyle> {
        None
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        true
    }

    fn clear_background(&self, _tab: &Self::Tab) -> bool {
        true
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [true, true]
    }
}
