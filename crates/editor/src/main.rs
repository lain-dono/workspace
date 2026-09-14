use self::ui::EditorPanel;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow, WindowTheme};
use bevy::winit::{UpdateMode, WinitSettings};

pub mod inspector;
pub mod panel;
pub mod reflect_editor;
pub mod ui;
pub mod util;

pub mod example;

fn main() {
    //crate::util::enable_tracing();

    let mut app = App::new();

    app.insert_resource(ClearColor(Color::CRIMSON))
        .insert_resource(Msaa::Off)
        // Optimal power saving and present mode settings for desktop apps.
        .insert_resource(WinitSettings {
            return_from_run: true,
            //focused_mode: UpdateMode::Continuous,
            focused_mode: UpdateMode::Reactive {
                wait: std::time::Duration::from_secs_f64(1.0 / 60.0),
            },
            unfocused_mode: UpdateMode::ReactiveLowPower {
                wait: std::time::Duration::from_secs(60),
            },
        });

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: String::from("ShaderLab"),
            //resolution: (500., 300.).into(),
            present_mode: PresentMode::AutoNoVsync,
            // Tells wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            window_theme: Some(WindowTheme::Dark),
            decorations: true,
            ..default()
        }),
        ..default()
    }));
    //.add_plugins_with(DefaultPlugins, |group| group.disable::<LogPlugin>());

    app.add_plugins(LogDiagnosticsPlugin::default());

    app.add_plugins(self::ui::EditorUiPlugin);
    app.add_plugins(self::inspector::DefaultInspectorConfigPlugin);

    self::inspector::integration::WorldInspectorTab::add_panel(&mut app);
    self::inspector::integration::AllAssetsTab::add_panel(&mut app);
    self::inspector::integration::EntityQueryTab::<Without<EditorPanel>>::add_panel(&mut app);

    self::panel::ViewStyleTab::add_panel(&mut app);

    self::panel::PlaceholderTab::add_panel(&mut app);
    self::panel::HierarchyTab::add_panel(&mut app);
    self::panel::SceneTab::<SceneFilter>::add_panel(&mut app);
    self::panel::OutlinerTab::add_panel(&mut app);
    // self::panel::WorkspaceTab::add_panel(&mut app);

    // app.add_plugins(self::anima::AnimaPlugin);
    app.add_systems(Startup, crate::example::setup_entity_tree);
    app.add_systems(Startup, crate::example::setup_pleasure_room);
    app.add_systems(Startup, setup);

    //app.add_systems(Update, .after(EditorStage::Tabs));

    /*
    app.add_plugins(crate::scene::component::EditorPlugin);
    app.add_plugins(crate::scene::GizmoPlugin);

    app.add_plugins(self::inspector::InspectorPlugin);

    app.add_editor_tab::<self::scene::FileBrowser>();
    app.add_editor_tab::<self::scene::Hierarchy>();
    app.add_editor_tab::<self::scene::Inspector>();
    app.add_editor_tab::<self::scene::SceneTab>();
    */

    app.run();
}


fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    //mut spawner: ResMut<ReflectSceneSpawner>,
    //mut scenes: ResMut<Assets<ReflectScene>>,
    _type_registry: Res<AppTypeRegistry>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {

type SceneFilter = (Without<EditorPanel>, Without<PrimaryWindow>);

    bevy::log::info!("setup");

    // use crate::anima::{Animation2d, TimelinePanel};

    //use crate::scene::{FileBrowser, Hierarchy, Inspector, SceneTab};
    use self::inspector::integration::*;
    use self::panel::*;
    use crate::ui::*;

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
            self.tab(icon, title, PlaceholderTab)
        }
    }

    impl<'s, 'w> SpawnTab for Commands<'s, 'w> {
        fn tab<T: Component>(&mut self, icon: char, title: &str, tab: T) -> Tab {
            //let name = Name::new(pretty_type_name::pretty_type_name::<T>());
            let name = Name::new(title.to_string());
            let entity = self.spawn((name, EditorPanel::default(), tab)).id();
            Tab::new(icon, title, entity)
        }
    }

    let node_tree = commands.placeholder(icon::NODETREE, "Node Tree");

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

    commands.insert_resource(tree);
}
