//! The fly's body, built out of boxes.
//!
//! Blocky to match the voxel humans the rest of the game wants, but shaped for
//! one job above all others: to say *which way it is pointing*. At seven
//! millimetres the silhouette is nearly all the information there is, and the
//! first attempt failed at it badly enough to read as flying backwards. Three
//! things were wrong, and all three are worth remembering because none of them
//! were visible in the code:
//!
//! - The abdomen was one plain box, making the body near enough symmetric front
//!   to back. It now tapers over three segments.
//! - The eyes were small. A housefly's eyes really are most of its head, and
//!   they are the one feature that says where its face is, so they are now
//!   bigger than the head they sit on.
//! - The wings swept *inward* — a sign error — and pivoted about their centres,
//!   which threw half of each blade forward past the head. They now hinge at a
//!   shoulder, like the legs, so they hang backward at any sweep angle.
//!
//! **The wings do not flap.** A housefly beats them around two hundred times a
//! second, which outruns the frame rate and aliases into a slow, wrong-looking
//! flutter — the helicopter-rotor artefact, with no shutter angle to hide behind.
//! What a real fly's wings look like is a translucent smear that widens and
//! fades the harder it is working, so that is what these are. Cheaper than
//! animating them and more accurate.
//!
//! **The legs are a gauge.** Splayed when perched, tucked when flying, reaching
//! when the player asks to land. That is the only landing indicator in a game
//! with no HUD, and it is the shape of the thing itself — the whole no-HUD
//! argument in miniature. If a player learns to read it unprompted the pillar
//! holds; if nobody notices, it needs to be bigger.
//!
//! **`FLY_MODEL=glb`** swaps all of this for `assets/fly.glb`. That model is
//! Tripo output and faces its own **−X** with **+Y** up, so [`facing`] yaws it a
//! quarter turn — worked out from the mesh, not guessed: reflecting the point
//! cloud across each plane puts the bilateral symmetry at Z (1.000 of points map
//! onto the cloud, against 0.30 and 0.05), so Z is lateral; and along X one end
//! tapers to a thin tip below the centreline while the other carries the tallest,
//! widest mass. Abdomen and thorax. Its rig is not usable — no wing bones, four
//! coincident limb roots, two joints driving nothing — so under `FLY_MODEL=glb`
//! the wing and leg systems below have nothing to drive and the landing tell
//! falls back to the nose-up flare in [`crate::fly`].

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::fly::{Fly, Intent, Stance};

// ---------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------

/// Measured head-to-abdomen along the model's own X. Re-measure if the export
/// changes; everything else scales off it.
const MODEL_LENGTH: f32 = 1.0;

/// Height of the model's own centre above its origin, in model units.
///
/// The rigged export stands on its origin — Y runs 0 to 0.789 rather than either
/// side of zero — so dropping it in unmodified hangs the fly a body-height above
/// where it should be. Lowering it by half the height both centres the body on
/// the collision sphere and, as it happens, puts the feet within a twentieth of a
/// millimetre of a surface the fly is perched on.
const MODEL_MIDPOINT: f32 = 0.3943;

/// How long the fly actually is, in centimetres — the real figure, six and a
/// half millimetres.
///
/// This is the one measurement in the build that a player can check against
/// their own memory, so it is also the reference everything else is judged
/// against: a "fast" fly is only fast relative to how big it looks.
const FLY_LENGTH: f32 = 0.65;

/// How the rigged model is turned and seated.
/// Its nose is its own +X. Settled by rendering it and the procedural fly from
/// the same camera angle: the built one showed its back, the model showed a
/// profile, so it is a quarter turn out. Measured, not read off a filename.
fn rigged_facing() -> Quat {
    Quat::from_rotation_y(FRAC_PI_2)
}
const RIGGED_SEAT: f32 = 0.41;

/// Turn the model's −X nose into Bevy's −Z forward.
fn facing() -> Quat {
    Quat::from_rotation_y(-FRAC_PI_2)
}

/// Which body the fly wears.
///
/// The rigged model is the default. It is a far better-looking animal than the
/// boxes below — chitin, compound eyes, translucent wings, six jointed legs —
/// and it walks off its own clip.
///
/// It costs the wingbeat. The procedural body is the only one of the three
/// whose wings beat, because the beat is driven by systems that look for parts
/// this file built, and the model brings one clip and it is a walk. A fly in
/// the air therefore holds its wings still, which is the one place the boxes
/// are still ahead. Driving the model's own wing bones is the fix and is not
/// done yet.
///
/// `FLY_MODEL=built` goes back to the procedural body, wingbeat and all.
#[derive(Clone, Copy, PartialEq)]
enum Worn {
    /// Boxes and lathes built in this file. Wings and legs both driven.
    Built,
    /// `assets/fly.glb`. Tripo output whose rig is not usable — no wing bones,
    /// four coincident limb roots, two joints driving nothing.
    Early,
    /// `assets/characters/fly/fly-legs.glb` — the supplied rig with six leg
    /// bones added by `tools/rig-the-legs.py`, because the one it came with
    /// could not move a leg without moving the body.
    Rigged,
}

/// `FLY_MODEL=built` for the one built here, `glb` for the early model,
/// anything else (or nothing) for the rigged one.
fn worn() -> Worn {
    match std::env::var("FLY_MODEL").as_deref() {
        Ok("built") => Worn::Built,
        Ok("glb") => Worn::Early,
        _ => Worn::Rigged,
    }
}

