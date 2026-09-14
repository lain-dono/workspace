use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct DecalPlugin;

impl Plugin for DecalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<DecalMaterial> {
            prepass_enabled: false,
            ..default()
        });
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DecalMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material for DecalMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/decal.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/decal.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[derive(Component, Debug, Default)]
pub struct Decal;

#[derive(Bundle, Debug, Default)]
pub struct DecalBundle {
    pub mesh: Mesh3d,

    pub decal: Decal,
    pub material: MeshMaterial3d<DecalMaterial>,

    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The inherited visibility of the entity.
    pub inherited_visibility: InheritedVisibility,
    /// The view visibility of the entity.
    pub view_visibility: ViewVisibility,
    /// The transform of the entity.
    pub transform: Transform,
    /// The global transform of the entity.
    pub global_transform: GlobalTransform,
}
