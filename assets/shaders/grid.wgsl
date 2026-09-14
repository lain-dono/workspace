#import bevy_render::view::View
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_pbr::view_transformations::uv_to_ndc

struct Settings {
    size: vec2<f32>,
    offset_y: f32,
    padding: f32,
}

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: Settings;
@group(0) @binding(3) var<uniform> view: View;


@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let clip_pos = uv_to_ndc(in.uv); // Convert from uv to clip space
    let screen = textureSample(screen_texture, texture_sampler, in.uv);

    let near_factor = view.clip_from_view[3][2];
    let far_factor = 0.00001;

    let near = position_ndc_to_world(vec3(clip_pos, near_factor)) - vec3(0.0, settings.offset_y, 0.0);
    let far = position_ndc_to_world(vec3(clip_pos, far_factor)) - vec3(0.0, settings.offset_y, 0.0);

    let t = -near.y / (far.y - near.y);
    let pos = near + t * (far - near);
    let size = 1.0 / settings.size;

    var color =          grid_axis(pos, size       ) * (0.1     / t);
    // color = blend(color, grid_base(pos, size * 10.0) * (0.00001 / t) * vec4(1.0, 0.0, 0.0, 1.0));
    color = blend(color, grid_base(pos, size       ) * (0.0001  / t));
    color = blend(color, grid_base(pos, size * 0.1 ) * (0.001   / t));
    color = blend(color, grid_base(pos, size * 0.01) * (0.01    / t));

    return blend(screen, color * f32(t > 0.0));
}

// OVER operator using premultiplied alpha
// see: https://en.wikipedia.org/wiki/Alpha_compositing
fn blend(src: vec4<f32>, dst: vec4<f32>) -> vec4<f32> {
    return clamp(vec4(
        src.rgb + (1.0 - src.a) * dst.rgb,
        src.a   + (1.0 - src.a) * dst.a,
    ), vec4(0.0), vec4(1.0));
}

fn position_clip_to_world(clip_pos: vec4<f32>) -> vec3<f32> {
    let world_pos = view.world_from_clip * clip_pos;
    return world_pos.xyz;
}

fn position_ndc_to_world(ndc_pos: vec3<f32>) -> vec3<f32> {
    let world_pos = view.world_from_clip * vec4(ndc_pos, 1.0);
    return world_pos.xyz / world_pos.w;
}

/// Retrieve the perspective camera near clipping plane
fn perspective_camera_near() -> f32 {
    return view.clip_from_view[3][2];
}

/*
/// Convert linear view z to ndc depth.
/// Note: View z input should be negative for values in front of the camera as -z is forward
fn view_z_to_depth_ndc(view_z: f32) -> f32 {
#ifdef VIEW_PROJECTION_PERSPECTIVE
    return -perspective_camera_near() / view_z;
#else ifdef VIEW_PROJECTION_ORTHOGRAPHIC
    return view_bindings::view.clip_from_view[3][2] + view_z * view_bindings::view.clip_from_view[2][2];
#else
    let ndc_pos = view_bindings::view.clip_from_view * vec4(0.0, 0.0, view_z, 1.0);
    return ndc_pos.z / ndc_pos.w;
#endif
}
*/

fn grid_alpha(pos: vec3<f32>, scale: vec2<f32>) -> vec3<f32> {
    let coord = pos.xz * scale; // use the scale variable to set the distance between the lines
    let derivative = fwidth(coord);
    let grid = abs(fract(coord - 0.5) - 0.5) / derivative;
    let line = min(grid.x, grid.y);
    let min_z = min(derivative.y, 1.0);
    let min_x = min(derivative.x, 1.0);
    let alpha = 1.0 - min(line, 1.0);
    return vec3(min_x, alpha, min_z);
}

fn grid_base(pos: vec3<f32>, scale: vec2<f32>) -> vec4<f32> {
    let grid = grid_alpha(pos, scale);
    let min_z = grid.z;
    let min_x = grid.x;
    let alpha = grid.y;
    // return vec4(1.0, 1.0, 1.0, alpha) * alpha;
    return vec4(0.5, 0.5, 0.5, 1.0) * alpha;
}

fn grid_axis(pos: vec3<f32>, scale: vec2<f32>) -> vec4<f32> {
    let grid = grid_alpha(pos, scale);
    let min_z = grid.z;
    let min_x = grid.x;
    let alpha = grid.y;
    // var color = vec4(1.0) * alpha;
    var color = vec4(0.0);

    let extra_x = 1.0 / scale.x;
    let extra_z = 1.0 / scale.y;

    // z axis
    if (pos.x > -extra_x * min_x && pos.x < extra_x * min_x) {
        color = vec4(0.0, 0.0, 1.0, 1.0) * alpha;
    }
    // x axis
    if (pos.z > -extra_z * min_z && pos.z < extra_z * min_z) {
        color = vec4(1.0, 0.0, 0.0, 1.0) * alpha;
    }

    return color;
}