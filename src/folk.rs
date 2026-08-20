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

use bevy::animation::{AnimatedBy, AnimationTargetId};
use bevy::prelude::*;

use crate::world::{Home, Solid, Stuff, UNITS_PER_METRE};

/// The model he is made of.
const FATHER: &str = "characters/dad/dad-idle.glb";

/// And hers.
const DAUGHTER: &str = "characters/daughter/daughter-walk.glb";

/// Somebody who lives here.
struct Resident {
    /// The model that is their body.
    model: &'static str,
    /// Folders to take movement clips from, in order.
    ///
    /// More than one, because a body's own export may have no animation in it —
    /// `daughter-walk.glb` has none at all despite the name. Borrowing works
    /// because every one of these rigs is the same forty-one bones under the
    /// same names, and Bevy matches a clip to a skeleton by the name path of
    /// each bone rather than by which file they arrived in. One set of clips
    /// can move the whole family.
    clips: &'static [&'static str],
    /// Height in centimetres. The models are normalised to one unit tall, so
    /// this is the only thing that decides how big somebody is.
    tall: f32,
    /// Which room they start in, and where in it as a fraction of its width and
    /// depth. They wander from there, but only within the room they are in —
    /// doorways are not waypoints yet.
    room: &'static str,
    at: (f32, f32),
    facing: f32,
}

const HOUSEHOLD: &[Resident] = &[
    Resident {
        model: FATHER,
        clips: &["characters/dad"],
        tall: 178.0,
        room: "great room",
        at: (0.55, 0.62),
        facing: -1.2,
    },
    Resident {
        model: DAUGHTER,
        // Her own folder first, so a clip of her own wins the moment one
        // arrives; her father's after it, which is what she moves on today.
        clips: &["characters/daughter", "characters/dad"],
        tall: 138.0,
        room: "bedroom two",
        at: (0.5, 0.55),
        facing: 0.7,
    },
];

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
    /// What this posture is called in an animation file, if a clip for it
    /// might exist. Checked before the hand-built bone table is used.
    clip: Option<&'static str>,
    /// Roughly where the model's own origin sits relative to the floor. Only a
    /// starting guess: a body settles onto the floor by measuring itself, so
    /// this only has to be close enough that the first frame is not absurd.
    lift: f32,
}

pub const STANDING: Posture = Posture {
    clip: Some("idle"),
    lift: 0.0,
};

pub const SEATED: Posture = Posture {
    clip: Some("sit"),
    lift: -47.0,
};

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

/// What a movement is called: the file's name with the family's own name taken
/// off the front, so `dad-idle` in `dad/` is `idle`.
///
/// Only a fallback. A file that names its own clips is believed instead — these
/// exports call theirs `preset:biped:walk` — and this is what a file with an
/// unnamed clip gets called.
fn movement_name(family: &str, file: &str) -> String {
    let plain = file.to_lowercase();
    let movement = plain
        .strip_prefix(&family.to_lowercase())
        .unwrap_or(&plain)
        .trim_start_matches(['-', '_', ' '])
        .to_string();
    if movement.is_empty() { plain } else { movement }
}

/// Every glTF in the characters folder, as (movement, path).
///
/// Scanned rather than listed, so a new movement is a file drop and not a code
/// change. The name comes from the file: `DadWalk.glb` is `walk`, because the
/// part that varies between one export and the next is the part worth reading.
fn movements(folders: &[&str]) -> Vec<(String, String)> {
    // Resolved the way Bevy resolves it, not from the working directory.
    //
    // This scanned `assets`, `../assets` and `../../assets` relative to the
    // process's cwd, which is the repository root when the game is started by
    // hand and is *not* the app's folder when a launcher starts it. So the
    // shipped build found no movement files, fell through to a hand-written
    // pose that had been authored for the previous rig's T-pose, and testers
    // got a man standing in his living room with his arms crossed over the
    // wrong shoulders and no idea how to walk. It ran perfectly from the repo.
    //
    // `get_base_path` is what the asset server itself uses: the manifest
    // directory under `cargo run`, and the executable's own folder in a build
    // anybody else can start.
    let root = bevy::asset::io::file::FileAssetReader::get_base_path().join("assets");
    let mut found: Vec<(String, String)> = Vec::new();
    for folder in folders {
        let here = root.join(folder);
        let Ok(entries) = std::fs::read_dir(&here) else {
            warn!("no movements: cannot read {}", here.display());
            continue;
        };
        // The movement is whatever the file is called once the family's own
        // name is out of the way: `dad-idle` in `dad/` is `idle`. Only a
        // fallback — a file that names its own clips is believed instead.
        let family = std::path::Path::new(folder)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("glb") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            found.push((
                movement_name(&family, name),
                std::path::Path::new(folder)
                    .join(path.file_name().unwrap())
                    .to_string_lossy()
                    .into_owned(),
            ));
        }
    }
    found.sort();
    found
}

