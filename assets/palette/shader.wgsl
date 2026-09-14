#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions::alpha_discard,
    mesh_view_bindings::view,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif


@group(#{MATERIAL_BIND_GROUP}) @binding(0) var pal_base_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var pal_base_sampler: sampler;

@group(#{MATERIAL_BIND_GROUP}) @binding(2) var pal_attr_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var pal_attr_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_vertex_output(mesh, is_front, false);

    let attr = textureSample(pal_attr_texture, pal_attr_sampler, mesh.uv);

    let time = 123.0;
    let t = (time % 36.0) / 36.0;

    let base = textureSample(pal_base_texture, pal_base_sampler, vec2(mesh.uv.x, mesh.uv.y - attr.b * t));

    let emissive = clamp(1.0 - attr.a, 0.0, 1.0);

    pbr_input.material.base_color = base;
    // pbr_input.material.emissive = base * (1.0 - attr.a) * 10.0;
    // pbr_input.material.emissive = select(vec4(base.rgb * 100000.0, 1.0), vec4(0.0, 0.0, 0.0, 1.0), emissive < 0.5);
    // pbr_input.material.emissive = vec4(emissive * base.rgb * 100000.0, 1.0);
    // pbr_input.material.emissive = vec4(emissive * base.rgb * 100000.0 /  view.exposure, 1.0);
    pbr_input.material.emissive = vec4(emissive * base.rgb * 200.0 / view.exposure, 1.0);
    //vec4(base.rgb * emissive, emissive);

    pbr_input.material.metallic = attr.r;
    pbr_input.material.perceptual_roughness = 1.0 - attr.g;

    // pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(mesh, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    //out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    //out.color = out.color * 2.0;
#endif

    return out;
}