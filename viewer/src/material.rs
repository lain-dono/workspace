use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    image::{ImageLoaderSettings, ImageSampler},
    mesh::MeshVertexBufferLayoutRef,
    pbr::MaterialPipeline,
    prelude::*,
    render::render_resource::*,
    shader::ShaderRef,
};

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PaletteMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub base: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub attr: Handle<Image>,
}

impl Material for PaletteMaterial {
    fn fragment_shader() -> ShaderRef {
        "assets/palette/shader.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Back);
        Ok(())
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add = on_add_use_palette_material)]
pub struct UsePaletteMaterial;

/// The on_add hook that will run when the component is
/// added when spawning the glTF scene.
fn on_add_use_palette_material(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let palette = world.resource::<Palette>().material.clone();

    world
        .commands()
        .entity(entity)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert(MeshMaterial3d(palette));
}

#[derive(Resource)]
struct Palette {
    material: Handle<PaletteMaterial>,
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<PaletteMaterial>>,
) {
    let base = asset_server.load_with_settings(
        "assets/palette/base.png",
        |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest(),
    );
    let attr = asset_server.load_with_settings(
        "assets/palette/attr.png",
        |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest(),
    );

    let material = materials.add(PaletteMaterial { base, attr });
    commands.insert_resource(Palette { material });
}

pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<PaletteMaterial>::default())
        .register_type::<UsePaletteMaterial>()
        .add_systems(Startup, setup);
}
