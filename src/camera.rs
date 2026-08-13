//! The eye.
//!
//! Two toggles live here, and they exist because neither question can be settled
//! from a desk. They are most of the reason this spike is worth building.
//!
//! **What happens to the horizon when you land on a ceiling?** (`R`)
//!
//! - `WorldUp` keeps the room upright and draws the fly upside down. The player
//!   never loses their bearings; the fly reads as a thing you are watching.
//! - `BodyUp` rolls with the fly, so the room turns over instead. The player is
//!   the fly; the cost is that "up" stops meaning anything, and every landing is
//!   a 180° roll.
//!
//! Both are defensible and they feel completely different. There is no way to
//! know which is right without a ceiling, a fly, and ten minutes.
//!
//! **How close is too close?** (`Q`)
//!
//! First person is the honest answer to what a fly sees and is a strong candidate
//! for making people ill — at this scale the camera translates through several
//! body-lengths in a tenth of a second. The short chase cam is the safe default,
//! and it is the default here, but the comparison needs to be one keypress away
//! or it will never actually get made.
//!
//! The wide field of view in both modes is a gesture at compound eyes. A real fly
//! sees very nearly everywhere at once at terrible resolution; a 95° frustum is
//! not that, but it does the one thing that matters at this scale, which is keep
//! enough of the room on screen to know where you are.

use bevy::camera::Exposure;
use bevy::prelude::*;

use crate::fly::{BODY_RADIUS, Fly};
use crate::world::Home;

/// Near plane. Half a millimetre: close enough that a wall the fly is pressed
/// against still draws.
const NEAR: f32 = 0.05;
/// Far plane. The house is under nine metres corner to corner.
const FAR: f32 = 3000.0;

const CHASE_FOV: f32 = 95.0;
const FIRST_PERSON_FOV: f32 = 110.0;

/// How far behind the fly the chase camera sits, in centimetres — about three and
/// a half body lengths. Close enough that the fly fills a useful part of the
/// frame, far enough that its own body never hides what it is about to hit.
///
/// This is the *true* distance now. It used to be 4.5 and play at more like 11,
/// because the eye smoothed its absolute position and therefore trailed the fly
/// by `speed / FOLLOW_RATE` whenever it was moving — see [`FOLLOW_RATE`].
const CHASE_DISTANCE: f32 = 2.2;
/// And how far above the fly's own up-axis.
const CHASE_LIFT: f32 = 0.55;

/// The camera keeps at least this much clear of anything it would otherwise be
/// buried in. The kitchen makes this earn its keep.
const EYE_CLEARANCE: f32 = 0.6;

/// Exponential smoothing rates, in 1/seconds, applied as `1 - exp(-rate * dt)` so
/// the feel does not change with frame rate.
///
/// What is smoothed matters far more than the rate. This used to ease the
/// camera's *absolute position* toward its target, and easing toward a moving
/// target trails it forever: the steady-state lag is `speed / rate`, so at
/// 180 cm/s and rate 26 the eye sat a further 6.9 cm back than it was asked to.
/// That is half again the entire chase distance — and in first person, where the
/// target is the fly's own head, it is the whole difference between first person
/// and a mediocre third person.
///
/// It now smooths the *offset from the fly*, which has no velocity term at all.
/// Flying in a straight line puts the eye exactly where the constants say, and
/// the smoothing does only what it was wanted for: taking the edge off changes of
/// direction. The rate is a taste setting again, rather than something that has
/// to be raised every time `CRUISE` is.
///
/// The up-vector rate is deliberately much slower, so that landing on a ceiling
/// in `BodyUp` reads as a roll rather than a cut.
const FOLLOW_RATE: f32 = 22.0;
const ROLL_RATE: f32 = 5.0;

/// `FLY_INSPECT=<degrees>` parks the camera close to the fly at that azimuth and
/// stares at it: 0 is directly behind, 180 head-on, 90 side-on. It is for
/// *looking at the model* rather than playing, and it exists because judging a
/// seven-millimetre silhouette from the chase camera is hopeless — the first
/// attempt at this fly read as facing backwards and no in-game screenshot was
/// ever going to show why.
pub fn inspect_azimuth() -> Option<f32> {
    std::env::var("FLY_INSPECT").ok()?.parse::<f32>().ok()
}

/// How far the inspection camera sits from the fly, in centimetres.
const INSPECT_DISTANCE: f32 = 2.1;

/// `FLY_ROOM=<name>` stands in a room and looks across it.
///
/// The plan view says whether a house has the right rooms; it says nothing
/// about whether a room is worth being in. This is the other half: a fixed
/// viewpoint per room, from a corner at standing height, so two passes over the
/// same room can be compared frame to frame rather than from memory.
/// `FLY_ROOM=kitchen` or `FLY_ROOM=kitchen:ne` — the room, and optionally which
/// corner to stand in. One diagonal cannot show a whole room: whichever two
/// walls the camera has its back to are invisible, and in a kitchen that is
/// every counter in it. Two opposite corners see everything.
pub fn room_view() -> Option<String> {
    std::env::var("FLY_ROOM").ok().filter(|r| !r.is_empty())
}