/// Measured nose-to-tail extent of the boxes below, before scaling. Everything
/// in the trunk is authored at whatever size read well and then scaled as a
/// group to [`FLY_LENGTH`], so the proportions can be edited without anyone
/// having to keep a division in their head.
const PROCEDURAL_LENGTH: f32 = 0.795;

const CHITIN: Color = Color::srgb(0.155, 0.160, 0.172);
const EYE_RED: Color = Color::srgb(0.50, 0.12, 0.09);
const WING: Color = Color::srgba(0.82, 0.86, 0.92, 0.30);

/// Wing chord multipliers at rest and at full effort.
///
/// Folded is *full* chord, not a sliver. Narrowing the resting wing to a third
/// of its width — which is what this did first — turns each one into a blade and
/// the fly into a small helicopter. A fly at rest folds its wings flat along the
/// abdomen at their true width; what widens is the blur when they are working.
const WING_FOLDED: f32 = 1.0;
const WING_BLURRED: f32 = 1.85;

/// How fast the pose catches up, in 1/seconds.
const POSE_RATE: f32 = 14.0;

/// The two segments of a leg, in centimetres. Together they can reach a little
/// over three millimetres, which is what a fly of this size has.
const FEMUR: f32 = 0.15;
const TIBIA: f32 = 0.16;

#[derive(Component)]
struct Wing;

/// One leg: where it is hung, where its foot wants to be, and when in the gait
/// it is allowed to move.
///
/// Flies walk an **alternating tripod** — front and hind on one side step with
/// the middle leg of the other, so three feet are always down and the insect
/// never balances. It is why a fly can walk up a wall, and it is what makes six
/// legs read as walking rather than as six legs twitching.
///
/// The leg is posed by its *foot*, not by its joints. A target is worked out in
/// the body's frame and the femur and tibia are solved to reach it, which is
/// the only way to keep a planted foot actually planted: the body moves, the
/// target does not, and the knee bends to take up the difference. Posing the
/// joints directly and hoping — which is what this did first — swings the foot
/// on an arc through whatever it is standing on.
#[derive(Component)]
struct Leg {
    /// Where the leg meets the thorax, in the body's frame.
    anchor: Vec3,
    /// Where the foot rests when standing still, in the body's frame.
    home: Vec3,
    /// Where the foot goes when the legs fold up for flight.
    folded: Vec3,
    femur: f32,
    tibia: f32,
    /// Which tripod: 0.0 or 0.5 of a cycle.
    phase: f32,
    /// Which way the knee breaks — outward, and away from the body.
    side: f32,
    /// The tibia's pivot, hung at the far end of the femur.
    knee: Entity,
}

#[derive(Resource)]
struct WingSkin(Handle<StandardMaterial>);

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        // `PostStartup` so the fly entity from `fly::hatch` already exists.
        app.add_systems(PostStartup, grow_the_body).add_systems(
            Update,
            (
                present_the_fly,
                work_the_wings,
                pose_the_legs,
                walk_the_model,
                find_the_model_wings,
                beat_the_model_wings,
                find_the_model_legs,
                walk_the_model_legs,
            )
                .chain(),
        );
    }
}

