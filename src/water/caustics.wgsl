#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

#import bevy_water::water_bindings
#import bevy_water::water_functions as water_fn

struct CausticsMaterial {
    plane: vec4<f32>,
    light: vec4<f32>,
};
@group(2) @binding(0) var<uniform> material: CausticsMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    // @location(1) blend_color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) old_pos: vec3<f32>,
    @location(1) new_pos: vec3<f32>,
};

fn line_plane_intercept(line_pos: vec3<f32>, line_normal: vec3<f32>, plane: vec4<f32>) -> vec3<f32> {
    // Unoptimized
    let distance: f32 = (plane.w - dot(plane.xyz, line_pos)) / dot(line_normal, plane.xyz);

    // Optimized (assumes planeN always points up)
    // let distance: f32 = (planeD - line_pos.y) / line_normal.y;

    return line_pos + line_normal * distance;
}

const IOR_AIR: f32 = 1.0;
const IOR_WATER: f32 = 1.333;

fn project(origin: vec3<f32>, ray: vec3<f32>, refractedLight: vec3<f32>, plane: vec4<f32>) -> vec3<f32> {
    // This is close to the above.  But the cube has some different aspects when
    // the ray intersects with the side of the cube.  It has a softer patch.
    // plane.w = -_MaxDepth;
    let o = line_plane_intercept(origin, ray, plane);
    // o is now where the ray intersects the plane on the water surface.
    // let tplane: f32 = (-1.0 - o.y) / refractedLight.y;
    // tplane is the distance to the XZ plane situated at (0, -1, 0).
    // var proj: vec3<f32> = o + refractedLight * tplane;
    // proj is the intersection of the refractedLight onto the XZ plane at (0, -1, 0).
    // The following preserves the behavior entirely.
    // plane.w = -1.0;
    let proj = line_plane_intercept(o, refractedLight, plane);
    // let distance: f32 = (planeD - dot(planeN, lineP)) / dot(lineN, planeN);
    // return origin + refractedLight * tplane;
    return proj;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let w_pos = water_fn::uv_to_coord(vertex.uv);

    // Calculate normal.
    let delta = 0.2;
    let height = water_fn::get_wave_height(w_pos);
    let height_dx = water_fn::get_wave_height(w_pos + vec2<f32>(delta, 0.0));
    let height_dz = water_fn::get_wave_height(w_pos + vec2<f32>(0.0, delta));
    let normal = vec3<f32>(height - height_dx, delta, height - height_dz);


    let refracted_light = refract(-material.light.xyz, material.plane.xyz, IOR_AIR / IOR_WATER);
    let ray = refract(-material.light.xyz, normal, IOR_AIR / IOR_WATER);
    let uv_pos = vec3<f32>(vertex.uv.x, 0.5, vertex.uv.y) * 2.0 - 1.0;
    out.old_pos = project(uv_pos, refracted_light, refracted_light, material.plane);
    out.new_pos = project(uv_pos, ray, refracted_light, material.plane);
    out.clip_position = vec4<f32>(out.new_pos.xz + refracted_light.xz / refracted_light.y, 0.0, 1.0);
    // out.vertex.y *= -1;


    // let world_pos = mesh_position_local_to_world(
    //     get_world_from_local(vertex.instance_index),
    //     vec4<f32>(vertex.position, 1.0),
    // );

    return out;
}

struct FragmentInput {
    @location(0) old_pos: vec3<f32>,
    @location(1) new_pos: vec3<f32>,
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let old_area = length(dpdx(input.old_pos)) * length(dpdy(input.old_pos));
    let new_area = length(dpdx(input.new_pos)) * length(dpdy(input.new_pos));
    return vec4<f32>(old_area / new_area * 0.2, 0.0, 0.0, 1.0);
}
