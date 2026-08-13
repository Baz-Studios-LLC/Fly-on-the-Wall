//! The house, as far as this spike is concerned: two rooms, some furniture, and
//! one door that actually moves.
//!
//! Everything solid is an oriented box. That is not a placeholder for a physics
//! engine — it is the whole collision model, and it is enough because the only
//! three questions the fly ever asks are *how far is the nearest surface*, *which
//! way does it face*, and *what does this ray hit*. A slab raycast and a
//! closest-point clamp answer all three in about sixty lines, and owning them
//! means the landing code can ask for exactly the probe it wants instead of
//! negotiating with someone else's character controller.
//!
//! **Two rooms that differ in air, not in floor plan.** A pair of empty cubes
//! would teach nothing. The living room is tall and mostly empty — long
//! uninterrupted flight lines, a handful of low obstacles. The kitchen is small
//! and stuffed with hard edges, overhangs and undersides. If the flight model
//! feels right in both, that is a real result; if it only feels right in one,
//! that is a more useful result still.

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Scale
// ---------------------------------------------------------------------------

/// **One world unit is one centimetre.**
///
/// This is the decision the whole project inherits, so it is worth stating why.
/// The alternative — metres, as in every other game here — puts the fly at 0.007
/// units and its collision radius at 0.003, which is where `f32` collision
/// margins, near planes and normalisation tolerances all start behaving badly at
/// once. Centimetres put the fly at just under one unit, which is the range every
/// default in the engine was tuned for, and they keep the numbers mentally cheap:
/// a crumb is 3 units, a countertop is 90, a room is 500.
///
/// Rooms are still *authored* in metres through [`m`], because that is how anyone
/// thinks about a house.
pub const UNITS_PER_METRE: f32 = 100.0;

/// Metres to world units, for authoring. `m(2.4)` is a ceiling.
pub const fn m(metres: f32) -> f32 {
    metres * UNITS_PER_METRE
}

// ---------------------------------------------------------------------------
// Solids
// ---------------------------------------------------------------------------

/// What a surface is made of.
///
/// Nothing here reads this yet except the greybox tint. It exists because the
/// list of things a fly cares about — is it warm, is it sticky, does it smell of
/// food, will it hold my feet — is going to hang off exactly this enum, and
/// putting it in now costs one field.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stuff {
    Plaster,
    Grass,
    Floorboard,
    Wood,
    Fabric,
    Metal,
    Stone,
    Glass,
}

impl Stuff {
    /// Greybox tint. Muted and low-contrast on purpose: at fly scale the eye
    /// needs edges and shading to read the space, and saturated colour just makes
    /// the geometry harder to parse.
    fn tint(self) -> Color {
        match self {
            Stuff::Plaster => Color::srgb(0.82, 0.80, 0.76),
            Stuff::Floorboard => Color::srgb(0.45, 0.33, 0.22),
            Stuff::Wood => Color::srgb(0.58, 0.42, 0.27),
            Stuff::Fabric => Color::srgb(0.37, 0.36, 0.42),
            Stuff::Grass => Color::srgb(0.30, 0.40, 0.22),
            Stuff::Metal => Color::srgb(0.66, 0.68, 0.71),
            Stuff::Stone => Color::srgb(0.30, 0.30, 0.32),
            Stuff::Glass => Color::srgba(0.78, 0.87, 0.92, 0.13),
        }
    }

    fn perceptual_roughness(self) -> f32 {
        match self {
            Stuff::Metal => 0.35,
            Stuff::Glass => 0.05,
            Stuff::Stone => 0.55,
            Stuff::Fabric => 0.95,
            Stuff::Grass => 0.95,
            _ => 0.8,
        }
    }

    /// Glass is drawn see-through and, more importantly, is not allowed to cast
    /// a shadow — a pane that shadowed would close the window it fills.
    fn is_glass(self) -> bool {
        matches!(self, Stuff::Glass)
    }
}

