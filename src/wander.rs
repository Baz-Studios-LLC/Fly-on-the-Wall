//! People moving about the house on their own.
//!
//! Not a script and not a path graph: somebody stands for a while, picks
//! somewhere in the room they are in, walks there, and stands again. That is
//! most of what anybody does at home, and it is the whole of what a fly needs
//! them to do — the interest is in a body that is somewhere else than it was,
//! not in where it decided to go.
//!
//! ## Walking at the speed of the walk
//!
//! The clip Brett exported has its travel baked in: the hip ramps a metre and a
//! half forward over two and a third seconds, which on a body scaled to a
//! hundred and seventy-eight centimetres is about one and a fifth metres a
//! second. Left alone that carries the *drawn* body across the room while the
//! entity stands still, and the first capture of it had him walk into the
//! camera.
//!
//! Cancelling it and then moving him at a speed chosen here would be the usual
//! answer, and it would be wrong twice: it throws away the animator's timing,
//! and any disagreement between the two speeds shows up as feet skating on the
//! floorboards. So the travel is *taken* instead. Each frame the hip's sideways
//! motion is read, applied to the body's place in the room, and put back. He
//! moves exactly as fast as he is animated to move, because it is the same
//! motion.
//!
//! That transfer only happens while he is walking. A standing idle also swings
//! the hips, but a standing body's feet are planted and its hips move *against*
//! them; carrying that into the room would slide him sideways while he stood
//! still.

use bevy::math::Affine3A;
use bevy::prelude::*;

use crate::folk::{Person, Repertoire};
use crate::world::Home;

/// How close is close enough to have arrived, in centimetres. A person aiming
/// at a point does not hit it.
const ARRIVED: f32 = 26.0;

/// How fast somebody turns on the spot, in radians a second.
const TURN: f32 = 2.4;

/// How far he will consider walking, and how near is not worth the trip.
const FURTHEST: f32 = 420.0;
const NEAREST: f32 = 120.0;

/// A person's own run of numbers, so what they do is the same every time the
/// house is built. Nothing here is allowed to be genuinely random: a body that
/// wanders differently on each run makes every visual regression an argument.
#[derive(Component)]
pub struct Whim(u32);

impl Whim {
    fn next(&mut self) -> f32 {
        // Numerical Recipes' constants. A person needs a few decisions a
        // minute, not a statistician's stream.
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / ((1u32 << 24) as f32)
    }
    fn between(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.next()
    }
}

/// What somebody is up to.
#[derive(Component)]
pub enum Doing {
    /// Standing, with this long left before deciding to move.
    Waiting(f32),
    /// Walking to a place in the room, with this long before giving up on it.
    Walking { to: Vec3, patience: f32 },
}

/// The bone a clip's travel is baked into.
///
/// Found by name once the skeleton is up. `rest` is where the bone sits when it
/// is not travelling, and `up` is whichever of its local axes points at the
/// ceiling — rigs converted out of a Z-up tool keep their own idea of up, and
/// on this one the hip's height is its local `z`.
#[derive(Component)]
pub struct Gait {
    hip: Entity,
    up: Vec3,
    /// Where the bone sits *across* its own up axis when it is not travelling.
    ///
    /// Flattened deliberately. It was the whole bind translation at first, and
    /// putting the bone back then read `rest + up * height`, which adds the
    /// bind height to the animated height rather than replacing it. The hip's
    /// bind height on this rig is half the body, so he walked about the room
    /// with his feet ninety centimetres off the floorboards — and stood on them
    /// perfectly whenever he stopped, because the idle never runs this.
    rest_across: Vec3,
    previous: Option<Vec3>,
}

/// Where a person's collision was built, so it can be carried to where they are
/// now instead of being rebuilt.
#[derive(Component)]
pub struct Filed {
    pub at: Affine3A,
    pub hull: usize,
}

pub struct WanderPlugin;

impl Plugin for WanderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (find_the_hip, decide, steer).chain())
            .add_systems(
                PostUpdate,
                (
                    take_the_stride.after(bevy::app::AnimationSystems),
                    carry_the_collision.after(TransformSystems::Propagate),
                )
                    .chain(),
            );
    }
}

