//! The fly: how it flies, how it lands, and how it walks on a ceiling.
//!
//! This is the file the whole spike exists to write. Everything else here is
//! scaffolding for the question it answers — *is being a fly worth doing?* — so
//! it is built to the standard of something that will be tuned for months.
//!
//! Five decisions carry the feel:
//!
//! - **A fly darts; it does not hover.** There is no lift, no stall, no angle of
//!   attack — thrust is applied in the heading's frame and drag is isotropic — so
//!   the movement code that fits is Quake's `accelerate`/`friction` pair with the
//!   ground case deleted. But the first version made the drag high enough to hold
//!   station, and that is a *hoverfly*, a different family. A housefly commits to
//!   a direction, carries, snaps, and commits again, and it answers "stop flying"
//!   by landing rather than by parking in the air. Low drag, a real sag, and a
//!   floor under the drag so a fly that stops pushing glides rather than halts,
//!   are what buy that.
//! - **Friction sets the top speed, thrust sets the response.** Same lesson as
//!   the walk in Flat Earth Simulator: clamping velocity feels like a governor,
//!   letting drag find the equilibrium feels like a body.
//! - **Contact is landing.** Touch anything at any speed and the fly is on it.
//!   This was built the other way first — sticking only below a speed threshold,
//!   sliding otherwise, on the theory that being glued to a lampshade you meant
//!   to pass would be infuriating. Playing it settled the argument: skidding off
//!   a wall you plainly hit reads as a bug, not a mechanic, and a housefly does
//!   not bounce. Taking off again costs one keypress, which is far cheaper than
//!   never being sure a landing will take. The corollary is that the collision
//!   test has to be *swept*, because at real speed the fly crosses more ground
//!   in a tick than the window pane is thick.
//! - **A perch is stored in the surface's own frame,** not the world's. One
//!   consequence pays for the whole design: a fly standing on the door is still
//!   standing on the door after the door swings, with no code that knows the door
//!   exists.
//! - **Walking wraps around edges in both directions.** Convex (off the lip of a
//!   table, onto its side) and concave (along the floor, up the wall). Without
//!   both, "walk on any surface" is a demo of walking on one surface.
//!
//! What is deliberately absent: any threat, any need, any human. A movement model
//! with nothing at stake can feel excellent and still be wrong, because pressure
//! changes how a control scheme reads. This spike cannot answer that and should
//! not pretend to.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::world::Home;

// ---------------------------------------------------------------------------
// Tuning
// ---------------------------------------------------------------------------

/// Simulation rate. Movement is fixed-step and the body is interpolated between
/// ticks, so the feel is identical at 60 and 144 fps.
pub const TICK_RATE: f64 = 64.0;

/// The fly's collision sphere, in centimetres. A housefly's body is 6 to 7 mm
/// long and about 2.5 mm across, so 2.6 mm of radius is the body, not the legs —
/// which is the right thing to collide with, since legs that clip a wall are
/// invisible and a thorax that does is not.
pub const BODY_RADIUS: f32 = 0.26;

/// Cruise speed, cm/s. **The real figure**, and it stayed.
///
/// Commonly cited for *Musca domestica* is about two metres a second. This was
/// turned down to 180 for a while on the theory that faithful speed would read
/// as twitchy rather than fast — and then put back, by someone who had actually
/// played it, once the camera stopped trailing and contact started meaning
/// landing. Worth recording, because the instinct was wrong in an instructive
/// way: what felt like too much speed was mostly a camera lagging `speed /
/// FOLLOW_RATE` behind the thing it was framing. Fix the framing and the speed
/// is fine.
///
/// It is still worth knowing what it means at this scale. The fly is 0.65 cm, so
/// 200 cm/s is **308 body-lengths per second**, where a sprinting human manages
/// about three. That ratio is genuinely what being a fly is like. The greybox
/// living room crosses in 2.5 seconds.
const CRUISE: f32 = 200.0;

/// Quake's two knobs. Acceleration is enormous relative to the body — a fly is
/// nearly all flight muscle — so the response is near-instant.
const THRUST: f32 = 12.0;