fn grow_the_body(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    flies: Query<Entity, With<Fly>>,
) {
    let Ok(fly) = flies.single() else {
        return;
    };

    // The fly entity itself carries the transform; the body hangs off it, so the
    // whole thing follows one write per frame.
    commands
        .entity(fly)
        .insert((Transform::default(), Visibility::default()));

    match worn() {
        Worn::Early => {
            commands.spawn((
                Name::new("Fly model"),
                ChildOf(fly),
                WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset("fly.glb"))),
                {
                    let scale = FLY_LENGTH / MODEL_LENGTH;
                    Transform::from_rotation(facing())
                        .with_scale(Vec3::splat(scale))
                        .with_translation(Vec3::new(0.0, -MODEL_MIDPOINT * scale, 0.0))
                },
            ));
            return;
        }
        Worn::Rigged => {
            // Normalised to a unit across, so the scale is the fly's length.
            // Facing and seat height are unknown until it has been looked at;
            // both are settled below once there is a capture to settle them
            // against, rather than guessed at from the file.
            commands.spawn((
                Name::new("Fly model"),
                ChildOf(fly),
                WorldAssetRoot(
                    assets.load(GltfAssetLabel::Scene(0).from_asset("characters/fly/fly-legs.glb")),
                ),
                Transform::from_rotation(rigged_facing())
                    .with_scale(Vec3::splat(FLY_LENGTH))
                    .with_translation(Vec3::new(0.0, -RIGGED_SEAT * FLY_LENGTH, 0.0)),
            ));
            return;
        }
        Worn::Built => {}
    }

    let chitin = materials.add(StandardMaterial {
        base_color: CHITIN,
        perceptual_roughness: 0.42,
        metallic: 0.25,
        ..default()
    });
    let eye = materials.add(StandardMaterial {
        base_color: EYE_RED,
        perceptual_roughness: 0.22,
        ..default()
    });
    let wing_skin = materials.add(StandardMaterial {
        base_color: WING,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.15,
        ..default()
    });
    commands.insert_resource(WingSkin(wing_skin.clone()));

    let box_mesh =
        |meshes: &mut Assets<Mesh>, x: f32, y: f32, z: f32| meshes.add(Cuboid::new(x, y, z));

    // Everything below hangs off one scaled root, so the fly's real size is a
    // single constant rather than a factor smeared through forty numbers.
    let trunk = commands
        .spawn((
            ChildOf(fly),
            Transform::from_scale(Vec3::splat(FLY_LENGTH / PROCEDURAL_LENGTH)),
            Visibility::default(),
        ))
        .id();

    // -- Trunk. Forward is -Z. ---------------------------------------------
    //
    // Shaped for one job above all others: to say *which way it is pointing*
    // from any angle. The first version failed at that — a plain box abdomen and
    // small eyes gave a silhouette that was near enough symmetric front to back,
    // and it read as facing backwards as often as not. So the abdomen now tapers
    // over three segments to a blunt point, and the eyes are enormous. That
    // proportion is not a stylisation: a housefly's eyes really are most of its
    // head, and they are the one feature that tells you where its face is.
    let parts: [(Vec3, Vec3, &Handle<StandardMaterial>); 8] = [
        // Abdomen, front to back, each segment smaller than the last.
        (
            Vec3::new(0.0, 0.0, 0.11),
            Vec3::new(0.225, 0.215, 0.14),
            &chitin,
        ),
        (
            Vec3::new(0.0, -0.005, 0.235),
            Vec3::new(0.195, 0.185, 0.12),
            &chitin,
        ),
        (
            Vec3::new(0.0, -0.012, 0.345),
            Vec3::new(0.145, 0.135, 0.11),
            &chitin,
        ),
        // Thorax: the widest, tallest part, where the wings and legs hang from.
        (
            Vec3::new(0.0, 0.012, -0.07),
            Vec3::new(0.25, 0.24, 0.27),
            &chitin,
        ),
        // A small head, mostly hidden behind the eyes.
        (
            Vec3::new(0.0, 0.02, -0.275),
            Vec3::new(0.15, 0.145, 0.135),
            &chitin,
        ),
        // The eyes: bigger than the head they sit on, bulging past it on both
        // sides and above. All of the character is here.
        (
            Vec3::new(-0.077, 0.04, -0.285),
            Vec3::new(0.10, 0.175, 0.175),
            &eye,
        ),
        (
            Vec3::new(0.077, 0.04, -0.285),
            Vec3::new(0.10, 0.175, 0.175),
            &eye,
        ),
        // Proboscis. Tiny, but it puts a point on the front end.
        (
            Vec3::new(0.0, -0.055, -0.355),
            Vec3::new(0.055, 0.06, 0.08),
            &chitin,
        ),
    ];
    for (offset, size, material) in parts {
        let mesh = box_mesh(&mut meshes, size.x, size.y, size.z);
        commands.spawn((
            ChildOf(trunk),
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(offset),
        ));
    }

    // -- Wings --------------------------------------------------------------
    //
    // Hinged at the root like the legs, for the same reason and after the same
    // mistake: rotating a centre-pivoted blade swung half of it forward past the
    // head, so the fly appeared to have two propellers growing out of its face.
    // A wing hangs *backward* from a shoulder, and modelling the shoulder is the
    // only way to get that for free at every sweep angle.
    const WING_LENGTH: f32 = 0.55;
    let wing_mesh = box_mesh(&mut meshes, 0.26, 0.006, WING_LENGTH);
    // Back, a little out, a little up.
    let wing_dir = Vec3::new(0.40, 0.07, 0.92);
    for side in [-1.0f32, 1.0] {
        let dir = Vec3::new(wing_dir.x * side, wing_dir.y, wing_dir.z).normalize();
        let shoulder = commands
            .spawn((
                ChildOf(trunk),
                Transform::from_translation(Vec3::new(side * 0.095, 0.135, -0.02))
                    .looking_to(dir, Vec3::Y),
                Visibility::default(),
            ))
            .id();
        commands.spawn((
            ChildOf(shoulder),
            Wing,
            Mesh3d(wing_mesh.clone()),
            MeshMaterial3d(wing_skin.clone()),
            // Hung off −Z, where `looking_to` aims, so the blade runs from the
            // shoulder outward rather than through it.
            Transform::from_translation(Vec3::new(0.0, 0.0, -WING_LENGTH * 0.5))
                .with_scale(Vec3::new(WING_FOLDED, 1.0, 1.0)),
        ));
    }

    // -- Legs ---------------------------------------------------------------
    //
    // Two segments and a knee. A fly's leg is femur, tibia and a tarsus that is
    // mostly hairs, and the knee is the whole silhouette: the femur goes up and
    // out from the thorax and the tibia comes back down to the foot, which is
    // why an insect standing still looks coiled rather than propped. One
    // straight stick per leg — which is what this was — cannot do that, and its
    // foot can only travel on an arc about the anchor.
    let femur_mesh = box_mesh(&mut meshes, 0.030, 0.028, FEMUR);
    let tibia_mesh = box_mesh(&mut meshes, 0.018, 0.018, TIBIA);
    let foot_mesh = box_mesh(&mut meshes, 0.020, 0.010, 0.030);

    // Anchor, resting foot, and folded foot, given for the right side; the left
    // mirrors in x. Fore feet plant ahead of the shoulder and hind feet behind,
    // as a fly's do, and the middle pair reach furthest out — that wide middle
    // stance is most of what makes the tripod look stable.
    let plan: [(Vec3, Vec3, Vec3); 3] = [
        (
            Vec3::new(0.10, -0.045, -0.15),
            Vec3::new(0.20, -0.255, -0.25),
            Vec3::new(0.12, -0.02, -0.19),
        ),
        (
            Vec3::new(0.115, -0.05, -0.02),
            Vec3::new(0.27, -0.255, -0.02),
            Vec3::new(0.14, -0.01, 0.06),
        ),
        (
            Vec3::new(0.10, -0.05, 0.10),
            Vec3::new(0.21, -0.255, 0.25),
            Vec3::new(0.13, -0.02, 0.24),
        ),
    ];

    for side in [-1.0f32, 1.0] {
        let mirror = |v: Vec3| Vec3::new(v.x * side, v.y, v.z);
        for (row, (anchor, home, folded)) in plan.into_iter().enumerate() {
            // Fore and hind on one side step with the middle of the other.
            let phase = if (row % 2 == 0) == (side > 0.0) {
                0.0
            } else {
                0.5
            };

            // The knee is spawned first so the leg can hold onto it: posing
            // needs to write both rotations, and finding a child by walking the
            // hierarchy every frame for six legs is work for nothing.
            let knee = commands
                .spawn((
                    Transform::from_translation(Vec3::new(0.0, 0.0, -FEMUR)),
                    Visibility::default(),
                ))
                .id();
            commands.spawn((
                ChildOf(knee),
                Mesh3d(tibia_mesh.clone()),
                MeshMaterial3d(chitin.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, -TIBIA * 0.5)),
            ));
            commands.spawn((
                ChildOf(knee),
                Mesh3d(foot_mesh.clone()),
                MeshMaterial3d(chitin.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, -TIBIA)),
            ));

            let pivot = commands
                .spawn((
                    ChildOf(trunk),
                    Leg {
                        anchor: mirror(anchor),
                        home: mirror(home),
                        folded: mirror(folded),
                        femur: FEMUR,
                        tibia: TIBIA,
                        phase,
                        side,
                        knee,
                    },
                    Transform::from_translation(mirror(anchor)),
                    Visibility::default(),
                ))
                .id();
            commands.entity(knee).insert(ChildOf(pivot));
            commands.spawn((
                ChildOf(pivot),
                Mesh3d(femur_mesh.clone()),
                MeshMaterial3d(chitin.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.0, -FEMUR * 0.5)),
            ));
        }
    }
}

