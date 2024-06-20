#import bevy_pbr::{
    mesh_view_bindings::view,
parallax_mapping::parallaxed_uv,
    mesh_functions,
    skinning,
    morph::morph,
    prepass_utils::prepass_depth,
    forward_io::{Vertex, VertexOutput},
        view_transformations::{position_world_to_clip, depth_ndc_to_view_z, uv_to_ndc, ndc_to_uv},
}

#import bevy_water::caustics_binding

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let weight_count = bevy_pbr::morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph(vertex.index, bevy_pbr::morph::position_offset, i);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph(vertex.index, bevy_pbr::morph::normal_offset, i);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph(vertex.index, bevy_pbr::morph::tangent_offset, i), 0.0);
#endif
    }
    return vertex;
}
#endif

fn light_project(in: VertexOutput) -> vec4<f32> {

    // let position = vertex.position + vec3<f32>(0, vertex.position.x, 0);
    //
    //
    let v_ray = in.world_position.xyz - caustics_binding::material.light_dir.xyz;
    let view_ray = normalize(view.world_position - in.world_position.xyz);
    let model = bevy_pbr::mesh_functions::get_model_matrix(in.instance_index);
    let scale = (model * vec4(1.0, 1.0, 1.0, 0.0)).xyz;

    // view vector
    let V = normalize(v_ray);
    let N = in.world_normal;
    let T = in.world_tangent.xyz / scale;
    let B = in.world_tangent.w * cross(N, T); // bitangent
    // Transform V from fragment to camera in world space to tangent space.
    let Vt = vec3(dot(V, T), dot(V, B), dot(V, N));
    let uv = parallaxed_uv(1.0, 1.0, 0u, in.position.xy, Vt); // is this uv or clip space?
    // let depth_pass_depth = depth_ndc_to_view_z(prepass_depth(vec4<f32>(ndc_to_uv(uv), 0, 0), 0u));
    let depth_pass_depth = prepass_depth(vec4<f32>(ndc_to_uv(uv), 0, 0), 0u);
    // return vec4<f32>(uv_to_ndc(uv.xy), in.position.zw);
    return vec4<f32>(uv_to_ndc(uv.xy), depth_pass_depth, in.position.w);
}

@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var model = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var model = mesh_functions::get_model_matrix(vertex_no_morph.instance_index);
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        model,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif
    out.position = light_project(out);

    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
#ifdef VERTEX_COLORS
    return mesh.color;
#else
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
#endif
}
