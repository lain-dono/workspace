use super::{floor::Building, workspace::Workspace};
use bevy::{
    asset::AssetMut,
    image::{ImageSampler, ImageSamplerDescriptor},
    light,
    mesh::PlaneMeshBuilder,
    prelude::*,
    render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat},
};
use image::{DynamicImage, ImageBuffer, imageops::FilterType};

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ReferenceImage;

#[derive(Resource, Debug, Default, Clone)]
pub struct ReferenceImageHandle(pub Handle<Image>);

#[derive(Resource, Deref)]
pub struct DefaultSampler(ImageSamplerDescriptor);

pub fn plugin(app: &mut App) {
    let image_plugin = app.get_added_plugins::<ImagePlugin>();
    let Some(image_plugin) = image_plugin.first() else {
        warn!("No ImagePlugin found. Try adding MipmapGeneratorPlugin after DefaultPlugins");
        return;
    };

    app.insert_resource(DefaultSampler(image_plugin.default_sampler.clone()));

    app.add_systems(Startup, setup_system);
    app.add_systems(Update, (update_ground, check_textures));
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = Vec2::new(2066.0, 3539.0);

    let mesh = PlaneMeshBuilder::from_size(size).subdivisions(10);
    let texture = asset_server.load("neuschwanstein_plan.png");
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.5),
        base_color_texture: Some(texture.clone()),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(ReferenceImageHandle(texture));

    commands.spawn((
        ReferenceImage,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        light::NotShadowCaster,
        light::NotShadowReceiver,
        Pickable::IGNORE,
    ));
}

fn update_ground(
    mut ground: Query<&mut Transform, With<ReferenceImage>>,
    workspace: Res<Workspace>,
    building: Res<Building>,
) {
    let ground = ground.single_mut().ok();

    if let Some(mut transform) = ground
        && let Some(floor) = building.floors.get(workspace.current_floor)
    {
        transform.translation.x = floor.ref_offset_x * floor.ref_scale;
        transform.translation.y = floor.min;
        transform.translation.z = -floor.ref_offset_y * floor.ref_scale;
        transform.scale = Vec3::splat(floor.ref_scale);
    }
}

fn check_textures(
    default_sampler: Res<DefaultSampler>,
    wait_handle: Res<ReferenceImageHandle>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = &wait_handle.0;
    for event in events.read() {
        if event.is_loaded_with_dependencies(&wait_handle.0)
            && let Some(image) = images.get_mut(handle)
        {
            generate_mipmap(image, &default_sampler, 8, FilterType::CatmullRom);
        }
    }
}

fn generate_mipmap(
    mut image: AssetMut<Image>,
    default_sampler: &ImageSamplerDescriptor,
    anisotropy_clamp: u16,
    filter: FilterType,
) {
    let &TextureDescriptor {
        dimension,
        format,
        size,
        mip_level_count,
        ..
    } = &image.texture_descriptor;

    let mut sampler = match &image.sampler {
        ImageSampler::Default => default_sampler.clone(),
        ImageSampler::Descriptor(descriptor) => descriptor.clone(),
    };

    sampler.anisotropy_clamp = anisotropy_clamp;

    let sampler = ImageSampler::Descriptor(sampler);

    let Extent3d {
        width: mut w,
        height: mut h,
        depth_or_array_layers,
    } = size;

    if image.is_compressed() {
        warn!("Compressed images not supported");
        return;
    }
    if mip_level_count != 1 {
        return;
    }
    if dimension != TextureDimension::D2 {
        warn!("Image has dimension {dimension:?} but only TextureDimension::D2 is supported.");
        return;
    }
    if depth_or_array_layers != 1 {
        warn!("Image contains {depth_or_array_layers} layers only a single layer is supported.");
        return;
    }

    let buf = image.data.clone().unwrap();

    let source = match format {
        TextureFormat::R8Unorm => ImageBuffer::from_raw(w, h, buf).map(DynamicImage::ImageLuma8),
        TextureFormat::Rg8Unorm => ImageBuffer::from_raw(w, h, buf).map(DynamicImage::ImageLumaA8),
        TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {
            ImageBuffer::from_raw(w, h, buf).map(DynamicImage::ImageRgba8)
        }
        format => {
            warn!("Conversion into dynamic image not supported for {format:?}.");
            return;
        }
    };

    let Some(mut source) = source else {
        warn!("Failed to convert into {format:?}.");
        return;
    };

    let mip_count = u32::max(w, h).ilog2();
    let mut data = source.as_bytes().to_vec();

    for _ in 0..mip_count {
        w = u32::max(w / 2, 1);
        h = u32::max(h / 2, 1);
        source = source.resize_exact(w, h, filter);
        data.extend(source.as_bytes());
    }

    image.texture_descriptor.mip_level_count = mip_count + 1; // +1 for mip0
    image.sampler = sampler;
    image.data = Some(data);
}
