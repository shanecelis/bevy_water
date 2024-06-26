#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::view_transformations

#import bevy_water::water_bindings
#import bevy_water::water_functions as water_fn
#import bevy_water::caustics_functions as caustics_fn

struct CausticsMaterial {
    plane: vec4<f32>,
    light: vec4<f32>,
};
@group(2) @binding(0) var<uniform> material: CausticsMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) old_pos: vec3<f32>,
    @location(1) new_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) height: f32,
    @location(4) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let w_pos = water_fn::uv_to_coord(vertex.uv);
    // let w_pos = vertex.uv;

    // Calculate normal.
    // let delta = 0.2;
    let height = water_fn::get_wave_height(w_pos);
    // let height_dx = water_fn::get_wave_height(w_pos + vec2<f32>(delta, 0.0));
    // let height_dz = water_fn::get_wave_height(w_pos + vec2<f32>(0.0, delta));
    var normal = water_fn::get_wave_normal(w_pos);

    // normal = normalize(vertex.normal + normal);
    // let normal = vec3<f32>(height - height_dx, delta, height - height_dz);
    let light = material.light.xyz;


    let refracted_light = refract(-light, material.plane.xyz, caustics_fn::IOR);
    let ray = refract(-light, normal, caustics_fn::IOR);
    let uv_pos = vec3<f32>(vertex.uv.x, 0.5, vertex.uv.y) * 2.0 - 1.0;

    // let uv_pos = vec3<f32>(vertex.uv.x, vertex.position.z, vertex.uv.y) * 2.0 - 1.0;
    // let uv_pos = vertex.position.xzy * 2  - 1.;
    // let uv_pos = vec3<f32>(vertex.uv.x /2, 0.0, vertex.uv.y / 2);
    // let uv_pos = vertex.uv;
    out.uv = vertex.uv;
    out.height = height;
    out.normal = normal;
    let flip_y = vec2<f32>(1.0, -1.0);
    // let flip_y = vec2<f32>(1.0, 1.0);
    // out.old_pos = caustics_fn::project(uv_pos, refracted_light, refracted_light, material.plane);
    out.old_pos = caustics_fn::project(uv_pos, refracted_light, refracted_light, material.plane);
    out.new_pos = caustics_fn::project(uv_pos + material.plane.xyz * height, ray, refracted_light, material.plane);
    out.clip_position = vec4<f32>(0.75 * (out.new_pos.xz + refracted_light.xz / refracted_light.y), 0.0, 1.0);
    // out.clip_position = vec4<f32>(vertex.uv * 2.0 - 1.0, 0.0, 1.0);
    // out.clip_position = vec4<f32>(vertex.uv, 0.0, 1.0);


    //
    let ndc = view_transformations::uv_to_ndc(vertex.uv);
    // out.clip_position = vec4<f32>(ndc * flip_y, 0.0, 1.0);
    // out.clip_position = vec4<f32>(vertex.uv, 0.0, 1.0);
    // out.clip_position = vec4<f32>(uv_pos.xz, 0.0, 1.0);

    // out.vertex.y *= -1;


    // let world_pos = mesh_position_local_to_world(
    //     get_world_from_local(vertex.instance_index),
    //     vec4<f32>(vertex.position, 1.0),
    // );

    return out;
}

struct FragmentInput {
    // @builtin(position) clip_position: vec4<f32>,
    @location(0) old_pos: vec3<f32>,
    @location(1) new_pos: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) height: f32,
    @location(4) uv: vec2<f32>,
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let old_area = length(dpdx(input.old_pos)) * length(dpdy(input.old_pos));
    let new_area = length(dpdx(input.new_pos)) * length(dpdy(input.new_pos));
    var col: vec4<f32>;
    // return input.clip_position;
    col = vec4<f32>(old_area / new_area * 0.5, input.height, input.normal.x, input.normal.z);
    // return vec4<f32>(old_area / new_area * 0.2, input.height, input.normal.x, 1.0);
    // col = vec4<f32>(old_area / new_area * 0.2, 1.0, 0.0, 1.0);
    // col = vec4<f32>(old_area / new_area, 1.0, 0.0, 1.0);


    // col = vec4<f32>(abs(old_area)* 100000., abs(new_area) * 10000., 0.0, 1.0);

    // col = vec4<f32>(-input.normal, 1.0);
    // col = vec4<f32>(input.uv % 0.5, 0.0, 1.0);

    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    // return vec4<f32>(input.old_pos - input.new_pos, 1.0);
    // return vec4<f32>(abs(old_area - new_area) * 100.0, 0.0, 0.0, 1.0);
    // return vec4<f32>(abs(old_area - new_area) * 100000.0, 0.0, 0.0, 1.0);
    // return vec4<f32>(input.new_pos, 1.0);
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    //
    // col = vec4<f32>(pow(col.rgb, 2.2 * vec3<f32>(1,1,1)), 1.0); // convert from linear to srgb
    return col;
}