/// Copy the interpolated body pose onto the entity every frame.

fn present_the_fly(fixed: Res<Time<Fixed>>, mut flies: Query<(&Fly, &mut Transform)>) {
    let alpha = fixed.overstep_fraction();
    for (fly, mut transform) in &mut flies {
        let (position, rotation) = fly.presented(alpha);
        transform.translation = position;
        transform.rotation = rotation;
    }
}

fn work_the_wings(
    time: Res<Time>,
    skin: Option<Res<WingSkin>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    flies: Query<&Fly>,
    mut wings: Query<&mut Transform, With<Wing>>,
) {
    let Ok(fly) = flies.single() else {
        return;
    };
    let effort = fly.effort();
    let want = WING_FOLDED + (WING_BLURRED - WING_FOLDED) * effort.min(1.0);
    let blend = 1.0 - (-POSE_RATE * time.delta_secs()).exp();

    for mut transform in &mut wings {
        transform.scale.x += (want - transform.scale.x) * blend;
    }

    // Faster work, fainter smear.
    if let Some(skin) = skin
        && let Some(mut material) = materials.get_mut(&skin.0)
    {
        let alpha = 0.42 - 0.22 * effort.min(1.0);
        material.base_color = WING.with_alpha(alpha);
    }
}

/// Two bones to a foot.
///
/// Given where the leg is hung and where its foot must be, work out the femur
/// and tibia directions that put it there. `pole` decides which way the knee
/// breaks — outward and up, so the leg reads as an insect's rather than a
/// bird's.
fn reach(anchor: Vec3, foot: Vec3, femur: f32, tibia: f32, pole: Vec3) -> (Vec3, Vec3) {
    let v = foot - anchor;
    let far = v.length();
    if far < 1e-6 {
        return (Vec3::NEG_Y, Vec3::NEG_Y);
    }
    let n = v / far;
    // Never quite straight and never folded flat: at full stretch the knee has
    // no plane to break in and the leg snaps between solutions.
    let d = far.clamp((femur - tibia).abs() + 0.01, femur + tibia - 0.004);
    let foot = anchor + n * d;

    let cos_alpha = ((femur * femur + d * d - tibia * tibia) / (2.0 * femur * d)).clamp(-1.0, 1.0);
    let alpha = cos_alpha.acos();
    let across = pole - n * pole.dot(n);
    let across = if across.length_squared() < 1e-8 {
        n.any_orthonormal_vector()
    } else {
        across.normalize()
    };

    let femur_dir = (n * alpha.cos() + across * alpha.sin()).normalize();
    let knee = anchor + femur_dir * femur;
    (femur_dir, (foot - knee).normalize_or_zero())
}

/// A leg on the rigged model: the pose it stands in, and where it sits in the
/// tripod.
#[derive(Component)]
struct ModelLeg {
    rest: Quat,
    /// 0.0 or 0.5. Insects walk an alternating tripod — front and rear on one
    /// side with the middle of the other — so three feet are always down.
    phase: f32,
    /// Which of the three segments this is. A fly's leg is femur, tibia and
    /// tarsus, and they do different jobs: the femur swings the leg, the tibia
    /// folds it out of the way, and the tarsus keeps the foot pointing at the
    /// floor while the other two move.
    segment: Segment,
}

#[derive(Clone, Copy, PartialEq)]
enum Segment {
    Femur,
    Tibia,
    Tarsus,
}

