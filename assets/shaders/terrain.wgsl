// https://www.youtube.com/watch?v=5zlfJW2VGLM
// https://github.com/vercidium-patreon/glvertexid

#import bevy_pbr::{
    mesh_functions,
    //forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,

    mesh_bindings::mesh,
    mesh_view_bindings::view,
    pbr_types,
    pbr_functions,
}

#import "shaders/terrain_biplanar.wgsl"::BiplanarMapping
#import "shaders/terrain_triplanar.wgsl"::TriplanarMapping

struct HeightmapUniform {
    scale: f32,
    size: u32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> heightmap: HeightmapUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var heightmap_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) instance_index: u32,
}

@vertex
fn vertex(
    @builtin(vertex_index) index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let scale = heightmap.scale;
    let mesh_coord = coord_from_vertex_index(heightmap.size, index);

    let level = 0u; // 0 - 512, 1 - 256, 2 - 128, 3 - 64, 4 - 32, 5 - 16, 6 - 8

    let dimensions = textureDimensions(heightmap_texture, level).xy;
    let texel = vec2<i32>(dimensions) / i32(heightmap.size);
    let coord = mesh_coord * texel;
    let uv = vec2<f32>(mesh_coord) / f32(heightmap.size);

    let elevation = textureLoad(heightmap_texture, coord, level).r;
    let elevation_nu = textureLoad(heightmap_texture, coord + vec2(-texel.x, 0), level).r; // 01
    let elevation_pu = textureLoad(heightmap_texture, coord + vec2( texel.x, 0), level).r; // 21
    let elevation_nv = textureLoad(heightmap_texture, coord + vec2(0, -texel.y), level).r; // 10
    let elevation_pv = textureLoad(heightmap_texture, coord + vec2(0,  texel.y), level).r; // 12

    let du = (elevation_nu - elevation_pu) * scale;
    let dv = (elevation_nv - elevation_pv) * scale;
    let normal = vec3(du * f32(dimensions.x), 2.0, dv * f32(dimensions.y));
    // let normal = vec3(du, 2.0, dv);
    let position = vec3(uv.x, elevation * scale, uv.y);

    let normal_sobel = sample_sobelFilter3x3(texel, coord, 16, level);

    let n = vec2<f32>(texel * 2) / f32(heightmap.size);
    let va = normalize(vec3(n.x, -du, 0.0));
    let vb = normalize(vec3(0.0, -dv, n.y));
    let normal_cross = cross(vb, va);

    let world_from_local = mesh_functions::get_world_from_local(instance_index);

    var out: VertexOutput;
    // out.world_normal = normalize(normal_sobel);
    // out.world_normal = normal;
    out.world_normal = mesh_functions::mesh_normal_local_to_world(normal, instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
    out.uv = uv;
    return out;
}

fn coord_from_vertex_index(size: u32, index: u32) -> vec2<i32> {
    let stride = (size - 1u) * 2u + 4u;
    let cindex = clamp(i32(index % stride) - 1, 0, i32(stride) - 3);
    return vec2(cindex % 2 + i32(index / stride), cindex / 2);
}

fn sample_sobelFilter3x3(texel: vec2<i32>, coord: vec2<i32>, scale: f32, level: u32) -> vec3<f32> {
    let s0 = textureLoad(heightmap_texture, coord + vec2( texel.x,   texel.y), level).r;
    let s1 = textureLoad(heightmap_texture, coord + vec2( 0,         texel.y), level).r;
    let s2 = textureLoad(heightmap_texture, coord + vec2(-texel.x,   texel.y), level).r;
    let s3 = textureLoad(heightmap_texture, coord + vec2( texel.x,   0), level).r;

    let s5 = textureLoad(heightmap_texture, coord + vec2(-texel.x,   0), level).r;
    let s6 = textureLoad(heightmap_texture, coord + vec2( texel.x,  -texel.y), level).r;
    let s7 = textureLoad(heightmap_texture, coord + vec2( 0,        -texel.y), level).r;
    let s8 = textureLoad(heightmap_texture, coord + vec2(-texel.x,  -texel.y), level).r;

    return vec3(
        scale * (s2 - s0 + 2 * (s5 - s3) + s8 - s6),
        1.0,
        scale * (s6 - s0 + 2 * (s7 - s1) + s8 - s2),
    );
}

fn sample_scharrFilter3x3(texel: vec2<i32>, coord: vec2<i32>, scale: f32, level: u32) -> vec3<f32> {
    let s0 = textureLoad(heightmap_texture, coord + vec2( texel.x,   texel.y), level).r;
    let s1 = textureLoad(heightmap_texture, coord + vec2( 0,         texel.y), level).r;
    let s2 = textureLoad(heightmap_texture, coord + vec2(-texel.x,   texel.y), level).r;
    let s3 = textureLoad(heightmap_texture, coord + vec2( texel.x,   0), level).r;

    let s5 = textureLoad(heightmap_texture, coord + vec2(-texel.x,   0), level).r;
    let s6 = textureLoad(heightmap_texture, coord + vec2( texel.x,  -texel.y), level).r;
    let s7 = textureLoad(heightmap_texture, coord + vec2( 0,        -texel.y), level).r;
    let s8 = textureLoad(heightmap_texture, coord + vec2(-texel.x,  -texel.y), level).r;

    return vec3(
        scale * (3 * (s2 - s0) + 10 * (s5 - s3) + 3 * (s8 - s6)),
        1.0,
        scale * (3 * (s6 - s0) + 10 * (s7 - s1) + 3 * (s8 - s2)),
    );
}


@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    // let pos = in.world_position.xyz / in.world_position.w;
    // let world_normal = normalize(cross(dpdy(pos), dpdx(pos)));

    var pbr_input: pbr_types::PbrInput = pbr_types::pbr_input_new();
    pbr_input.flags = mesh[in.instance_index].flags;

    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.V = pbr_functions::calculate_view(in.world_position, pbr_input.is_orthographic);
    pbr_input.frag_coord = in.position;
    pbr_input.world_position = in.world_position;

    // pbr_input.material.base_color = in.color;
    pbr_input.material.base_color = vec4(0.75);
    pbr_input.material.metallic = 0.0;
    pbr_input.material.perceptual_roughness = 1.0;

    pbr_input.world_normal = pbr_functions::prepare_world_normal(in.world_normal, false, is_front);

    pbr_input.N = normalize(pbr_input.world_normal);

    return pbr_functions::apply_pbr_lighting(pbr_input);
    // return vec4(pbr_input.N, 1.0);
    // return vec4(in.uv, 0.0, 1.0);
    // return vec4(in.world_normal, 1.0);
    // return in.marker;
}