//! Collision for made models.
//!
//! A model arrives as ten thousand triangles and the rest of this house
//! collides as oriented boxes. That is exact where it is used — a wall *is* a
//! box, so nothing is being approximated — but a couch is not a box, and the
//! first version of this file turned one into a voxel hull and argued that
//! coarse was fine because "the upholstery does not need to be accurate to a
//! quarter of a centimetre".
//!
//! That reasoning is wrong for this game and the note is kept here because it
//! is the kind of wrong that sounds sensible. Seven centimetres is nothing in a
//! game about a person and **fourteen body lengths** to the thing landing on it
//! here. The hull stood proud of the arms, filled the dip in the seat, and
//! bridged the gap between the cushions.
//!
//! So a model collides against its own triangles: the mesh you can see is the
//! surface you land on. The grid below is a filing system for those triangles —
//! it decides how many a query has to consider and nothing whatever about
//! accuracy.
//!
//! Everything downstream already worked at this scale and needed no changes.
//! Flight sweeps from the previous position, so nothing tunnels; walking
//! re-seats onto whatever is underfoot every step, so it crawls the real
//! cushions and takes their normals. Both go through `Home`, and `Home` now
//! asks the hulls as well as the boxes.

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

use crate::world::Home;

/// How big a lookup cell is, in centimetres. This is only a filing system for
/// triangles now, not the collision itself — it decides how many triangles a
/// query has to consider, and nothing about accuracy.
pub const CELL: f32 = 12.0;

/// A model whose collision has not been worked out yet.
#[derive(Component)]
pub struct NeedsHull {
    /// The solid carrying the model: a perch is stored relative to it, and it
    /// is what moves when arrange mode moves the piece.
    pub solid: usize,
}

pub struct MadePlugin;

impl Plugin for MadePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fit_collision);
    }
}

/// Walk a scene and collect every triangle in it, in world centimetres.
///
/// Public because a person is now the same problem as a made model: a hierarchy
/// of meshes that has to become something the fly can land on.
pub fn triangles(
    root: Entity,
    children: &Query<&Children>,
    drawn: &Query<(&Mesh3d, &GlobalTransform)>,
    meshes: &Assets<Mesh>,
    out: &mut Vec<[Vec3; 3]>,
) {
    if let Ok((handle, transform)) = drawn.get(root) {
        if let Some(mesh) = meshes.get(&handle.0) {
            let Some(VertexAttributeValues::Float32x3(points)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                return;
            };
            let to_world = transform.affine();
            let at = |i: usize| {
                let p = points[i];
                to_world.transform_point3(Vec3::new(p[0], p[1], p[2]))
            };
            match mesh.indices() {
                Some(indices) => {
                    let list: Vec<usize> = indices.iter().collect();
                    for tri in list.chunks_exact(3) {
                        out.push([at(tri[0]), at(tri[1]), at(tri[2])]);
                    }
                }
                None => {
                    for tri in (0..points.len()).collect::<Vec<_>>().chunks_exact(3) {
                        out.push([at(tri[0]), at(tri[1]), at(tri[2])]);
                    }
                }
            }
        }
    }
    if let Ok(kids) = children.get(root) {
        for kid in kids.iter() {
            triangles(kid, children, drawn, meshes, out);
        }
    }
}

