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

/// The model he is made of.
const FATHER: &str = "characters/DadRigged.glb";

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

/// A way of holding a body: the bones that differ from the bind pose, and how
/// far the whole body drops from where its feet would otherwise be.
///
/// One posture was enough while there was one person standing in one place.
/// There are four people coming and the interesting thing any of them does is
/// sit down, so a posture is a value now rather than a constant.
pub struct Posture {
    bones: &'static [(&'static str, [f32; 3])],
    /// What this posture is called in an animation file, if a clip for it
    /// might exist. Checked before the hand-built bone table is used.
    clip: Option<&'static str>,
    /// Roughly where the model's own origin sits relative to the floor. Only a
    /// starting guess: a body settles onto the floor by measuring itself, so
    /// this only has to be close enough that the first frame is not absurd.
    lift: f32,
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
const STANDING_BONES: &[(&str, [f32; 3])] = &[
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

pub const STANDING: Posture = Posture {
    bones: STANDING_BONES,
    clip: Some("idle"),
    lift: 0.0,
};

/// Sitting back on a sofa, watching the television.
///
/// Hips and knees near a right angle, but not at one: a man on a sofa is not a
/// man on a dining chair. He is reclined a few degrees, his knees are a little
/// higher than his hips because a sofa cushion is lower than a chair, and his
/// feet are flat and apart.
const SEATED_BONES: &[(&str, [f32; 3])] = &[
    // Flexion is about x. It was written about z first, which is abduction —
    // he sat with his legs straight out sideways in the splits.
    ("L_Thigh", [1.80, 0.06, 0.16]),
    ("R_Thigh", [1.80, -0.06, -0.16]),
    ("L_Calf", [-1.80, 0.0, 0.0]),
    ("R_Calf", [-1.80, 0.0, 0.0]),
    ("L_Foot", [-0.25, 0.0, 0.0]),
    ("R_Foot", [-0.25, 0.0, 0.0]),
    // Reclined into the back of it.
    ("Hip", [0.0, 0.0, 0.0]),
    ("Waist", [-0.10, 0.0, 0.0]),
    ("Spine01", [-0.06, 0.0, 0.0]),
    ("NeckTwist01", [0.12, 0.0, 0.0]),
    // Arms along the cushions, elbows bent, hands in his lap.
    ("L_Clavicle", [0.0, 0.0, -0.05]),
    ("R_Clavicle", [0.0, 0.0, 0.05]),
    ("L_Upperarm", [0.22, 0.0, -1.22]),
    ("R_Upperarm", [0.22, 0.0, 1.22]),
    ("L_Forearm", [0.0, 0.55, -0.45]),
    ("R_Forearm", [0.0, -0.55, 0.45]),
];

pub const SEATED: Posture = Posture {
    bones: SEATED_BONES,
    clip: Some("sit"),
    lift: -47.0,
};

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

/// The glTF files a person's movements live in, kept so their clips can be
/// found once they load. A scene is a separate asset and does not carry them.
///
/// More than one, because the rigging tool writes a single animation per
/// export. There is nothing to merge: Bevy matches a clip to a skeleton by the
/// *name path* of each bone, so a walk exported from this rig plays on the
/// body already standing in the room, and the duplicate mesh in the walk file
/// is simply never used. Combining the files would save disk and change
/// nothing else.
#[derive(Component)]
struct CameFrom(Vec<(String, Handle<Gltf>)>);

/// Which clip is which, once they are all in one graph, so a person can be
/// told to do something else later without reloading anything.
///
/// Kept rather than recomputed because the graph is built once, out of a
/// folder scan and every file in it: recovering this map later would mean
/// doing all of that again.
#[allow(
    dead_code,
    reason = "the handle a state machine will change poses with"
)]
#[derive(Component)]
pub struct Repertoire(pub std::collections::HashMap<String, AnimationNodeIndex>);

/// What to call the movement in a file, given the base model's name.
///
/// The part that varies between one export and the next is the part worth
/// reading: `DadRigged` and `DadWalk` share `Dad`, so the second is `walk`.
/// A file that shares nothing, or everything, keeps its own name — the base
/// model itself lands there and contributes no clips anyway.
fn movement_name(model: &str, file: &str) -> String {
    let shared = file
        .chars()
        .zip(model.chars())
        .take_while(|(a, b)| a == b)
        .count();
    let movement = file[shared..].trim_matches(['_', '-', ' ']).to_lowercase();
    if movement.is_empty() {
        file.to_lowercase()
    } else {
        movement
    }
}

