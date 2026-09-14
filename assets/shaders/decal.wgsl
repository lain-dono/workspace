#import bevy_pbr::{
    mesh_view_bindings::depth_prepass_texture,
    mesh_bindings,
    mesh_functions,
    view_transformations::{uv_to_ndc, frag_coord_to_uv, position_ndc_to_world, position_world_to_clip},
}
#import bevy_render::maths::mat2x4_f32_to_mat3x3_unpack

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) instance_index: u32,
}

@vertex
fn vertex(
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    let model = mesh_functions::get_world_from_local(instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(model, generate_cube_strip(vertex_index));
    var out: VertexOutput;
    out.instance_index = instance_index;
    out.position = position_world_to_clip(world_position.xyz);
    return out;
}

@fragment
fn fragment(
#ifdef MULTISAMPLED
    @builtin(sample_index) sample_index: u32,
#endif
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) instance_index: u32,
) -> @location(0) vec4<f32> {
#ifdef MULTISAMPLED
    let ndc_depth = textureLoad(depth_prepass_texture, vec2<i32>(position.xy), i32(sample_index));
#else // MULTISAMPLED
    let ndc_depth = textureLoad(depth_prepass_texture, vec2<i32>(position.xy), 0);
#endif // MULTISAMPLED

    let world_to_local = get_inverse_model_matrix(instance_index);

    let ndc_position = vec3(uv_to_ndc(frag_coord_to_uv(position.xy)), ndc_depth);
    let world_position = position_ndc_to_world(ndc_position);
    let local_position = world_to_local * vec4(world_position, 1.0);

    let color = textureSample(texture, texture_sampler, local_position.xz + 0.5);

    let x = local_position.x;
    let y = local_position.y;
    let z = local_position.z;

    if (ndc_depth <= -0.0 || abs(x) > 0.5 || abs(y) > 0.5 || abs(z) > 0.5) {
        discard;
    } else {
        return color;
    }
}

fn generate_cube_strip(vertex_index: u32) -> vec4<f32> {
    let mask = 1u << vertex_index;
    return vec4(
        f32((0x287Au & mask) != 0u) - 0.5,
        f32((0x02AFu & mask) != 0u) - 0.5,
        f32((0x31E3u & mask) != 0u) - 0.5,
        1.0,
    );
}

fn get_inverse_model_matrix(instance_index: u32) -> mat4x4<f32> {
    let affine = mesh_bindings::mesh[instance_index].world_from_local;
    let mx = transpose(mat2x4_f32_to_mat3x3_unpack(
        mesh_bindings::mesh[instance_index].local_from_world_transpose_a,
        mesh_bindings::mesh[instance_index].local_from_world_transpose_b,
    ));
    let tx = vec3(affine.x[3], affine.y[3], affine.z[3]);
    return mat4x4(vec4(mx.x, 0.0), vec4(mx.y, 0.0), vec4(mx.z, 0.0), vec4(-(mx * tx), 1.0));
}

// fn inverse3(m: mat3x3<f32>) -> mat3x3<f32> {
//     let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2];
//     let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2];
//     let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2];

//     let b01 =  a22 * a11 - a12 * a21;
//     let b11 = -a22 * a10 + a12 * a20;
//     let b21 =  a21 * a10 - a11 * a20;

//     let det = a00 * b01 + a01 * b11 + a02 * b21;

//     return mat3x3(
//         vec3(b01, (-a22 * a01 + a02 * a21), ( a12 * a01 - a02 * a11)) / det,
//         vec3(b11, ( a22 * a00 - a02 * a20), (-a12 * a00 + a02 * a10)) / det,
//         vec3(b21, (-a21 * a00 + a01 * a20), ( a11 * a00 - a01 * a10)) / det,
//     );
// }

// fn inverse4(m: mat4x4<f32>) -> mat4x4<f32> {
//     let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2]; let a03 = m[0][3];
//     let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2]; let a13 = m[1][3];
//     let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2]; let a23 = m[2][3];
//     let a30 = m[3][0]; let a31 = m[3][1]; let a32 = m[3][2]; let a33 = m[3][3];

//     let b00 = a00 * a11 - a01 * a10;
//     let b01 = a00 * a12 - a02 * a10;
//     let b02 = a00 * a13 - a03 * a10;
//     let b03 = a01 * a12 - a02 * a11;
//     let b04 = a01 * a13 - a03 * a11;
//     let b05 = a02 * a13 - a03 * a12;
//     let b06 = a20 * a31 - a21 * a30;
//     let b07 = a20 * a32 - a22 * a30;
//     let b08 = a20 * a33 - a23 * a30;
//     let b09 = a21 * a32 - a22 * a31;
//     let b10 = a21 * a33 - a23 * a31;
//     let b11 = a22 * a33 - a23 * a32;

//     let det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;

//     return mat4x4(
//         vec4(
//             a11 * b11 - a12 * b10 + a13 * b09,
//             a02 * b10 - a01 * b11 - a03 * b09,
//             a31 * b05 - a32 * b04 + a33 * b03,
//             a22 * b04 - a21 * b05 - a23 * b03,
//         ) / det,
//         vec4(
//             a12 * b08 - a10 * b11 - a13 * b07,
//             a00 * b11 - a02 * b08 + a03 * b07,
//             a32 * b02 - a30 * b05 - a33 * b01,
//             a20 * b05 - a22 * b02 + a23 * b01,
//         ) / det,
//         vec4(
//             a10 * b10 - a11 * b08 + a13 * b06,
//             a01 * b08 - a00 * b10 - a03 * b06,
//             a30 * b04 - a31 * b02 + a33 * b00,
//             a21 * b02 - a20 * b04 - a23 * b00,
//         ) / det,
//         vec4(
//             a11 * b07 - a10 * b09 - a12 * b06,
//             a00 * b09 - a01 * b07 + a02 * b06,
//             a31 * b01 - a30 * b03 - a32 * b00,
//             a20 * b03 - a21 * b01 + a22 * b00,
//         ) / det,
//     );
// }