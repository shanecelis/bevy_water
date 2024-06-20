#define_import_path bevy_water::caustics_functions

const IOR_AIR: f32 = 1.0;
const IOR_WATER: f32 = 1.333;
const IOR: f32 = IOR_AIR / IOR_WATER;
const underwater_color: vec4<f32> = vec4<f32>(0.4, 0.9, 1.0, 1.0);

fn line_plane_intercept(line_pos: vec3<f32>, line_normal: vec3<f32>, plane: vec4<f32>) -> vec3<f32> {
    // Unoptimized
    let distance: f32 = (plane.w - dot(plane.xyz, line_pos)) / dot(line_normal, plane.xyz);

    // Optimized (assumes planeN always points up)
    // let distance: f32 = (planeD - line_pos.y) / line_normal.y;

    return line_pos + line_normal * distance;
}

fn intersect_cube(origin: vec3<f32>, ray: vec3<f32>, cubeMin: vec3<f32>, cubeMax: vec3<f32>) -> vec2<f32> {
    let tMin: vec3<f32> = (cubeMin - origin) / ray;
    let tMax: vec3<f32> = (cubeMax - origin) / ray;
    let t1: vec3<f32> = min(tMin, tMax);
    let t2: vec3<f32> = max(tMin, tMax);
    let tNear: f32 = max(max(t1.x, t1.y), t1.z);
    let tFar: f32 = min(min(t2.x, t2.y), t2.z);
    return vec2<f32>(tNear, tFar);
}

fn project0(origin_: vec3<f32>, ray: vec3<f32>, refracted_light: vec3<f32>, plane: vec4<f32>) -> vec3<f32> {
    var origin = origin_;
    let tcube = intersect_cube(origin, ray, vec3<f32>(-1.0, plane.w, -1.0), vec3<f32>(1.0, 2.0, 1.0));
    origin += ray * tcube.y;
    let tplane = (-origin.y - 1.0) / refracted_light.y;
    return origin + refracted_light * tplane;
}

fn project(origin: vec3<f32>, ray: vec3<f32>, refracted_light: vec3<f32>, plane: vec4<f32>) -> vec3<f32> {
    let o = line_plane_intercept(origin, ray, plane);
    // o is now where the ray intersects the plane on the water surface.
    // let tplane: f32 = (-1.0 - o.y) / refracted_light.y;
    // tplane is the distance to the XZ plane situated at (0, -1, 0).
    // var proj: vec3<f32> = o + refracted_light * tplane;
    // proj is the intersection of the refracted_light onto the XZ plane at (0, -1, 0).
    // The following preserves the behavior entirely.
    // plane.w = -1.0;
    let proj = line_plane_intercept(o, refracted_light, vec4<f32>(plane.xyz, 0));
    // let distance: f32 = (planeD - dot(planeN, lineP)) / dot(lineN, planeN);
    // return origin + refracted_light * tplane;
    return proj;
}

/* Signed distance of position to the plane. Negative implies it's under the plane. */
fn distance_to_plane(position: vec3<f32>, plane: vec4<f32>) -> f32 {
  return dot(plane.xyz, position) - plane.w;
}