fn fit_collision(
    mut commands: Commands,
    mut home: ResMut<Home>,
    waiting: Query<(Entity, &NeedsHull)>,
    children: Query<&Children>,
    drawn: Query<(&Mesh3d, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // `FLY_HULL=1` draws what the fly will actually hit. Collision that cannot
    // be seen is collision nobody can check, and a hull derived from a mesh is
    // exactly the kind of thing that looks right in a log line and is wrong in
    // the room.
    let show = std::env::var("FLY_HULL").is_ok();
    for (root, needs) in &waiting {
        let mut tris = Vec::new();
        triangles(root, &children, &drawn, &meshes, &mut tris);
        // The scene loads over several frames; nothing yet just means not yet.
        if tris.is_empty() {
            continue;
        }

        // The mesh itself is the collision now. A voxel hull was the first
        // answer and it was a seven-centimetre shell round a couch: proud of
        // the arms, filling the dip in the seat, bridging the gap between the
        // cushions. Fine for a person-sized game and hopeless here, where the
        // thing landing on it is five millimetres long.
        let count = tris.len();
        let hull = crate::world::Hull::new(needs.solid, tris, CELL);

        if show {
            probe(&mut commands, &hull, &mut meshes, &mut materials);
        }

        let (low, high) = hull.bounds();
        // The height of whatever is on top at the middle of it: a seat, a
        // table top, a shelf. It is the number anybody actually wants when
        // putting something — or somebody — on a model, and reading it off the
        // collision means it stays right when the model is replaced.
        // A profile across it, so the shape of a seat is legible from the log:
        // a sofa reads as a tall back and a low seat, and which side is which
        // says which way it faces. Guessing that from the code that placed it
        // does not work — the generated sofa was thrown away and the model
        // brought its own orientation with it.
        let across: Vec<String> = (0..=6)
            .map(|k| {
                let x = low.x + (high.x - low.x) * k as f32 / 6.0;
                let from = Vec3::new(x, high.y + 10.0, (low.z + high.z) * 0.5);
                let h = hull
                    .raycast(from, Vec3::NEG_Y, high.y - low.y + 40.0)
                    .map(|(d, _)| from.y - d)
                    .unwrap_or(low.y);
                format!("{h:.0}")
            })
            .collect();
        info!("   surface across x, west to east: {}", across.join("  "));
        let middle = Vec3::new(
            (low.x + high.x) * 0.5,
            high.y + 10.0,
            (low.z + high.z) * 0.5,
        );
        let surface = hull
            .raycast(middle, Vec3::NEG_Y, high.y - low.y + 40.0)
            .map(|(d, _)| middle.y - d)
            .unwrap_or(low.y);
        info!(
            "made model: piece {} — {count} triangles as collision, in {} cm cells; \
             {:.0} x {:.0} x {:.0} cm about ({:.0}, {:.0}, {:.0}), top surface at {surface:.0}",
            home.solids[needs.solid].piece,
            CELL,
            high.x - low.x,
            high.y - low.y,
            high.z - low.z,
            (low.x + high.x) * 0.5,
            low.y,
            (low.z + high.z) * 0.5,
        );
        home.hulls.push(hull);
        commands.entity(root).remove::<NeedsHull>();
    }
}

/// A drawn collision speck. Marked so the turntable keeps it: the studio hides
/// everything that is not a person, and hiding the collision would defeat the
/// only diagnostic there is for it.
#[derive(Component)]
pub struct Probe;

/// Draw what the fly will actually hit, as a grid of specks dropped onto it.
///
/// Collision that cannot be seen is collision nobody can check, and a hull
/// derived from a mesh is exactly the kind of thing that looks right in a log
/// line and is wrong in the room. Each speck sits where the fly's feet would.
///
/// The grid is sized from the hull's own bounds, so it works for a couch, an
/// armchair or a man without anybody choosing a number for each.
pub fn probe(
    commands: &mut Commands,
    hull: &crate::world::Hull,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    const STEP: f32 = 2.5;
    let (low, high) = hull.bounds();
    let (cube, skin) = (
        meshes.add(Cuboid::from_length(1.0)),
        materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.35, 0.15),
            unlit: true,
            ..default()
        }),
    );
    let span = high - low;
    let (across, along) = (
        (span.x / STEP).ceil() as i32 + 1,
        (span.z / STEP).ceil() as i32 + 1,
    );
    let mut landed = 0;
    for gx in 0..across {
        for gz in 0..along {
            let from = Vec3::new(
                low.x + gx as f32 * STEP,
                high.y + 20.0,
                low.z + gz as f32 * STEP,
            );
            if let Some((d, _)) = hull.raycast(from, Vec3::NEG_Y, span.y + 60.0) {
                commands.spawn((
                    Probe,
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(skin.clone()),
                    Transform::from_translation(from + Vec3::NEG_Y * d)
                        .with_scale(Vec3::splat(0.9)),
                    bevy::light::NotShadowCaster,
                ));
                landed += 1;
            }
        }
    }
    info!("hull probe: {landed} of {} landed", across * along);
}