/// Bones whose roll a clip gets wrong, and which way their own length runs.
///
/// The presets Brett's rigging tool writes are retargeted from another
/// skeleton, and the retarget puts a large constant roll on both forearms:
/// measured against this rig's own bind pose it is 69 degrees on the right and
/// **159** on the left, both about the forearm's own length. Asymmetric by
/// ninety degrees is not something an animator did on purpose.
///
/// What it looks like in the room is palms turned to face forward and a pinch
/// at each elbow, and it is in the walk as well as the idle — the same bad
/// baseline under both. The bind pose the model arrived in is correct, which is
/// how it can be told apart from a badly built mesh: `FLY_MOVE=none` shows arms
/// hanging properly with the palms to the thighs.
///
/// So the roll is taken out and nothing else is. A rotation splits cleanly into
/// a swing and a twist about a chosen axis; dropping the twist leaves every
/// bend the animator wrote and removes only the spin the retarget added.
const UNROLL: &[&str] = &["L_Forearm", "R_Forearm"];

/// Bones the retarget leaves permanently bent, and the turn that puts them
/// back, as Euler XYZ in the bone's own frame.
///
/// The same fault as [`UNROLL`] and it shows up as a stoop: head hanging, spine
/// curved, "he looks like he has scoliosis". Measured against the rig's bind
/// pose, `NeckTwist01` sits 12.7 degrees forward through the whole idle and the
/// three spine bones add three and a half each — some twenty-three degrees of
/// lean, and thirty-four in the walk.
///
/// What makes it correctable is that it barely moves: the neck's deviation
/// ranges over four degrees across fifteen seconds and the spine's over one.
/// It is an offset, not a performance. Subtracting a constant therefore takes
/// out the stoop and leaves every bit of the animation intact, which is why
/// this is a fixed turn rather than a pull back toward the bind pose — that
/// would flatten a sitting clip into standing.
///
/// The walk legitimately leans further than the idle, so it is deliberately
/// under-corrected: what is removed is roughly the idle's offset, which leaves
/// a walker with the forward lean a walker should have.
const STRAIGHTEN: &[(&str, [f32; 3])] = &[
    ("Waist", [0.052, 0.0, 0.0]),
    ("Spine01", [0.052, 0.0, 0.0]),
    ("Spine02", [0.052, 0.0, 0.0]),
    ("NeckTwist01", [0.216, 0.0, 0.0]),
];

/// A bone that should not roll, the pose it rests in, and its own length axis.
#[derive(Component)]
struct Steady {
    /// Bone, its rest pose, and the axis its roll is taken about.
    rolling: Vec<(Entity, Quat, Vec3)>,
    /// Bone, its rest pose, and the constant turn added back to it.
    bent: Vec<(Entity, Quat, Quat)>,
}

/// Split off the spin about `axis` and return what is left.
fn without_twist(turn: Quat, axis: Vec3) -> Quat {
    let along = axis * turn.xyz().dot(axis);
    let twist = Quat::from_xyzw(along.x, along.y, along.z, turn.w);
    // A half turn exactly across the axis has no twist to speak of and
    // normalising it would be dividing by nothing.
    if twist.length_squared() < 1e-8 {
        return turn;
    }
    turn * twist.normalize().inverse()
}

