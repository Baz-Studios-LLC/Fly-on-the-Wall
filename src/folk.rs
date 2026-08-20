//! The people who live here.
//!
//! One family, and nearly the whole cast of the game: the fly, four of these,
//! and the odd visitor. That ratio is the argument for how much care goes in
//! here — a prop that is slightly wrong costs a glance, and a person who is
//! slightly wrong is most of what anybody looks at.
//!
//! The father was built in this file twice. First as boxes, in the voxel
//! language the game was described in but with elbows and knees, so an arm was
//! upper arm, forearm and hand hung off each other rather than one swinging
//! plank. Then as lathed profiles — surfaces of revolution with leaning rings,
//! which is a genuinely good way to build a body and produced a correctly
//! proportioned man with a bowl cut and no life in him.
//!
//! Both are gone. He is a made model now, rigged, and he arrives with a
//! forty-one bone humanoid skeleton — `L_Upperarm`, `R_Calf`, `Head`, the usual
//! names. That skeleton is worth more than the geometry it carries: it is what
//! a walk cycle, a turn of the head and a hand on a door will hang off.
//!
//! What stayed from the hand-built versions is everything around the body: the
//! marker the camera and the turntable look for, and collision taken from his
//! own triangles rather than a box round his shoulders.

use bevy::prelude::*;

use crate::world::{Home, Solid, Stuff, UNITS_PER_METRE};

/// How tall he stands, in centimetres.
const TALL: f32 = 178.0;

/// A person. The studio and the camera both look for this.
#[derive(Component)]
pub struct Person;

/// How tall this person is, in centimetres, measured from their own origin.
///
/// The camera needs it and has no other way to know. It used to assume the
/// person's transform sat at hip height, because the hand-built body was
/// authored from the pelvis outward; a made model stands on the floor instead,
/// and the first capture after the swap was framed on a patch of air six
/// centimetres below his shoe.
#[derive(Component)]
pub struct Stature(pub f32);

/// A person whose collision has not been worked out yet.
#[derive(Component)]
struct NeedsBody {
    solid: usize,
}

/// A pose, as rotations on named bones.
///
/// A rig arrives in its bind pose, which for this one is a T — arms straight
/// out, palms down. Nobody stands like that. The bones are named the way every
/// humanoid rig names them, so a pose is a short table rather than anything
/// clever, and it is authored here in radians for the same reason the house is:
/// it can be tuned by changing a number instead of re-exporting a file.
///
/// Angles are Euler XYZ in the bone's own space.
const AT_EASE: &[(&str, [f32; 3])] = &[
    // Arms down. Most of the angle is a single swing at the shoulder; the rest
    // is what stops him standing to attention — a little forward, a little out
    // from the ribs, and a real bend at the elbow.
    ("L_Clavicle", [0.0, 0.0, -0.05]),
    ("R_Clavicle", [0.0, 0.0, 0.05]),
    ("L_Upperarm", [0.10, 0.0, -1.32]),
    ("R_Upperarm", [0.10, 0.0, 1.32]),
    ("L_Forearm", [0.0, 0.28, -0.18]),
    ("R_Forearm", [0.0, -0.28, 0.18]),
    ("L_Hand", [0.0, 0.0, 0.10]),
    ("R_Hand", [0.0, 0.0, -0.10]),
    // Weight on one leg. A body with both legs identical reads as a mannequin
    // however well it is built.
    ("R_Thigh", [0.06, 0.0, -0.04]),
    ("R_Calf", [-0.12, 0.0, 0.0]),
    ("L_Thigh", [-0.03, 0.0, 0.02]),
    // And not quite square to the room.
    ("Spine02", [0.0, 0.05, 0.0]),
    ("NeckTwist01", [0.02, -0.10, 0.0]),
];

