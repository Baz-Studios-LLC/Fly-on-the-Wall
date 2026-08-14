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
const CELL: f32 = 12.0;

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
fn triangles(
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

        // A grid of probes straight down onto it, drawn as specks. It is the
        // only way to see collision that has no geometry of its own: each speck
        // sits exactly where the fly's feet would.
        if show {
            let (cube, skin) = (
                meshes.add(Cuboid::from_length(1.0)),
                materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.35, 0.15),
                    unlit: true,
                    ..default()
                }),
            );
            let mut probes = 0;
            for gx in 0..44 {
                for gz in 0..70 {
                    let from = Vec3::new(
                        hull.bounds().0.x + gx as f32 * 2.5,
                        hull.bounds().1.y + 20.0,
                        hull.bounds().0.z + gz as f32 * 2.5,
                    );
                    if let Some((d, _)) = hull.raycast(from, Vec3::NEG_Y, 260.0) {
                        commands.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(skin.clone()),
                            Transform::from_translation(from + Vec3::NEG_Y * d)
                                .with_scale(Vec3::splat(0.9)),
                            bevy::light::NotShadowCaster,
                        ));
                        probes += 1;
                    }
                }
            }
            info!("made model: {probes} probes landed on the mesh");
        }

        info!(
            "made model: {count} triangles as collision, in {} cm cells",
            CELL
        );
        home.hulls.push(hull);
        commands.entity(root).remove::<NeedsHull>();
    }
}