/// The entity holding a person's `AnimationPlayer`, which the glTF loader puts
/// on whichever node roots the animated hierarchy rather than on the person.
#[derive(Component)]
pub struct Plays(pub Entity);

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
    /// The posture that asked for a seat. Somebody who is not sitting down has
    /// no business being moved onto the furniture.
    wanted: &'static Posture,
}

/// The posture somebody was asked to hold, until a clip is playing for it.
///
/// It used to carry a table of bone angles as well, applied when no clip could
/// be found. Those angles were authored for the hand-built body this file began
/// as, and then for a rig whose bind pose was a T; against the arms-down rig
/// that replaced both they fold a man's arms across the wrong shoulders. The
/// launcher build hit exactly that, because it could not find its clips.
///
/// A fallback that is worse than doing nothing is not a fallback. A rigged
/// model arrives in a pose its author chose, and standing still in it is the
/// right thing to do when there is nothing to play.
#[derive(Component)]
struct Wants(&'static Posture);

pub struct FolkPlugin;

impl Plugin for FolkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, raise_the_folk)
            .add_systems(
                Update,
                (wire_up_borrowed_bones, play_what_he_has, take_a_seat).chain(),
            )
            // After the transforms have propagated, not merely after the pose
            // has been *set*. Posing writes local rotations; the world
            // positions those imply are worked out later in the frame, and a
            // collision hull built before that lands on the bind pose no matter
            // what the screen shows. It cost a man a hundred and seventeen
            // centimetres across the shoulders to find out.
            .add_systems(
                PostUpdate,
                (
                    hold_the_roll.after(bevy::app::AnimationSystems),
                    make_him_solid.after(TransformSystems::Propagate),
                ),
            );
    }
}

