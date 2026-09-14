[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let uv = vec2<f32>(f32(i32((vertex_index << 1u) & 2u)), f32(i32(vertex_index & 2u)));
    return vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.02, 0.02, 0.02, 1.0);
}

fn builtin_dither_threshold(uv: vec2<f32>) -> f32 {
    let DITHER_THRESHOLDS = [
        1.0 / 17.0,  9.0 / 17.0,  3.0 / 17.0, 11.0 / 17.0,
        13.0 / 17.0,  5.0 / 17.0, 15.0 / 17.0,  7.0 / 17.0,
        4.0 / 17.0, 12.0 / 17.0,  2.0 / 17.0, 10.0 / 17.0,
        16.0 / 17.0,  8.0 / 17.0, 14.0 / 17.0,  6.0 / 17.0
    ];
    return (u32(uv.x) % 4) * 4u + u32(uv.y) % 4u;
}

fn builtin_dither(alpha: f32, uv: vec2<f32>) -> f32 {
    return alpha - builtin_dither_threshold(uv);
}