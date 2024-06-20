#define_import_path bevy_water::caustics_binding

struct CausticsParallaxExtension {
    water_world_to_uv: mat4x4<f32>,
    water_plane: vec4<f32>,
    light_dir: vec4<f32>,
    // quantize_steps: u32,
}

@group(2) @binding(200)
var<uniform> material: CausticsParallaxExtension;
