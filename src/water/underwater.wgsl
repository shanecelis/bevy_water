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
    water_world_to_uv: mat4x4<f32>,
    water_plane: vec4<f32>,
    water_color: vec4<f32>,
    light_dir: vec4<f32>,
    // quantize_steps: u32,
}

@group(2) @binding(200)
var<uniform> material: UnderwaterMaterial;

@group(2) @binding(201) var caustics_texture: texture_2d<f32>;
@group(2) @binding(202) var caustics_sampler: sampler;
// @group(2) @binding(203) var water_world_to_uv: mat4x4<f32>;

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
    let water_uv = (material.water_world_to_uv * vec4<f32>(in.world_position.xyz, 1.0)).xz;
    // let water_uv = in.world_position.xz / 10.0 + 0.5;
    // let water_uv = (in.world_position * material.water_world_to_uv).xz;

    let w_pos = water_fn::uv_to_coord(water_uv);
    // let w_pos = water_uv;
    let height = water_fn::get_wave_height(w_pos); // Water height from water_plane.
    // let height = caustic.g;

    let no_light = vec4<f32>(0,0,0,1);
    let light_color = vec4<f32>(1,1,1,1);
        // pbr_input.material.emissive = light_color * 10.;// * caustic.r * 100.0;// caustic.r * 1000.0;
    let depth = caustics_fn::distance_to_plane(in.world_position.xyz, material.water_plane) - height;
    var caustic = no_light;
    // var caustic = vec4<f32>(1.0, 0, 0, 1);
    if (depth < 0.0) {
        // We're underwater.
        // pbr_input.material.base_color *= material.water_color;
        let refracted_light = refract(-material.light_dir.xyz, material.water_plane.xyz, caustics_fn::IOR);

        let plane_intersect = caustics_fn::line_plane_intercept(in.world_position.xyz, refracted_light, material.water_plane);

        pbr_input.material.base_color *= caustics_fn::underwater_color;//material.water_color;
        let caustic_uv = (material.water_world_to_uv * vec4<f32>(plane_intersect, 1.0)).xz;
        if (caustic_uv.x >= 0. && caustic_uv.x < 1. && caustic_uv.y >= 0. && caustic_uv.y < 1.) {
        caustic = textureSample(caustics_texture, caustics_sampler, caustic_uv);
        // caustic = textureSample(caustics_texture, caustics_sampler, water_uv);
        let area_ratio = caustic.r / 0.5;
        // let area_ratio = caustic.r;

        /// I'd like to change the lighting intensity here.
        // pbr_input.material.base_color = mix(pbr_input.material.base_color, material.water_color, saturate(abs(depth * 3.0)));// caustic.r * 1000.0;
        // pbr_input.material.base_color = mix(pbr_input.material.base_color, light_color, caustic.r * 4.0);// caustic.r * 1000.0;
        //
        // The problem here is emissive makes it lighter, but caustic.r > 1 is lighter, and caustic.r < 1 is darker.
        pbr_input.attenuation = vec3<f32>(max(area_ratio * 10, 0.9));
        if (area_ratio > 1) {
            //
            //pbr_input.material.emissive = mix(no_light, light_color, (area_ratio - 1) * 50);
            // pbr_input.material.emissive = 100.0 * light_color;//mix(-light_color * 1000, light_color, (caustic.r) * 10000.0);// caustic.r * 1000.0;


        } else {
            // pbr_input.material.base_color *= (1 - area_ratio) * 10; // looks like it brightens everything.
            //pbr_input.material.base_color *= pow(area_ratio, 0.2);

            // pbr_input.material.emissive = light_color * -1;

        }
        }
        //

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

    if (depth < 0.0) {
        // out.color = abs(caustic);
      // out.color = vec4<f32>(caustic.r, caustic.r, caustic.r, 1.0);
    }
    // XXX: This does not make any sense!
    // out.color = vec4<f32>(water_uv.x % 1.0, 0., 0. * depth, 1.0);

    // out.color = vec4<f32>(w_pos.x % 1.0, 0., 0. * depth, 1.0);
    // out.color = vec4<f32>(in.world_position.xz % 1.0, 0. * depth, 1.0);
    // out.color = vec4<f32>(in.world_position.y + 10.2, 0.0, 0. * depth, 1.0);
    // out.color = out.color * 2.0;
#endif

    return out;
}
