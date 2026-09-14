fn triplanar_default(position: vec3<f32>, normal: vec3<f32>, tile: f32, blend: f32) -> vec4<f32> {
    let node_uv: vec3<f32> = position * tile;
    var node_blend: vec3<f32> = pow(abs(normal), blend);
    node_blend /= dot(node_blend, 1.0);

    let node_x: vec4<f32> = sample_texture2d(texture, sampler, node_uv.zy);
    let node_y: vec4<f32> = sample_texture2d(texture, sampler, node_uv.xz);
    let node_z: vec4<f32> = sample_texture2d(texture, sampler, node_uv.xy);
    return node_x * node_blend.x + node_y * node_blend.y + node_z * node_blend.z;
}

fn triplanar_normal() -> vec4<f32> {
    let node_uv: vec3<f32> = position * tile;
    var node_blend: vec3<f32> = max(pow(abs(normal), blend), 0);
    node_blend /= (node_blend.x + node_blend.y + node_blend.z).xxx;

    var node_x: vec3<f32> = unpack_normalmap_rg_or_ag(sample_texture2d(texture, sampler, node_uv.zy));
    var node_y: vec3<f32> = unpack_normalmap_rg_or_ag(sample_texture2d(texture, sampler, node_uv.xz));
    var node_z: vec3<f32> = unpack_normalmap_rg_or_ag(sample_texture2d(texture, sampler, node_uv.xy));
    node_x = vec3<f32>(node_x.xy + normal.zy, abs(node_x.z) * normal.x);
    node_y = vec3<f32>(node_y.xy + normal.xz, abs(node_y.z) * normal.y);
    node_z = vec3<f32>(node_z.xy + normal.xy, abs(node_z.z) * normal.z);

    var out: vec4<f32> = vec4<f32>(normalize(node_x.zyx * node_blend.x + node_y.xzy * node_blend.y + node_z.xyz * node_blend.z), 1);
    mat3x3<f32> node_transform = mat3x3<f32>(in.worldspace_tangent, in.worldspace_bitangent, in.worldspace_normal);
    out.rgb = transform_world_to_tangent(out.rgb, node_transform);
    return out;
}

fn rotate_about_normalized_axis(input: vec3<f32>, axis: vec3<f32>, rotation: f32) -> vec3<f32> {
    let s = sin(rotation);
    let c = cos(rotation);
    let one_minus_c = 1.0 - c;

    let matrix = mat3x3<f32>(
        one_minus_c * axis.x * axis.x + c, one_minus_c * axis.x * axis.y - axis.z * s, one_minus_c * axis.z * axis.x + axis.y * s,
        one_minus_c * axis.x * axis.y + axis.z * s, one_minus_c * axis.y * axis.y + c, one_minus_c * axis.y * axis.z - axis.x * s,
        one_minus_c * axis.z * axis.x - axis.y * s, one_minus_c * axis.y * axis.z + axis.x * s, one_minus_c * axis.z * axis.z + c
    );

    return matrix * input;
}

fn rotate_about_axis(input: vec3<f32>, axis: vec3<f32>, rotation: f32) -> vec3<f32> {
    return rotate_about_normalized_axis(input, normalize(axis), rotation)
}

fn sphere_mask(coords: vec4<f32>, center: vec4<f32>, radius: f32, hardness: f32) -> vec4<f32> {
    return vec4<f32>(1.0) - saturate((distance(coords, center) - vec4<f32>(radius)) / (1.0 - hardness));
}

fn projection(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return b * dot(a, b) / dot(b, b);
}
fn rejection(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return a - (b * dot(a, b) / dot(b, b));
}

fn dither(input: vec4<f32>, screen_position: vec4<f32>, screen_params: vec4<f32>) -> vec4<f32> {
    let dither_thresholds[16] = [
        1.0 / 17.0,  9.0 / 17.0,  3.0 / 17.0, 11.0 / 17.0,
        13.0 / 17.0,  5.0 / 17.0, 15.0 / 17.0,  7.0 / 17.0,
        4.0 / 17.0, 12.0 / 17.0,  2.0 / 17.0, 10.0 / 17.0,
        16.0 / 17.0,  8.0 / 17.0, 14.0 / 17.0,  6.0 / 17.0
    ];

    let uv = screen_position.xy * screenparams.xy;
    let index = (u32(uv.x) % 4) * 4 + u32(uv.y) % 4;
    return input - dither_thresholds[index];
}