/// Every glTF in the characters folder, as (movement, path).
///
/// Scanned rather than listed, so a new movement is a file drop and not a code
/// change. The name comes from the file: `DadWalk.glb` is `walk`, because the
/// part that varies between one export and the next is the part worth reading.
fn movements(model: &str) -> Vec<(String, String)> {
    let folder = std::path::Path::new(model)
        .parent()
        .map(|p| p.to_owned())
        .unwrap_or_default();
    let stem = std::path::Path::new(model)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    // The longest run of leading letters shared by the base model's name is
    // taken as the family prefix: `DadRigged` and `DadWalk` share `Dad`.
    let mut found: Vec<(String, String)> = Vec::new();
    for root in ["assets", "../assets", "../../assets"] {
        let here = std::path::Path::new(root).join(&folder);
        let Ok(entries) = std::fs::read_dir(&here) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("glb") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            found.push((
                movement_name(stem, name),
                folder
                    .join(path.file_name().unwrap())
                    .to_string_lossy()
                    .into_owned(),
            ));
        }
        if !found.is_empty() {
            break;
        }
    }
    found.sort();
    found
}

/// A person playing a clip of their own rather than a pose built here.
#[derive(Component)]
struct Animated;

/// Somebody who wants to sit on the named model and has not found it yet.
#[derive(Component)]
struct NeedsSeat {
    /// The model to sit on.
    on: &'static str,
    /// What to do instead if there is nothing there to sit on.
    otherwise: &'static Posture,
}

/// A rig that has not been posed yet, and the posture it is waiting for.
#[derive(Component)]
struct NeedsPose(&'static Posture);

pub struct FolkPlugin;

impl Plugin for FolkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, raise_the_father)
            .add_systems(
                Update,
                (play_what_he_has, take_a_seat, pose_him, breathe).chain(),
            )
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

/// Play a clip the model brought with it, if it brought one.
///
/// The rigged father arrived with a skeleton and no animation, so everything he
/// does is authored here as tables of bone angles. That is the right answer for
/// a body that has nothing else, and the wrong one the moment a file turns up
/// with a real walk in it — hand-written sine waves should not be fighting an
/// animator for the same bones.
///
/// So: if the file has clips, one of them plays and the hand-built pose and
/// idle stand down. The clip is chosen by name where the name says what it is —
/// a posture asks for `sit`, and anything called `idle` will do otherwise —
/// falling back to the first clip in the file. glTF holds any number of named
/// clips in one file, so this does not need one file per movement.
fn play_what_he_has(
    mut commands: Commands,
    files: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    folk: Query<(Entity, &CameFrom, &NeedsPose), Without<Animated>>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    idling: Query<Entity, With<Idling>>,
) {
    for (person, came_from, wanted) in &folk {
        // Every file has to be in before the graph is built, or a person ends
        // up able to stand and not to walk depending on load order.
        if came_from.0.iter().any(|(_, file)| !files.contains(file)) {
            continue;
        }
        let mut clips: Vec<(String, Handle<AnimationClip>)> = Vec::new();
        for (movement, handle) in &came_from.0 {
            let Some(file) = files.get(handle) else {
                continue;
            };
            // A file that names its clips is believed; one that does not is
            // named after itself, which is the whole point of one per export.
            for (name, clip) in &file.named_animations {
                clips.push((name.to_lowercase(), clip.clone()));
            }
            if file.named_animations.is_empty() {
                for (i, clip) in file.animations.iter().enumerate() {
                    let name = if i == 0 {
                        movement.clone()
                    } else {
                        format!("{movement}{i}")
                    };
                    clips.push((name, clip.clone()));
                }
            }
        }
        if clips.is_empty() {
            continue;
        }

        // The loader puts a player on whichever entity roots the animated
        // hierarchy, so finding it is also how we know the skeleton is here.
        let mut root = None;
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            if players.contains(entity) {
                root = Some(entity);
                break;
            }
        }
        let Some(root) = root else {
            continue;
        };

        // One graph with everything in it, so changing what somebody is doing
        // is choosing a node rather than loading a file.
        let mut graph = AnimationGraph::new();
        let blend = graph.add_blend(1.0, graph.root);
        let mut repertoire = std::collections::HashMap::new();
        for (name, clip) in &clips {
            repertoire.insert(name.clone(), graph.add_clip(clip.clone(), 1.0, blend));
        }

        // `FLY_MOVE=<name>` plays a named clip instead of the one the posture
        // asks for. There is no other way to look at a movement: a person does
        // what the room needs, and checking that a walk cycle came out of the
        // exporter intact should not require making him walk somewhere first.
        let asked = std::env::var("FLY_MOVE").ok();
        let wants = asked.as_deref().or(wanted.0.clip).unwrap_or("idle");
        let chosen = repertoire
            .iter()
            .find(|(name, _)| name.contains(wants))
            .or_else(|| repertoire.iter().find(|(name, _)| name.contains("idle")))
            .map(|(name, node)| (name.clone(), *node));
        let Some((playing, node)) = chosen.or_else(|| {
            clips
                .first()
                .and_then(|(name, _)| repertoire.get(name).map(|n| (name.clone(), *n)))
        }) else {
            continue;
        };

        if let Ok(mut player) = players.get_mut(root) {
            player.play(node).repeat();
        }
        commands
            .entity(root)
            .insert(AnimationGraphHandle(graphs.add(graph)));

        // The hand-built idle would keep writing the same bones every frame and
        // win, because it runs after the animation.
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            if idling.contains(entity) {
                commands.entity(entity).remove::<Idling>();
            }
        }

        info!(
            "a person is animated: playing {playing:?} of {:?}",
            clips.iter().map(|(n, _)| n).collect::<Vec<_>>()
        );
        commands
            .entity(person)
            .insert((Animated, Repertoire(repertoire)))
            .remove::<NeedsPose>();
    }
}