/// Drag, which is what actually sets both the top speed and the coast.
///
/// This was 6.0, and that was the single biggest thing wrong with how the fly
/// felt. At 6.0 releasing thrust stopped it dead inside sixteen centimetres,
/// which is a quadcopter holding station — and a hovering fly is a *hoverfly*,
/// a different family entirely. A housefly darts: it commits to a direction,
/// carries, snaps, and commits again. At 2.2 it bleeds from cruise down to a
/// glide over about seventy-five centimetres — momentum the player has to plan
/// around, which is most of what makes flight a skill.
const DRAG: f32 = 2.2;

/// The speed drag will not take the fly below while it is airborne, cm/s.
///
/// A housefly cannot hover. It can lose its push — and it does, fast — but what
/// is left underneath is a glide, not a stop. Letting drag run all the way to
/// zero gave a fly that coasted to a halt and hung there, which is the same
/// wrong answer the old `STOP_SPEED` gave more abruptly.
///
/// Combined with [`IDLE_SINK`] this produces the behaviour without anything
/// having to model it: let go of everything and the fly settles into a shallow
/// descending drift, still going where it was going, on its way to the floor.
const GLIDE_SPEED: f32 = 30.0;

/// Downward acceleration when nothing at all is being asked for, cm/s².
///
/// Not gravity — the wings are still working. It is what stops mid-air from
/// being a free parking space. Against [`DRAG`] it settles at about 48 cm/s, so
/// letting go of everything means visibly starting to fall, and holding a
/// position becomes something the player is *doing*.
///
/// The deeper version of this is that a real housefly answers "stop flying" by
/// landing, not by sinking, and lands far more often than anything in this build
/// yet gives it reason to. That wants a fatigue budget, which is a need, which
/// is out of scope for a movement spike.
const IDLE_SINK: f32 = 105.0;

/// Downward acceleration while flying but not actively climbing, cm/s².
///
/// Much gentler than [`IDLE_SINK`]: a fly driving itself forward is generating
/// lift, and sinking at the idle rate while crossing the room would put it on
/// the floor before it got there. At about 15 cm/s settled, level flight across
/// the greybox living room costs some 36 cm of height — enough that the player
/// has to keep flying it, not so much that they are fighting it.
const CRUISE_SINK: f32 = 32.0;

/// Walk speed, cm/s. **Three times the real figure**, and the one number here
/// that is deliberately, knowingly wrong.
///
/// A housefly walks at one to two centimetres a second. At that speed crossing
/// the living room on foot takes four minutes, which is accurate and unplayable
/// — walking would never be chosen for anything, and walking is the whole reason
/// the gap under the door exists. At 6 the same crossing is about eighty
/// seconds: still a serious commitment against a 200 cm/s flight, still slow
/// enough that the room feels enormous on foot, but a route rather than a
/// punishment.
///
/// Worth recording that this dial moved *up* while every other realism dial
/// moved down. Small creatures are relatively far faster than us in the air and
/// no faster on foot, so faithful flight reads as twitchy and faithful walking
/// reads as broken.
const WALK: f32 = 6.0;

/// How far a deliberate landing reaches. Holding the land button makes the fly
/// grab anything within four body-lengths, which is the difference between
/// landing on a shelf edge and bouncing off it.
const LAND_REACH: f32 = 2.6;

/// How much of the view direction has to survive projection onto a surface
/// before it is trusted as "forward" — squared length, so this is a projection
/// of 0.2, about 11° off the normal.
///
/// Below it the direction is numerical noise rather than intent, and using it
/// anyway is what made a fresh landing feel unresponsive. See
/// [`surface_forward`].
const FLATTEN_FLOOR: f32 = 0.04;

/// Push-off speed when leaving a surface, cm/s.
const TAKEOFF: f32 = 90.0;

/// How long after taking off the fly cannot re-land, seconds. Without it, pushing
/// off a ceiling re-grabs the ceiling on the next tick.
const TAKEOFF_LOCKOUT: f32 = 0.18;

/// Pitch limit, radians. Short of vertical so the heading's `cross` with world up
/// never degenerates.
const PITCH_LIMIT: f32 = 88.0_f32.to_radians();

/// Mouse sensitivity, degrees of turn per unit of mouse motion.
const SENSITIVITY: f32 = 0.09;

