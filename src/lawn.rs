//! The lawn: real blades of grass, generated on the GPU.
//!
//! Ported from Brett's Grass Test project — the Ghost-of-Tsushima family of
//! technique. There is no blade mesh and no per-blade data anywhere in memory:
//! each chunk draws a dummy mesh whose only purpose is to run the vertex
//! shader `blades × 7` times, and the shader builds every vertex from a hash
//! of its own index. See `assets/shaders/lawn_blade.wgsl`.
//!
//! Cut down from the demo on purpose. The ground here is a flat plane, so the
//! per-chunk height grid is gone; nothing rolls across this lawn, so the
//! trample map and push are gone; the game runs neither SSAO nor temporal
//! anti-aliasing, so the custom prepass is gone. What stays is what a window
//! looks out on: clumped blades, dry strays, wind with gusts, and the
//! screen-space width floor that keeps the field stable at a distance.
//!
//! Everything is deterministic: blades come from integer chunk coordinates and
//! wind from elapsed time, so two captures at the same delay are comparable.

use bevy::{
    asset::RenderAssetUsages,
    camera::{primitives::Aabb, visibility::NoAutoAabb},
    light::NotShadowCaster,
    mesh::{Indices, MeshTag},
    prelude::*,
    render::{
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderType},
        storage::ShaderBuffer,
    },
    shader::{Shader, ShaderRef},
};

const LAWN_SHADER: &str = "shaders/lawn.wgsl";
/// The shared blade module. Declared with `#define_import_path`, so it must be
/// loaded explicitly and its handle held — an import missing from the shader
/// cache does not error, the material simply never draws.
const LAWN_BLADE_SHADER: &str = "shaders/lawn_blade.wgsl";

/// One chunk of lawn, in centimetres. Mirrored in `lawn_blade.wgsl`.
const CHUNK: f32 = 400.0;
/// Spine segments per blade; vertices are `2n + 1`. Mirrored in the shader.
const SEGMENTS: u32 = 3;
const VERTS: u32 = 2 * SEGMENTS + 1;

/// Blades in every chunk. Around 800 a square metre: the lawn is one lot seen
/// mostly through windows, and this machine plays far heavier games.
const BLADES: u32 = 13000;

/// How high the ground plane sits. The house's lawn slab tops out at -10, so
/// blades root there and their tips stay below the interior floors at zero.
const GROUND: f32 = -10.0;

/// Per-chunk record, indexed by [`MeshTag`]. Mirrored in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct ChunkMeta {
    chunk_coord: IVec2,
    blade_count: u32,
    pad: u32,
}

/// Global lawn parameters, packed into vec4s to keep uniform padding sane.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct LawnParams {
    base_color: Vec4,
    tip_color: Vec4,
    dry_color: Vec4,
    /// `xy` unit wind direction, `z` strength, `w` speed.
    wind: Vec4,
    /// `x` height, `y` height jitter, `z` width, `w` curvature.
    shape: Vec4,
    /// `x` clump radius, `y` clump pull, `z` clump height variation,
    /// `w` blades per clump.
    clump: Vec4,
    /// `x` dry fraction, `y` colour jitter, `z` baked AO, `w` translucency.
    look: Vec4,
    /// `x` gust wavelength, `y` min pixel width, `z` time, `w` ground height.
    misc: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct LawnMaterial {
    #[uniform(0)]
    params: LawnParams,
    #[storage(1, read_only)]
    chunk_meta: Handle<ShaderBuffer>,
}

impl Material for LawnMaterial {
    fn vertex_shader() -> ShaderRef {
        LAWN_SHADER.into()
    }
    fn fragment_shader() -> ShaderRef {
        LAWN_SHADER.into()
    }
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        // Blades are single-sided ribbons seen from both sides; the fragment
        // shader flips the normal for back faces instead of culling them away.
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

#[derive(Resource)]
struct LawnAssets {
    material: Handle<LawnMaterial>,
    _blade_module: Handle<Shader>,
}

pub struct LawnPlugin;

impl Plugin for LawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LawnMaterial>::default())
            .add_systems(Startup, plant_the_lawn)
            .add_systems(Update, let_the_wind_blow);
    }
}

/// Which chunks get grass: everything on the lot except ground the house or
/// its paving already owns. A chunk fully under the building or the drive
/// would pay for blades nothing can ever see.
fn wanted(coord: IVec2) -> bool {
    let lo = coord.as_vec2() * CHUNK;
    let hi = lo + Vec2::splat(CHUNK);

    let inside = |b_lo: Vec2, b_hi: Vec2| -> bool {
        lo.x >= b_lo.x && hi.x <= b_hi.x && lo.y >= b_lo.y && hi.y <= b_hi.y
    };

    // The house, porches included, with the walls' thickness around it.
    let (bmin, bmax) = crate::house::bounds();
    if inside(bmin - 30.0, bmax + 30.0) {
        return false;
    }
    // The drive.
    let garage = crate::house::room("garage");
    if inside(
        Vec2::new(garage.middle().x - 290.0, garage.max.y),
        Vec2::new(garage.middle().x + 290.0, garage.max.y + 950.0),
    ) {
        return false;
    }
    true
}

