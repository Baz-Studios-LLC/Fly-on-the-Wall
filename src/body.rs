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

/// Turn the model's −X nose into Bevy's −Z forward.
fn facing() -> Quat {
    Quat::from_rotation_y(-FRAC_PI_2)
}

/// The procedural fly is the default while the model is still being worked on.
/// `FLY_MODEL=glb` swaps in `assets/fly.glb`.
fn use_the_model() -> bool {
    std::env::var("FLY_MODEL").as_deref() == Ok("glb")
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

#[derive(Component)]
struct Wing;

/// A leg's pivot, carrying the two poses it interpolates between and which
/// half of the gait it belongs to.
///
/// Flies walk an **alternating tripod**: front and hind on one side move with
/// the middle leg of the other, so three feet are always down and the insect
/// never has to balance. It is the reason a fly can walk up a wall without
/// falling off, and it is what makes six legs read as walking rather than as
/// six legs twitching.
#[derive(Component)]
struct Leg {
    planted: Quat,
    tucked: Quat,
    /// True for one tripod, false for the other.
    tripod: bool,
}

#[derive(Resource)]
struct WingSkin(Handle<StandardMaterial>);

pub struct BodyPlugin;

impl Plugin for BodyPlugin {
    fn build(&self, app: &mut App) {
        // `PostStartup` so the fly entity from `fly::hatch` already exists.
        app.add_systems(PostStartup, grow_the_body)
            .add_systems(Update, (present_the_fly, work_the_wings, pose_the_legs));
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

    if use_the_model() {
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
    // A pivot per leg with the segment hung off its −Z, so a pose is one
    // `looking_to` and each leg can be written as the direction it obviously
    // points. The previous version composed two Euler rotations, which meant the
    // forward-reaching front legs silently swung *inward* once their pitch took
    // them past vertical — the kind of bug that is invisible in the code and
    // obvious the moment you look at the thing.
    let segment = box_mesh(&mut meshes, 0.020, 0.020, 0.26);

    // Anchor and planted direction, given for the right side; the left mirrors
    // in x. Fore legs reach forward, hind legs trail back, as a fly's do.
    let plan: [(Vec3, Vec3); 3] = [
        (
            Vec3::new(0.10, -0.045, -0.15),
            Vec3::new(0.62, -0.60, -0.58),
        ),
        (Vec3::new(0.115, -0.05, -0.02), Vec3::new(0.85, -0.58, 0.08)),
        (Vec3::new(0.10, -0.05, 0.10), Vec3::new(0.60, -0.55, 0.66)),
    ];
    // In flight everything folds back along the body.
    let tucked_dir = Vec3::new(0.30, -0.26, 0.92);

    for side in [-1.0f32, 1.0] {
        let mirror = |v: Vec3| Vec3::new(v.x * side, v.y, v.z);
        for (row, (anchor, planted_dir)) in plan.into_iter().enumerate() {
            let pose = |d: Vec3| {
                Transform::default()
                    .looking_to(mirror(d).normalize(), Vec3::Y)
                    .rotation
            };
            let planted = pose(planted_dir);
            let tucked = pose(tucked_dir);
            // Fore and hind on one side, middle on the other.
            let tripod = (row % 2 == 0) == (side > 0.0);

            let pivot = commands
                .spawn((
                    ChildOf(trunk),
                    Leg {
                        planted,
                        tucked,
                        tripod,
                    },
                    Transform::from_translation(mirror(anchor)).with_rotation(planted),
                    Visibility::default(),
                ))
                .id();
            commands.spawn((
                ChildOf(pivot),
                Mesh3d(segment.clone()),
                MeshMaterial3d(chitin.clone()),
                // Hung off −Z, which is where `looking_to` aims.
                Transform::from_translation(Vec3::new(0.0, 0.0, -0.13)),
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

fn pose_the_legs(
    time: Res<Time>,
    flies: Query<(&Fly, &Intent)>,
    mut legs: Query<(&Leg, &mut Transform)>,
    mut gait: Local<f32>,
    mut forced: Local<Option<Option<f32>>>,
) {
    /// How far the fly walks per full step cycle, in centimetres. Driving the
    /// cycle off *distance* rather than time is what stops the feet skating:
    /// walk slowly and the legs move slowly, by construction.
    const STRIDE: f32 = 1.6;
    /// How far a leg swings fore and aft, and how far the foot lifts.
    const SWING: f32 = 0.34;
    const LIFT: f32 = 0.22;

    let Ok((fly, intent)) = flies.single() else {
        return;
    };
    // Reaching counts as planted: asking to land puts the legs out early, which
    // is the tell that the fly is about to grab something.
    let perched = matches!(fly.stance, Stance::Perched(_));
    let planted = perched || intent.land;
    let blend = 1.0 - (-POSE_RATE * time.delta_secs()).exp();

    let speed = fly.vel.length();
    let walking = perched && speed > 0.35;
    if walking {
        *gait += speed * time.delta_secs() / STRIDE * std::f32::consts::TAU;
    }
    // A capture cannot press a key, so the cycle can be posed from outside.
    // Read once and kept: this runs every frame.
    let forced =
        *forced.get_or_insert_with(|| std::env::var("FLY_GAIT").ok().and_then(|v| v.parse().ok()));
    let phase = forced.map(|p| p * std::f32::consts::TAU).unwrap_or(*gait);

    for (leg, mut transform) in &mut legs {
        let mut want = if planted { leg.planted } else { leg.tucked };
        if walking || forced.is_some() {
            let a = phase
                + if leg.tripod {
                    0.0
                } else {
                    std::f32::consts::PI
                };
            // Swing about the body's up axis is the step; the lift is only on
            // the half of the cycle where the foot is off the ground, so the
            // three legs that are down stay down.
            let swing = Quat::from_rotation_y(a.sin() * SWING);
            let lift = Quat::from_rotation_x(a.cos().max(0.0) * LIFT);
            want = lift * swing * want;
        }
        transform.rotation = transform.rotation.slerp(want, blend);
    }
}