/// Find the bone the travel is baked into, and which way is up in its frame.
fn find_the_hip(
    mut commands: Commands,
    folk: Query<Entity, (With<Person>, With<Repertoire>, Without<Gait>)>,
    children: Query<&Children>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    local: Query<&Transform>,
    placed: Query<&GlobalTransform>,
) {
    for person in &folk {
        let mut hip = None;
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
            if names
                .get(entity)
                .is_ok_and(|n| matches!(n.as_str(), "Hip" | "Hips" | "Pelvis"))
            {
                hip = Some(entity);
                break;
            }
        }
        let Some(hip) = hip else {
            continue;
        };
        let (Ok(rest), Ok(above)) = (local.get(hip), parents.get(hip)) else {
            continue;
        };
        let Ok(frame) = placed.get(above.parent()) else {
            continue;
        };
        // Whichever local axis of the parent points most nearly at the ceiling.
        let up = [Vec3::X, Vec3::Y, Vec3::Z]
            .into_iter()
            .max_by(|a, b| {
                let lift = |v: Vec3| {
                    frame
                        .affine()
                        .transform_vector3(v)
                        .normalize_or_zero()
                        .y
                        .abs()
                };
                lift(*a).total_cmp(&lift(*b))
            })
            .unwrap_or(Vec3::Y);
        info!(
            "a person strides from {:?}, whose local {up:?} is up",
            names.get(hip).map(|n| n.to_string()).unwrap_or_default()
        );
        commands.entity(person).insert((
            Gait {
                hip,
                up,
                rest_across: rest.translation - up * rest.translation.dot(up),
                previous: None,
            },
            Doing::Waiting(1.5),
            Whim(0x5eed_1e55),
        ));
    }
}

/// Stand for a while, then pick somewhere to go.
fn decide(
    clock: Res<Time>,
    home: Res<Home>,
    mut folk: Query<(
        &mut Doing,
        &mut Whim,
        &Transform,
        &Repertoire,
        &crate::folk::Plays,
    )>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (mut doing, mut whim, standing, knows, plays) in &mut folk {
        let Doing::Waiting(left) = &mut *doing else {
            continue;
        };
        *left -= clock.delta_secs();
        if *left > 0.0 {
            continue;
        }

        let from = standing.translation;
        let Some(room) = crate::house::room_at(from.xz()) else {
            *doing = Doing::Waiting(4.0);
            continue;
        };
        // Six tries and then give up until the next tick. A room with the
        // furniture pushed about can genuinely have nowhere worth standing, and
        // a body that searches until it finds one stalls the frame.
        let mut chosen = None;
        for _ in 0..6 {
            let margin = 62.0;
            let to = Vec3::new(
                whim.between(room.min.x + margin, room.max.x - margin),
                from.y,
                whim.between(room.min.y + margin, room.max.y - margin),
            );
            let step = to - from;
            let far = step.length();
            if !(NEAREST..FURTHEST).contains(&far) {
                continue;
            }
            if walkable(&home, from, to) {
                chosen = Some(to);
                break;
            }
        }
        let Some(to) = chosen else {
            *doing = Doing::Waiting(whim.between(1.0, 3.0));
            continue;
        };

        if let Some(node) = clip(knows, "walk")
            && let Ok(mut player) = players.get_mut(plays.0)
        {
            player.stop_all();
            player.play(node).repeat();
        }
        // Patience is generous: it exists to catch a body wedged on a chair
        // leg, not to hurry anybody along.
        info!(
            "somebody sets off for ({:.0}, {:.0}) in the {}",
            to.x, to.z, room.name
        );
        *doing = Doing::Walking {
            to,
            patience: 4.0 + (to - from).length() / 60.0,
        };
    }
}