/// Give a model that brought no animation the wiring to borrow one.
///
/// Bevy builds an `AnimationPlayer`, and the target ids that address each bone,
/// only when a glTF actually contains animations. `daughter-walk.glb` contains
/// none — no `animations` key at all, seven accessors against her father's
/// hundred and thirty-three — so there was nothing on her for a borrowed clip
/// to drive, and she stood still while he walked about.
///
/// A clip addresses a bone by a hash of the *name path* from the animation root
/// down to it. Her rig is her father's rig — the same forty-one bones under the
/// same names in the same nesting — so the same paths hash to the same ids, and
/// building them by hand makes his clips hers. That is the whole reason one set
/// of animations can move a family.
fn wire_up_borrowed_bones(
    mut commands: Commands,
    folk: Query<Entity, (With<Person>, Without<Animated>)>,
    children: Query<&Children>,
    names: Query<&Name>,
    players: Query<(), With<AnimationPlayer>>,
    already: Query<(), With<AnimationTargetId>>,
) {
    for person in &folk {
        // The armature is the animation root in these exports, and the path a
        // clip was keyed on starts with its name.
        let mut root = None;
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            if players.contains(entity) {
                // It brought its own animation and its own wiring with it.
                root = None;
                break;
            }
            if names.get(entity).is_ok_and(|n| n.as_str() == "Armature") {
                root = Some(entity);
            }
        }
        let Some(root) = root else {
            continue;
        };
        if already.contains(root) {
            continue;
        }

        let mut wired = 0;
        let mut walk = vec![(root, Vec::<Name>::new())];
        while let Some((entity, path)) = walk.pop() {
            let Ok(name) = names.get(entity) else {
                continue;
            };
            let mut here = path.clone();
            here.push(name.clone());
            commands
                .entity(entity)
                .insert((AnimationTargetId::from_names(here.iter()), AnimatedBy(root)));
            wired += 1;
            if let Ok(kids) = children.get(entity) {
                for kid in kids.iter() {
                    walk.push((kid, here.clone()));
                }
            }
        }
        commands.entity(root).insert(AnimationPlayer::default());
        info!("a person borrows animation: {wired} bones wired to a player of their own");
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
    folk: Query<(Entity, &CameFrom, &Wants), Without<Animated>>,
    children: Query<&Children>,
    names: Query<&Name>,
    local: Query<&Transform>,
    mut players: Query<&mut AnimationPlayer>,
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
        // `FLY_MOVE=none` plays nothing, which is the only way to see what a
        // model actually arrived as. A clip drives every bone it has a channel
        // for, so anything wrong with the bind pose is invisible the moment one
        // starts — and anything wrong with the clip looks exactly like a
        // badly built mesh until you can see the two apart.
        let asked = std::env::var("FLY_MOVE").ok();
        if asked.as_deref() == Some("none") {
            info!("a person is left in the pose the model arrived in");
            commands.entity(person).insert(Animated).remove::<Wants>();
            continue;
        }
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

        info!(
            "a person is animated: playing {playing:?} of {:?}",
            clips.iter().map(|(n, _)| n).collect::<Vec<_>>()
        );
        // Read the rest pose *now*, before the graph is inserted and a clip
        // has ever written to these bones. One frame later it is gone.
        let mut steady = Vec::new();
        let mut bent = Vec::new();
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            let kids: Vec<Entity> = children
                .get(entity)
                .map(|k| k.iter().collect())
                .unwrap_or_default();
            stack.extend(kids.iter().copied());
            let Ok(name) = names.get(entity) else {
                continue;
            };
            let Ok(rest) = local.get(entity) else {
                continue;
            };
            if let Some((_, back)) = STRAIGHTEN.iter().find(|(bone, _)| *bone == name.as_str()) {
                bent.push((
                    entity,
                    rest.rotation,
                    Quat::from_euler(EulerRot::XYZ, back[0], back[1], back[2]),
                ));
            }
            if !UNROLL.contains(&name.as_str()) {
                continue;
            }
            // Which way the bone runs is where its longest child sits. On this
            // rig a forearm's hand is at (0, 0.14, 0), so the axis is its own
            // local y — and reading it rather than assuming it is what lets a
            // differently built rig through unharmed.
            let along = kids
                .iter()
                .filter_map(|kid| local.get(*kid).ok())
                .map(|t| t.translation)
                .max_by(|a, b| a.length_squared().total_cmp(&b.length_squared()))
                .unwrap_or(Vec3::Y)
                .normalize_or(Vec3::Y);
            steady.push((entity, rest.rotation, along));
        }
        if !steady.is_empty() || !bent.is_empty() {
            info!(
                "a person's retarget is corrected: {} bones unrolled, {} straightened",
                steady.len(),
                bent.len()
            );
            commands.entity(person).insert(Steady {
                rolling: steady,
                bent,
            });
        }

        commands
            .entity(person)
            .insert((Animated, Plays(root), Repertoire(repertoire)))
            .remove::<Wants>();
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
        if !std::ptr::eq(
            wanted.wanted,
            wanted
                .otherwise
                .clip
                .map_or(wanted.wanted, |_| wanted.wanted),
        ) || wanted.wanted.clip != Some("sit")
        {
            commands.entity(person).remove::<NeedsSeat>();
            continue;
        }
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
                .insert(Wants(wanted.otherwise))
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
/// Stand the household up.
///
/// One entry per person in [`HOUSEHOLD`], and nothing here knows which of them
/// is the father. It did, for as long as there was one — and the moment a
/// second body arrived, every hard-coded thing about him was a thing to
/// untangle. Height, room, clip folders and facing are the whole of what makes
/// one resident different from another.
pub fn raise_the_folk(mut commands: Commands, mut home: ResMut<Home>, assets: Res<AssetServer>) {
    for who in HOUSEHOLD {
        let moves = movements(who.clips);
        // Sit somebody down only if there is a clip for sitting. There is idle
        // and there is walking, and a standing idle played by somebody
        // positioned as though seated puts a man standing on his own sofa.
        let posture = if moves.iter().any(|(name, _)| name.contains("sit")) {
            &SEATED
        } else {
            &STANDING
        };

        let room = crate::house::room(who.room);
        let stand = Vec3::new(
            room.min.x + room.wide() * who.at.0,
            posture.lift,
            room.min.y + room.deep() * who.at.1,
        );
        let turn = Quat::from_rotation_y(who.facing);

        // The models are normalised to one unit tall, so the scale is a height
        // in metres. Everything downstream multiplies by a hundred.
        let mut solid = Solid::between(
            stand - Vec3::splat(2.0),
            stand + Vec3::splat(2.0),
            Stuff::Fabric,
        );
        solid.model = Some(who.model);
        solid.rot = turn;
        solid.unseen = true;
        solid.scale = who.tall / UNITS_PER_METRE;
        let index = home.solids.len();
        // Each is their own piece, so arrange mode can place them. They were
        // deliberately left out of it while the father was eighty hand-built
        // boxes — dragging him would have moved the collision and left the man
        // standing there. A model moves as one thing: solid, hull and scene.
        solid.piece = index as u32;
        home.solids.push(solid);

        info!(
            "{} stands in the {} at {:.0} cm, with {} movement files",
            who.model,
            who.room,
            who.tall,
            moves.len()
        );

        commands.spawn((
            Person,
            Stature(who.tall),
            CameFrom(
                moves
                    .into_iter()
                    .map(|(movement, path)| (movement, assets.load(path)))
                    .collect(),
            ),
            Wants(posture),
            NeedsSeat {
                on: "models/couch.glb",
                otherwise: &STANDING,
                wanted: posture,
            },
            NeedsBody { solid: index },
            crate::world::Part { solid: index },
            Name::new(who.model),
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(who.model))),
            Transform::from_translation(stand)
                .with_rotation(turn)
                .with_scale(Vec3::splat(who.tall)),
        ));
    }
}