/// Find the six leg bones added by `tools/rig-the-legs.py`.
///
/// They exist because the supplied rig could not drive a leg. Its front-left
/// was fifty-nine per cent weighted to `bone_7`, which is the *body*, and its
/// front-right ninety-nine per cent to `tripo::0_Left_Limb_6`, which also holds
/// most of the other five: any rotation that moved a leg moved half the animal,
/// which is exactly what `preset:hexapod:walk` looked like.
///
/// The fix is a re-skin rather than a correction. The father's faults were
/// constants sitting under good animation and subtracting a constant fixed
/// them; this one is the skeleton disagreeing with the mesh about what a leg
/// is, and nothing at runtime can talk it round.
fn find_the_model_legs(
    mut commands: Commands,
    flies: Query<Entity, With<Fly>>,
    children: Query<&Children>,
    names: Query<&Name>,
    poses: Query<&Transform>,
    mut looked: Local<bool>,
) {
    if worn() != Worn::Rigged || *looked {
        return;
    }
    let Ok(fly) = flies.single() else {
        return;
    };
    let mut found = 0;
    let mut stack = vec![fly];
    while let Some(entity) = stack.pop() {
        if let Ok(kids) = children.get(entity) {
            stack.extend(kids.iter());
        }
        let Ok(name) = names.get(entity) else {
            continue;
        };
        let Some(leg) = name
            .as_str()
            .strip_prefix("leg_")
            .and_then(|rest| rest.rsplit_once('_'))
        else {
            continue;
        };
        let (which, part) = leg;
        let phase = match which {
            "front_left" | "middle_right" | "rear_left" => 0.0,
            "front_right" | "middle_left" | "rear_right" => 0.5,
            _ => continue,
        };
        let segment = match part {
            "femur" => Segment::Femur,
            "tibia" => Segment::Tibia,
            "tarsus" => Segment::Tarsus,
            _ => continue,
        };
        let Ok(rest) = poses.get(entity) else {
            continue;
        };
        commands.entity(entity).insert(ModelLeg {
            rest: rest.rotation,
            phase,
            segment,
        });
        found += 1;
    }
    if found == 18 {
        *looked = true;
        info!("the rigged fly walks on six legs of its own: femur, tibia, tarsus");
    }
}

/// Walk the model's legs, driven by how far the body has actually gone.
///
/// One rotation per leg, swept fore and aft about the fly's own lateral axis,
/// with a lift through the return so a foot passes over the floor rather than
/// through it. No knee: a single bone per leg is what the re-skin gives, and at
/// the size a fly is drawn the swing is the whole of what reads.
///
/// Phase advances with distance travelled, not with time, which is the same
/// rule the built legs and the father's walk follow — feet keep up with the
/// floor by construction rather than by a constant somebody tuned.
fn walk_the_model_legs(
    time: Res<Time>,
    flies: Query<&Fly>,
    parents: Query<&ChildOf>,
    placed: Query<&GlobalTransform>,
    mut legs: Query<(Entity, &ModelLeg, &mut Transform)>,
    mut was: Local<Option<Vec3>>,
    mut gait: Local<f32>,
    mut stepping: Local<f32>,
    mut ticks: Local<u32>,
) {
    let Ok(fly) = flies.single() else {
        return;
    };
    /// How far a thigh swings fore and aft, radians.
    ///
    /// Larger than it looks in a close-up, because a close-up is not where
    /// anybody sees it. In chase view the fly is a few dozen pixels across and
    /// fifteen degrees of thigh is under a pixel of foot travel.
    const SWING: f32 = 0.44;
    /// How far it lifts on the way through.
    const LIFT: f32 = 0.20;
    /// How far the tibia folds at the top of the return, and how much of that
    /// the tarsus takes back so the foot still meets the floor flat rather
    /// than curling under the leg.
    const FOLD: f32 = 0.52;
    const FLATTEN: f32 = 0.30;
    /// Body travel for one full stride, in centimetres.
    ///
    /// **Chosen so the gait can be drawn, not from anatomy.** A fly's real
    /// stride is about a third of its body length, which on this body is under
    /// two millimetres — and at the six centimetres a second `fly::WALK`
    /// crawls, that is thirty-three gait cycles every second. No screen can
    /// show that. Set there, the legs ran the whole time and aliased into
    /// looking perfectly still, which is exactly the trap the wingbeat has a
    /// paragraph about a hundred lines further down this file. Twice.
    ///
    /// A centimetre a stride puts it at six cycles a second at full walking
    /// speed: brisk, and visible. The feet slide a little for it, and sliding
    /// feet beat invisible ones.
    const STRIDE: f32 = 1.0;
    /// And a ceiling on top, in cycles per frame, so a burst of speed cannot
    /// alias the gait however fast the fly is dragged along.
    const FASTEST: f32 = 0.11;

    let here = fly.pos;
    let moved = was.map(|w| here - w).unwrap_or(Vec3::ZERO);
    *was = Some(here);
    let walking = matches!(fly.stance, Stance::Perched(_));
    let travelled = if walking { moved.length() } else { 0.0 };
    *gait = (*gait + (travelled / STRIDE).min(FASTEST)).fract();

    // `FLY_STEP=1` says what the gait is actually being fed. Twice now the
    // legs have been reported still while a capture showed them stepping, and
    // the difference between "the signal is zero" and "the movement is too
    // small to see" is not something either of us can tell by looking.
    if std::env::var("FLY_STEP").is_ok() {
        *ticks += 1;
        if *ticks % 30 == 0 {
            info!(
                "gait: {} travelled {:.4} cm/frame  phase {:.2}  stepping {:.2}",
                if walking { "walking" } else { "flying" },
                travelled,
                *gait,
                *stepping
            );
        }
    }

    // A capture cannot press a key.
    let forced = std::env::var("FLY_GAIT")
        .ok()
        .and_then(|v| v.parse::<f32>().ok());
    if let Some(p) = forced {
        *gait = p;
    }

    // Legs settle to standing when the fly is not going anywhere, rather than
    // freezing mid-stride.
    let blend = 1.0 - (-POSE_RATE * time.delta_secs()).exp();
    let want = if forced.is_some() || travelled > 1e-5 {
        1.0
    } else {
        0.0
    };
    *stepping += (want - *stepping) * blend;

    let along = fly.body * Vec3::NEG_Z;
    let lateral = fly.body * Vec3::X;
    for (entity, leg, mut pose) in &mut legs {
        let Some(frame) = parents
            .get(entity)
            .ok()
            .and_then(|c| placed.get(c.parent()).ok())
        else {
            continue;
        };
        let (_, turn, _) = frame.to_scale_rotation_translation();
        let sideways = turn.inverse() * lateral;
        let forward = turn.inverse() * along;

        let t = (*gait + leg.phase).fract();
        // The second half of the cycle is the return: no weight on that foot,
        // so it is the half that lifts and folds.
        let swinging = if t > 0.5 {
            ((t - 0.5) * std::f32::consts::PI).sin()
        } else {
            0.0
        };

        pose.rotation = match leg.segment {
            Segment::Femur => {
                let sweep = (t * std::f32::consts::TAU).cos() * SWING * *stepping;
                let lift = swinging * LIFT * *stepping;
                Quat::from_axis_angle(sideways, sweep)
                    * Quat::from_axis_angle(forward, lift)
                    * leg.rest
            }
            // The tibia folds through the return and straightens to plant. It
            // never goes fully straight: a leg at full stretch has no plane to
            // bend in and snaps between solutions.
            Segment::Tibia => {
                Quat::from_axis_angle(sideways, FOLD * swinging * *stepping) * leg.rest
            }
            // And the tarsus gives some of that back, so the foot arrives flat
            // instead of tucked under the leg it hangs from.
            Segment::Tarsus => {
                Quat::from_axis_angle(sideways, -FLATTEN * swinging * *stepping) * leg.rest
            }
        };
    }
}