/// `FLY_PLAN=1` looks straight down at the whole house from above it.
///
/// Not a play mode — a way to *see a drawn house at all*. A fly's own camera is
/// six body-lengths off the floor and usually under furniture, which is a fine
/// way to judge flight and a hopeless way to answer "did the import work, and is
/// the place lit". The first ranch loaded read as a black rectangle from down
/// there whether the fault was the geometry, the lamps or the table it happened
/// to be sitting under.
pub fn plan_view() -> bool {
    std::env::var("FLY_PLAN").as_deref() == Ok("1")
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum View {
    #[default]
    Chase,
    FirstPerson,
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Roll {
    /// The room stays upright; the fly goes upside down.
    #[default]
    WorldUp,
    /// The fly stays upright; the room goes upside down.
    BodyUp,
}

#[derive(Component)]
pub struct Eye {
    /// Smoothed camera up, so the roll between conventions is continuous.
    up: Vec3,
    /// Smoothed offset from the fly. Smoothing this rather than the world
    /// position is what keeps the eye from trailing at speed.
    offset: Vec3,
    /// Whether the rig has been placed at least once. The first frame must snap
    /// rather than ease, or the camera spends a second flying in from the origin.
    seated: bool,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<View>()
            .init_resource::<Roll>()
            .add_systems(Startup, spawn_eye)
            .add_systems(Update, (choose_view, place_the_eye).chain());
    }
}

fn spawn_eye(mut commands: Commands) {
    commands.spawn((
        Name::new("Eye"),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: CHASE_FOV.to_radians(),
            near: NEAR,
            far: FAR,
            ..default()
        }),
        // Bevy's default exposure is `BLENDER`, at EV100 9.7 — calibrated for
        // daylight. A room lit by an 800-lumen bulb sits in the tens of lux, and
        // against that stop it renders essentially black, which is exactly what
        // the first pass looked like. EV100 7 is the indoor stop and is worth
        // more than any amount of tuning the lights themselves: the physics was
        // already right, the camera was stopped down for a beach.
        Exposure::INDOOR,
        Transform::default(),
        bevy::ui::IsDefaultUiCamera,
        Eye {
            up: Vec3::Y,
            offset: Vec3::ZERO,
            seated: false,
        },
    ));
}

fn choose_view(keys: Res<ButtonInput<KeyCode>>, mut view: ResMut<View>, mut roll: ResMut<Roll>) {
    if keys.just_pressed(KeyCode::KeyQ) {
        *view = match *view {
            View::Chase => View::FirstPerson,
            View::FirstPerson => View::Chase,
        };
    }
    if keys.just_pressed(KeyCode::KeyR) {
        *roll = match *roll {
            Roll::WorldUp => Roll::BodyUp,
            Roll::BodyUp => Roll::WorldUp,
        };
    }
}