/// One box. `rot` is identity for everything except the door.
#[derive(Clone, Debug)]
pub struct Solid {
    pub center: Vec3,
    pub half: Vec3,
    pub rot: Quat,
    pub stuff: Stuff,
    /// A colour from outside, overriding [`Stuff::tint`].
    ///
    /// A hand-authored solid says what it is *made of* and takes its greybox
    /// colour from that. A solid that came out of Opificium has already been
    /// painted by someone with an eye, and second-guessing that from a material
    /// enum would be throwing away the better answer.
    pub paint: Option<Color>,
    /// How much this surface emits. A ceiling fixture lit only by the lamp
    /// inside it is a grey lump: the lamp points down and away from the thing
    /// it is supposed to be shining out of.
    pub glow: f32,
    /// Drawn see-through and excluded from shadow casting: glass.
    pub sheer: bool,
    /// Overhead: the roof, or a ceiling under it. Hidden in the plan view,
    /// which is the only way to see a drawn house's rooms — from above, a house
    /// is a roof, and once it has a ceiling it is a ceiling.
    pub roof: bool,
}

/// The nearest point on a solid to some query point.
///
/// `distance` is negative when the query point is *inside* the box, which is how
/// the flight collision pass detects that it has integrated through a wall.
#[derive(Clone, Copy, Debug)]
pub struct Near {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

/// A ray hit.
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub solid: usize,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

impl Solid {
    /// An axis-aligned solid from its bounds. This is the authoring form —
    /// reading a room off a list of min/max corners is far easier to check than
    /// reading it off centres and half-extents.
    pub fn between(min: Vec3, max: Vec3, stuff: Stuff) -> Self {
        Solid {
            center: (min + max) * 0.5,
            half: (max - min) * 0.5,
            rot: Quat::IDENTITY,
            sheer: stuff.is_glass(),
            paint: None,
            roof: false,
            glow: 0.0,
            stuff,
        }
    }

    pub fn to_local(&self, p: Vec3) -> Vec3 {
        self.rot.inverse() * (p - self.center)
    }

    pub fn to_world(&self, local: Vec3) -> Vec3 {
        self.center + self.rot * local
    }

    /// Rotate a direction out of the solid's frame into the world's.
    pub fn dir_to_world(&self, local: Vec3) -> Vec3 {
        self.rot * local
    }

    /// Closest point on the box surface, its outward normal, and the distance to
    /// it.
    ///
    /// The inside case matters more than it looks: it is what a landing probe
    /// gets when the fly has clipped a corner, and answering it with the *least
    /// penetrated* face is what makes the recovery push the fly back out the way
    /// it came instead of squirting it through the wall.
    pub fn nearest(&self, p: Vec3) -> Near {
        let local = self.to_local(p);
        let clamped = local.clamp(-self.half, self.half);
        let outside = local - clamped;

        if outside.length_squared() > 1e-12 {
            let distance = outside.length();
            return Near {
                point: self.to_world(clamped),
                normal: self.dir_to_world(outside / distance),
                distance,
            };
        }

        // Inside. Find the face we are least deep behind.
        let gap = self.half - local.abs();
        let axis = if gap.x <= gap.y && gap.x <= gap.z {
            0
        } else if gap.y <= gap.z {
            1
        } else {
            2
        };
        let sign = if local[axis] >= 0.0 { 1.0 } else { -1.0 };

        let mut surface = local;
        surface[axis] = self.half[axis] * sign;
        let mut normal = Vec3::ZERO;
        normal[axis] = sign;

        Near {
            point: self.to_world(surface),
            normal: self.dir_to_world(normal),
            distance: -gap[axis],
        }
    }