fn plant_the_lawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut materials: ResMut<Assets<LawnMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // The lot: past the walls on every side, out to the sidewalk in front and
    // the fence line behind. Beyond it the flat green plane carries on, which
    // at that distance reads as lawn mown shorter.
    let (min_c, max_c) = (IVec2::new(-3, -2), IVec2::new(7, 6));

    let mut coords = Vec::new();
    for z in min_c.y..=max_c.y {
        for x in min_c.x..=max_c.x {
            let coord = IVec2::new(x, z);
            if wanted(coord) {
                coords.push(coord);
            }
        }
    }

    let meta: Vec<ChunkMeta> = coords
        .iter()
        .map(|&chunk_coord| ChunkMeta {
            chunk_coord,
            blade_count: BLADES,
            pad: 0,
        })
        .collect();
    let chunk_meta = buffers.add(ShaderBuffer::from(meta));

    let material = materials.add(LawnMaterial {
        params: LawnParams::default(),
        chunk_meta: chunk_meta.clone(),
    });

    let mesh = meshes.add(dummy_mesh(BLADES));

    for (slot, &coord) in coords.iter().enumerate() {
        let origin = coord.as_vec2() * CHUNK;
        commands.spawn((
            Name::new(format!("lawn {},{}", coord.x, coord.y)),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            MeshTag(slot as u32),
            // Blades position themselves in world space from the chunk coord;
            // the transform exists so something owns the AABB.
            Transform::IDENTITY,
            // The dummy mesh is all zeros, so the AABB must be authored or
            // every chunk culls to a point at the origin.
            Aabb {
                center: Vec3::new(origin.x + CHUNK * 0.5, GROUND + 6.0, origin.y + CHUNK * 0.5)
                    .into(),
                half_extents: Vec3::new(CHUNK * 0.5 + 40.0, 16.0, CHUNK * 0.5 + 40.0).into(),
            },
            NoAutoAabb,
            // Blades are far below shadow-cascade resolution; their depth
            // comes from the baked base darkening instead.
            NotShadowCaster,
        ));
    }

    commands.insert_resource(LawnAssets {
        material,
        _blade_module: asset_server.load::<Shader>(LAWN_BLADE_SHADER),
    });
}

/// The dummy mesh: positions all zero and never read. Indexed, so a blade is
/// `2n + 1` vertices rather than the `6n - 3` of a raw triangle list.
fn dummy_mesh(blades: u32) -> Mesh {
    let vertex_count = (blades * VERTS) as usize;
    let mut indices = Vec::with_capacity((blades * (2 * SEGMENTS - 1) * 3) as usize);
    for blade in 0..blades {
        let base = blade * VERTS;
        for s in 0..SEGMENTS - 1 {
            let l0 = base + 2 * s;
            indices.extend_from_slice(&[l0, l0 + 1, l0 + 2, l0 + 1, l0 + 3, l0 + 2]);
        }
        let l = base + 2 * (SEGMENTS - 1);
        indices.extend_from_slice(&[l, l + 1, base + VERTS - 1]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0f32, 0.0, 0.0]; vertex_count],
    )
    .with_inserted_indices(Indices::U32(indices))
}

fn let_the_wind_blow(
    time: Res<Time>,
    assets: Option<Res<LawnAssets>>,
    mut materials: ResMut<Assets<LawnMaterial>>,
) {
    let Some(assets) = assets else {
        return;
    };
    let Some(mut material) = materials.get_mut(&assets.material) else {
        return;
    };

    let wind_dir = Vec2::from_angle(0.6);
    material.params = LawnParams {
        base_color: Vec4::from_array(
            LinearRgba::from(Color::srgb(0.16, 0.30, 0.09)).to_f32_array(),
        ),
        tip_color: Vec4::from_array(LinearRgba::from(Color::srgb(0.45, 0.65, 0.23)).to_f32_array()),
        dry_color: Vec4::from_array(LinearRgba::from(Color::srgb(0.56, 0.50, 0.21)).to_f32_array()),
        wind: Vec4::new(wind_dir.x, wind_dir.y, 0.32, 0.6),
        // Suburban lawn, not meadow: short blades, moderate spread.
        shape: Vec4::new(7.0, 0.4, 1.1, 1.25),
        clump: Vec4::new(16.0, 0.45, 0.45, 90.0),
        look: Vec4::new(0.08, 0.26, 0.45, 0.5),
        misc: Vec4::new(2200.0, 1.3, time.elapsed_secs(), GROUND),
    };
}