/// A wing on the rigged model, and the pose it hangs in at rest.
#[derive(Component)]
struct ModelWing {
    rest: Quat,
    /// −1 or +1: which side of the body it is on, so the pair sweeps apart
    /// rather than both the same way.
    side: f32,
}

/// Find the model's wing bones.
///
/// The rig names almost everything `bone_N`, so they are identified by what
/// they *hold* rather than what they are called: weighing every vertex against
/// every bone puts `bone_12` and `bone_14` alone up at y≈0.70, out at z≈∓0.31,
/// and carrying twenty-odd units of weight each — thin membranes, high on the
/// body, symmetric about the centreline. Nothing else in the skeleton looks
/// remotely like that.
///
/// The legs are a different story and are left alone. `tripo::0_Left_Limb_6`
/// carries eight hundred and eighty units spread across ninety-six per cent of
/// the model's width, so the auto-rig has hung several legs off one bone; there
/// is no per-leg control to drive a tripod gait with, and pretending otherwise
/// would swing half the undercarriage at once.
fn find_the_model_wings(
    mut commands: Commands,
    flies: Query<Entity, With<Fly>>,
    children: Query<&Children>,
    names: Query<&Name>,
    poses: Query<&Transform>,
    already: Query<(), With<ModelWing>>,
    mut looked: Local<bool>,
) {
    if worn() != Worn::Rigged || *looked {
        return;
    }
    let Ok(fly) = flies.single() else {
        return;
    };
    let mut found = 0;
    let mut stack = vec![fly];
    while let Some(entity) = stack.pop() {
        if let Ok(kids) = children.get(entity) {
            stack.extend(kids.iter());
        }
        let Ok(name) = names.get(entity) else {
            continue;
        };
        let side = match name.as_str() {
            "bone_12" => -1.0,
            "bone_14" => 1.0,
            _ => continue,
        };
        if already.contains(entity) {
            continue;
        }
        let Ok(rest) = poses.get(entity) else {
            continue;
        };
        commands.entity(entity).insert(ModelWing {
            rest: rest.rotation,
            side,
        });
        found += 1;
    }
    if found == 2 {
        *looked = true;
        info!("the rigged fly's wings are wired up");
    }
}