/// Sit somebody on the piece of furniture they were told to sit on.
///
/// The seat is found from the furniture's own collision rather than written
/// down here. Brett resized the sofa in arrange mode and saved it, which is
/// exactly the event that should break nothing — and it left the father sitting
/// in the air beside a smaller sofa, because he had been placed at coordinates
/// measured off one afternoon's log.
///
/// Runs until the furniture's hull exists, which takes a few frames.
fn take_a_seat(
    mut commands: Commands,
    mut home: ResMut<crate::world::Home>,
    mut folk: Query<(Entity, &NeedsSeat, &mut Transform)>,
) {
    for (person, wanted, mut standing) in &mut folk {
        let Some(furniture) = home.solids.iter().position(|s| s.model == Some(wanted.on)) else {
            // No sofa, so he stands where he is rather than sitting on the
            // floor in the shape of a chair. Furniture can be removed from the
            // house and a person should not be the thing that breaks.
            warn!(
                "nobody can sit on {}: no such model in the house",
                wanted.on
            );
            commands
                .entity(person)
                .insert(NeedsPose(wanted.otherwise))
                .remove::<NeedsSeat>();
            continue;
        };
        let Some(seat) = home
            .hulls
            .iter()
            .find(|h| h.belongs_to() == furniture)
            .and_then(crate::made::seat)
        else {
            continue;
        };

        // The model faces its own +x, so this is the turn that points that at
        // whatever the sofa is pointing at.
        let turn = Quat::from_rotation_y(seat.facing.z.atan2(seat.facing.x) * -1.0);
        standing.translation = Vec3::new(seat.at.x, standing.translation.y, seat.at.z);
        standing.rotation = turn;
        let mine = home
            .solids
            .iter()
            .position(|s| s.model == Some(FATHER))
            .unwrap_or(furniture);
        home.solids[mine].center.x = seat.at.x;
        home.solids[mine].center.z = seat.at.z;
        home.solids[mine].rot = turn;
        info!(
            "somebody sits at ({:.0}, {:.0}, {:.0}) facing ({:.2}, {:.2})",
            seat.at.x, seat.at.y, seat.at.z, seat.facing.x, seat.facing.z
        );
        commands.entity(person).remove::<NeedsSeat>();
    }
}

