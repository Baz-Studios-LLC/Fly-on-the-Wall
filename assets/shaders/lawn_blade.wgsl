// Shared lawn blade construction.
//
// Ported from the Grass Test project and cut down for this game: the ground is
// a flat plane, nothing rolls over the lawn to trample or push it, and there is
// no prepass because the game runs neither SSAO nor temporal anti-aliasing. One
// world unit is one centimetre, so every length in here is a hundred times the
// demo's.
//
// There is no blade mesh. Each lawn chunk draws a dummy mesh whose only job is
// to make the GPU run this shader the right number of times; the shader reads
// the vertex index, works out which blade of which clump it is, and builds the
// vertex from a hash. Nothing about any individual blade is ever stored.

#define_import_path lawn::blade

#import bevy_pbr::{
    mesh_bindings::mesh,
    mesh_functions,
    mesh_view_bindings::view,
}

// Mirrored from src/lawn.rs.
const CHUNK_SIZE: f32 = 400.0;
const SEGMENTS: u32 = 3u;
const VERTS: u32 = 7u;
const TAU: f32 = 6.2831853;

struct LawnParams {
    base_color: vec4<f32>,
    tip_color: vec4<f32>,
    dry_color: vec4<f32>,
    // xy unit wind direction, z strength, w speed
    wind: vec4<f32>,
    // x height, y height jitter, z width, w curvature
    shape: vec4<f32>,
    // x clump radius, y clump pull, z clump height variation, w blades per clump
    clump: vec4<f32>,
    // x dry fraction, y colour jitter, z baked AO, w translucency
    look: vec4<f32>,
    // x gust wavelength, y min pixel width, z time, w ground height
    misc: vec4<f32>,
}