/// Work the model's wings, the way the built ones are worked.
///
/// Not a flap. A housefly beats at about two hundred a second and a screen
/// draws sixty, so an honest flap aliases into a slow, wrong-looking flutter —
/// the built wings answer that by *smearing*, widening into the arc they sweep
/// and thinning as they work harder. The same answer here: the wings sweep up
/// and forward into the beat as effort rises, and spread wider as they go.
///
/// `FLY_BEAT=<0..1>` forces the effort, because a capture cannot hold a key and
/// a wing at rest tells you nothing about a wing at work.
fn beat_the_model_wings(
    time: Res<Time>,
    flies: Query<&Fly>,
    mut wings: Query<(&ModelWing, &mut Transform)>,
) {
    let Ok(fly) = flies.single() else {
        return;
    };
    let forced = std::env::var("FLY_BEAT")
        .ok()
        .and_then(|v| v.parse::<f32>().ok());
    let effort = forced.unwrap_or_else(|| fly.effort()).clamp(0.0, 1.0);
    let blend = 1.0 - (-POSE_RATE * time.delta_secs()).exp();

    /// How far the wing sweeps up out of rest at full effort, radians.
    const SWEEP: f32 = 0.40;
    /// How much wider the membrane reads when it is working.
    const SMEAR: f32 = 2.0;
    /// The buzz, in beats a second. Deliberately far below a housefly's two
    /// hundred and deliberately above what a screen can resolve: at sixty
    /// frames it aliases, and aliasing a *small* amplitude is what a blur
    /// looks like. Aliasing a large one is a slow flap, which is what this
    /// looked like when the number was twelve.
    const BUZZ: f32 = 27.0;
    /// How far it shivers about the smear. Small on purpose.
    const SHIVER: f32 = 0.14;
    /// How much the whole wing rocks fore and aft across a beat, which is what
    /// stops a smear reading as a stuck decal.
    const ROCK: f32 = 0.09;

    let clock = time.elapsed_secs();
    for (wing, mut pose) in &mut wings {
        // Held pose: swept up and out into the stroke as effort rises.
        let held = wing.rest * Quat::from_rotation_x(SWEEP * effort * wing.side);
        pose.rotation = pose.rotation.slerp(held, blend);

        // Then the buzz on top, applied *after* the smoothing so it is not
        // smoothed away — a shiver eased at POSE_RATE is no shiver at all.
        // The two wings run a half beat apart, because a pair in perfect step
        // reads as one object.
        let beat = (clock * BUZZ + wing.side * 0.25) * std::f32::consts::TAU;
        pose.rotation *= Quat::from_rotation_x(beat.sin() * SHIVER * effort * wing.side)
            * Quat::from_rotation_z(beat.cos() * ROCK * effort);

        let widen = 1.0 + (SMEAR - 1.0) * effort;
        pose.scale = pose.scale.lerp(Vec3::new(1.0, widen, 1.0), blend);
    }
}

/// Drive the rigged model's own walk clip from how fast the fly is actually
/// crossing the floor. **Off unless `FLY_WALK=1`, because the clip does not fit
/// the rig.**
///
/// `preset:hexapod:walk` leaves the root, the spine and the head alone — good —
/// and then swings three bones named `tripo::0_Left_Limb_2`, `_4` and `_6`
/// through sixty to ninety degrees. Weighing the mesh against each bone says
/// what they actually hold: `_6` alone carries eight hundred and eighty units
/// of vertex weight spread across ninety-six per cent of the model's width.
/// That is not one leg, it is most of the undercarriage, and a preset that
/// takes it for a femur throws the animal about.
///
/// So this is not a constant offset that can be subtracted, the way the
/// father's stoop and forearm roll were. The preset and the rig disagree about
/// what the bones *are*. The bind pose is excellent, so standing in it beats
/// thrashing in a clip written for a different skeleton.
///
/// The same weighing does say where the wings are, which is worth having for
/// whoever rigs them: `bone_12` and `bone_14`, high on the body at y≈0.70 and
/// symmetric about the centreline at z≈±0.31.
///
/// The model brings one clip, a hexapod walk, and a fly spends most of its life
/// off the ground — so the clip is not simply left looping. Its speed is the
/// fly's ground speed over the pace the clip was authored at, which means the
/// feet keep up with the floor instead of skating on it, and it stops dead when
/// the fly does. In the air it is still, because a fly in flight is not walking.
///
/// The same argument as the father's walk, and the same trap avoided: picking a
/// playback rate by eye guarantees the legs and the floor disagree.
fn walk_the_model(
    flies: Query<&Fly>,
    mut players: Query<&mut AnimationPlayer>,
    mut clip: Local<Option<AnimationNodeIndex>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    files: Res<Assets<Gltf>>,
    assets: Res<AssetServer>,
    mut handle: Local<Option<Handle<Gltf>>>,
    mut commands: Commands,
    flies_entity: Query<Entity, With<Fly>>,
    children: Query<&Children>,
    has_player: Query<(), With<AnimationPlayer>>,
) {
    if worn() != Worn::Rigged || std::env::var("FLY_WALK").as_deref() != Ok("1") {
        return;
    }
    let file = handle
        .get_or_insert_with(|| assets.load("characters/fly/fly-legs.glb"))
        .clone();

    // Start it once the skeleton and the file are both here.
    // Found by walking the fly's own hierarchy, not by asking the world for an
    // animation player. There is another body in this house with a skeleton in
    // it, and a global query happily handed the fly's walk to the father: the
    // graph went on his root, the fly's player never got one, and the legs sat
    // perfectly still while the log said the clip was playing.
    let fly_root = flies_entity.single().ok().and_then(|fly| {
        let mut stack = vec![fly];
        while let Some(entity) = stack.pop() {
            if has_player.contains(entity) {
                return Some(entity);
            }
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
        }
        None
    });

    if clip.is_none()
        && let Some(loaded) = files.get(&file)
        && let Some(first) = loaded.animations.first()
        && let Some(root) = fly_root
    {
        let (graph, node) = AnimationGraph::from_clip(first.clone());
        commands
            .entity(root)
            .insert(AnimationGraphHandle(graphs.add(graph)));
        if let Ok(mut player) = players.get_mut(root) {
            player.play(node).repeat();
        }
        *clip = Some(node);
        info!(
            "the fly wears a rigged body: {} clips",
            loaded.animations.len()
        );
    }
    let Some(node) = *clip else {
        return;
    };
    let Ok(fly) = flies.single() else {
        return;
    };

    /// How fast the fly crosses the floor when the clip runs at its authored
    /// speed, in centimetres a second. Tuned against the model's own stride.
    const PACE: f32 = 2.6;
    // A capture cannot press a key, so the walk can be run from outside — the
    // same reason `FLY_GAIT` exists for the built legs.
    let forced = std::env::var("FLY_GAIT").is_ok();
    let ground = if forced {
        PACE
    } else if matches!(fly.stance, Stance::Perched(_)) {
        (fly.pos - fly.prev_pos).with_y(0.0).length() * crate::fly::TICK_RATE as f32
    } else {
        0.0
    };
    if let Some(root) = fly_root
        && let Ok(mut player) = players.get_mut(root)
        && let Some(playing) = player.animation_mut(node)
    {
        playing.set_speed((ground / PACE).clamp(0.0, 4.0));
    }
}

