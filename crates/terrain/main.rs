use bevy::{
    asset::RenderAssetUsages,
    camera::{primitives::Aabb, visibility::NoAutoAabb},
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    image::ImageLoaderSettings,
    mesh::Indices,
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    reflect::TypePath,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

pub mod mipgen;
pub mod render;

const SHADER_ASSET_PATH: &str = "shaders/terrain.wgsl";
const NORMAL_SHADER_ASSET_PATH: &str = "shaders/normal.wgsl";

fn main() {
    #[cfg(feature = "dev")]
    use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::FifoRelaxed,
                ..default()
            }),
            ..default()
        }),
        FreeCameraPlugin,
        MaterialPlugin::<TerrainMaterial>::default(),
        MaterialPlugin::<NormalMaterial>::default(),
        WireframePlugin::default(),
        #[cfg(feature = "dev")]
        FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    // Here we define size of our overlay
                    font_size: FontSize::Px(32.0),
                    // If we want, we can use a custom font
                    font: default(),
                    // We could also disable font smoothing,
                    font_smoothing: bevy::text::FontSmoothing::None,
                    ..default()
                },
                // We can also change color of the overlay
                text_color: Color::WHITE,
                // We can also set the refresh interval for the FPS counter
                refresh_interval: core::time::Duration::from_millis(100),
                enabled: true,
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: true,
                    // The minimum acceptable fps
                    min_fps: 30.0,
                    // The target fps
                    target_fps: 60.0,
                },
            },
        },
        crate::mipgen::plugin,
    ));

    app.insert_resource(WireframeConfig {
        global: false,
        default_color: bevy::color::palettes::css::WHITE.into(),
        default_topology: bevy::pbr::wireframe::WireframeTopology::Triangles,
        default_line_width: 1.0,
    })
    .add_systems(Startup, setup)
    .add_systems(Update, animate_light_direction)
    .add_systems(Update, load_terrain);

    app.run();
}

#[derive(Component)]
struct HeightMapImage(Handle<Image>);

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut tmaterials: ResMut<Assets<TerrainMaterial>>,
    mut nmaterials: ResMut<Assets<NormalMaterial>>,
    mut smaterials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,

    mut waiters: ResMut<mipgen::ImageWaiter>,
) {
    let heightmap_image: Handle<Image> = asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.is_srgb = false;
            settings.texture_format = Some(bevy::render::render_resource::TextureFormat::R16Unorm);
            settings.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
        })
        .load("heightmap.png");

    waiters.add(heightmap_image.clone());

    commands.spawn(HeightMapImage(heightmap_image.clone()));

    // terrain
    commands.spawn(terrain_bundle(
        32,
        1.0,
        heightmap_image,
        &mut meshes,
        &mut tmaterials,
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.5))),
        MeshMaterial3d(nmaterials.add(NormalMaterial {})),
        Transform::from_xyz(-0.5, 0.7, 0.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.2))),
        MeshMaterial3d(nmaterials.add(NormalMaterial {})),
        Transform::from_xyz(-0.2, 0.7, 0.0),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.5))),
        MeshMaterial3d(smaterials.add(StandardMaterial::default())),
        Transform::from_xyz(-1.0, 0.7, 0.0),
    ));
    // commands.spawn((
    //     Mesh3d(meshes.add(Plane3d::default())),
    //     MeshMaterial3d(smaterials.add(StandardMaterial::default())),
    //     Transform::from_xyz(-1.0, 0.5, 0.0),
    // ));

    // commands.spawn((PointLight::default(), Transform::from_xyz(2.0, 4.0, 2.0)));

    // camera
    commands.spawn((
        Camera3d::default(),
        FreeCamera::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        // bevy::render::view::Hdr,
    ));

    // directional 'sun' light
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::PI / 4.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        bevy::light::CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .build(),
    ));
}

fn terrain_bundle(
    size: u32,
    scale: f32,
    image: Handle<Image>,
    meshes: &mut Assets<Mesh>,
    tmaterials: &mut Assets<TerrainMaterial>,
) -> impl Bundle {
    let aabb = Vec3A::new(1.0, scale, 1.0) / 2.0;
    (
        Mesh3d(meshes.add(gen_terrain(size))),
        MeshMaterial3d(tmaterials.add(TerrainMaterial {
            heightmap_scale: scale,
            heightmap_size: size,
            heightmap_image: image,
        })),
        Transform::from_xyz(-1.5, 0.0, -1.5),
        NoAutoAabb,
        Aabb {
            center: aabb,
            half_extents: aabb,
        },
        ShowAabbGizmo::default(),
    )
}

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct NormalMaterial {}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for NormalMaterial {
    fn fragment_shader() -> ShaderRef {
        NORMAL_SHADER_ASSET_PATH.into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

// This struct defines the data that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TerrainMaterial {
    #[uniform(0)]
    heightmap_scale: f32,

    #[uniform(0)]
    heightmap_size: u32,

    // #[texture(1, sample_type = "u_int")]
    #[texture(1)]
    heightmap_image: Handle<Image>,
}

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

fn gen_terrain(heightmap_size: u32) -> Mesh {
    let size = heightmap_size as usize - 1;
    let vertices_per_run = size * 2 + 4;
    let vertices_per_chunk = dbg!(vertices_per_run * size);
    let primitive_topology = bevy::mesh::PrimitiveTopology::TriangleStrip;

    Mesh::new(primitive_topology, RenderAssetUsages::RENDER_WORLD).with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![Vec3::ZERO; vertices_per_chunk],
    )
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs() * 0.5);
    }
}

