
#import bevy_pbr::{
    pbr_functions,
    mesh_functions,
    mesh_view_bindings::globals,
    view_transformations::position_world_to_clip,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
}
#import bevy_render::color_operations::hsv_to_rgb;

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> frequency: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var<uniform> intensity: f32;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
    @location(30) text_attribute: vec4<f32>,
#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif
};

struct VertexOutput1 {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    @location(6) @interpolate(flat) instance_index: u32,
#endif
#ifdef VISIBILITY_RANGE_DITHER
    @location(7) @interpolate(flat) visibility_range_dither: i32,
#endif
    @location(666) text_attribute: vec4<f32>,
}
@vertex
fn vertex(vertex: Vertex) -> VertexOutput1 {
    var out: VertexOutput1;

    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);

#ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index
    );
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    let t = globals.time * frequency * 5.0 + vertex.text_attribute.x * 12.0;
    // Wobble effect.
    let x = sin(t) * cos(t * 1.3 + vertex.text_attribute.x * 8.0);
    let y = cos(t) * sin(t * 3.7 + vertex.text_attribute.x * 3.0);
    let pos = out.world_position.xyz + vec3(x, y, 0.0) * intensity * vertex.text_attribute.y;
    // Add a wave effect to non-wobbling items.
    let y2 = cos(t / 3.0 + vertex.text_attribute.a / 2.0) * 4.0; 
    let pos2 = pos + vec3(0.0, y2, 0.0) * (1.0 - vertex.text_attribute.y);
    out.position = position_world_to_clip(pos2);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

    out.text_attribute = vertex.text_attribute;

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}

@fragment
fn fragment(in: VertexOutput1, @builtin(front_facing) is_front: bool,) -> FragmentOutput {
    var vertex_output: VertexOutput;
    vertex_output.position = in.position;
    vertex_output.world_position = in.world_position;
    vertex_output.world_normal = in.world_normal;
#ifdef VERTEX_UVS_A
    vertex_output.uv = in.uv;
#endif
#ifdef VERTEX_COLORS
    vertex_output.color = in.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    vertex_output.instance_index = in.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    vertex_output.visibility_range_dither = in.visibility_range_dither;
#endif

    var pbr_input = pbr_input_from_standard_material(vertex_output, is_front);
    var out: FragmentOutput;
    out.color = pbr_input.material.base_color;
    out.color *= vec4(hsv_to_rgb(vec3(in.text_attribute.z + globals.time, 1.0, 0.5)), 1.0);
    return out;
}