fn pose_the_legs(
    time: Res<Time>,
    flies: Query<(&Fly, &Intent)>,
    legs: Query<(Entity, &Leg)>,
    mut poses: Query<&mut Transform>,
    mut was: Local<Option<Vec3>>,
    mut gait: Local<f32>,
    mut axis: Local<Vec3>,
    mut stepping: Local<f32>,
    mut extend: Local<f32>,
    mut forced: Local<Option<Option<f32>>>,
) {
    /// Fraction of the cycle a foot spends on the ground. Insects run about
    /// six tenths at a walk, and it has to be over a half or the tripods swap
    /// with nothing down in between.
    const STANCE: f32 = 0.62;
    /// Half a stride, in centimetres. Two millimetres a step for a fly this
    /// size.
    const AMP: f32 = 0.11;
    /// How far a foot clears the ground on the way through.
    const LIFT: f32 = 0.06;

    let Ok((fly, intent)) = flies.single() else {
        return;
    };
    let dt = time.delta_secs();
    let blend = 1.0 - (-POSE_RATE * dt).exp();

    // Distance actually travelled, not speed.
    //
    // `walk_about` zeroes the velocity every tick — a perched fly is placed,
    // not integrated — so the old test of `vel.length() > 0.35` was never once
    // true and this animation had never played. Measuring the body's own
    // movement is also what makes the gait immune to how walking is
    // implemented: the feet keep up with the fly by construction.
    let here = fly.pos;
    let moved = was.map(|w| here - w).unwrap_or(Vec3::ZERO);
    *was = Some(here);

    let walking = matches!(fly.stance, Stance::Perched(_));
    let travelled = if walking { moved.length() } else { 0.0 };

    // One cycle carries the body exactly one stance's worth of foot travel, so
    // a planted foot does not slide: over the stance the foot goes back by two
    // amplitudes while the body goes forward by the same.
    const CYCLE: f32 = 2.0 * AMP / STANCE;
    *gait = (*gait + travelled / CYCLE).fract();

    if travelled > 1e-5 {
        let want = (fly.body.inverse() * (moved / travelled)).with_y(0.0);
        if want.length_squared() > 1e-6 {
            *axis = axis.lerp(want.normalize(), blend).normalize_or_zero();
        }
    }
    if axis.length_squared() < 0.5 {
        *axis = Vec3::NEG_Z;
    }

    // Ease the stride in and out rather than freezing a leg mid-swing the
    // instant somebody lets go of the key.
    //
    // Speed comes from the last *tick*, not from this frame. The simulation
    // runs at a fixed sixty-four hertz and drawing runs faster, so on most
    // frames the fly has not moved at all and a per-frame speed flickers
    // between walking pace and nothing — which would leave the stride damped
    // to some average of the two and the legs shuffling.
    let speed = (fly.pos - fly.prev_pos).length() * crate::fly::TICK_RATE as f32;
    let moving = if speed > 0.6 { 1.0 } else { 0.0 };
    *stepping += (moving - *stepping) * blend;
    // Reaching counts as planted: asking to land puts the legs out early, and
    // that is the tell that the fly is about to grab something.
    let out = if walking || intent.land { 1.0 } else { 0.0 };
    *extend += (out - *extend) * blend;

    // A capture cannot press a key, so the cycle can be posed from outside.
    // Read once and kept: this runs every frame.
    let forced = *forced.get_or_insert_with(|| {
        std::env::var("FLY_GAIT")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
    });
    let (phase, stride) = match forced {
        Some(p) => (p, 1.0),
        None => (*gait, *stepping),
    };

    for (pivot, leg) in &legs {
        let t = (phase + leg.phase).fract();
        let (along, lift) = if t < STANCE {
            // On the ground, going straight back at the speed the body is
            // going forward.
            (AMP - 2.0 * AMP * (t / STANCE), 0.0)
        } else {
            // Through the air, eased at both ends so the foot sets down
            // rather than arriving.
            let u = (t - STANCE) / (1.0 - STANCE);
            let eased = u * u * (3.0 - 2.0 * u);
            (
                -AMP + 2.0 * AMP * eased,
                (u * std::f32::consts::PI).sin() * LIFT,
            )
        };

        let planted = leg.home + *axis * (along * stride) + Vec3::Y * (lift * stride);
        let foot = leg.folded.lerp(planted, *extend);
        let pole = (Vec3::new(leg.side, 0.0, 0.0) * 0.55 + Vec3::Y * 0.85).normalize();
        let (femur_dir, tibia_dir) = reach(leg.anchor, foot, leg.femur, leg.tibia, pole);

        let hip = Transform::default().looking_to(femur_dir, Vec3::Y).rotation;
        let shin = Transform::default().looking_to(tibia_dir, Vec3::Y).rotation;
        if let Ok(mut femur) = poses.get_mut(pivot) {
            femur.rotation = hip;
        }
        // The knee hangs off the femur, so its rotation is relative to it.
        if let Ok(mut tibia) = poses.get_mut(leg.knee) {
            tibia.rotation = hip.inverse() * shin;
        }
    }
}