/// Somewhere to sit on a piece of furniture, worked out from its collision.
pub struct Seat {
    /// The middle of the cushion, at cushion height.
    pub at: Vec3,
    /// The way somebody sitting on it would face: away from the back.
    pub facing: Vec3,
}

/// Find the seat on a sofa or a chair.
///
/// Brett resized the sofa in arrange mode and saved it, which is exactly the
/// event that should not break anything and did: the father had been sat at
/// hand-measured coordinates, and the moment the furniture under him changed
/// size he was sitting in the air beside it. Coordinates copied out of a log
/// are a snapshot of one afternoon.
///
/// So the seat is found rather than remembered. Probes are dropped over the
/// model's own footprint and sorted by how high they land: a cushion is the
/// low plateau, a back is the high one, and the direction from one to the other
/// is the way the thing faces. Everything is measured as a fraction of the
/// piece's own height, so it survives being scaled.
pub fn seat(hull: &crate::world::Hull) -> Option<Seat> {
    const STEP: f32 = 3.0;
    let (low, high) = hull.bounds();
    let tall = high.y - low.y;
    if tall < 20.0 {
        return None;
    }

    // Drop a probe every three centimetres and keep where each one landed.
    let mut hits: Vec<Vec3> = Vec::new();
    let mut back = (Vec3::ZERO, 0.0f32);
    let mut x = low.x;
    while x <= high.x {
        let mut z = low.z;
        while z <= high.z {
            let from = Vec3::new(x, high.y + 10.0, z);
            if let Some((d, _)) = hull.raycast(from, Vec3::NEG_Y, tall + 40.0) {
                let hit = Vec3::new(x, from.y - d, z);
                let up = (hit.y - low.y) / tall;
                if (0.18..0.66).contains(&up) {
                    hits.push(hit);
                } else if up > 0.74 {
                    back.0 += hit;
                    back.1 += 1.0;
                }
            }
            z += STEP;
        }
        x += STEP;
    }

    // A cushion is a *plateau*, and saying "anything at roughly seat height" is
    // not the same thing. A ray dropped just off the back edge of a sofa grazes
    // the sloping outside of it and lands anywhere between the top and the
    // floor, so a band alone collects a scatter of hits behind the seat and
    // drags the back edge a foot into the upholstery — which is where the
    // father sat, inside his own sofa.
    //
    // So: bin the heights, take the fullest bin, and keep only what is level
    // with it. Graze hits spread across every bin; a cushion fills one.
    const BIN: f32 = 2.0;
    let mut bins: std::collections::HashMap<i32, u32> = Default::default();
    for hit in &hits {
        *bins.entry((hit.y / BIN).round() as i32).or_default() += 1;
    }
    let (level, count) = bins.into_iter().max_by_key(|&(_, n)| n)?;
    if count < 8 {
        return None;
    }
    let level = level as f32 * BIN;
    let cushion: Vec<Vec3> = hits
        .into_iter()
        .filter(|hit| (hit.y - level).abs() < 4.0)
        .collect();
    if cushion.len() < 8 {
        return None;
    }

    let middle = cushion.iter().copied().sum::<Vec3>() / cushion.len() as f32;
    let facing = if back.1 > 3.0 {
        (middle - back.0 / back.1).with_y(0.0).normalize_or(Vec3::X)
    } else {
        Vec3::X
    };
    // Not the middle of the cushion: the back of it. People sit back, and a
    // body placed at the centroid of a two-metre sofa has its knees over the
    // front edge and its shoulders a foot clear of the cushions.
    let hard_back = cushion
        .iter()
        .map(|p| p.dot(facing))
        .fold(f32::INFINITY, f32::min);
    let at = middle + facing * (hard_back - middle.dot(facing) + 6.0);
    Some(Seat {
        at: at.with_y(level),
        facing,
    })
}