    /// Slab test. Returns the entry point only — a ray that starts inside the box
    /// misses, which is what every probe in `fly.rs` wants, since they all start
    /// just off a surface and reach outward.
    pub fn raycast(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<(f32, Vec3)> {
        let o = self.to_local(origin);
        let d = self.rot.inverse() * dir;

        let mut t_min = 0.0f32;
        let mut t_max = max;
        let mut axis = 0usize;
        let mut sign = 1.0f32;

        for i in 0..3 {
            if d[i].abs() < 1e-8 {
                // Parallel to this slab: a miss unless we already lie between its
                // planes.
                if o[i] < -self.half[i] || o[i] > self.half[i] {
                    return None;
                }
                continue;
            }
            let inv = 1.0 / d[i];
            let mut t1 = (-self.half[i] - o[i]) * inv;
            let mut t2 = (self.half[i] - o[i]) * inv;
            let mut face = -1.0;
            if t1 > t2 {
                core::mem::swap(&mut t1, &mut t2);
                face = 1.0;
            }
            if t1 > t_min {
                t_min = t1;
                axis = i;
                sign = face;
            }
            t_max = t_max.min(t2);
            if t_min > t_max {
                return None;
            }
        }

        if t_min <= 0.0 {
            return None;
        }
        let mut normal = Vec3::ZERO;
        normal[axis] = sign;
        Some((t_min, self.dir_to_world(normal)))
    }
}

// ---------------------------------------------------------------------------
// The house
// ---------------------------------------------------------------------------

/// Where the house on screen came from.
///
/// Three, not two, since the procedural house arrived: it is the goal and so it
/// is the default, the greybox is the movement test kept for its known
/// dimensions and its pass test, and a drawn house is reference behaviour that
/// is no longer what anybody boots into.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Origin {
    Procedural,
    Greybox,
    Drawn,
}

/// Every solid in the world, plus the index of the one that swings.
#[derive(Resource)]
pub struct Home {
    pub solids: Vec<Solid>,
    /// Index into `solids` of the swinging door panel, when there is one. Kept
    /// so the door can be rebuilt in place each tick without a search, and so a
    /// perch can tell it is riding it.
    ///
    /// `None` for an imported house: the bench bakes a doorway as an *absence*
    /// of boxes, so an opening has no leaf to swing. Giving imported openings
    /// real doors is the one thing this game needs from the bench that the
    /// village never did — see the note in `blueprint.rs`.
    pub door: Option<usize>,
    /// Where a new fly starts. A property of the house rather than a constant,
    /// now that the house can arrive from a file.
    pub spawn: Vec3,
}

impl Home {
    /// Nearest surface across the whole house, ignoring anything further than
    /// `within`. Linear over ~30 boxes, which at 64 Hz is nothing; when the house
    /// grows past a few hundred this wants a grid, and not before.
    pub fn nearest(&self, p: Vec3, within: f32) -> Option<(usize, Near)> {
        let mut best: Option<(usize, Near)> = None;
        for (i, solid) in self.solids.iter().enumerate() {
            let near = solid.nearest(p);
            if near.distance > within {
                continue;
            }
            if best.is_none_or(|(_, b)| near.distance < b.distance) {
                best = Some((i, near));
            }
        }
        best
    }

