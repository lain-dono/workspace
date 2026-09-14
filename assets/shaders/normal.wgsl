#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_functions::prepare_world_normal,
}

@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let world_normal = prepare_world_normal(in.world_normal, false, is_front);
    let N = normalize(world_normal);
    return vec4(N, 1.0);
}