/// Turn toward where he is going, and notice when he gets there.
fn steer(
    clock: Res<Time>,
    mut folk: Query<(
        &mut Doing,
        &mut Whim,
        &mut Transform,
        &Repertoire,
        &crate::folk::Plays,
    )>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (mut doing, mut whim, mut walking, knows, plays) in &mut folk {
        let Doing::Walking { to, patience } = &mut *doing else {
            continue;
        };
        *patience -= clock.delta_secs();

        let step = (*to - walking.translation).with_y(0.0);
        if step.length() < ARRIVED || *patience <= 0.0 {
            if let Some(node) = clip(knows, "idle")
                && let Ok(mut player) = players.get_mut(plays.0)
            {
                player.stop_all();
                player.play(node).repeat();
            }
            *doing = Doing::Waiting(whim.between(3.0, 11.0));
            continue;
        }

        // The model faces its own +x, so this is the turn that points that at
        // where he is going.
        let want = Quat::from_rotation_y((-step.z).atan2(step.x));
        walking.rotation = walking
            .rotation
            .rotate_towards(want, TURN * clock.delta_secs());
    }
}

/// Take the travel out of the clip and give it to the body.
fn take_the_stride(
    mut folk: Query<(&mut Transform, &mut Gait, &Doing), With<Person>>,
    mut bones: Query<&mut Transform, Without<Person>>,
    parents: Query<&ChildOf>,
    placed: Query<&GlobalTransform>,
) {
    for (mut body, mut gait, doing, ..) in &mut folk {
        let Ok(mut hip) = bones.get_mut(gait.hip) else {
            continue;
        };
        let along = hip.translation - gait.up * hip.translation.dot(gait.up);

        if matches!(doing, Doing::Walking { .. }) {
            if let Some(was) = gait.previous {
                let moved = along - was;
                // A clip that loops jumps its whole travel backwards in one
                // frame. Nothing a walking body does covers a fifth of its own
                // height between frames, so that is the tell.
                if moved.length() < 0.2
                    && let Ok(frame) = placed.get(
                        parents
                            .get(gait.hip)
                            .map(|c| c.parent())
                            .unwrap_or(gait.hip),
                    )
                {
                    body.translation += frame.affine().transform_vector3(moved);
                }
            }
            gait.previous = Some(along);
            // Put the bone back, keeping the rise and fall of the stride: that
            // is the body bobbing over its own legs, not travel.
            hip.translation = gait.rest_across + gait.up * hip.translation.dot(gait.up);
        } else {
            gait.previous = None;
        }
    }
}

/// Carry a person's collision to wherever they have got to.
fn carry_the_collision(
    mut home: ResMut<Home>,
    folk: Query<(&GlobalTransform, &Filed), With<Person>>,
) {
    for (now, filed) in &folk {
        let (_, turn, at) = now.to_scale_rotation_translation();
        // Rigid only: the scale is the model's own and is already baked into
        // the triangles, so it must not be applied twice.
        let placed = Affine3A::from_rotation_translation(turn, at);
        let shift = placed * filed.at.inverse();
        if let Some(hull) = home.hulls.get_mut(filed.hull) {
            hull.carry(shift);
        }
        let solid = home.hulls[filed.hull].belongs_to();
        home.solids[solid].center = at;
        home.solids[solid].rot = turn;
    }
}

fn clip(knows: &Repertoire, want: &str) -> Option<AnimationNodeIndex> {
    knows
        .0
        .iter()
        .find(|(name, _)| name.contains(want))
        .map(|(_, node)| *node)
}

/// Is there room to walk from here to there?
///
/// Three lines abreast at the width of a body, sampled at shin, hip and
/// shoulder. A single ray down the middle walks somebody through the arm of a
/// sofa and under a table.
fn walkable(home: &Home, from: Vec3, to: Vec3) -> bool {
    let step = (to - from).with_y(0.0);
    let far = step.length();
    if far < 1.0 {
        return false;
    }
    let ahead = step / far;
    let across = Vec3::new(-ahead.z, 0.0, ahead.x) * 22.0;
    // Start clear of the body doing the asking. Somebody's own collision is in
    // the same list as the walls, and a ray fired from inside their own chest
    // reports the room impassable in every direction — which is exactly what
    // happened: he stood still for twenty seconds looking for somewhere to go.
    const CLEAR: f32 = 36.0;
    if far < CLEAR {
        return false;
    }
    for side in [-1.0f32, 0.0, 1.0] {
        for height in [16.0f32, 92.0, 158.0] {
            let origin = from + across * side + Vec3::Y * height + ahead * CLEAR;
            if home.raycast(origin, ahead, far - CLEAR + 24.0).is_some() {
                return false;
            }
        }
    }
    true
}