/// How far the body's rendered heading may drift from its true one before it
/// snaps, radians.
///
/// This is the saccade. Real flies do not turn — they fly straight and then
/// change direction in about thirty milliseconds, which is faster than a frame,
/// and it is the single most recognisable thing about how a fly moves. Smoothly
/// interpolating the body's yaw throws that away and gets a hummingbird. So the
/// *body* snaps in discrete jumps while the *camera* follows smoothly: the
/// control stays readable, and the thing you are looking at moves like an insect.
const SACCADE: f32 = 14.0_f32.to_radians();

/// The slow catch-up underneath the saccade, in 1/seconds. Small heading changes
/// would otherwise never show at all.
const BODY_DRIFT: f32 = 6.0;

/// How far the nose comes up when the player asks to land, radians.
const FLARE: f32 = 22.0_f32.to_radians();

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Where the fly is standing, in the surface's own coordinates.
///
/// Storing the *local* position and normal rather than world ones is what makes a
/// perch survive its surface moving. The door swings; the fly's `local_pos` never
/// changes; the world position falls out of the solid's current transform.
#[derive(Clone, Copy, Debug)]
pub struct Perch {
    pub solid: usize,
    pub local_pos: Vec3,
    pub local_normal: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub enum Stance {
    Flying,
    Perched(Perch),
}

impl Stance {
    pub fn perch(&self) -> Option<Perch> {
        match self {
            Stance::Perched(p) => Some(*p),
            Stance::Flying => None,
        }
    }
}

#[derive(Component)]
pub struct Fly {
    pub pos: Vec3,
    pub vel: Vec3,
    /// Heading, which the mouse drives. In flight this is where thrust points; on
    /// a surface it is only used to work out which way "forward" is.
    pub yaw: f32,
    pub pitch: f32,
    pub stance: Stance,
    /// Counts down after a takeoff.
    pub lockout: f32,
    /// The body's rendered orientation, which lags the true heading in jumps.
    pub body: Quat,

    // Previous tick, for render interpolation.
    pub prev_pos: Vec3,
    pub prev_body: Quat,
}

impl Default for Fly {
    fn default() -> Self {
        Fly {
            pos: Vec3::ZERO,
            vel: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            stance: Stance::Flying,
            lockout: 0.0,
            body: Quat::IDENTITY,
            prev_pos: Vec3::ZERO,
            prev_body: Quat::IDENTITY,
        }
    }
}

impl Fly {
    /// Unit vector the fly is aiming along.
    pub fn heading(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(-sy * cp, sp, -cy * cp)
    }

    /// Where the body should be drawn this frame, between the last two ticks.
    pub fn presented(&self, alpha: f32) -> (Vec3, Quat) {
        (
            self.prev_pos.lerp(self.pos, alpha),
            self.prev_body.slerp(self.body, alpha),
        )
    }

    /// How hard the wings are working, 0..1. Drives the wingbeat's pitch and the
    /// width of the blur quads, and is the only "gauge" in the game.
    pub fn effort(&self) -> f32 {
        match self.stance {
            Stance::Perched(_) => 0.0,
            Stance::Flying => (self.vel.length() / CRUISE).clamp(0.0, 1.4),
        }
    }
}

/// Input, latched in `Update` and consumed in `FixedUpdate`.
///
/// Latched rather than read directly because a frame may not contain a tick: a
/// press sampled with `just_pressed` inside `FixedUpdate` is silently dropped
/// whenever the frame rate runs ahead of the tick rate, which at 144 fps is most
/// of them.
#[derive(Component, Default)]
pub struct Intent {
    /// x = strafe, y = climb, z = forward. Each in -1..=1.
    pub thrust: Vec3,
    /// Held: reach for a surface.
    pub land: bool,
    /// Latched: leave the surface.
    pub launch: bool,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct FlyPlugin;

impl Plugin for FlyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(TICK_RATE))
            .add_systems(Startup, hatch)
            .add_systems(Update, (grab_the_cursor, gather_intent, look_around).chain())
            .add_systems(FixedUpdate, step_the_fly);
    }
}