/// A slow movement laid over the resting pose: bone, axis scaled to the
/// amplitude in radians, cycles per second, and phase in turns.
///
/// Nobody stands still. A body that does is the single loudest thing in a room
/// — more than a blank face, more than a bad texture — because stillness is the
/// one property no living thing has. None of these amplitudes is meant to be
/// noticed on its own; a hundredth of a radian at the spine is a millimetre at
/// the shoulder. What is noticed is their absence.
///
/// Rates are deliberately coprime-ish so the whole thing does not visibly loop.
/// Breathing runs at about fourteen a minute, the weight shift at four, and the
/// head drifts slower still.
const IDLE: &[(&str, [f32; 3], f32, f32)] = &[
    // Breathing: the chest rises and the neck takes it back out, so his head
    // does not nod along with his lungs.
    ("Spine01", [0.010, 0.0, 0.0], 0.235, 0.0),
    ("Spine02", [0.008, 0.0, 0.0], 0.235, 0.02),
    ("NeckTwist02", [-0.011, 0.0, 0.0], 0.235, 0.05),
    // Weight shifting from one foot to the other.
    ("Hip", [0.0, 0.0, 0.019], 0.061, 0.0),
    ("Waist", [0.0, 0.0, -0.011], 0.061, 0.04),
    // Looking about the room, slowly.
    ("Head", [0.015, 0.105, 0.0], 0.037, 0.31),
    // Arms hanging, not pinned. Deliberately smaller than they want to be:
    // his collision is built once, from the pose he settles into, and a
    // hundredth of a radian at the shoulder is already most of a centimetre at
    // the hand. That is two body lengths to the thing landing on it. Until the
    // collision follows the bones, the amplitude here is bounded by how far the
    // hand may drift from the surface the fly can feel.
    ("L_Upperarm", [0.008, 0.0, 0.013], 0.083, 0.0),
    ("R_Upperarm", [0.008, 0.0, -0.013], 0.083, 0.5),
    ("L_Forearm", [0.0, 0.011, 0.0], 0.083, 0.12),
    ("R_Forearm", [0.0, -0.011, 0.0], 0.083, 0.62),
];

/// A bone that moves, and the pose it moves around.
#[derive(Component)]
struct Idling {
    rest: Quat,
    turn: Vec3,
    rate: f32,
    phase: f32,
}

/// A rig that has not been posed yet.
#[derive(Component)]
struct NeedsPose;

pub struct FolkPlugin;

impl Plugin for FolkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, raise_the_father)
            .add_systems(Update, (pose_him, breathe).chain())
            // After the transforms have propagated, not merely after the pose
            // has been *set*. Posing writes local rotations; the world
            // positions those imply are worked out later in the frame, and a
            // collision hull built before that lands on the bind pose no matter
            // what the screen shows. It cost a man a hundred and seventeen
            // centimetres across the shoulders to find out.
            .add_systems(
                PostUpdate,
                make_him_solid.after(TransformSystems::Propagate),
            );
    }
}

/// Put him at ease.
///
/// Runs until it finds the bones, because a glTF scene arrives over several
/// frames and the skeleton is not there on the first one.
fn pose_him(
    mut commands: Commands,
    waiting: Query<Entity, With<NeedsPose>>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut bones: Query<&mut Transform>,
) {
    for person in &waiting {
        let mut found = 0;
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            let Ok(name) = names.get(entity) else {
                continue;
            };
            let Some((_, angles)) = AT_EASE.iter().find(|(bone, _)| *bone == name.as_str()) else {
                continue;
            };
            if let Ok(mut bone) = bones.get_mut(entity) {
                // Composed onto the bind rotation, not replacing it. A bind
                // pose already carries the bone's own orientation, and throwing
                // that away snaps every limb onto the armature's axes.
                bone.rotation *= Quat::from_euler(EulerRot::XYZ, angles[0], angles[1], angles[2]);
                found += 1;
            }
        }
        if found == 0 {
            continue;
        }

        // The idle hangs off whatever pose the bone ended on, so the resting
        // rotation has to be read *after* the pose, not before it.
        let mut moving = 0;
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            let Ok(name) = names.get(entity) else {
                continue;
            };
            let Some((_, turn, rate, phase)) =
                IDLE.iter().find(|(bone, ..)| *bone == name.as_str())
            else {
                continue;
            };
            let Ok(bone) = bones.get(entity) else {
                continue;
            };
            commands.entity(entity).insert(Idling {
                rest: bone.rotation,
                turn: Vec3::from_array(*turn),
                rate: *rate,
                phase: *phase,
            });
            moving += 1;
        }

        info!(
            "a person is posed: {found} of {} bones set, {moving} of {} breathing",
            AT_EASE.len(),
            IDLE.len()
        );
        commands.entity(person).remove::<NeedsPose>();
    }
}

