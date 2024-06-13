#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
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

#import bevy_water::water_functions as water_fn
#import bevy_water::caustics_functions as caustics_fn

struct UnderwaterMaterial {
    water_plane: vec4<f32>,
    water_color: vec4<f32>,
    light_dir: vec4<f32>,
    // quantize_steps: u32,
}

@group(2) @binding(200)
var<uniform> material: UnderwaterMaterial;

@group(2) @binding(201) var caustics_texture: texture_2d<f32>;
@group(2) @binding(202) var caustics_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    // pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    // XXX: Use our own UV.

    let w_pos = water_fn::uv_to_coord(in.uv);
    let height = water_fn::get_wave_height(w_pos); // Water height from water_plane.
    let depth = caustics_fn::distance_to_plane(in.world_position.xyz, material.water_plane) - height;
    if (depth < 0.0) {
        // We're underwater.
        // pbr_input.material.base_color *= material.water_color;
        let refracted_light = refract(-material.light_dir.xyz, material.water_plane.xyz, caustics_fn::IOR);

        let plane_intersect = caustics_fn::line_plane_intercept(in.world_position.xyz, refracted_light, material.water_plane);
        let caustic = textureSample(caustics_texture, caustics_sampler, in.uv);

        /// I'd like to change the lighting intensity here.
        pbr_input.material.base_color = mix(pbr_input.material.base_color, material.water_color, saturate(abs(depth / 0.01)));// caustic.r * 1000.0;
    }

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    // out.color = vec4<f32>(vec4<u32>(out.color * f32(material.quantize_steps))) / f32(material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // out.color = vec4<f32>(caustic.r, caustic.r, caustic.r, 1.0);
    // out.color = out.color * 2.0;
#endif

    return out;
}
