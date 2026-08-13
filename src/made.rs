//! Collision for made models.
//!
//! A model arrives as ten thousand triangles and this game's collision is
//! oriented boxes — a slab raycast and a closest-point clamp, which the whole
//! flight model is built on and which is not worth replacing to seat one couch.
//! So the triangles are turned into boxes: the mesh is voxelised on a coarse
//! grid and the occupied cells are merged back into as few boxes as will cover
//! them.
//!
//! That gives collision that follows the actual shape — you can land on the arm
//! of the couch and not on the gap under it — without a second collision system
//! to keep working, and without anyone hand-authoring a proxy per model. Drop a
//! model in and it gets its own.
//!
//! The grid is deliberately coarse. A fly is a quarter of a centimetre and the
//! upholstery it lands on does not need to be accurate to that: what matters is
//! that the seat is where the seat looks, and that the space under the couch is
//! open, because that is exactly the kind of place a fly goes.

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

use crate::world::{Home, Solid, Stuff};

/// How big a collision cell is, in centimetres. Fine enough that a cushion and
/// the gap beside it are different places; coarse enough that a couch is a
/// hundred boxes and not ten thousand.
const CELL: f32 = 7.0;

/// A model whose collision has not been worked out yet.
#[derive(Component)]
pub struct NeedsHull {
    /// The piece the boxes should join, so arrange mode moves them with it.
    pub piece: u32,
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
    let (cube, skin) = if show {
        (
            Some(meshes.add(Cuboid::from_length(1.0))),
            Some(materials.add(StandardMaterial {
                base_color: Color::srgba(0.95, 0.45, 0.25, 0.30),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
        )
    } else {
        (None, None)
    };
    for (root, needs) in &waiting {
        let mut tris = Vec::new();
        triangles(root, &children, &drawn, &meshes, &mut tris);
        // The scene loads over several frames; nothing yet just means not yet.
        if tris.is_empty() {
            continue;
        }

        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = Vec3::splat(f32::NEG_INFINITY);
        for t in &tris {
            for p in t {
                lo = lo.min(*p);
                hi = hi.max(*p);
            }
        }

        let dims = ((hi - lo) / CELL).ceil().max(Vec3::ONE);
        let (nx, ny, nz) = (dims.x as usize, dims.y as usize, dims.z as usize);
        let mut filled = vec![false; nx * ny * nz];
        let index = |x: usize, y: usize, z: usize| (y * nz + z) * nx + x;

        // Occupancy by sampling each triangle rather than by exact
        // triangle-box overlap. At seven centimetres a barycentric sweep at
        // half-cell spacing misses nothing that matters and is a tenth of the
        // code an exact test would be.
        for t in &tris {
            let a = t[1] - t[0];
            let b = t[2] - t[0];
            let steps = ((a.length() + b.length()) / (CELL * 0.5)).ceil().max(1.0) as usize;
            for i in 0..=steps {
                for j in 0..=(steps - i) {
                    let u = i as f32 / steps as f32;
                    let v = j as f32 / steps as f32;
                    let p = t[0] + a * u + b * v;
                    let cell = ((p - lo) / CELL).floor();
                    let (x, y, z) = (cell.x as isize, cell.y as isize, cell.z as isize);
                    if x < 0 || y < 0 || z < 0 {
                        continue;
                    }
                    let (x, y, z) = (x as usize, y as usize, z as usize);
                    if x < nx && y < ny && z < nz {
                        filled[index(x, y, z)] = true;
                    }
                }
            }
        }

        // Greedy merge: run along x, widen along z, then raise along y. Boxes
        // rather than cells, because collision walks every solid in the house
        // on every query and six hundred of them for one couch would be felt.
        let mut used = vec![false; filled.len()];
        let mut added = 0usize;
        for y in 0..ny {
            for z in 0..nz {
                for x in 0..nx {
                    if !filled[index(x, y, z)] || used[index(x, y, z)] {
                        continue;
                    }
                    let mut w = 1;
                    while x + w < nx && filled[index(x + w, y, z)] && !used[index(x + w, y, z)] {
                        w += 1;
                    }
                    let mut d = 1;
                    'deeper: while z + d < nz {
                        for i in 0..w {
                            if !filled[index(x + i, y, z + d)] || used[index(x + i, y, z + d)] {
                                break 'deeper;
                            }
                        }
                        d += 1;
                    }
                    let mut h = 1;
                    'taller: while y + h < ny {
                        for k in 0..d {
                            for i in 0..w {
                                if !filled[index(x + i, y + h, z + k)]
                                    || used[index(x + i, y + h, z + k)]
                                {
                                    break 'taller;
                                }
                            }
                        }
                        h += 1;
                    }
                    for k in 0..h {
                        for j in 0..d {
                            for i in 0..w {
                                used[index(x + i, y + k, z + j)] = true;
                            }
                        }
                    }

                    let min = lo + Vec3::new(x as f32, y as f32, z as f32) * CELL;
                    let max = min + Vec3::new(w as f32, h as f32, d as f32) * CELL;
                    let mut solid = Solid::between(min, max, Stuff::Fabric);
                    solid.unseen = true;
                    solid.piece = needs.piece;
                    if let (Some(cube), Some(skin)) = (&cube, &skin) {
                        commands.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(skin.clone()),
                            Transform::from_translation(solid.center).with_scale(solid.half * 2.0),
                            bevy::light::NotShadowCaster,
                        ));
                    }
                    home.solids.push(solid);
                    added += 1;
                }
            }
        }

        info!(
            "made model: {} triangles -> {} collision boxes at {CELL:.0} cm",
            tris.len(),
            added
        );
        commands.entity(root).remove::<NeedsHull>();
    }
}
