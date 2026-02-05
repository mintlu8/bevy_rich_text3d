
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool,) -> FragmentOutput {
    var out: FragmentOutput;
    if in.uv_b.x == 0.0 {
        out.color = vec4(1.0, 0.0, 0.0, 1.0);
    }
    if in.uv_b.x == 1.0 {
        out.color = vec4(0.0, 1.0, 0.0, 1.0);
    }
    if in.uv_b.x == 2.0 {
        out.color = vec4(0.0, 0.0, 1.0, 1.0);
    }
    return out;
}