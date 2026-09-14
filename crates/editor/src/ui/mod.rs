pub mod app;
pub mod icon;
pub mod style;
pub mod tabs;
pub mod widget;

pub use bevy_egui as shell;

pub use self::app::{EditorPanel, EditorTab};
pub use self::style::Style;
pub use self::tabs::{NodeIndex, Split, SplitTree, Tab, TreeNode};

use bevy::ecs::system::StaticSystemParam;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy_egui::EguiContexts;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum EditorStage {
    Root,   // PreUpdate
    Tabs,   // Update
    Finish, // PostUpdate
}

pub struct EditorUiPlugin;

impl bevy::app::Plugin for EditorUiPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        use self::app::*;
        use bevy::prelude::*;

        app.configure_sets(
            PreUpdate,
            (EditorStage::Root, EditorStage::Tabs, EditorStage::Finish)
                .chain()
                .after(shell::EguiSet::BeginFrame),
        );

        app.add_plugins(self::shell::EguiPlugin)
            .init_resource::<self::style::Style>()
            .init_resource::<SharedData>()
            .add_systems(Startup, setup_egui_style)
            .add_systems(
                PreUpdate,
                (ui_root, ui_tabs, update_panel_render_target)
                    .chain()
                    .in_set(EditorStage::Root),
            )
            .add_systems(PostUpdate, ui_finish.in_set(EditorStage::Finish));
    }
}

fn setup_egui_style(mut context: EguiContexts) {
    let blender_icons = egui::FontData::from_static(include_bytes!("icon.ttf"));
    let inter_variable = egui::FontData::from_static(include_bytes!(
        "../../../../assets/fonts/Inter/InterVariable.ttf"
    ));

    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert("blender".to_owned(), blender_icons);
    fonts
        .font_data
        .insert("InterVariable".to_owned(), inter_variable);

    let blender_icons = "blender";
    let inter_variable = "InterVariable";

    fonts.families.insert(
        egui::FontFamily::Name(blender_icons.into()),
        vec!["Hack".to_owned(), blender_icons.into()],
    );

    let proportional = fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default();

    proportional.insert(0, inter_variable.to_owned());
    proportional.push(blender_icons.to_owned());

    let monospace = fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default();

    monospace.insert(0, inter_variable.to_owned());
    monospace.push(blender_icons.to_owned());

    let style = Style::default();
    let context = context.ctx_mut();
    context.set_fonts(fonts);
    style.init(context);
}

pub trait AddEditorTab {
    fn add_editor_tab<T: EditorTab + 'static>(&mut self) -> &mut Self;
}

impl AddEditorTab for bevy::app::App {
    fn add_editor_tab<T: EditorTab + 'static>(&mut self) -> &mut Self {
        fn system<T: EditorTab>(
            mut context: EguiContexts,
            mut panel_view: Query<(Entity, &EditorPanel, &mut T)>,
            query: StaticSystemParam<T::Param>,
        ) {
            let ctx = context.ctx_mut();
            let mut query = query.into_inner();
            for (entity, viewport, mut tab) in panel_view.iter_mut() {
                if let Some(viewport) = viewport.viewport {
                    let mut ui = egui::Ui::new(
                        ctx.clone(),
                        egui::LayerId::background(),
                        egui::Id::new(entity),
                        viewport,
                        viewport,
                    );
                    tab.ui(&mut ui, entity, &mut query);
                }
            }
        }

        self.add_systems(Update, system::<T>.in_set(EditorStage::Tabs));
        self
    }
}

#[derive(Default, Component)]
pub struct PanelRenderTarget {
    pub texture_id: Option<egui::TextureId>,
}

impl PanelRenderTarget {
    pub fn create_render_target(images: &mut Assets<Image>) -> RenderTarget {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

        let mut image = Image {
            texture_descriptor: wgpu::TextureDescriptor {
                label: None,
                size,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);

        RenderTarget::Image(images.add(image))
    }
}

pub fn update_panel_render_target(
    mut context: self::shell::EguiContexts,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<(&mut PanelRenderTarget, &EditorPanel, &Camera)>,
) {
    let ctx = context.ctx_mut();
    let ppi = ctx.pixels_per_point();

    for (mut panel, tab, camera) in query.iter_mut() {
        let RenderTarget::Image(handle) = &camera.target else {
            continue;
        };

        if let Some((image, viewport)) = images.get_mut(handle).zip(tab.viewport) {
            let width = (viewport.width() * ppi) as u32;
            let height = (viewport.height() * ppi) as u32;

            panel.texture_id = Some(context.add_image(handle.clone_weak()));

            image.resize(wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            });
        }
    }
}
