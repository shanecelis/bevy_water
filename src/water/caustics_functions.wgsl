#define_import_path bevy_water::caustics_functions

const IOR_AIR: f32 = 1.0;
const IOR_WATER: f32 = 1.333;
const IOR: f32 = IOR_AIR / IOR_WATER;

fn line_plane_intercept(line_pos: vec3<f32>, line_normal: vec3<f32>, plane: vec4<f32>) -> vec3<f32> {
    // Unoptimized
    let distance: f32 = (plane.w - dot(plane.xyz, line_pos)) / dot(line_normal, plane.xyz);

    // Optimized (assumes planeN always points up)
    // let distance: f32 = (planeD - line_pos.y) / line_normal.y;

    return line_pos + line_normal * distance;
}

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

/* Distance of position to the water_plane. Negative implies it's under the water_plane. */
fn distance_to_plane(position: vec3<f32>, water_plane: vec4<f32>) -> f32 {
  return dot(water_plane.xyz, position) - water_plane.w;
}
