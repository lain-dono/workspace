// https://github.com/bonsairobo/bevy_triplanar_splatting

//#define_import_path trimap::triplanar

struct TriplanarMapping {
    // weights for blending between the planes
    weights: vec3<f32>,

    uv_x: vec2<f32>,
    uv_y: vec2<f32>,
    uv_z: vec2<f32>,
}

fn calculate_triplanar_mapping(p: vec3<f32>, n: vec3<f32>, k: f32) -> TriplanarMapping {
    var w = pow(abs(n), vec3(k));
    w = w / (w.x + w.y + w.z);
    return TriplanarMapping(w, p.yz, p.zx, p.xy);
}

fn triplanar_normal_to_world_splatted(
    t: texture_2d_array<f32>,
    s: sampler,
    w_mtl: vec4<f32>,
    world_normal: vec3<f32>,
    map: TriplanarMapping,
) -> vec3<f32> {
    // Conditional sampling improves performance quite a bit.
    var sum = vec3(0.0);
    if (w_mtl.r > 0.0) { sum += w_mtl.r * triplanar_normal_to_world(t, s, 0, world_normal, map); }
    if (w_mtl.g > 0.0) { sum += w_mtl.g * triplanar_normal_to_world(t, s, 1, world_normal, map); }
    if (w_mtl.b > 0.0) { sum += w_mtl.b * triplanar_normal_to_world(t, s, 2, world_normal, map); }
    if (w_mtl.a > 0.0) { sum += w_mtl.a * triplanar_normal_to_world(t, s, 3, world_normal, map); }
    return normalize(sum);
}

fn triplanar_normal_to_world(
    t: texture_2d_array<f32>,
    s: sampler,
    layer: i32,
    world_normal: vec3<f32>,
    map: TriplanarMapping,
) -> vec3<f32> {
    // Tangent space normals.
    var tnormalx = sample_normal_map_xy(t, s, map.uv_x, layer);
    var tnormaly = sample_normal_map_xy(t, s, map.uv_y, layer);
    var tnormalz = sample_normal_map_xy(t, s, map.uv_z, layer);

    // Whiteout blend
    // https://bgolus.medium.com/normal-mapping-for-a-triplanar-shader-10bf39dca05a#ce80
    //
    // Swizzles are adapted to be compatible with biplanar mapping.
    tnormalx = vec3(tnormalx.xy + world_normal.yz, abs(tnormalx.z) * world_normal.x);
    tnormaly = vec3(tnormaly.xy + world_normal.zx, abs(tnormaly.z) * world_normal.y);
    tnormalz = vec3(tnormalz.xy + world_normal.xy, abs(tnormalz.z) * world_normal.z);

    // Swizzle tangent normals to match world orientation and triblend
    return normalize(
        tnormalx.zxy * map.weights.x +
        tnormaly.yzx * map.weights.y +
        tnormalz.xyz * map.weights.z
    );
}

fn sample_normal_map(
    two_component_normal_map: bool,
    flip_normal_map_y: bool,
    t: texture_2d_array<f32>,
    s: sampler,
    uv: vec2<f32>,
    layer: i32
) -> vec3<f32> {
    var Nt = textureSample(t, s, uv, layer).rgb;
    if (two_component_normal_map) {
        Nt = vec3<f32>(Nt.rg * 2.0 - 1.0, 0.0);
        // Nt.z = sqrt(1.0 - Nt.x * Nt.x - Nt.y * Nt.y);
        Nt.z = sqrt(1.0 - dot(Nt));
    } else {
        Nt = Nt * 2.0 - 1.0;
    }
    if (flip_normal_map_y) {
        Nt.y = -Nt.y;
    }
    return normalize(Nt);
}

fn sample_normal_map_xy(t: texture_2d_array<f32>, s: sampler, coord: vec2<f32>, layer: i32) -> vec3<f32> {
    return normalize(unpack_xy(textureSample(t, s, coord, layer).rg));
}

fn unpack_xy(n: vec2<f32>) -> vec3<f32> {
    let nn = n * 2.0 - 1.0;
    return vec3<f32>(nn, sqrt(1.0 - dot(nn)));
}

fn unpack_xyz(n: vec3<f32>) -> vec3<f32> {
    return n * 2.0 - 1.0;
}