    pub fn raycast(&self, origin: Vec3, dir: Vec3, max: f32) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        for (i, solid) in self.solids.iter().enumerate() {
            let Some((distance, normal)) = solid.raycast(origin, dir, max) else {
                continue;
            };
            if best.is_none_or(|b| distance < b.distance) {
                best = Some(Hit {
                    solid: i,
                    distance,
                    point: origin + dir * distance,
                    normal,
                });
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------
// The door
// ---------------------------------------------------------------------------

/// The doorway's clear opening, in world coordinates. The door panel fills this
/// when closed.
const DOOR_WIDTH: f32 = m(0.8);
const DOOR_HEIGHT: f32 = m(2.0);
const DOOR_THICKNESS: f32 = 4.0;

/// The gap under a closed door — a real one is about this, and it is the reason
/// walking is a traversal verb here and not an idle pose. The fly's body is a
/// little over half a centimetre, so this is passable on foot and only on foot.
pub const UNDER_GAP: f32 = 1.2;

/// Where the hinge is: the low-`z` jamb, in the middle of the wall's thickness.
const HINGE: Vec3 = Vec3::new(510.0, UNDER_GAP + (DOOR_HEIGHT - UNDER_GAP) * 0.5, 60.0);

/// How far open "open" is. Nearly flat against the kitchen wall.
const OPEN_ANGLE: f32 = 85.0_f32.to_radians();

/// How fast the door swings, in radians per second. Slow enough that a fly can be
/// standing on it while it moves, which is the point of animating it at all.
const SWING_RATE: f32 = 1.6;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DoorWant {
    Open,
    /// The interesting one. A slot to thread rather than a hole to fly through.
    Ajar,
    #[default]
    Closed,
}

#[derive(Resource)]
pub struct Door {
    pub want: DoorWant,
    /// Current angle, radians. Lerped toward the target so the panel actually
    /// swings.
    pub angle: f32,
    /// The ajar angle, live-tunable with `[` and `]`.
    ///
    /// This is the single most important number in the spike, and the obvious
    /// way to reason about it is wrong. A hinged door spans its whole opening at
    /// every angle — swinging it does not uncover a strip of doorway the way a
    /// sliding one would. What opens is the slot between the door's *free edge*
    /// and the far jamb, and that slot is `DOOR_WIDTH * (1 - cos)`, which is
    /// quadratic in the angle and therefore tiny for a long time. At 2°, a door
    /// that looks visibly ajar has a four-tenths-of-a-millimetre gap and a fly
    /// cannot get through it.
    ///
    /// Ten degrees puts the slot at about 1.2 cm — two fly-lengths — and also
    /// swings the free edge just clear of the wall's thickness, so both possible
    /// constrictions open at roughly the same moment. That coincidence is what
    /// makes it a threadable gap rather than an arbitrary one.
    pub ajar_angle: f32,
}

impl Default for Door {
    fn default() -> Self {
        Door {
            want: DoorWant::Closed,
            angle: 0.0,
            ajar_angle: 10.0_f32.to_radians(),
        }
    }
}

impl Door {
    fn target(&self) -> f32 {
        match self.want {
            DoorWant::Open => OPEN_ANGLE,
            DoorWant::Ajar => self.ajar_angle,
            DoorWant::Closed => 0.0,
        }
    }

    /// The clear slot between the door's free edge and the far jamb, in
    /// centimetres — the thing the fly actually has to fit through.
    pub fn gap(&self) -> f32 {
        DOOR_WIDTH * (1.0 - self.angle.cos())
    }

    /// The panel as a solid at the current angle.
    fn panel(&self) -> Solid {
        let rot = Quat::from_rotation_y(self.angle);
        Solid {
            center: HINGE + rot * Vec3::new(0.0, 0.0, DOOR_WIDTH * 0.5),
            half: Vec3::new(
                DOOR_THICKNESS * 0.5,
                (DOOR_HEIGHT - UNDER_GAP) * 0.5,
                DOOR_WIDTH * 0.5,
            ),
            rot,
            glow: 0.0,
            stuff: Stuff::Wood,
            paint: None,
            sheer: false,
            roof: false,
        }
    }
}

/// Marker for the door's rendered mesh, so its transform can follow the solid.
#[derive(Component)]
struct DoorPanel;

// ---------------------------------------------------------------------------
// Building it
// ---------------------------------------------------------------------------

/// Wall and ceiling thickness. Only matters where the fly can get behind
/// something, which — this being a house seen by a fly — is more often than you
/// would think.
const T: f32 = 20.0;

/// The living room window, in the west wall. Sill at knee height, head just
/// under the lintel, and wide enough that the shaft it throws crosses most of
/// the floor in the afternoon.
const WINDOW_SILL: f32 = 90.0;
const WINDOW_HEAD: f32 = 205.0;
const WINDOW_NEAR: f32 = 130.0;
const WINDOW_FAR: f32 = 300.0;

fn build_home() -> Home {
    use Stuff::*;
    let mut s: Vec<Solid> = Vec::new();
    let mut add = |min: Vec3, max: Vec3, stuff: Stuff| s.push(Solid::between(min, max, stuff));

    // -- Shell -------------------------------------------------------------
    // One continuous floor under both rooms.
    add(
        Vec3::new(-T, -T, -T),
        Vec3::new(m(8.4), 0.0, m(4.2)),
        Floorboard,
    );

    // Living room: 5 m x 4 m, 2.7 m to the ceiling. Tall and mostly empty.
    add(
        Vec3::new(0.0, m(2.7), 0.0),
        Vec3::new(m(5.0), m(2.7) + T, m(4.0)),
        Plaster,
    );
    // West wall, in four pieces around a window.
    //
    // The window is not scenery. Sealed rooms cannot be lit: a directional light
    // outside a closed box contributes exactly nothing indoors, which left the
    // whole house on flat ambient with no shading anywhere. A hole in a wall is
    // the fix, and it happens to be the best-looking thing in the build — a fly
    // crossing a shaft of afternoon sun is most of the reason to be a fly.
    add(
        Vec3::new(-T, 0.0, -T),
        Vec3::new(0.0, WINDOW_SILL, m(4.2)),
        Plaster,
    ); // under the sill
    add(
        Vec3::new(-T, WINDOW_HEAD, -T),
        Vec3::new(0.0, m(2.9), m(4.2)),
        Plaster,
    ); // over the head
    add(
        Vec3::new(-T, WINDOW_SILL, -T),
        Vec3::new(0.0, WINDOW_HEAD, WINDOW_NEAR),
        Plaster,
    );
    add(
        Vec3::new(-T, WINDOW_SILL, WINDOW_FAR),
        Vec3::new(0.0, WINDOW_HEAD, m(4.2)),
        Plaster,
    );

    // The pane itself. It is a real solid, so the fly can land on it and bump
    // along it, which is the single most recognisable thing a housefly does. It
    // is excluded from shadow casting in `dress_the_set`, or it would block the
    // very light the window exists to admit.
    add(
        Vec3::new(-11.0, WINDOW_SILL, WINDOW_NEAR),
        Vec3::new(-9.0, WINDOW_HEAD, WINDOW_FAR),
        Glass,
    );
    add(
        Vec3::new(-T, 0.0, -T),
        Vec3::new(m(8.4), m(2.9), 0.0),
        Plaster,
    ); // north, shared
    add(
        Vec3::new(-T, 0.0, m(4.0)),
        Vec3::new(m(5.2), m(2.9), m(4.2)),
        Plaster,
    ); // south

    // The wall between the rooms, in three pieces around the opening. The
    // opening runs z 60..140 and stops at 2 m; above it is a lintel.
    add(
        Vec3::new(m(5.0), 0.0, -T),
        Vec3::new(m(5.2), m(2.9), 60.0),
        Plaster,
    );
    add(
        Vec3::new(m(5.0), 0.0, 140.0),
        Vec3::new(m(5.2), m(2.9), m(4.2)),
        Plaster,
    );
    add(
        Vec3::new(m(5.0), DOOR_HEIGHT, 60.0),
        Vec3::new(m(5.2), m(2.9), 140.0),
        Plaster,
    );

    // Kitchen: 3 m x 2.5 m, 2.4 m ceiling. Small, hard, and full of edges.
    add(
        Vec3::new(m(5.2), m(2.4), 0.0),
        Vec3::new(m(8.2), m(2.4) + T, m(2.5)),
        Plaster,
    );
    add(
        Vec3::new(m(8.2), 0.0, -T),
        Vec3::new(m(8.4), m(2.6), m(2.7)),
        Plaster,
    ); // east
    add(
        Vec3::new(m(5.2), 0.0, m(2.5)),
        Vec3::new(m(8.4), m(2.6), m(2.7)),
        Plaster,
    ); // south

    // -- Living room: few obstacles, all low, long lines above them ---------
    add(
        Vec3::new(60.0, 0.0, 300.0),
        Vec3::new(260.0, 45.0, 390.0),
        Fabric,
    ); // sofa seat
    add(
        Vec3::new(60.0, 45.0, 362.0),
        Vec3::new(260.0, 88.0, 390.0),
        Fabric,
    ); // sofa back

    // Coffee table. The underside of the top is the first inverted landing most
    // players will try, so it is deliberately reachable from a standing start.
    add(
        Vec3::new(120.0, 38.0, 180.0),
        Vec3::new(280.0, 42.0, 265.0),
        Wood,
    );
    for (x, z) in [
        (126.0, 186.0),
        (268.0, 186.0),
        (126.0, 253.0),
        (268.0, 253.0),
    ] {
        add(
            Vec3::new(x, 0.0, z),
            Vec3::new(x + 6.0, 38.0, z + 6.0),
            Wood,
        );
    }

    // A bookcase: the only tall thing in the room, so there is something to climb
    // that is not a wall. It stands against the north wall rather than the west
    // one, where it used to sit directly in front of the window and block the
    // only real light in the house.
    add(
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(470.0, 180.0, 28.0),
        Wood,
    );
    for y in [45.0, 90.0, 135.0] {
        add(
            Vec3::new(300.0, y, 0.0),
            Vec3::new(470.0, y + 4.0, 34.0),
            Wood,
        ); // shelves, proud of the case
    }

    // -- Kitchen: overhangs, undersides, and a gap to get lost behind -------
    add(
        Vec3::new(m(5.4), 0.0, 0.0),
        Vec3::new(m(8.0), 90.0, 60.0),
        Wood,
    ); // counter body
    add(
        Vec3::new(m(5.35), 90.0, 0.0),
        Vec3::new(m(8.05), 94.0, 64.0),
        Stone,
    ); // countertop, overhanging

    // Wall cabinets. The underside at 150 is the best inverted perch in the
    // house and the natural target for the pass test.
    add(
        Vec3::new(m(5.6), 150.0, 0.0),
        Vec3::new(m(7.8), 230.0, 35.0),
        Wood,
    );

    // Fridge, standing off the wall. The 15 cm behind it is unreachable in
    // flight and trivial on foot, which is the whole argument for walking.
    add(
        Vec3::new(m(5.4), 0.0, 170.0),
        Vec3::new(m(6.0), 170.0, 220.0),
        Metal,
    );

    // A floating shelf, and a small table with an overhang.
    add(
        Vec3::new(640.0, 120.0, 215.0),
        Vec3::new(760.0, 126.0, 245.0),
        Wood,
    );
    add(
        Vec3::new(620.0, 0.0, 100.0),
        Vec3::new(760.0, 72.0, 160.0),
        Wood,
    );
    add(
        Vec3::new(612.0, 72.0, 92.0),
        Vec3::new(768.0, 76.0, 168.0),
        Wood,
    );

    // The door goes last so its index is known and stable.
    let door = s.len();
    s.push(Door::default().panel());

    Home {
        solids: s,
        door: Some(door),
        spawn: SPAWN,
    }
}

/// Where a new fly starts: hanging under the living room ceiling, at the far end
/// from the door.
///
/// Starting *perched and inverted* rather than hovering in mid-air is a small
/// decision with a large effect — the first thing the player does is fall off a
/// ceiling, which teaches takeoff, gravity and the camera's roll convention in
/// about a second and a half without a word of instruction.
pub const SPAWN: Vec3 = Vec3::new(120.0, m(2.7) - 5.0, 320.0);

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        // The procedural house unless something else is asked for by name. It
        // is the one being built, so it is the one that boots; `FLY_HOUSE=greybox`
        // gets the two-room movement test back, and `FLY_HOUSE=<name>` loads a
        // house drawn in Opificium, which is reference rather than the goal.
        let (home, origin) = match crate::blueprint::requested() {
            Some(name) if name == "greybox" => (build_home(), Origin::Greybox),
            Some(name) => match crate::blueprint::load(&name) {
                Ok(imported) => {
                    app.insert_resource(crate::blueprint::Marks(imported.marks));
                    (imported.home, Origin::Drawn)
                }
                Err(why) => {
                    error!("{why} — falling back to the procedural house");
                    (crate::house::build(), Origin::Procedural)
                }
            },
            None => (crate::house::build(), Origin::Procedural),
        };
        if origin == Origin::Procedural {
            crate::house::audit(&home);
        }

        app.insert_resource(origin)
            .insert_resource(home)
            .init_resource::<Door>()
            .add_systems(Startup, dress_the_set)
            // The door is written in `FixedUpdate` before the fly moves, so a fly
            // perched on it reads a panel that has already taken this tick's
            // swing rather than last tick's.
            .add_systems(FixedUpdate, swing_the_door.before(crate::fly::step_the_fly))
            .add_systems(Update, (choose_door_state, follow_the_door));
    }
}

/// Every solid in the house is a box, and a box is a unit cube with a scale on
/// it. Handing each one its own `Cuboid` mesh and its own material — which is
/// what this did first — cost 1559 meshes and 1549 materials for a house whose
/// entire vocabulary is one shape and about thirty colours, and it defeats
/// batching outright: the renderer groups draws by material, so a house where
/// no two boards share one is a house with no two boards in the same draw.
///
/// One cube, and a palette keyed by how a surface *looks*. Colour is quantised
/// to eight bits a channel before it becomes a key, which costs nothing visible
/// and means the deterministic grain on two neighbouring floorboards collapses
/// to one entry when it lands on the same byte.
fn dress_the_set(
    mut commands: Commands,
    home: Res<Home>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::from_length(1.0));
    let mut palette: std::collections::HashMap<[u8; 6], Handle<StandardMaterial>> =
        std::collections::HashMap::new();