fn place_the_eye(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    view: Res<View>,
    roll: Res<Roll>,
    home: Res<Home>,
    flies: Query<&Fly>,
    mut eyes: Query<(&mut Transform, &mut Projection, &mut Eye)>,
) {
    let Ok(fly) = flies.single() else {
        return;
    };
    let Ok((mut transform, mut projection, mut eye)) = eyes.single_mut() else {
        return;
    };

    // The body is simulated at 64 Hz and drawn between ticks. `alpha` is the
    // leftover fraction of the current tick, and using it here rather than the
    // raw tick position is the whole reason the fly does not stutter at 144 fps.
    let alpha = fixed.overstep_fraction();
    let (position, body) = fly.presented(alpha);
    let dt = time.delta_secs();

    // Standing in one room, looking across it.
    if let Some(spec) = room_view() {
        let (name, corner) = match spec.split_once(':') {
            Some((n, c)) => (n.to_string(), c.to_ascii_lowercase()),
            None => (spec.clone(), "sw".to_string()),
        };
        let r = crate::house::rooms()
            .into_iter()
            .find(|r| r.name.eq_ignore_ascii_case(&name));
        if let Some(r) = r {
            // Corner to opposite corner, which is the only line that sees a
            // whole room. Aiming at the *middle* instead — which is what this
            // did first — puts everything along the near two walls behind the
            // camera, and a kitchen's entire worth of counters with it.
            let (near, far) = match corner.as_str() {
                "ne" => (Vec2::new(0.90, 0.90), Vec2::new(0.15, 0.15)),
                "nw" => (Vec2::new(0.10, 0.90), Vec2::new(0.85, 0.15)),
                "se" => (Vec2::new(0.90, 0.10), Vec2::new(0.15, 0.85)),
                _ => (Vec2::new(0.10, 0.10), Vec2::new(0.85, 0.85)),
            };
            let mut from = Vec3::new(
                r.min.x + r.wide() * near.x,
                165.0,
                r.min.y + r.deep() * near.y,
            );
            let at = Vec3::new(r.min.x + r.wide() * far.x, 55.0, r.min.y + r.deep() * far.y);

            // A corner is exactly where furniture goes, and two garage captures
            // in a row came back as a close-up of the back of a shelf unit.
            // Walk the viewpoint along its own sightline until it is clear of
            // everything, which costs one short loop and makes the corner views
            // trustworthy without hand-placing a camera per room.
            let step = (at - from).normalize_or_zero();
            for _ in 0..12 {
                let tight = home
                    .solids
                    .iter()
                    .any(|s| !s.sheer && s.nearest(from).distance < 75.0);
                if !tight {
                    break;
                }
                from += step * 16.0;
            }
            transform.translation = from;
            transform.look_at(at, Vec3::Y);
            if let Projection::Perspective(perspective) = &mut *projection {
                perspective.fov = 62.0_f32.to_radians();
            }
            eye.seated = true;
            return;
        }
        warn!("no room called '{name}'");
    }

    // Straight down at the whole house, framed by its own bounds.
    if plan_view() {
        let mut low = Vec3::splat(f32::INFINITY);
        let mut high = Vec3::splat(f32::NEG_INFINITY);
        for solid in &home.solids {
            let reach = (solid.rot * solid.half).abs().max(solid.half);
            low = low.min(solid.center - reach);
            high = high.max(solid.center + reach);
        }
        if low.x.is_finite() {
            let middle = (low + high) * 0.5;
            let span = (high - low).max_element();
            transform.translation = Vec3::new(middle.x, high.y + span * 0.9, middle.z);
            transform.look_at(Vec3::new(middle.x, low.y, middle.z), Vec3::NEG_Z);
            if let Projection::Perspective(perspective) = &mut *projection {
                perspective.fov = 60.0_f32.to_radians();
            }
        }
        eye.seated = true;
        return;
    }

    // Inspection overrides everything: fixed offset, no smoothing, looking
    // straight at the fly.
    if let Some(azimuth) = inspect_azimuth() {
        let offset =
            Quat::from_rotation_y(azimuth.to_radians()) * Vec3::new(0.0, 0.42, 1.0).normalize();
        transform.translation = position + offset * INSPECT_DISTANCE;
        transform.look_at(position, Vec3::Y);
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = 40.0_f32.to_radians();
        }
        eye.seated = true;
        return;
    }

    let want_up = match *roll {
        Roll::WorldUp => Vec3::Y,
        Roll::BodyUp => body * Vec3::Y,
    };
    // Slerp-ish easing on the up vector. Normalising a lerp is enough here and
    // stays well-behaved right up until the two are opposed, which is exactly the
    // ceiling case — so nudge off the pole when it happens.
    let blend = 1.0 - (-ROLL_RATE * dt).exp();
    let mut up = if eye.seated {
        eye.up.lerp(want_up, blend)
    } else {
        want_up
    };
    if up.length_squared() < 1e-4 {
        up = (want_up + body * Vec3::X * 0.01).normalize();
    }
    eye.up = up.normalize();

    let aim = fly.heading();

    let (want_offset, fov) = match *view {
        View::FirstPerson => {
            // Just ahead of the thorax, roughly where the head is.
            (aim * (BODY_RADIUS * 1.2), FIRST_PERSON_FOV)
        }
        View::Chase => {
            // Behind along the *aim*, not along the body. The body saccades in
            // discrete jumps by design, and a camera that copied that would be
            // unusable.
            //
            // The lift, though, follows the *fly's* up rather than the world's.
            // Lifting along world up parks the camera inside the ceiling every
            // time the fly lands on one — and since the clearance cast below
            // then pulls the eye in to avoid the plaster, the result was the
            // camera collapsing onto the fly's back. Following the body's up
            // means the lift is always away from whatever it is standing on.
            let lift = body * Vec3::Y;
            let mut back = -aim * CHASE_DISTANCE + lift * CHASE_LIFT;

            // Do not sit inside the cabinets. Cast from the fly outward and pull
            // the eye in to the first thing in the way.
            let distance = back.length();
            if distance > 1e-4
                && let Some(hit) = home.raycast(position, back / distance, distance)
            {
                back = (back / distance) * (hit.distance - EYE_CLEARANCE).max(0.0);
            }
            (back, CHASE_FOV)
        }
    };

    // First person is never smoothed. Easing the eye toward the head leaves it
    // behind the head by `speed / FOLLOW_RATE` for as long as the fly is moving,
    // which is not a slightly-late first person — it is third person.
    let follow = if eye.seated && *view == View::Chase {
        1.0 - (-FOLLOW_RATE * dt).exp()
    } else {
        1.0
    };
    eye.offset = eye.offset.lerp(want_offset, follow);
    transform.translation = position + eye.offset;

    // The look target is not smoothed. Smoothing the aim as well as the position
    // stacks two lags and the result feels like steering a boat.
    let target = match *view {
        View::FirstPerson => transform.translation + aim,
        View::Chase => position + aim * CHASE_DISTANCE * 0.5,
    };
    let direction = target - transform.translation;
    if direction.length_squared() > 1e-6 {
        transform.look_to(direction.normalize(), eye.up);
    }

    if let Projection::Perspective(perspective) = &mut *projection {
        perspective.fov = fov.to_radians();
    }
    eye.seated = true;
}
