
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions,
    pbr_fragment::pbr_input_from_standard_material,
    mesh_view_bindings::globals,
}
#import bevy_render::color_operations::hsv_to_rgb;

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> since: f32;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var<uniform> speed: f32;

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool,) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    var out: FragmentOutput;
    out.color = pbr_input.material.base_color;
    out.color.a *= clamp((globals.time - since) * speed - in.uv_b.x, 0.0, 1.0);
    return out;
}