/// Put him at ease.
///
/// Runs until it finds the bones, because a glTF scene arrives over several
/// frames and the skeleton is not there on the first one.
fn pose_him(
    mut commands: Commands,
    waiting: Query<(Entity, &NeedsPose)>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut bones: Query<&mut Transform>,
) {
    for (person, wanted) in &waiting {
        let mut found = 0;
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            let Ok(name) = names.get(entity) else {
                continue;
            };
            let Some((_, angles)) = wanted
                .0
                .bones
                .iter()
                .find(|(bone, _)| *bone == name.as_str())
            else {
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
            wanted.0.bones.len(),
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
pub fn raise_the_father(mut commands: Commands, mut home: ResMut<Home>, assets: Res<AssetServer>) {
    // On the sofa, watching the television.
    //
    // He stood in the middle of the great room facing nothing for as long as he
    // was hand-built, and it was the oddest thing in the house — a man does not
    // stand in the centre of his own living room. The sofa faces east at the
    // television, so he does too.
    //
    // The seat is measured, not guessed: `made` reports the top surface of
    // every model it collides, and the sofa's is forty-three centimetres. He
    // sits a shade back from the middle of it, toward one end.
    let posture = &SEATED;
    let sit = Vec3::new(968.0, posture.lift, 1000.0);
    // The model faces its own +x, so east is no turn at all.
    let turn = Quat::IDENTITY;
    let stand = sit;

    // The model is normalised to one unit tall, so the scale is his height in
    // metres. Everything downstream multiplies by a hundred.
    let mut solid = Solid::between(
        stand - Vec3::splat(2.0),
        stand + Vec3::splat(2.0),
        Stuff::Fabric,
    );
    solid.model = Some(FATHER);
    solid.rot = turn;
    solid.unseen = true;
    solid.scale = TALL / UNITS_PER_METRE;
    let index = home.solids.len();
    // He is his own piece, so arrange mode can place him. He was deliberately
    // left out of it while he was built here out of eighty boxes — dragging him
    // would have moved the collision and left the man standing there. A model
    // moves as one thing: solid, hull and scene all follow.
    solid.piece = index as u32;
    home.solids.push(solid);

    commands.spawn((
        Person,
        Stature(TALL),
        CameFrom(
            movements(FATHER)
                .into_iter()
                .inspect(|(movement, path)| info!("a movement is available: {movement} — {path}"))
                .map(|(movement, path)| (movement, assets.load(path)))
                .collect(),
        ),
        NeedsPose(posture),
        NeedsSeat {
            on: "models/couch.glb",
            otherwise: &STANDING,
        },
        NeedsBody { solid: index },
        crate::world::Part { solid: index },
        Name::new("Father"),
        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(FATHER))),
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
    mut stance: Query<&mut Transform>,
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

        // Stand him on the floor by measuring where he actually reaches.
        //
        // Every pose changes it. A model's origin is between its feet only
        // while it is standing in the pose it was exported in; fold the legs to
        // sit somebody down and the feet end up half a metre in front of the
        // origin and well above it. Authoring that offset per pose is a number
        // nobody can get right by looking at it — the seated father had his
        // shoes fifteen centimetres into the floorboards, with one visible
        // under the sofa.
        //
        // So the drop is measured off the collision that was just built, and
        // the whole body moves by it. One correction is exact; the loop is only
        // there because the transforms have to propagate again before the hull
        // can be rebuilt to match.
        if low.y.abs() > 0.5 {
            if let Ok(mut standing) = stance.get_mut(person) {
                standing.translation.y -= low.y;
            }
            home.solids[needs.solid].center.y -= low.y;
            continue;
        }
        info!(
            // The span across matters as much as the height: it is the one
            // number that tells you at a glance whether the collision is in the
            // pose you can see or still in the T it was bound in.
            "a person is solid: {} triangles, {:.0} cm tall and {:.0} cm across, \
             feet at {:.0}, crown at {:.0}, x {:.0}..{:.0} z {:.0}..{:.0}",
            hull.count(),
            high.y - low.y,
            (high.xz() - low.xz()).max_element(),
            low.y,
            high.y,
            low.x,
            high.x,
            low.z,
            high.z
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

#[cfg(test)]
mod tests {
    use super::movement_name;

    #[test]
    fn a_movement_is_named_after_what_differs() {
        // The rigging tool writes one animation per export, so the files arrive
        // as a family and the family name is the part they share.
        assert_eq!(movement_name("DadRigged", "DadWalk"), "walk");
        assert_eq!(movement_name("DadRigged", "DadSitting"), "sitting");
        assert_eq!(movement_name("DadRigged", "Dad_Idle"), "idle");
        // The base model shares its whole name with itself.
        assert_eq!(movement_name("DadRigged", "DadRigged"), "dadrigged");
        // And something unrelated dropped in the same folder keeps its name
        // rather than becoming a suffix of somebody else's.
        assert_eq!(movement_name("DadRigged", "MumWave"), "mumwave");
    }
}