    for (i, solid) in home.solids.iter().enumerate() {
        let colour = solid.paint.unwrap_or_else(|| solid.stuff.tint());
        let rgba = colour.to_linear();
        let byte = |c: f32| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
        let key = [
            byte(rgba.red),
            byte(rgba.green),
            byte(rgba.blue),
            byte(rgba.alpha),
            solid.stuff as u8,
            solid.sheer as u8 | (byte(solid.glow / 24.0) << 1),
        ];
        let material = palette
            .entry(key)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: colour,
                    emissive: (rgba * solid.glow).with_alpha(1.0),
                    perceptual_roughness: solid.stuff.perceptual_roughness(),
                    metallic: if solid.stuff == Stuff::Metal {
                        0.6
                    } else {
                        0.0
                    },
                    alpha_mode: if solid.sheer {
                        AlphaMode::Blend
                    } else {
                        AlphaMode::Opaque
                    },
                    ..default()
                })
            })
            .clone();

        let mut entity = commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(solid.center)
                .with_rotation(solid.rot)
                .with_scale(solid.half * 2.0),
        ));
        if home.door == Some(i) {
            entity.insert(DoorPanel);
        }
        if solid.sheer {
            entity.insert(bevy::light::NotShadowCaster);
        }
        // The plan view looks straight down, so the roof is all it would ever
        // see. It stays solid — only the drawing of it goes.
        if solid.roof && crate::camera::plan_view() {
            entity.insert(Visibility::Hidden);
        }
    }
}