fn load_terrain(
    mut commands: Commands,
    q: Query<(Entity, &HeightMapImage)>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut nmaterials: ResMut<Assets<NormalMaterial>>,
    mut smaterials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, m) in q {
        let Some(image) = images.get(&m.0) else {
            continue;
        };
        if asset_server.is_loaded(&m.0) {
            commands.entity(entity).remove::<HeightMapImage>().insert((
                Mesh3d(meshes.add(init_indices(image))),
                MeshMaterial3d(nmaterials.add(NormalMaterial {})),
                // MeshMaterial3d(smaterials.add(StandardMaterial::default())),
                Transform::from_xyz(-0.2, 0.0, 0.0),
            ));
        }
    }
}

// https://github.com/emeiri/ogldev/blob/master/Terrain6/geomip_grid.cpp
fn init_indices(image: &Image) -> Mesh {
    let terrain = Terrain {
        size: 512,
        world_scale: 0.01,
        texture_scale: 1.0,
    };
    let patch_size = 511;
    let width: usize = 511;
    let depth: usize = 511;

    let patch = patch_size - 1;
    let num_quads = patch * patch;

    let elevation_at = |u, v| -> f32 {
        let elevation = image.get_color_at(u as u32, v as u32);
        100.0 * elevation.map_or(0.0, |c| c.to_linear().red)
    };

    let recommended = ((width - 1 + patch) / patch) * patch + 1;
    assert!(
        (width - 1).is_multiple_of(patch),
        "width {width}-1 must be divisible by {patch}, try {recommended}",
    );

    let recommended = ((depth - 1 + patch) / patch) * patch + 1;
    assert!(
        (depth - 1).is_multiple_of(patch),
        "depth {depth}-1 must be multiple of {patch}, try {recommended}",
    );

    assert!(patch >= 2, "The minimum patch size is 3");
    assert!(patch.is_multiple_of(2), "Patch size must be an odd number",);

    let mut indices: Vec<u32> = vec![];
    for v in (0..patch).step_by(2) {
        for u in (0..patch).step_by(2) {
            let map = |(a, b)| ((v + a) * width + u + b) as u32;

            indices.extend([(1, 1), (0, 0), (1, 0)].map(map));
            indices.extend([(1, 1), (1, 0), (2, 0)].map(map));
            indices.extend([(1, 1), (2, 0), (2, 1)].map(map));
            indices.extend([(1, 1), (2, 1), (2, 2)].map(map));
            indices.extend([(1, 1), (2, 2), (1, 2)].map(map));
            indices.extend([(1, 1), (1, 2), (0, 2)].map(map));
            indices.extend([(1, 1), (0, 2), (0, 1)].map(map));
            indices.extend([(1, 1), (0, 1), (0, 0)].map(map));
        }
    }

    // for i in indices.chunks_exact(3) {
    //     println!("{} {} {}", i[0], i[1], i[2]);
    // }

    assert_eq!(indices.len(), num_quads * 6,);

    // panic!("{indices:?}");

    let vtx_count = width * depth;
    println!("vtx_count: {vtx_count}");

    let mut positions = Vec::with_capacity(vtx_count);
    let mut normals = Vec::with_capacity(vtx_count);
    let mut uvs = Vec::with_capacity(vtx_count);

    let uv_scale = terrain.texture_scale / terrain.size as f32;
    for v in 0..depth {
        for u in 0..width {
            positions.push(Vec3::new(u as f32, elevation_at(u, v), v as f32) * terrain.world_scale);
            uvs.push(Vec2::new(u as f32, v as f32) * uv_scale);
        }
    }

    let accurate = true;

    if accurate {
        normals.extend((0..depth * width).map(|_| Vec3::ZERO));

        // Accumulate each triangle normal into each of the triangle vertices
        for v in (0..depth - 1).step_by(patch) {
            for u in (0..width - 1).step_by(patch) {
                for i in indices.as_chunks().0 {
                    let [i0, i1, i2] = i.map(|i| v * width + u + i as usize);
                    let a = positions[i1] - positions[i0];
                    let b = positions[i2] - positions[i0];
                    let normal = a.cross(b).normalize();

                    normals[i0] += normal;
                    normals[i1] += normal;
                    normals[i2] += normal;
                }
            }
        }
    } else {
        for v in 0..depth {
            for u in 0..width {
                let nu = elevation_at(u.saturating_sub(1), v); // 01
                let pu = elevation_at((u + 1).min(image.size().x as usize), v); // 21
                let nv = elevation_at(u, v.saturating_sub(1)); // 10
                let pv = elevation_at(u, (v + 1).min(image.size().y as usize)); // 12

                let du = nu - pu;
                let dv = nv - pv;
                normals.push(vec3(du, 2.0, dv));
            }
        }
    }

    // Normalize all the vertex normals
    for n in &mut normals {
        *n = n.normalize()
    }

    let primitive_topology = bevy::mesh::PrimitiveTopology::TriangleList;
    let asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;

    Mesh::new(primitive_topology, asset_usage)
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

#[derive(Default)]
struct Terrain {
    size: u32,
    world_scale: f32,
    texture_scale: f32,
}

impl Terrain {
    fn elevation(&self, u: usize, v: usize) -> f32 {
        0.0
    }
}
