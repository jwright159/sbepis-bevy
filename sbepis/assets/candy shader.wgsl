#import bevy_pbr::mesh_view_bindings::globals;

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
}
#endif

@fragment
fn fragment(
    in: VertexOutput,
) -> FragmentOutput {
	let dis = 0.5;
	let o = in.uv_b;
	let angle = atan2(o.x, o.y);
	let l = length(o);
	let offset = l + (angle / radians(360.0)) * dis;
	let circles = select(
		vec4(0.992, 0, 0, 1.0),
		vec4(0.094, 0.906, 0, 1.0),
		(offset - globals.time * 0.2) % dis / dis > -0.5
	);

    var out: FragmentOutput;
    out.color = circles;
    return out;
}
