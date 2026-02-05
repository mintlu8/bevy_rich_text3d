
#import bevy_pbr::{
    pbr_functions,
    mesh_functions,
    forward_io::{Vertex, VertexOutput},
    mesh_view_bindings::globals,
    view_transformations::position_world_to_clip,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> frequency: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var<uniform> intensity: f32;


@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);

#ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index
    );
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    let t = globals.time * frequency * 5.0 + vertex.uv_b.x * 12.0;
    let x = sin(t) * cos(t * 1.3 + vertex.uv_b.x * 8.0);
    let y = cos(t) * sin(t * 3.7 + vertex.uv_b.x * 3.0);
    out.position = position_world_to_clip(out.world_position.xyz + vec3(x, y, 0.0) * intensity * vertex.uv_b.y);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}