struct ChunkMeta {
    chunk_coord: vec2<i32>,
    blade_count: u32,
    pad: u32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: LawnParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> chunk_meta: array<ChunkMeta>;

// "lowbias32" (Chris Wellons).
fn hash_u32(x: u32) -> u32 {
    var h = x;
    h ^= h >> 16u;
    h *= 0x7feb352du;
    h ^= h >> 15u;
    h *= 0x846ca68bu;
    h ^= h >> 16u;
    return h;
}

// Four random values from a single hash, by slicing the result into bytes.
fn rand4(seed: u32) -> vec4<f32> {
    let h = hash_u32(seed);
    return vec4<f32>(
        f32(h & 0xffu),
        f32((h >> 8u) & 0xffu),
        f32((h >> 16u) & 0xffu),
        f32((h >> 24u) & 0xffu),
    ) * (1.0 / 255.0);
}

struct BladeVertex {
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
    color: vec3<f32>,
    // 0 at the base, 1 at the tip.
    height_t: f32,
}

fn bezier(p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>, t: f32) -> vec3<f32> {
    let u = 1.0 - t;
    return u * u * p0 + 2.0 * u * t * p1 + t * t * p2;
}

fn bezier_tangent(p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>, t: f32) -> vec3<f32> {
    let u = 1.0 - t;
    return 2.0 * u * (p1 - p0) + 2.0 * t * (p2 - p1);
}

// How many pixels one world centimetre spans at a given distance.
fn pixels_per_unit(distance_to_camera: f32) -> f32 {
    return 0.5 * view.viewport.w * view.clip_from_view[1][1] / max(distance_to_camera, 0.01);
}

// Build one vertex of one blade, entirely from its index.
fn blade_vertex(instance_index: u32, vertex_index: u32, time: f32) -> BladeVertex {
    let slot = mesh_functions::get_tag(instance_index);
    let chunk = chunk_meta[slot];

    // `vertex_index` counts from the start of the shared vertex slab, not from
    // zero — Bevy packs meshes together and draws at an offset. Without this
    // subtraction every blade id lands past `blade_count` and the whole lawn
    // silently renders as nothing.
    let local_index = vertex_index - mesh[instance_index].first_vertex_index;

    let blade_id = local_index / VERTS;
    let local_vert = local_index % VERTS;

    // Seed from the integer chunk coordinate: two captures of the same lawn
    // have to be comparable frame to frame.
    let coord_seed = hash_u32(
        bitcast<u32>(chunk.chunk_coord.x) * 73856093u ^ bitcast<u32>(chunk.chunk_coord.y) * 19349663u
    );

    // Blades are generated per clump: clump id is the blade id divided by the
    // clump size, which needs one hash instead of a neighbourhood search.
    let blades_per_clump = max(1u, u32(params.clump.w));
    let clump_id = blade_id / blades_per_clump;

    let clump_r = rand4(coord_seed + clump_id * 2654435761u);
    let blade_r = rand4(coord_seed ^ (blade_id * 2246822519u));
    let extra_r = rand4((coord_seed ^ 0x5bf03635u) + blade_id * 2166136261u);

    // --- placement ---
    let clump_centre = clump_r.xy * CHUNK_SIZE;
    let angle = blade_r.x * TAU;
    let radius = params.clump.x * pow(blade_r.y, 0.5 + params.clump.y * 1.5);
    let local_xz = clump_centre + vec2<f32>(cos(angle), sin(angle)) * radius;

    let chunk_origin = vec2<f32>(f32(chunk.chunk_coord.x), f32(chunk.chunk_coord.y)) * CHUNK_SIZE;
    let ground = params.misc.w;
    let base_world = vec3<f32>(chunk_origin.x + local_xz.x, ground, chunk_origin.y + local_xz.y);

    // Blades past the chunk's count collapse to a point and cost only their
    // vertex invocation.
    if blade_id >= chunk.blade_count {
        var dead: BladeVertex;
        dead.world_position = base_world;
        dead.world_normal = vec3<f32>(0.0, 1.0, 0.0);
        dead.color = vec3<f32>(0.0);
        dead.height_t = 0.0;
        return dead;
    }

    // --- height ---
    let clump_height_mul = 1.0 + (clump_r.z - 0.5) * 2.0 * params.clump.z;
    // Capped: the paving slabs top out just above the soil, and a blade that
    // outgrows them spears up through the driveway. 8.5 cm keeps the tallest
    // upright blade half a centimetre under the lowest slab top.
    let blade_height = clamp(
        params.shape.x * (1.0 + (blade_r.z - 0.5) * 2.0 * params.shape.y) * clump_height_mul,
        0.5,
        8.5,
    );

    // --- bending ---
    let own_angle = blade_r.w * TAU;
    let own_dir = vec2<f32>(cos(own_angle), sin(own_angle));
    let wind_dir = params.wind.xy;
    let world_xz = base_world.xz;

    // A gust wave travelling along the wind, two incommensurate frequencies so
    // it reads as weather rather than a repeating pulse.
    let gust_phase = dot(world_xz, wind_dir) / max(params.misc.x, 0.1) - time * params.wind.w * 0.35;
    let gust = 0.5 + 0.5 * sin(gust_phase * TAU);
    let gust_fine = 0.5 + 0.5 * sin(gust_phase * TAU * 2.37 + 1.7);
    let gust_mix = mix(gust, gust_fine, 0.35);
    let flutter = sin(time * (2.5 + extra_r.z * 2.0) * params.wind.w + extra_r.z * TAU) * 0.11;
    let wind_amount = params.wind.z * (0.35 + 0.65 * gust_mix + flutter);

    // Droop varies per clump and per blade, so tufts lean together while the
    // blades within one still spread.
    let droop = params.shape.w * (0.08 + 0.8 * mix(clump_r.z, extra_r.w, 0.55));

    let bend_vec = own_dir * (droop * blade_height)
        + wind_dir * (wind_amount * blade_height * 0.8);

    let bend_len = length(bend_vec);
    var forward = own_dir;
    if bend_len > 1.0e-5 {
        forward = bend_vec / bend_len;
    }

    // Bending sideways shortens the blade vertically, or blades stretch as
    // they lean.
    let bend_frac = clamp(bend_len / blade_height, 0.0, 0.92);
    let tip_height = blade_height * sqrt(max(0.0, 1.0 - bend_frac * bend_frac));

    let p0 = vec3<f32>(0.0);
    let p1 = vec3<f32>(
        forward.x * bend_len * 0.3,
        tip_height * 0.55,
        forward.y * bend_len * 0.3,
    );
    let p2 = vec3<f32>(forward.x * bend_len, tip_height, forward.y * bend_len);

    // --- this vertex ---
    let is_tip = local_vert == VERTS - 1u;
    let segment = min(local_vert / 2u, SEGMENTS);
    var t = f32(segment) / f32(SEGMENTS);
    var side = select(-1.0, 1.0, (local_vert & 1u) == 1u);
    if is_tip {
        t = 1.0;
        side = 0.0;
    }

    let spine = bezier(p0, p1, p2, t);
    let tangent = normalize(bezier_tangent(p0, p1, p2, t));
    let side_axis = vec3<f32>(-forward.y, 0.0, forward.x);
    let face_normal = cross(side_axis, tangent);
    let taper = pow(max(1.0 - t, 0.0), 0.7);

    // Screen-space width floor: geometry narrower than a pixel turns the lawn
    // into crawling noise, so distant blades widen as their detail drops.
    let camera_distance = distance(base_world, view.world_position.xyz);
    let min_world_width = params.misc.y / max(pixels_per_unit(camera_distance), 1.0e-4);
    let width = max(params.shape.z, min_world_width) * taper;

    // --- colour ---
    var tint = mix(params.base_color.rgb, params.tip_color.rgb, pow(t, 1.35));
    if extra_r.x < params.look.x {
        tint = params.dry_color.rgb * (0.75 + 0.35 * t);
    }
    let jitter = 1.0
        + (clump_r.w - 0.5) * params.look.y
        + (extra_r.y - 0.5) * params.look.y * 0.5;
    let ao = mix(1.0 - params.look.z, 1.0, t);

    var out: BladeVertex;
    out.world_position = base_world + spine + side_axis * (side * width * 0.5);
    out.world_normal = normalize(face_normal + side_axis * (side * 0.55));
    out.color = max(tint * jitter * ao, vec3<f32>(0.0));
    out.height_t = t;
    return out;
}