/// The father, in the great room.
///
/// He was built here, out of boxes and then out of lathed profiles, and both
/// are gone. The lathes were the better version and they were still losing:
/// a face is not a stack of ellipses, and every pass fixed one feature and
/// exposed the next. What was left after a long afternoon of it was a man with
/// correct proportions and no life in him.
///
/// He is a made model now, rigged, and he arrives with a skeleton whose joints
/// are named the way every humanoid rig names them — `L_Upperarm`, `R_Calf`,
/// `Head`. That skeleton is worth more than the geometry: it is what a walk
/// cycle and a turn of the head will hang off later.
///
/// The pipeline underneath is the one the couch already uses. Two facts — a
/// path and a place — and the collision comes out of the model's own triangles.
fn raise_the_father(mut commands: Commands, mut home: ResMut<Home>, assets: Res<AssetServer>) {
    // Standing in the great room, off to one side, facing across it. Clear of
    // everything, which has to include the made furniture: the armchair came in
    // at a hundred and sixteen centimetres where the generated one was
    // ninety-two, and he was standing in it.
    let room = crate::house::room("great room");
    let stand = Vec3::new(
        room.min.x + room.wide() * 0.62,
        0.0,
        room.min.y + room.deep() * 0.22,
    );
    let turn = Quat::from_rotation_y(-0.7);

    // The model is normalised to one unit tall, so the scale is his height in
    // metres. Everything downstream multiplies by a hundred.
    let mut solid = Solid::between(
        stand - Vec3::splat(2.0),
        stand + Vec3::splat(2.0),
        Stuff::Fabric,
    );
    solid.model = Some("characters/DadRigged.glb");
    solid.rot = turn;
    solid.unseen = true;
    solid.scale = TALL / UNITS_PER_METRE;
    let index = home.solids.len();
    home.solids.push(solid);

    commands.spawn((
        Person,
        Stature(TALL),
        NeedsPose,
        NeedsBody { solid: index },
        crate::world::Part { solid: index },
        Name::new("Father"),
        WorldAssetRoot(
            assets.load(GltfAssetLabel::Scene(0).from_asset("characters/DadRigged.glb")),
        ),
        Transform::from_translation(stand)
            .with_rotation(turn)
            .with_scale(Vec3::splat(UNITS_PER_METRE * TALL / UNITS_PER_METRE)),
    ));
}

/// Move the moving bones.
///
/// Sine waves on top of a stored rest rotation, the way the fly's wings and
/// legs are driven. No animation clips: the file ships none, and a table of
/// numbers in this file can be tuned by editing a number, which is the same
/// argument the rest of this game makes for building things in code.
fn breathe(clock: Res<Time>, mut bones: Query<(&Idling, &mut Transform)>) {
    let now = clock.elapsed_secs();
    for (idle, mut bone) in &mut bones {
        let a = std::f32::consts::TAU * (idle.rate * now + idle.phase);
        bone.rotation = idle.rest * Quat::from_scaled_axis(idle.turn * a.sin());
    }
}