fn hatch(mut commands: Commands, home: Res<Home>) {
    let mut fly = Fly::default();

    // Inspection mode stands the fly on the living room floor instead, out in
    // the open where the camera can get all the way round it.
    // The spawn is the house's business now, not a constant — an imported floor
    // plan has no idea where the greybox put its living room.
    fly.pos = home.spawn;
    fly.prev_pos = home.spawn;
    let (from, toward) = if crate::camera::inspect_azimuth().is_some() {
        (home.spawn.with_y(6.0), Vec3::NEG_Y)
    } else {
        (home.spawn, Vec3::Y)
    };

    // Reach up and take hold of the ceiling, rather than starting in a hover.
    // The first thing the player does is fall off it, and that fall teaches
    // takeoff, the idle sink and the camera's roll convention in about a second
    // and a half without a word of instruction.
    if let Some(hit) = home.raycast(from, toward, 40.0) {
        let solid = &home.solids[hit.solid];
        fly.stance = Stance::Perched(Perch {
            solid: hit.solid,
            local_pos: solid.to_local(hit.point),
            local_normal: solid.rot.inverse() * hit.normal,
        });
        fly.pos = hit.point + hit.normal * BODY_RADIUS;
        fly.prev_pos = fly.pos;
        // Hung upside down from frame one. Without this the first frame draws it
        // upright and the correction is visible.
        let forward = Vec3::NEG_Z - hit.normal * Vec3::NEG_Z.dot(hit.normal);
        fly.body = Transform::default()
            .looking_to(forward.normalize_or_zero(), hit.normal)
            .rotation;
        fly.prev_body = fly.body;
    }

    commands.spawn((Name::new("Fly"), fly, Intent::default()));
}