/// Take the retarget's roll back out of the arms, every frame, after the clip
/// has had its say.
fn hold_the_roll(steady: Query<&Steady>, mut bones: Query<&mut Transform>) {
    for holding in &steady {
        for &(bone, rest, along) in &holding.rolling {
            let Ok(mut turn) = bones.get_mut(bone) else {
                continue;
            };
            let strayed = rest.inverse() * turn.rotation;
            turn.rotation = rest * without_twist(strayed, along);
        }
        for &(bone, rest, back) in &holding.bent {
            let Ok(mut turn) = bones.get_mut(bone) else {
                continue;
            };
            // Composed onto the deviation rather than onto the bone, so what is
            // removed is a constant and everything the clip does around it
            // survives.
            let strayed = rest.inverse() * turn.rotation;
            turn.rotation = rest * (back * strayed);
        }
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
    waiting: Query<(Entity, &NeedsBody), Without<Wants>>,
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
        // Remember where it was filed, so a body that walks can carry its
        // collision rather than rebuild it.
        let (_, turn, at) = placed
            .get(person)
            .map(|g| g.to_scale_rotation_translation())
            .unwrap_or((Vec3::ONE, Quat::IDENTITY, Vec3::ZERO));
        commands.entity(person).insert(crate::wander::Filed {
            at: bevy::math::Affine3A::from_rotation_translation(turn, at),
            hull: home.hulls.len(),
        });
        home.hulls.push(hull);
        commands.entity(person).remove::<NeedsBody>();
    }
}

#[cfg(test)]
mod tests {
    use super::movement_name;

    #[test]
    fn a_movement_is_named_after_what_differs() {
        // The rigging tool writes one animation per export, so a family arrives
        // as a folder of files whose shared part is the family's own name.
        assert_eq!(movement_name("dad", "dad-walking"), "walking");
        assert_eq!(movement_name("dad", "dad_idle"), "idle");
        assert_eq!(movement_name("daughter", "daughter-walk"), "walk");
        // A file that is only the family name keeps it, and contributes no
        // clips anyway.
        assert_eq!(movement_name("dad", "dad"), "dad");
        // And something unrelated dropped in the same folder keeps its own name
        // rather than becoming a suffix of somebody else's.
        assert_eq!(movement_name("dad", "mum-wave"), "mum-wave");
    }
}