/// Hand his own triangles to the collision, in the pose he is standing in.
///
/// He was scenery: you flew straight through him. The made furniture solved
/// this already — the mesh you can see is the surface you land on, because a
/// bounding box round a shoulder stands two centimetres proud of it, which is
/// four body lengths to the thing landing there.
///
/// A rigged body needs one thing more. Skinning happens on the graphics card,
/// so the mesh held in memory never leaves its bind pose: reading it back the
/// way the couch is read gives a man standing in a T with his arms out through
/// the walls, no matter what pose is on screen. So the vertices are skinned
/// here, on the processor, once — each one moved by its bones' matrices exactly
/// as the shader would move it. What the fly lands on is then what anybody can
/// see, which is the whole rule this file is built on.
///
/// Once, and only once: this is a standing man, not a walking one. A pose that
/// changes every frame would need a cheaper answer than four thousand triangles
/// refiled into a grid, and that is a problem for whoever gives him a walk.
fn make_him_solid(
    mut commands: Commands,
    mut home: ResMut<Home>,
    waiting: Query<(Entity, &NeedsBody), Without<NeedsPose>>,
    children: Query<&Children>,
    skinned: Query<(&Mesh3d, &bevy::mesh::skinning::SkinnedMesh)>,
    placed: Query<&GlobalTransform>,
    mut meshes: ResMut<Assets<Mesh>>,
    bindposes: Res<Assets<bevy::mesh::skinning::SkinnedMeshInverseBindposes>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    use bevy::render::mesh::VertexAttributeValues as Values;

    for (person, needs) in &waiting {
        let mut tris = Vec::new();
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            let Ok((handle, skin)) = skinned.get(entity) else {
                continue;
            };
            let (Some(mesh), Some(binds)) = (
                meshes.get(&handle.0),
                bindposes.get(&skin.inverse_bindposes),
            ) else {
                continue;
            };
            let (
                Some(Values::Float32x3(points)),
                Some(Values::Uint16x4(bones)),
                Some(Values::Float32x4(weights)),
            ) = (
                mesh.attribute(Mesh::ATTRIBUTE_POSITION),
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX),
                mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT),
            )
            else {
                continue;
            };

            // One matrix per bone: where that bone is now, undoing where it was
            // when the mesh was bound to it.
            let joints: Vec<Mat4> = skin
                .joints
                .iter()
                .enumerate()
                .map(|(i, &bone)| {
                    let now = placed
                        .get(bone)
                        .map(|at| at.affine().into())
                        .unwrap_or(Mat4::IDENTITY);
                    now * binds[i]
                })
                .collect();

            let at = |i: usize| -> Vec3 {
                let p = points[i];
                let local = Vec3::new(p[0], p[1], p[2]).extend(1.0);
                let mut out = Vec4::ZERO;
                for k in 0..4 {
                    let w = weights[i][k];
                    if w == 0.0 {
                        continue;
                    }
                    if let Some(joint) = joints.get(bones[i][k] as usize) {
                        out += *joint * local * w;
                    }
                }
                // A vertex with no weights at all belongs to nothing and would
                // collapse to the origin, dragging a triangle across the room
                // with it.
                if out.w.abs() < 1e-6 {
                    Vec3::new(p[0], p[1], p[2])
                } else {
                    out.truncate() / out.w
                }
            };

            match mesh.indices() {
                Some(indices) => {
                    let list: Vec<usize> = indices.iter().collect();
                    for tri in list.chunks_exact(3) {
                        tris.push([at(tri[0]), at(tri[1]), at(tri[2])]);
                    }
                }
                None => {
                    for tri in (0..points.len()).collect::<Vec<_>>().chunks_exact(3) {
                        tris.push([at(tri[0]), at(tri[1]), at(tri[2])]);
                    }
                }
            }
        }

        // Nothing yet just means the scene has not finished arriving.
        if tris.is_empty() {
            continue;
        }
        let hull = crate::world::Hull::new(needs.solid, tris, crate::made::CELL);
        let (low, high) = hull.bounds();
        info!(
            // The span across matters as much as the height: it is the one
            // number that tells you at a glance whether the collision is in the
            // pose you can see or still in the T it was bound in.
            "a person is solid: {} triangles, {:.0} cm tall and {:.0} cm across",
            hull.count(),
            high.y - low.y,
            (high.xz() - low.xz()).max_element()
        );
        // `FLY_HULL=1` draws it, because a person's collision is the one kind
        // with no geometry of its own to check against.
        if std::env::var("FLY_HULL").is_ok() {
            crate::made::probe(&mut commands, &hull, &mut meshes, &mut materials);
        }
        home.hulls.push(hull);
        commands.entity(person).remove::<NeedsBody>();
    }
}
