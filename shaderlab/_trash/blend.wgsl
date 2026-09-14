
fn blend_burn(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, 1.0 - (1.0 - blend) / base, opacity);
}

fn blend_darken(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, min(blend, base), opacity);
}

fn blend_difference(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, abs(blend - base), opacity);
}

fn blend_dodge(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, base / (1.0 - blend), opacity);
}

fn blend_divide(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, base / (blend + 0.000000000001), opacity);
}

fn blend_exclusion(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, blend + base - (2.0 * blend * base), opacity);
}

fn blend_hard_light(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    let result1 = 1.0 - 2.0 * (1.0 - base) * (1.0 - blend);
    let result2 = 2.0 * base * blend;
    let zeroorone = step(blend, 0.5);
    return mix(base, result2 * zeroorone + (1.0 - zeroorone) * result1, opacity);
}

fn blend_hard_mix(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, step(1 - base, blend), opacity);
}

fn blend_lighten(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, max(blend, base), opacity);
}

fn blend_linear_burn(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, base + blend - 1.0, opacity);
}

fn blend_linear_dodge(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, base + blend, opacity);
}

fn blend_linear_light(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, select(max(base + (2.0 * blend) - 1.0, 0.0), min(base + 2.0 * (blend - 0.5), 1.0), blend >= 0.5), opacity);
}

fn blend_linear_light_add_sub(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, blend + 2.0 * base - 1.0, opacity);
}

fn blend_multiply(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, base * blend, opacity);
}

fn blend_negation(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, 1.0 - abs(1.0 - blend - base), opacity);
}

fn blend_screen(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, 1.0 - (1.0 - blend) * (1.0 - base), opacity);
}

fn blend_subtract(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, base - blend, opacity);
}

fn blend_overwrite(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    return mix(base, blend, opacity);
}


fn blend_overlay(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    let result1 = 1.0 - 2.0 * (1.0 - base) * (1.0 - blend);
    let result2 = 2.0 * base * blend;
    let zeroorone = step(base, 0.5);
    return mix(base, result2 * zeroorone + (1.0 - zeroorone) * result1, opacity);
}

fn blend_pin_light(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    let check = step (0.5, blend);
    let result1 = check * max(2.0 * (base - 0.5), blend);
    return mix(base, result1 + (1.0 - check) * min(2.0 * base, blend), opacity);
}

fn blend_soft_light(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    let result1 = 2.0 * base * blend + base * base * (1.0 - 2.0 * blend);
    let result2 = sqrt(base) * (2.0 * blend - 1.0) + 2.0 * base * (1.0 - blend);
    let zeroorone = step(0.5, blend);
    return mix(base, result2 * zeroorone + (1.0 - zeroorone) * result1, opacity);
}

fn blend_vivid_light(base: vec4<f32>, blend: vec4<f32>, opacity: f32) -> vec4<f32> {
    let result1 = 1.0 - (1.0 - blend) / (2.0 * base);
    let result2 = blend / (2.0 * (1.0 - base));
    let zeroorone = step(0.5, base);
    return mix(base, result2 * zeroorone + (1.0 - zeroorone) * result1, opacity);
}