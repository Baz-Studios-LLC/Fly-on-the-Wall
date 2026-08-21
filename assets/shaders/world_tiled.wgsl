// World-space planar texturing for a house made of boxes.
//
// Every solid in the house is a scaled unit cube sharing one mesh, so mesh UVs
// stretch with each box's size — a texture applied that way smears along every
// long board. Instead the UV here comes from *world position*, projected onto
// the plane a fragment's normal faces: floors map by xz, walls by whichever of
// xy/zy they face. Boxes have axis faces, so the hard select never blends, and
// the texture runs continuously across every box that shares it — one floor
// reads as one floor, not forty separately-wallpapered planks.
//
// The normal map is applied in the projection's own frame, orthonormalised
// against the geometric normal, so rotated solids (roof slopes) stay correct.

#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    forward_io::{VertexOutput, FragmentOutput},
}

struct TileParams {
    // x: world size of one texture repeat, in cm
    // y: normal map strength
    // z: 1.0 swaps the uv axes (roof courses that must run the other way)
    // w: unused
    params: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> tile: TileParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var normal_tex: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var normal_smp: sampler;

@fragment
fn fragment(vin: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var in = vin;

    let n = normalize(in.world_normal);
    let an = abs(n);
    let p = in.world_position.xyz / max(tile.params.x, 1.0);

    var uv: vec2<f32>;
    var t_axis: vec3<f32>;
    var b_axis: vec3<f32>;
    if an.y >= an.x && an.y >= an.z {
        uv = vec2<f32>(p.x, p.z);
        t_axis = vec3<f32>(1.0, 0.0, 0.0);
        b_axis = vec3<f32>(0.0, 0.0, 1.0);
    } else if an.x >= an.z {
        uv = vec2<f32>(p.z, -p.y);
        t_axis = vec3<f32>(0.0, 0.0, 1.0);
        b_axis = vec3<f32>(0.0, -1.0, 0.0);
    } else {
        uv = vec2<f32>(p.x, -p.y);
        t_axis = vec3<f32>(1.0, 0.0, 0.0);
        b_axis = vec3<f32>(0.0, -1.0, 0.0);
    }
    if tile.params.z > 0.5 {
        uv = uv.yx;
        let keep = t_axis;
        t_axis = b_axis;
        b_axis = keep;
    }
    in.uv = uv;

    // Sampled before any branching downstream, and always: a texture sample in
    // a vertex-uniform spot keeps the derivatives honest.
    let sampled = textureSample(normal_tex, normal_smp, uv).xyz * 2.0 - 1.0;

    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // The projection frame, orthonormalised against the lit-side normal.
    let nn = pbr_input.N;
    var tt = t_axis - nn * dot(t_axis, nn);
    if length(tt) > 1.0e-4 {
        tt = normalize(tt);
        let bb = normalize(cross(nn, tt) * sign(dot(cross(nn, tt), b_axis) + 1.0e-6));
        let s = tile.params.y;
        pbr_input.N = normalize(tt * sampled.x * s + bb * sampled.y * s + nn * max(sampled.z, 0.05));
    }

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
    return out;
}
