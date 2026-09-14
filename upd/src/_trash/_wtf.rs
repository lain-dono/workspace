/*
fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    //mut spawner: ResMut<ReflectSceneSpawner>,
    //mut scenes: ResMut<Assets<ReflectScene>>,
    _type_registry: Res<AppTypeRegistry>,
    // primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    // type SceneFilter = (Without<EditorPanel>, Without<PrimaryWindow>);

    bevy::log::info!("setup");

    // use crate::anima::{Animation2d, TimelinePanel};

    //use crate::scene::{FileBrowser, Hierarchy, Inspector, SceneTab};
    // use self::inspector::integration::*;
    // use self::panel::*;
    // use crate::ui::*;

    /*
    if false {
        let first_pass_layer = bevy::render::view::RenderLayers::layer(1);
        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            first_pass_layer,
        ));
    }

    {
        let world = example_scene();
        let scene = ReflectScene::from_world(&world, &type_registry.0);
        let scene = scenes.add(scene);
        spawner.spawn(scene.clone());
        commands.insert_resource(CurrentScene(scene));
    }
    */

    trait SpawnTab {
        fn tab<T: Component>(&mut self, icon: char, title: &str, tab: T) -> Tab;

        fn placeholder(&mut self, icon: char, title: &str) -> Tab {
            self.tab(icon, title, panel::PlaceholderTab)
        }
    }

    impl SpawnTab for Commands<'_, '_> {
        fn tab<T: Component>(&mut self, icon: char, title: &str, tab: T) -> Tab {
            //let name = Name::new(pretty_type_name::pretty_type_name::<T>());
            let name = Name::new(title.to_string());
            let entity = self.spawn((name, EditorPanel::default(), tab)).id();
            Tab::new(icon, title, entity)
        }
    }

    let node_tree = commands.placeholder(icon::NODETREE, "Node Tree");
    let filler = commands.placeholder(icon::NODETREE, "Filler");

    /*
    //let scene = SceneTab::spawn(&mut commands, &mut images);
    //let hierarchy = commands.tab(icon::OUTLINER, "Hierarchy", Hierarchy::default());
    //let inspector = commands.tab(icon::PROPERTIES, "Inspector", Inspector::default());
    //let files = commands.tab(icon::FILEBROWSER, "File Browser", FileBrowser::default());

    //let assets = commands.placeholder(icon::ASSET_MANAGER, "Asset Manager");

    let view_style = commands.tab(icon::ASSET_MANAGER, "View egui::Style", ViewStyleTab);

    let assets = commands.tab(icon::ASSET_MANAGER, "Asset Manager", AllAssetsTab);
    let files = commands.placeholder(icon::FILEBROWSER, "File Browser");
    let hierarchy = commands.tab(icon::OUTLINER, "Hierarchy", HierarchyTab);

    // let anim = Animation2d::spawn(&mut commands, &mut images);

    // let workspace = commands.tab(icon::NODETREE, "Workspace", WorkspaceTab::default());
    // let timeline = commands.tab(icon::TIME, "Timeline", TimelinePanel::default());

    let outliner = commands.tab(icon::OUTLINER, "Outliner", OutlinerTab);
    let world_inspector = commands.tab(icon::PROPERTIES, "World Inspector", WorldInspectorTab);
    let filtered_inspector = commands.tab(
        icon::PROPERTIES,
        "Entity Inspector",
        EntityQueryTab::<Without<EditorPanel>>::default(),
    );

    let scene = SceneTab::<SceneFilter>::spawn(&mut commands, &mut images);

    if let Ok(window_entity) = primary_window.get_single() {
        commands
            .entity(window_entity)
            .insert(SpatialBundle::default())
            .add_child(view_style.entity)
            .add_child(node_tree.entity)
            .add_child(assets.entity)
            .add_child(files.entity)
            .add_child(hierarchy.entity)
            // .add_child(anim.entity)
            // .add_child(timeline.entity)
            .add_child(outliner.entity)
            .add_child(world_inspector.entity)
            .add_child(filtered_inspector.entity)
            .add_child(scene.entity);
    }

    // let root = TreeNode::leaf_with(vec![scene, anim, workspace, node_tree, view_style]);
    let root = TreeNode::leaf_with(vec![scene, node_tree, view_style]);
    let mut tree = SplitTree::new(root);

    let root_tabs = vec![outliner, world_inspector, filtered_inspector];
    let [a, b] = tree.right(NodeIndex::root(), 0.7, root_tabs);
    // let [_, _] = tree.below(a, 0.8, vec![timeline]);
    let [_, _] = tree.below(b, 0.5, vec![hierarchy, files, assets]);
    */

    let tree = Tree::new(vec![node_tree, filler]);

    commands.insert_resource(tree);
}

pub use self::app::{EditorPanel, EditorTab};
pub use self::dock::{Node, NodeIndex, Split, Tab, Tree};
pub use self::style::Style;

use bevy::ecs::system::StaticSystemParam;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum EditorStage {
    Root,   // PreUpdate
    Tabs,   // Update
    Finish, // PostUpdate
}

pub struct EditorUiPlugin;

fn editor_ui_plugin(app: &mut bevy::app::App) {
    use bevy::prelude::*;

    app.configure_sets(
        PreUpdate,
        (EditorStage::Root, EditorStage::Tabs, EditorStage::Finish)
            .chain()
            .after(bevy_egui::EguiPreUpdateSet::BeginPass),
    );

    app.init_resource::<self::style::Style>()
        .init_resource::<crate::app::SharedData>()
        .add_systems(
            EguiPrimaryContextPass,
            |mut context: bevy_egui::EguiContexts| -> Result {
                setup_egui_style(context.ctx_mut()?);
                Ok(())
            },
        )
        .add_systems(
            EguiPrimaryContextPass,
            // (ui_root, ui_tabs, update_panel_render_target)
            (crate::app::ui_root, crate::app::ui_tabs)
                .chain()
                .in_set(EditorStage::Root),
        )
        .add_systems(
            EguiPrimaryContextPass,
            crate::app::ui_finish.in_set(EditorStage::Finish),
        );
}

fn setup_egui_style(ctx: &mut egui::Context) {
    let blender_icons = egui::FontData::from_static(include_bytes!("icon.ttf"));
    let inter_variable =
        egui::FontData::from_static(include_bytes!("../../assets/fonts/Inter/InterVariable.ttf"));

    let mut fonts = egui::FontDefinitions::default();

    fonts
        .font_data
        .insert("blender".to_owned(), blender_icons.into());
    fonts
        .font_data
        .insert("InterVariable".to_owned(), inter_variable.into());

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

    ctx.set_fonts(fonts);

    let style = Style::default();
    style.init(ctx);
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
        ) -> Result {
            let ctx = context.ctx_mut()?;
            let mut query = query.into_inner();
            for (entity, viewport, mut tab) in panel_view.iter_mut() {
                if let Some(viewport) = viewport.viewport {
                    let id = egui::Id::new(entity);
                    let ui_builder = egui::UiBuilder::new()
                        .layer_id(egui::LayerId::background())
                        .max_rect(viewport);

                    let mut ui = egui::Ui::new(ctx.clone(), id, ui_builder);
                    tab.ui(&mut ui, entity, &mut query);
                }
            }
            Ok(())
        }

        self.add_systems(Update, system::<T>.in_set(EditorStage::Tabs));
        self
    }
}

/*
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

    */

    */
