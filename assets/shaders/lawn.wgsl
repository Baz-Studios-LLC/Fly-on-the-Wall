// Lawn, main pass.
//
// The vertex stage builds blades from nothing (see lawn_blade.wgsl). The
// fragment stage assembles a PbrInput by hand and hands it to Bevy's own
// lighting, which gets the sun, its shadows and tonemapping for free — the
// house's shadow falls across the lawn like anything else's.

#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::view,
    pbr_functions,
    pbr_types,
    view_transformations::position_world_to_clip,
}
#import lawn::blade::{blade_vertex, params}

struct VertexIn {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

// A private vertex output: the dummy mesh deliberately has nothing but
// positions, so bevy's forward_io::VertexOutput (which gates fields behind
// mesh-derived shader defs) cannot be used.
struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) height_t: f32,
}

@vertex
fn vertex(in: VertexIn) -> VertexOut {
    let blade = blade_vertex(in.instance_index, in.vertex_index, params.misc.z);

    var out: VertexOut;
    out.world_position = vec4<f32>(blade.world_position, 1.0);
    out.clip_position = position_world_to_clip(blade.world_position);
    out.world_normal = blade.world_normal;
    out.color = blade.color;
    out.height_t = blade.height_t;
    return out;
}

@fragment
fn fragment(in: VertexOut, @builtin(front_facing) is_front: bool) -> @location(0) vec4<f32> {
    var pbr_input = pbr_types::pbr_input_new();

    pbr_input.material.base_color = vec4<f32>(in.color, 1.0);
    // Waxy: rough enough not to look wet, smooth enough to catch the sheen
    // that makes a field of blades read as three dimensional.
    pbr_input.material.perceptual_roughness = 0.52;
    pbr_input.material.reflectance = vec3<f32>(0.06);
    pbr_input.material.metallic = 0.0;

    pbr_input.frag_coord = in.clip_position;
    pbr_input.world_position = in.world_position;
    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;

    // Blades are single-sided ribbons drawn with culling off; flip the normal
    // on back faces or half the field lights as though facing away.
    let normal = select(-in.world_normal, in.world_normal, is_front);
    pbr_input.world_normal = normal;
    pbr_input.N = normalize(normal);
    pbr_input.V = pbr_functions::calculate_view(in.world_position, pbr_input.is_orthographic);

    var color = pbr_functions::apply_pbr_lighting(pbr_input);

    // Wrap lighting, approximating sun through a thin blade: the unlit side
    // still glows rather than going flat black. Strongest at the tips.
    let wrap = params.look.w * in.height_t;
    if wrap > 0.0 {
        let ndotl = dot(pbr_input.N, pbr_input.V);
        let transmitted = max(0.0, -ndotl) * wrap;
        color = vec4<f32>(color.rgb + in.color * transmitted * 0.35, color.a);
    }

    return pbr_functions::main_pass_post_lighting_processing(pbr_input, color);
}