fn grab_the_cursor(
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut cursor) = cursors.single_mut() else {
        return;
    };
    if keys.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    } else if mouse.just_pressed(MouseButton::Left) && cursor.grab_mode == CursorGrabMode::None {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn gather_intent(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut intents: Query<&mut Intent>,
) {
    let axis = |plus: KeyCode, minus: KeyCode| -> f32 {
        (keys.pressed(plus) as i32 - keys.pressed(minus) as i32) as f32
    };

    for mut intent in &mut intents {
        intent.thrust = Vec3::new(
            axis(KeyCode::KeyD, KeyCode::KeyA),
            axis(KeyCode::Space, KeyCode::ControlLeft),
            axis(KeyCode::KeyW, KeyCode::KeyS),
        );
        intent.land = keys.pressed(KeyCode::KeyF) || mouse.pressed(MouseButton::Right);
        // Latched, not overwritten: a press must survive until a tick eats it.
        intent.launch |= keys.just_pressed(KeyCode::Space);
    }
}

/// Look is read raw, at frame rate, and never smoothed. Input smoothing is the
/// one thing players reliably notice and resent.
fn look_around(
    motion: Res<AccumulatedMouseMotion>,
    cursors: Query<&CursorOptions, With<PrimaryWindow>>,
    mut flies: Query<&mut Fly>,
) {
    if !cursors
        .single()
        .is_ok_and(|c| c.grab_mode == CursorGrabMode::Locked)
    {
        return;
    }
    let delta = motion.delta;
    if delta == Vec2::ZERO {
        return;
    }
    for mut fly in &mut flies {
        fly.yaw -= (delta.x * SENSITIVITY).to_radians();
        fly.pitch = (fly.pitch - (delta.y * SENSITIVITY).to_radians())
            .clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }
}

// ---------------------------------------------------------------------------
// The tick
// ---------------------------------------------------------------------------

pub fn step_the_fly(
    time: Res<Time<Fixed>>,
    home: Res<Home>,
    mut flies: Query<(&mut Fly, &mut Intent)>,
) {
    let dt = time.delta_secs();

    for (mut fly, mut intent) in &mut flies {
        fly.prev_pos = fly.pos;
        fly.prev_body = fly.body;
        fly.lockout = (fly.lockout - dt).max(0.0);

        match fly.stance {
            Stance::Flying => fly_along(&mut fly, &intent, &home, dt),
            Stance::Perched(perch) => walk_about(&mut fly, &intent, &home, perch, dt),
        }

        orient_the_body(&mut fly, &intent, &home, dt);
        intent.launch = false;
    }
}

// ---------------------------------------------------------------------------
// Flight
// ---------------------------------------------------------------------------

/// Quake's accelerate: it limits only the *projection* of velocity onto the wish
/// direction, which is why the result reads as momentum rather than as a speed
/// cap, and why turning while moving preserves speed the way a real flier does.
fn accelerate(vel: Vec3, wish_dir: Vec3, wish_speed: f32, accel: f32, dt: f32) -> Vec3 {
    let current = vel.dot(wish_dir);
    let shortfall = wish_speed - current;
    if shortfall <= 0.0 {
        return vel;
    }
    vel + wish_dir * (accel * wish_speed * dt).min(shortfall)
}

/// Air drag, and the floor underneath it.
///
/// Two departures from the Quake friction this started as. Quake's `STOP_SPEED`
/// — applying friction as though the body were moving faster than it is, so the
/// last of the motion dies quickly — is gone. That is a *ground* idea, the thing
/// that makes a player halt crisply when they release the stick, and in the air
/// it reads as the fly touching a brake. And the decay now bottoms out at
/// [`GLIDE_SPEED`] instead of at zero, because a fly that has stopped pushing is
/// still going somewhere.
fn apply_drag(vel: Vec3, dt: f32) -> Vec3 {
    let speed = vel.length();
    if speed <= GLIDE_SPEED {
        return vel;
    }
    let kept = (speed - speed * DRAG * dt).max(GLIDE_SPEED);
    vel * (kept / speed)
}

fn fly_along(fly: &mut Fly, intent: &Intent, home: &Home, dt: f32) {
    let forward = fly.heading();
    let right = forward.cross(Vec3::Y).normalize_or_zero();

    // Vertical thrust is world-relative, not body-relative. A fly's lift really
    // does tilt with its body, but steering height with a stick that changes
    // meaning as you pitch is miserable, and nobody watching can tell.
    let wish = forward * intent.thrust.z + right * intent.thrust.x + Vec3::Y * intent.thrust.y;
    let wish_dir = wish.normalize_or_zero();

    fly.vel = apply_drag(fly.vel, dt);
    if wish_dir != Vec3::ZERO {
        fly.vel = accelerate(fly.vel, wish_dir, CRUISE, THRUST, dt);
    }
    // Sag: hard when nothing at all is being asked for, gentle while the fly is
    // driving itself somewhere. A fly under power makes its own lift; a fly doing
    // nothing is not entitled to a parking space in mid-air.
    if intent.thrust.y == 0.0 {
        let idling = intent.thrust.x == 0.0 && intent.thrust.z == 0.0;
        fly.vel.y -= if idling { IDLE_SINK } else { CRUISE_SINK } * dt;
    }

    let previous = fly.pos;
    fly.pos += fly.vel * dt;

    // Just pushed off. Resolve overlaps and slide, but do not grab anything —
    // without this the takeoff re-lands on the surface it left on the next tick.
    if fly.lockout > 0.0 {
        for solid in home.solids.iter() {
            let near = solid.nearest(fly.pos);
            if near.distance >= BODY_RADIUS {
                continue;
            }
            fly.pos = near.point + near.normal * BODY_RADIUS;
            let into = fly.vel.dot(near.normal);
            if into < 0.0 {
                fly.vel -= near.normal * into;
            }
        }
        return;
    }

    // 1. Swept. Did the step *cross* a surface rather than end inside one?
    //
    // This is not an optimisation, it is a correctness fix that only became
    // visible once the speeds went real: at 200 cm/s the fly covers 3.1 cm in a
    // tick, and the thinnest solids in the house — the window pane at 2 cm, the
    // door at 4 — are thinner than that. A test that only asks "am I overlapping
    // anything *now*" passes straight through them.
    let step = fly.pos - previous;
    let travelled = step.length();
    if travelled > 1e-5
        && let Some(hit) = home.raycast(previous, step / travelled, travelled + BODY_RADIUS)
    {
        settle(fly, home, hit.solid, hit.point, hit.normal);
        return;
    }

    // 2. Touching. **Contact is landing** — no speed gate, no asking.
    //
    // The first version of this only stuck below a speed threshold and slid
    // along the surface otherwise, on the theory that being glued to a lampshade
    // you meant to pass would be infuriating. Played, it was worse the other
    // way: the fly skidded off walls it had plainly hit, which reads as a bug
    // rather than as a mechanic, and a real housefly does not bounce off things.
    // Getting off again is one press of `Space`, which is a much cheaper way to
    // recover from an unwanted landing than never being sure a wanted one will
    // take.
    let mut contact: Option<(usize, crate::world::Near)> = None;
    for (i, solid) in home.solids.iter().enumerate() {
        let near = solid.nearest(fly.pos);
        if near.distance < BODY_RADIUS
            && contact.is_none_or(|(_, c)| near.distance < c.distance)
        {
            contact = Some((i, near));
        }
    }
    if let Some((solid, near)) = contact {
        settle(fly, home, solid, near.point, near.normal);
        return;
    }

    // 3. Reaching. Nothing was touched, but the player asked to hold on and
    // something is close. This is what makes landing on a shelf edge possible.
    if intent.land
        && let Some((solid, near)) = home.nearest(fly.pos, LAND_REACH)
    {
        settle(fly, home, solid, near.point, near.normal);
    }
}

fn settle(fly: &mut Fly, home: &Home, solid: usize, point: Vec3, normal: Vec3) {
    let s = &home.solids[solid];
    fly.stance = Stance::Perched(Perch {
        solid,
        local_pos: s.to_local(point),
        local_normal: s.rot.inverse() * normal,
    });
    fly.vel = Vec3::ZERO;
    fly.pos = point + normal * BODY_RADIUS;
}

// ---------------------------------------------------------------------------
// Walking
// ---------------------------------------------------------------------------

/// A forward direction lying in a surface's plane, derived from where the player
/// is looking.
///
/// The obvious version — project the view onto the plane and normalise — fails
/// exactly when it matters most. Fly head-on into a wall and the view is very
/// nearly parallel to that wall's normal, so the projection collapses towards
/// zero and normalising it returns noise rather than a direction. The player
/// presses forward, the fly crawls off somewhere arbitrary, and it reads as the
/// controls having paused.
///
/// That was always latent, but it only became the *common* case when contact
/// started meaning landing: flying straight at a surface is now the ordinary way
/// to arrive on one.
///
/// Falling back to the view's own up-vector fixes it permanently. Up is
/// perpendicular to forward by construction, so whenever forward is parallel to
/// the normal, up is guaranteed to lie in the plane. It is also what a player
/// expects: looking straight down at the floor, "forward" is whichever way the
/// top of the screen points.
fn surface_forward(fly: &Fly, normal: Vec3) -> Vec3 {
    let look = fly.heading();
    let flattened = look - normal * look.dot(normal);
    if flattened.length_squared() > FLATTEN_FLOOR {
        return flattened.normalize();
    }
    // The view frame's own up. Pitch is clamped short of vertical, so `right` is
    // never degenerate and this is always a real vector.
    let right = look.cross(Vec3::Y).normalize_or_zero();
    let look_up = right.cross(look);
    (look_up - normal * look_up.dot(normal)).normalize_or_zero()
}

fn walk_about(fly: &mut Fly, intent: &Intent, home: &Home, perch: Perch, dt: f32) {
    let solid = &home.solids[perch.solid];
    let normal = (solid.rot * perch.local_normal).normalize_or_zero();
    let footing = solid.to_world(perch.local_pos);
    fly.pos = footing + normal * BODY_RADIUS;
    fly.vel = Vec3::ZERO;

    if intent.launch {
        fly.stance = Stance::Flying;
        fly.vel = normal * TAKEOFF;
        fly.lockout = TAKEOFF_LOCKOUT;
        return;
    }

    // A basis in the surface plane, oriented by where the player is looking.
    let forward = surface_forward(fly, normal);
    if forward == Vec3::ZERO {
        return;
    }
    let right = forward.cross(normal).normalize_or_zero();

    let wish = (forward * intent.thrust.z + right * intent.thrust.x).normalize_or_zero();
    if wish == Vec3::ZERO {
        return;
    }
    let step = WALK * dt;

    // 1. Concave wrap. Something ahead to climb onto — floor into wall, or the
    //    inside of a corner. Checked before moving, so the fly turns the corner
    //    instead of burying its face in it.
    // The reach is exactly one body radius past this tick's step, so the wrap
    // fires as the fly's leading edge touches the wall and not before. Reaching
    // further makes it snap across a visible gap; reaching less lets the body
    // clip in first.
    if let Some(hit) = home.raycast(fly.pos, wish, step + BODY_RADIUS)
        && hit.normal.dot(normal) < 0.98
    {
        settle(fly, home, hit.solid, hit.point, hit.normal);
        return;
    }

    // 2. The ordinary case: move along the plane, then re-seat onto whatever is
    //    underfoot. The probe is generous enough to absorb a small step.
    let target = fly.pos + wish * step;
    let probe = BODY_RADIUS * 3.0;
    if let Some(hit) = home.raycast(target + normal * probe, -normal, probe * 2.0) {
        settle(fly, home, hit.solid, hit.point, hit.normal);
        return;
    }

    // 3. Convex wrap. Nothing underfoot means we walked off a lip, so the face we
    //    want is behind and below: cast back under the edge to find it. This is
    //    the table-edge-to-table-side move, and the ceiling-to-wall one.
    let under = target - normal * probe;
    if let Some(hit) = home.raycast(under, -wish, step + probe * 2.0) {
        settle(fly, home, hit.solid, hit.point, hit.normal);
        return;
    }

    // 4. Nothing to hold. Let go — walking off the end of a shelf should drop
    //    you, not stop you.
    fly.stance = Stance::Flying;
    fly.pos = target;
    fly.lockout = TAKEOFF_LOCKOUT * 0.5;
}

// ---------------------------------------------------------------------------
// Presentation
// ---------------------------------------------------------------------------

fn orient_the_body(fly: &mut Fly, intent: &Intent, home: &Home, dt: f32) {
    let (want_forward, want_up) = match fly.stance {
        Stance::Perched(_) => {
            // Stood on a surface: up is the surface's own normal — read from the
            // solid, not carried over from the last orientation, or the fly never
            // turns over when it lands on a ceiling.
            let n = perch_normal(fly, home).unwrap_or(Vec3::Y);
            // Same basis the legs walk on, so the body faces where it is going
            // rather than somewhere the projection happened to land.
            (surface_forward(fly, n), n)
        }
        Stance::Flying => {
            // Point mostly where the player is aiming, leaned toward where the
            // fly is actually travelling, so drift stays legible.
            //
            // Pointing purely along velocity — which is what this did first —
            // was wrong twice over. The idle sink gives a hovering fly about
            // 7 cm/s straight down, so a fly holding station aimed its head at
            // the floor; and because that direction is parallel to world up, the
            // look-at behind it degenerated and the body span. Blending against
            // cruise speed keeps the aim dominant until the drift is genuinely
            // comparable to flying.
            let aim = fly.heading();
            let mut f = (aim * CRUISE + fly.vel).normalize_or_zero();
            if f == Vec3::ZERO {
                f = aim;
            }

            // Flare. Asking to land pitches the nose up, which is the landing
            // posture of every flying thing and — now that the model is a single
            // rigid mesh with no legs to splay — the only tell the player gets
            // that the fly is reaching. It has to be visible from behind.
            if intent.land {
                let right = f.cross(Vec3::Y).normalize_or_zero();
                if right != Vec3::ZERO {
                    f = Quat::from_axis_angle(right, FLARE) * f;
                }
            }

            // Bank into the turn, in proportion to how sideways the motion is.
            let lateral = fly.vel.dot(f.cross(Vec3::Y).normalize_or_zero());
            let roll = (-lateral / CRUISE).clamp(-1.0, 1.0) * 0.5;
            let mut up = Quat::from_axis_angle(f, roll) * Vec3::Y;
            // Guard the pole: aiming straight up or down leaves up parallel to
            // forward, and `looking_to` has nothing to work with.
            if f.cross(up).length_squared() < 1e-6 {
                up = Quat::from_axis_angle(f, roll) * Vec3::Z;
            }
            (f, up)
        }
    };

    if want_forward == Vec3::ZERO {
        return;
    }
    let target = Transform::default()
        .looking_to(want_forward, want_up)
        .rotation;

    // The saccade. Big reorientations happen in one tick; small ones creep.
    let error = fly.body.angle_between(target);
    fly.body = if error > SACCADE {
        target
    } else {
        fly.body.slerp(target, (BODY_DRIFT * dt).min(1.0))
    };
}

/// The perch's current surface normal in world space, for the camera and the
/// debug readout. `None` while flying.
pub fn perch_normal(fly: &Fly, home: &Home) -> Option<Vec3> {
    let perch = fly.stance.perch()?;
    let solid = home.solids.get(perch.solid)?;
    Some((solid.rot * perch.local_normal).normalize_or_zero())
}