fn choose_door_state(keys: Res<ButtonInput<KeyCode>>, mut door: ResMut<Door>) {
    if keys.just_pressed(KeyCode::KeyE) {
        door.want = match door.want {
            DoorWant::Closed => DoorWant::Ajar,
            DoorWant::Ajar => DoorWant::Open,
            DoorWant::Open => DoorWant::Closed,
        };
    }
    // Nudge the ajar angle while standing in it. Half a degree a press, which
    // near the default is a little over a millimetre of slot.
    let nudge = 0.5_f32.to_radians();
    let limit = 45.0_f32.to_radians();
    if keys.just_pressed(KeyCode::BracketRight) {
        door.ajar_angle = (door.ajar_angle + nudge).min(limit);
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        door.ajar_angle = (door.ajar_angle - nudge).max(0.0);
    }
}

fn swing_the_door(time: Res<Time<Fixed>>, mut door: ResMut<Door>, mut home: ResMut<Home>) {
    let Some(index) = home.door else {
        return;
    };
    let target = door.target();
    if (door.angle - target).abs() > 1e-5 {
        let step = SWING_RATE * time.delta_secs();
        door.angle += (target - door.angle).clamp(-step, step);
        home.solids[index] = door.panel();
    }
}

fn follow_the_door(home: Res<Home>, mut panels: Query<&mut Transform, With<DoorPanel>>) {
    let Some(index) = home.door else {
        return;
    };
    let solid = &home.solids[index];
    for mut transform in &mut panels {
        transform.translation = solid.center;
        transform.rotation = solid.rot;
    }
}
