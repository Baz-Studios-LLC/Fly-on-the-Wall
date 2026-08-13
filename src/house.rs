//! The house, generated.
//!
//! Not drawn in a bench and not typed as a list of boxes: described as a plan of
//! rooms and built from it by [`wall_run`], which is the only primitive that
//! matters. A wall knows its own openings, so a doorway is a *hole in a run*
//! rather than four hand-placed pieces to be kept in step every time a room
//! moves.
//!
//! **The plan is a real one.** It is the three-bedroom ranch Brett posted:
//! bedrooms and a bath down the left, a hall, an open kitchen-and-great-room
//! through the middle, laundry and the main suite down the right, and a two-car
//! garage on the end. Every dimension below is authored in **feet, off that
//! drawing**, so the two can be compared line by line.
//!
//! **Then it is scaled, once.** No habitable room may be under fifteen feet in
//! either direction, and the drawing's bedrooms are eleven foot six. Rather than
//! distort the plan — stretching one room and leaving the rest — the whole house
//! is multiplied by [`SCALE`], which is exactly `15 / 11.5`. Every proportion in
//! the drawing survives, the tightest bedroom lands on the minimum precisely,
//! and the house becomes 88'-8" by 45'.
//!
//! Habitable means living space: the great room, the kitchen and the three
//! bedrooms. A bathroom, laundry, closet, hall and garage are service space and
//! are not held to it — which is the ordinary reading of the word, and the only
//! one under which this plan survives at all, its laundry being six foot eight
//! deep.
//!
//! Nothing is trusted: [`audit`] measures the built result on every run.

use bevy::prelude::*;

use crate::world::{Home, Solid, Stuff};

// ---------------------------------------------------------------------------
// The laws, and the scale that satisfies them
// ---------------------------------------------------------------------------

/// Nine feet, finished floor to finished ceiling.
pub const CEILING: f32 = 274.32;

/// Fifteen feet: the least clear interior floor a habitable room may offer in
/// either direction.
pub const MIN_ROOM: f32 = 457.2;

/// How far under the minimum still counts as meeting it: half a millimetre.
///
/// `11.5 * (15 / 11.5) * 30.48` is 457.2 in arithmetic and 457.19998 in `f32`,
/// and a law enforced without a tolerance fails on its own scale factor. Half a
/// millimetre is far below anything a fly could find and far above the noise.
const HAIR: f32 = 0.05;

/// Centimetres per foot.
const FOOT: f32 = 30.48;

/// What the drawing is multiplied by so its tightest bedroom — eleven foot six —
/// reaches the fifteen-foot minimum. Applied to the whole plan, so nothing is
/// distorted relative to anything else.
const SCALE: f32 = 15.0 / 11.5;

/// Feet on the drawing to centimetres in the world.
fn ft(feet: f32) -> f32 {
    feet * SCALE * FOOT
}

const OUTER: f32 = 20.0;
const INNER: f32 = 12.0;
const SLAB: f32 = 12.0;
/// How far a wall carries on below the floor, so no room can open a slot at
/// its skirting by sitting a few centimetres lower than its neighbour.
const FOOTING: f32 = 30.0;
/// The tallest a piece of groundwork gets before it counts as building.
pub const STEP_HIGH: f32 = 26.0;

/// Skirting: how tall, and how far proud of the plaster it stands.
///
/// Two centimetres of relief is nothing to a person and a landable ledge running
/// the entire perimeter of every room to a fly — which is the whole argument for
/// modelling trim in a game seen from six millimetres up.
const SKIRT_HIGH: f32 = 9.0;
const SKIRT_PROUD: f32 = 2.0;

/// Cornice, at the other end of the wall. Same argument as the skirting: it
/// breaks an otherwise unbroken expanse of plaster where the wall meets the
/// ceiling, and it is a ledge running the perimeter of every room at the exact
/// height a fly wants to sit and watch from.
const CORNICE_HIGH: f32 = 7.0;
const CORNICE_PROUD: f32 = 3.0;
/// The furthest anything nailed to a wall may stand out from it: cornice,
/// skirting, cladding, window casing, a sill. The envelope law allows exactly
/// this much and no more, so a sill is trim and a shelf is an escapee.
pub const TRIM_PROUD: f32 = 20.0;

const DOOR_WIDE: f32 = 92.0;
const DOOR_HIGH: f32 = 205.0;
const SILL: f32 = 100.0;
const HEAD: f32 = 215.0;
const WINDOW_WIDE: f32 = 150.0;

// ---------------------------------------------------------------------------
// The plan, in feet
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Use {
    Living,
    Kitchen,
    Bed,
    Bath,
    Utility,
    Hall,
    Garage,
}

impl Use {
    /// Living space, and so held to the fifteen-foot minimum.
    pub fn habitable(self) -> bool {
        matches!(self, Use::Living | Use::Kitchen | Use::Bed)
    }
}

pub struct Room {
    pub name: &'static str,
    pub use_for: Use,
    /// Clear interior bounds in world centimetres, between finished surfaces.
    pub min: Vec2,
    pub max: Vec2,
}

impl Room {
    pub fn middle(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
    pub fn wide(&self) -> f32 {
        self.max.x - self.min.x
    }
    pub fn deep(&self) -> f32 {
        self.max.y - self.min.y
    }
}

/// The drawing: `(name, use, x0, z0, x1, z1)` in **feet**.
const PLAN: [(&str, Use, f32, f32, f32, f32); 10] = [
    // Left column: two bedrooms with the hall bath between them.
    ("bedroom three", Use::Bed, 0.0, 0.0, 12.0, 11.5),
    ("bathroom", Use::Bath, 0.0, 11.5, 12.0, 23.0),
    ("bedroom two", Use::Bed, 0.0, 23.0, 12.0, 34.5),
    // The hall, running the depth of the house.
    ("hall", Use::Hall, 12.0, 0.0, 16.0, 34.5),
    // The middle: one open volume, kitchen at the back, great room in front.
    ("kitchen", Use::Kitchen, 16.0, 0.0, 33.67, 14.0),
    ("great room", Use::Living, 16.0, 14.0, 33.67, 34.5),
    // Right column: laundry, the main suite's bath, the main bedroom.
    ("laundry", Use::Utility, 33.67, 0.0, 46.0, 6.67),
    ("main bath", Use::Bath, 33.67, 6.67, 46.0, 22.0),
    ("main bedroom", Use::Bed, 33.67, 22.0, 46.0, 34.5),
    // And the garage on the end.
    ("garage", Use::Garage, 46.0, 0.0, 68.0, 24.0),
];

pub fn rooms() -> Vec<Room> {
    PLAN.iter()
        .map(|&(name, use_for, x0, z0, x1, z1)| Room {
            name,
            use_for,
            min: Vec2::new(ft(x0), ft(z0)),
            max: Vec2::new(ft(x1), ft(z1)),
        })
        .collect()
}

pub fn room(named: &str) -> Room {
    rooms()
        .into_iter()
        .find(|r| r.name == named)
        .expect("a room by that name")
}

// ---------------------------------------------------------------------------
// Walls
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub struct Opening {
    pub at: f32,
    pub wide: f32,
    pub sill: f32,
    pub head: f32,
}

impl Opening {
    fn door(at: f32) -> Self {
        Opening {
            at,
            wide: DOOR_WIDE,
            sill: 0.0,
            head: DOOR_HIGH,
        }
    }
    fn window(at: f32) -> Self {
        Opening {
            at,
            wide: WINDOW_WIDE,
            sill: SILL,
            head: HEAD,
        }
    }
    fn cased(at: f32, wide: f32) -> Self {
        Opening {
            at,
            wide,
            sill: 0.0,
            head: 230.0,
        }
    }
}

/// One straight run of wall, with holes in it.
///
/// `a` and `b` are the run's *centreline* ends. The run is axis-aligned and
/// which axis is read off whichever end differs. Openings are measured along the
/// run from `a`.
fn wall_run(
    out: &mut Vec<Solid>,
    a: Vec2,
    b: Vec2,
    thick: f32,
    height: f32,
    stuff: Stuff,
    holes: &[Opening],
) {
    let along_x = (b.x - a.x).abs() >= (b.y - a.y).abs();
    let length = if along_x { b.x - a.x } else { b.y - a.y };
    let half = thick * 0.5;

    // A footing under the whole run, openings and all, from below the grass up
    // to the house's floor level.
    //
    // Every wall used to start at zero, which is right for every room whose
    // floor is also at zero — and wrong for the garage, whose slab is a step
    // down. That six-centimetre difference was a six-centimetre slot running
    // right around the garage with daylight and grass showing through it. A
    // stem wall is what a real slab sits against, and above the garage floor it
    // reads as exactly that.
    {
        let (min, max) = if along_x {
            (
                Vec3::new(a.x.min(b.x), -FOOTING, a.y - half),
                Vec3::new(a.x.max(b.x), 0.0, a.y + half),
            )
        } else {
            (
                Vec3::new(a.x - half, -FOOTING, a.y.min(b.y)),
                Vec3::new(a.x + half, 0.0, a.y.max(b.y)),
            )
        };
        out.push(Solid::between(min, max, Stuff::Stone));
    }

    let piece = |s: f32, e: f32, low: f32, high: f32, out: &mut Vec<Solid>| {
        if e - s < 0.01 || high - low < 0.01 {
            return;
        }
        let (min, max) = if along_x {
            (
                Vec3::new(a.x + s, low, a.y - half),
                Vec3::new(a.x + e, high, a.y + half),
            )
        } else {
            (
                Vec3::new(a.x - half, low, a.y + s),
                Vec3::new(a.x + half, high, a.y + e),
            )
        };
        out.push(Solid::between(min, max, stuff));

        // Lap siding, on whichever face of an exterior wall looks outward.
        //
        // The whole building was flat cream plaster. Boards are the cheapest
        // thing that puts a scale on an elevation: they tell you how big the
        // house is before you have looked at anything else in the frame. Only
        // exterior walls get them, and "exterior" is not a flag anyone has to
        // remember to set — it is a probe thirty centimetres off each face,
        // asking whether that is still indoors.
        if thick >= OUTER - 0.01 {
            for face in [-1.0f32, 1.0] {
                let probe = if along_x {
                    Vec2::new((a.x + s + a.x + e) * 0.5, a.y + face * (half + 30.0))
                } else {
                    Vec2::new(a.x + face * (half + 30.0), (a.y + s + a.y + e) * 0.5)
                };
                if inside_envelope(probe) {
                    continue;
                }
                // Boards lap, and a lapped board is not flat against the
                // wall: its bottom edge stands proud of its top, which is what
                // throws the shadow line that makes siding read as siding.
                //
                // Two wrong versions before this one, both of which stippled
                // the whole north elevation. Leaving a gap between courses
                // exposes the top face of every board — those face up, the sun
                // is up, and a hundred lit slivers three centimetres wide alias
                // into what looks exactly like shadow acne. Closing the gap by
                // overlapping them instead put every board's front face on the
                // same plane as its neighbour's, which is z-fighting in
                // three-centimetre bands. Tilting each board solves both: the
                // tops are covered and no two faces are coplanar.
                const BOARD: f32 = 18.0;
                const TILT: f32 = 0.15;
                let courses = ((high - low) / BOARD).floor().max(1.0) as usize;
                for k in 0..courses {
                    let y = low + BOARD * k as f32;
                    if y + BOARD * 1.4 > high {
                        break;
                    }
                    let n = grain(a.x + a.y + s + y * 2.3);
                    let size = if along_x {
                        Vec3::new(e - s, BOARD * 1.35, 3.4)
                    } else {
                        Vec3::new(3.4, BOARD * 1.35, e - s)
                    };
                    let mut board = Solid::between(-size * 0.5, size * 0.5, Stuff::Wood);
                    board.center = if along_x {
                        Vec3::new(
                            a.x + (s + e) * 0.5,
                            y + BOARD * 0.5,
                            a.y + face * (half + 1.9),
                        )
                    } else {
                        Vec3::new(
                            a.x + face * (half + 1.9),
                            y + BOARD * 0.5,
                            a.y + (s + e) * 0.5,
                        )
                    };
                    board.rot = if along_x {
                        Quat::from_rotation_x(face * TILT)
                    } else {
                        Quat::from_rotation_z(-face * TILT)
                    };
                    board.paint = Some(Color::srgb(
                        0.80 + n * 0.012,
                        0.785 + n * 0.012,
                        0.745 + n * 0.010,
                    ));
                    out.push(board);
                }
            }
        }

        // Cornice, on any piece that reaches the ceiling.
        if high >= CEILING - 0.01 && low < CEILING - CORNICE_HIGH {
            for face in [-1.0f32, 1.0] {
                // Stepped, like the skirting. A cornice run as one square
                // section reads as a stripe of paint where the wall meets the
                // ceiling; two steps and it reads as a moulding, because the
                // lower one is in shadow and the upper one is not.
                for (bottom, top, proud) in [
                    (
                        CEILING - CORNICE_HIGH,
                        CEILING - CORNICE_HIGH + 2.4,
                        CORNICE_PROUD - 1.4,
                    ),
                    (CEILING - CORNICE_HIGH + 2.4, CEILING, CORNICE_PROUD),
                ] {
                    let (cmin, cmax) = if along_x {
                        (
                            Vec3::new(a.x + s, bottom, a.y + face * half),
                            Vec3::new(a.x + e, top, a.y + face * (half + proud)),
                        )
                    } else {
                        (
                            Vec3::new(a.x + face * half, bottom, a.y + s),
                            Vec3::new(a.x + face * (half + proud), top, a.y + e),
                        )
                    };
                    let mut trim = Solid::between(cmin.min(cmax), cmin.max(cmax), Stuff::Wood);
                    trim.paint = Some(Color::srgb(0.93, 0.92, 0.90));
                    out.push(trim);
                }
            }
        }

        // Skirting, on any piece that reaches the floor.
        //
        // Emitted here rather than around the room afterwards, because a run
        // already knows where its solid parts are: a doorway gets no skirting
        // across it for free, and nothing has to be told where the openings
        // were. Both faces, since a partition has a room on each side.
        if low <= 0.01 && high > SKIRT_HIGH {
            for face in [-1.0f32, 1.0] {
                // A profile, not a plank. Skirting is a board with something on
                // top of it — here the board, then a bead set back from its
                // face — and the step between the two is what catches a line of
                // light all the way round a room at ankle height. It is two
                // boxes instead of one and it is in every room in the house.
                for (bottom, top, proud) in [
                    (0.0, SKIRT_HIGH - 2.6, SKIRT_PROUD),
                    (SKIRT_HIGH - 2.6, SKIRT_HIGH, SKIRT_PROUD - 0.9),
                ] {
                    let (smin, smax) = if along_x {
                        (
                            Vec3::new(a.x + s, bottom, a.y + face * half),
                            Vec3::new(a.x + e, top, a.y + face * (half + proud)),
                        )
                    } else {
                        (
                            Vec3::new(a.x + face * half, bottom, a.y + s),
                            Vec3::new(a.x + face * (half + proud), top, a.y + e),
                        )
                    };
                    let mut skirt = Solid::between(smin.min(smax), smin.max(smax), Stuff::Wood);
                    skirt.paint = Some(Color::srgb(0.90, 0.89, 0.86));
                    out.push(skirt);
                }
            }
        }
    };

    let mut cuts: Vec<Opening> = holes.to_vec();
    cuts.sort_by(|p, q| p.at.partial_cmp(&q.at).unwrap());

    let mut walked = 0.0;
    for hole in &cuts {
        let start = hole.at - hole.wide * 0.5;
        let end = hole.at + hole.wide * 0.5;
        piece(walked, start, 0.0, height, out);
        piece(start, end, 0.0, hole.sill, out);
        piece(start, end, hole.head, height, out);
        walked = end;
    }
    piece(walked, length, 0.0, height, out);
}

// ---------------------------------------------------------------------------
// Building it
// ---------------------------------------------------------------------------

/// Window positions in feet along their wall, shared by the openings and the
/// glazing so the two cannot drift apart.
const NORTH_WINDOWS: [f32; 4] = [6.0, 21.0, 29.0, 40.0];
const SOUTH_WINDOWS: [f32; 4] = [6.0, 21.0, 30.0, 41.0];
const WEST_WINDOWS: [f32; 3] = [6.0, 17.0, 29.0];
const EAST_WINDOW: f32 = 29.0;

/// A room's floor: boards, tile or concrete, by what the room is for.
///
/// Strips rather than one plane. It costs a few dozen solids per room, tiles the
/// same rectangle so collision is unchanged, and it is the difference between a
/// floor and a colour — the largest single surface in any shot of any room.
fn lay_floor(out: &mut Vec<Solid>, r: &Room) {
    let (stuff, base, run, tone, cross) = match r.use_for {
        // Tile, in squarer courses and a cooler colour.
        Use::Bath | Use::Utility => (
            Stuff::Stone,
            Color::srgb(0.72, 0.73, 0.71),
            34.0,
            0.05,
            true,
        ),
        // The garage is a poured slab: no courses at all, just a little
        // mottling, and it sits lower than the house.
        Use::Garage => (
            Stuff::Stone,
            Color::srgb(0.44, 0.44, 0.46),
            110.0,
            0.03,
            false,
        ),
        _ => (
            Stuff::Floorboard,
            Color::srgb(0.45, 0.33, 0.22),
            19.0,
            0.06,
            false,
        ),
    };

    let top = if r.use_for == Use::Garage { -6.0 } else { 0.0 };
    let lip = INNER;
    let (lo, hi) = (r.min - lip, r.max + lip);

    let mut at = lo.y;
    let mut i = 0;
    while at < hi.y {
        let to = (at + run).min(hi.y);
        let mut plank = Solid::between(Vec3::new(lo.x, -SLAB, at), Vec3::new(hi.x, top, to), stuff);
        // A hair of variation, and every fourth course a shade off, which is
        // what stops a run of them reading as stripes.
        let t = grain(at) * tone + if i % 4 == 0 { -tone * 0.5 } else { 0.0 };
        plank.paint = Some(Color::srgb(
            (base.to_srgba().red + t).clamp(0.0, 1.0),
            (base.to_srgba().green + t * 0.85).clamp(0.0, 1.0),
            (base.to_srgba().blue + t * 0.7).clamp(0.0, 1.0),
        ));
        out.push(plank);
        // Tile is cut across as well as along, so it reads as squares.
        if cross {
            let mut x = lo.x;
            while x < hi.x {
                let nx = (x + run).min(hi.x);
                let mut grout = Solid::between(
                    Vec3::new(nx - 1.0, -SLAB, at),
                    Vec3::new(nx, top + 0.2, to),
                    stuff,
                );
                grout.paint = Some(Color::srgb(0.60, 0.61, 0.60));
                out.push(grout);
                x = nx;
            }
        }
        at = to;
        i += 1;
    }
}

/// Deterministic grain, from a position. Never a generator: two captures of the
/// same house have to be comparable.
fn grain(v: f32) -> f32 {
    let x = (v * 0.37).sin() * 43758.547;
    (x - x.floor()) * 2.0 - 1.0
}

fn envelope() -> (f32, f32, f32, f32, f32, f32) {
    let h = OUTER * 0.5;
    (
        ft(0.0) - h,  // west
        ft(68.0) + h, // east, past the garage
        ft(0.0) - h,  // north
        ft(34.5) + h, // south
        ft(24.0) + h, // the garage's south wall
        ft(46.0) + h, // where the house ends and the garage begins
    )
}

/// Casing and a sill round every window, on whichever face is outdoors.
///
/// With the walls clad, a window became a rectangular hole punched in a run of
/// boards, which is the one thing siding never does — it is always trimmed out.
/// Four boards and a sill each, and the same probe the cladding uses to work
/// out which way is out.
fn window_trim(out: &mut Vec<Solid>) {
    let trim = Color::srgb(0.94, 0.93, 0.90);
    for (lo, hi) in window_openings() {
        let along_x = (hi.x - lo.x) > (hi.z - lo.z);
        let (mid_x, mid_z) = ((lo.x + hi.x) * 0.5, (lo.z + hi.z) * 0.5);
        for face in [-1.0f32, 1.0] {
            let probe = if along_x {
                Vec2::new(mid_x, mid_z + face * 40.0)
            } else {
                Vec2::new(mid_x + face * 40.0, mid_z)
            };
            if inside_envelope(probe) {
                continue;
            }
            let out_face = if along_x {
                mid_z + face * (OUTER * 0.5 + 3.0)
            } else {
                mid_x + face * (OUTER * 0.5 + 3.0)
            };
            let (wide, board) = if along_x {
                (hi.x - lo.x, 11.0)
            } else {
                (hi.z - lo.z, 11.0)
            };
            let mut put = |cx: f32, cy: f32, cz: f32, sx: f32, sy: f32, sz: f32| {
                let mut b = Solid::between(
                    Vec3::new(cx - sx * 0.5, cy - sy * 0.5, cz - sz * 0.5),
                    Vec3::new(cx + sx * 0.5, cy + sy * 0.5, cz + sz * 0.5),
                    Stuff::Wood,
                );
                b.paint = Some(trim);
                out.push(b);
            };
            let high = hi.y - lo.y.max(SILL);
            let cy = (SILL + HEAD) * 0.5;
            if along_x {
                for side in [-1.0f32, 1.0] {
                    put(
                        mid_x + side * (wide * 0.5 + board * 0.5),
                        cy,
                        out_face,
                        board,
                        high + board * 2.0,
                        5.0,
                    );
                }
                put(
                    mid_x,
                    HEAD + board * 0.5,
                    out_face,
                    wide + board * 2.0,
                    board,
                    6.0,
                );
                put(
                    mid_x,
                    SILL - 4.0,
                    out_face + face * 1.5,
                    wide + board * 2.6,
                    8.0,
                    9.0,
                );
            } else {
                for side in [-1.0f32, 1.0] {
                    put(
                        out_face,
                        cy,
                        mid_z + side * (wide * 0.5 + board * 0.5),
                        5.0,
                        high + board * 2.0,
                        board,
                    );
                }
                put(
                    out_face,
                    HEAD + board * 0.5,
                    mid_z,
                    6.0,
                    board,
                    wide + board * 2.0,
                );
                put(
                    out_face + face * 1.5,
                    SILL - 4.0,
                    mid_z,
                    9.0,
                    8.0,
                    wide + board * 2.6,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The grounds
// ---------------------------------------------------------------------------

/// Drive, path and planting.
///
/// The house had been standing on an infinite lawn with no way up to either of
/// its doors, which reads as a model of a house rather than a house. None of
/// this is tall enough to count as building, so the envelope law lets it lie
/// outside; the shrubs get away with it by being vegetation, which is what
/// `Stuff::Grass` already means.
fn grounds(out: &mut Vec<Solid>) {
    let (_w, _e, _n, so, garage_south, _he) = envelope();
    let paving = Color::srgb(0.63, 0.62, 0.59);
    let joint = Color::srgb(0.46, 0.46, 0.44);

    let mut pave = |min: Vec3, max: Vec3, tint: Color| {
        let mut s = Solid::between(min, max, Stuff::Stone);
        s.paint = Some(tint);
        s.outdoors = true;
        out.push(s);
    };

    // The drive, in bays, running out from under the garage door to the path.
    let (dx, drive_half) = (ft(57.0), ft(16.0) * 0.5 + 40.0);
    let drive_end = so + 190.0;
    const BAYS: usize = 5;
    pave(
        Vec3::new(dx - drive_half, -11.0, garage_south),
        Vec3::new(dx + drive_half, -3.0, drive_end),
        joint,
    );
    for k in 0..BAYS {
        let run = (drive_end - garage_south) / BAYS as f32;
        let z0 = garage_south + run * k as f32;
        pave(
            Vec3::new(dx - drive_half + 3.0, -11.0, z0 + 3.0),
            Vec3::new(dx + drive_half - 3.0, -2.0, z0 + run - 3.0),
            Color::srgb(
                0.63 + grain(dx + z0 * 1.7) * 0.02,
                0.62 + grain(z0 + dx * 1.7) * 0.02,
                0.59,
            ),
        );
    }

    // The path along the front, and the spur up to the step.
    let door_x = ft(25.0);
    let walk = so + 152.0;
    pave(
        Vec3::new(door_x - 70.0, -11.0, walk - 62.0),
        Vec3::new(dx - drive_half, -3.0, walk + 62.0),
        joint,
    );
    for k in 0..9 {
        let run = (dx - drive_half - door_x + 70.0) / 9.0;
        let x0 = door_x - 70.0 + run * k as f32;
        pave(
            Vec3::new(x0 + 3.0, -11.0, walk - 59.0),
            Vec3::new(x0 + run - 3.0, -2.0, walk + 59.0),
            Color::srgb(0.63 + grain(x0 + walk * 1.7) * 0.02, 0.62, 0.59),
        );
    }
    pave(
        Vec3::new(door_x - 62.0, -11.0, so + 30.0),
        Vec3::new(door_x + 62.0, -2.0, walk),
        paving,
    );

    // Foundation planting, between the front windows and clear of the door.
    //
    // Five boxes a shrub rather than three, turned to different angles and
    // overlapping hard, because a plant wants to read as one mass with a ragged
    // edge — three stacked cubes read as three stacked cubes. They are
    // `Stuff::Grass`, which is both what they are and how they get past the
    // envelope law: vegetation is allowed to stand outside the walls.
    for (x, tall) in [(ft(11.0), 98.0f32), (ft(35.0), 84.0), (ft(44.0), 110.0)] {
        for (k, (spread, lift)) in [
            (1.00f32, 0.26f32),
            (0.92, 0.44),
            (0.80, 0.60),
            (0.62, 0.76),
            (0.40, 0.90),
        ]
        .into_iter()
        .enumerate()
        {
            let n = grain(x + tall * 3.1 + k as f32 * 7.7);
            let wide = 86.0 * spread * (1.0 + n * 0.10);
            let y = tall * lift;
            let at = Vec3::new(x + n * 9.0, y, so + 40.0 + n * 7.0);
            let mut bush = Solid::between(
                at - Vec3::new(wide * 0.5, tall * 0.20, wide * 0.5),
                at + Vec3::new(wide * 0.5, tall * 0.20, wide * 0.5),
                Stuff::Grass,
            );
            bush.outdoors = true;
            bush.rot = Quat::from_rotation_y(k as f32 * 0.55 + n * 0.4);
            // Darker and greyer than lawn, and each layer a shade off the one
            // below so the mass has some depth in flat sun.
            bush.paint = Some(Color::srgb(
                0.13 + k as f32 * 0.014,
                0.23 + k as f32 * 0.022,
                0.13 + k as f32 * 0.010,
            ));
            out.push(bush);
        }
    }
}

// ---------------------------------------------------------------------------
// The neighbourhood
// ---------------------------------------------------------------------------

/// Everything past the property line: the street, the houses across it, the
/// trees, and what is in the two yards.
///
/// It exists to be *seen through a window*. Every room in this house has looked
/// out onto an unbroken green plane to the horizon, which is the one thing that
/// gives away that the building is a model of a house rather than a house in a
/// place. None of it needs to be good close up; it needs to have a silhouette,
/// a roof line and a colour that is not the lawn's.
fn neighbourhood(out: &mut Vec<Solid>) {
    let (w, e, n, so, _gs, _he) = envelope();

    fn place(out: &mut Vec<Solid>, min: Vec3, max: Vec3, stuff: Stuff, tint: Color) {
        let mut s = Solid::between(min, max, stuff);
        s.paint = Some(tint);
        s.outdoors = true;
        out.push(s);
    }

    // -- The street, out in front ------------------------------------------
    let walk_near = so + 900.0;
    let kerb = walk_near + 170.0;
    let road_far = kerb + 760.0;
    let walk_far = road_far + 20.0;
    let spread = 6000.0;

    place(
        out,
        Vec3::new(w - spread, -10.0, walk_near),
        Vec3::new(e + spread, -2.0, kerb),
        Stuff::Stone,
        Color::srgb(0.64, 0.63, 0.60),
    );
    place(
        out,
        Vec3::new(w - spread, -10.0, kerb),
        Vec3::new(e + spread, 4.0, kerb + 18.0),
        Stuff::Stone,
        Color::srgb(0.70, 0.69, 0.66),
    );
    place(
        out,
        Vec3::new(w - spread, -11.0, kerb + 18.0),
        Vec3::new(e + spread, -4.0, road_far),
        Stuff::Stone,
        Color::srgb(0.24, 0.24, 0.25),
    );
    place(
        out,
        Vec3::new(w - spread, -11.0, road_far),
        Vec3::new(e + spread, 4.0, road_far + 18.0),
        Stuff::Stone,
        Color::srgb(0.70, 0.69, 0.66),
    );
    place(
        out,
        Vec3::new(w - spread, -10.0, road_far + 18.0),
        Vec3::new(e + spread, -2.0, walk_far + 170.0),
        Stuff::Stone,
        Color::srgb(0.64, 0.63, 0.60),
    );
    // Centre line, dashed.
    let middle = (kerb + road_far) * 0.5;
    let mut dash = w - spread;
    while dash < e + spread {
        place(
            out,
            Vec3::new(dash, -11.0, middle - 7.0),
            Vec3::new(dash + 200.0, -3.5, middle + 7.0),
            Stuff::Stone,
            Color::srgb(0.72, 0.68, 0.36),
        );
        dash += 380.0;
    }
    // The drive crosses the verge to meet the road.
    place(
        out,
        Vec3::new(ft(57.0) - 330.0, -10.0, so + 190.0),
        Vec3::new(ft(57.0) + 330.0, -3.0, kerb + 20.0),
        Stuff::Stone,
        Color::srgb(0.63, 0.62, 0.59),
    );

    // -- A tree, since half of what makes a street a street is the trees ----
    fn tree(out: &mut Vec<Solid>, at: Vec2, tall: f32) {
        let seed = grain(at.x + at.y * 1.7);
        // A trunk that is two crossed boxes rather than one, so it has eight
        // faces at the corner instead of four and stops reading as a post.
        for k in 0..2 {
            let mut trunk = Solid::between(
                Vec3::new(-13.0, -10.0, -7.0),
                Vec3::new(13.0, tall * 0.44, 7.0),
                Stuff::Wood,
            );
            trunk.center = Vec3::new(at.x, (tall * 0.44 - 10.0) * 0.5 - 5.0, at.y);
            trunk.rot = Quat::from_rotation_y(k as f32 * std::f32::consts::FRAC_PI_2 + seed * 0.3);
            trunk.paint = Some(Color::srgb(0.29, 0.23, 0.18));
            trunk.outdoors = true;
            out.push(trunk);
        }
        // The canopy is a ball of lumps, not a stack of slabs. Twelve boxes on
        // a rough sphere, each turned, each about half the canopy across — the
        // first version stacked six wide flat boxes and read as a pile of
        // crates painted green.
        let heart = Vec3::new(at.x, tall * 0.74, at.y);
        let ball = tall * 0.36;
        const LUMPS: usize = 18;
        for k in 0..LUMPS {
            let a = k as f32 * 2.399_963;
            let up = 1.0 - 2.0 * (k as f32 + 0.5) / LUMPS as f32;
            let ring = (1.0 - up * up).max(0.0).sqrt();
            let dir = Vec3::new(a.cos() * ring, up * 0.82, a.sin() * ring);
            let m = grain(at.x + k as f32 * 13.0 + at.y * 0.7);
            // Smaller and more of them. Twelve at three-quarters of the ball
            // across is a heap of crates; eighteen at half is a canopy.
            let lump = ball * (0.50 + m * 0.16);
            let mut leaf = Solid::between(
                Vec3::splat(-lump * 0.5),
                Vec3::splat(lump * 0.5),
                Stuff::Grass,
            );
            leaf.center = heart + dir * ball * 0.74;
            leaf.rot = Quat::from_rotation_y(a) * Quat::from_rotation_x(0.4 + m * 0.5);
            leaf.paint = Some(Color::srgb(
                0.13 + m * 0.03 + up * 0.03,
                0.26 + m * 0.04 + up * 0.05,
                0.13 + m * 0.02,
            ));
            leaf.outdoors = true;
            out.push(leaf);
        }
    }

    // -- A house across the road, or beside this one -----------------------
    //
    // Built out of the same roof constructor the real one uses, so the ridges,
    // eaves and gables match and the street reads as one street.
    fn neighbour(out: &mut Vec<Solid>, min: Vec2, max: Vec2, along_x: bool, tone: f32) {
        let top = CEILING - 8.0;
        let clad = Color::srgb(0.62 + tone * 0.22, 0.60 + tone * 0.20, 0.55 + tone * 0.16);
        let mut body = Solid::between(
            Vec3::new(min.x, -12.0, min.y),
            Vec3::new(max.x, top, max.y),
            Stuff::Wood,
        );
        body.paint = Some(clad);
        body.outdoors = true;
        out.push(body);

        // Openings, punched as dark insets on the two long faces.
        let (from, to, face_lo, face_hi) = if along_x {
            (min.x, max.x, min.y, max.y)
        } else {
            (min.y, max.y, min.x, max.x)
        };
        let run = to - from;
        let holes = ((run / 260.0).round() as usize).max(2);
        for k in 0..holes {
            let along = from + run * (k as f32 + 0.5) / holes as f32;
            let door = k == holes / 2;
            let (lo_y, hi_y) = if door { (0.0, 200.0) } else { (95.0, 200.0) };
            for face in [face_lo, face_hi] {
                let side = if face == face_lo { -1.0 } else { 1.0 };
                let (a, b) = if along_x {
                    (
                        Vec3::new(along - 60.0, lo_y, face + side * 3.0),
                        Vec3::new(along + 60.0, hi_y, face - side * 3.0),
                    )
                } else {
                    (
                        Vec3::new(face + side * 3.0, lo_y, along - 60.0),
                        Vec3::new(face - side * 3.0, hi_y, along + 60.0),
                    )
                };
                let mut hole = Solid::between(a.min(b), a.max(b), Stuff::Glass);
                hole.paint = Some(if door {
                    Color::srgb(0.24, 0.18, 0.14)
                } else {
                    Color::srgb(0.16, 0.19, 0.22)
                });
                hole.sheer = false;
                hole.outdoors = true;
                out.push(hole);
            }
        }
        gable_roof(out, min, max, along_x, top + SLAB + 8.0);
    }

    // Two across the road, set back behind their own lawns, and one either
    // side of this house.
    // Across the road, set well back behind their own lawns. The first pass had
    // them close enough to the kerb to loom, which is not what a street of
    // these looks like and is not what a window onto one shows.
    for (x0, wide, deep, back, tone) in [
        (w - 900.0, 1560.0, 1020.0, 1500.0, 0.35f32),
        (w + 1300.0, 1420.0, 980.0, 1720.0, -0.42),
        (w + 3300.0, 1620.0, 1040.0, 1560.0, 0.18),
    ] {
        let z0 = walk_far + back;
        neighbour(
            out,
            Vec2::new(x0, z0),
            Vec2::new(x0 + wide, z0 + deep),
            true,
            tone,
        );
        // A drive out to the kerb and a path to the door, which is most of
        // what tells you a house is lived in from three gardens away.
        place(
            out,
            Vec3::new(x0 + wide - 500.0, -10.0, road_far + 18.0),
            Vec3::new(x0 + wide - 140.0, -3.0, z0),
            Stuff::Stone,
            Color::srgb(0.62, 0.61, 0.58),
        );
        place(
            out,
            Vec3::new(x0 + wide * 0.5 - 55.0, -10.0, road_far + 120.0),
            Vec3::new(x0 + wide * 0.5 + 55.0, -3.0, z0),
            Stuff::Stone,
            Color::srgb(0.65, 0.64, 0.61),
        );
    }
    neighbour(
        out,
        Vec2::new(w - 2560.0, n + 220.0),
        Vec2::new(w - 940.0, n + 1780.0),
        false,
        0.12,
    );
    neighbour(
        out,
        Vec2::new(e + 940.0, n + 300.0),
        Vec2::new(e + 2500.0, n + 1860.0),
        false,
        -0.20,
    );

    for (x, z, tall) in [
        (w - 420.0, walk_near + 80.0, 520.0),
        (ft(40.0), walk_near + 80.0, 460.0),
        (e + 520.0, walk_near + 80.0, 560.0),
        (w + 1300.0, walk_far + 300.0, 500.0),
        (w + 3400.0, walk_far + 320.0, 470.0),
        (w - 1400.0, n - 500.0, 620.0),
        (e + 900.0, so + 300.0, 540.0),
    ] {
        tree(out, Vec2::new(x, z), tall);
    }

    // -- The back yard ------------------------------------------------------
    let fence_z = n - 640.0;
    let (fw, fe) = (w - 340.0, e + 340.0);
    fn fence(out: &mut Vec<Solid>, a: Vec2, b: Vec2) {
        let along = (b - a).normalize_or_zero();
        let run = (b - a).length();
        let bays = (run / 200.0).round().max(1.0) as usize;
        for k in 0..=bays {
            let p = a + along * (run * k as f32 / bays as f32);
            place(
                out,
                Vec3::new(p.x - 7.0, -12.0, p.y - 7.0),
                Vec3::new(p.x + 7.0, 160.0, p.y + 7.0),
                Stuff::Wood,
                Color::srgb(0.40, 0.32, 0.24),
            );
        }
        let across = Vec2::new(along.y.abs(), along.x.abs());
        for (y, high) in [(24.0f32, 60.0f32), (94.0, 56.0)] {
            let lo = a - across * 5.0;
            let hi = b + across * 5.0;
            place(
                out,
                Vec3::new(lo.x.min(hi.x) - 4.0, y, lo.y.min(hi.y) - 4.0),
                Vec3::new(lo.x.max(hi.x) + 4.0, y + high, lo.y.max(hi.y) + 4.0),
                Stuff::Wood,
                Color::srgb(0.46, 0.37, 0.27),
            );
        }
    }
    fence(out, Vec2::new(fw, fence_z), Vec2::new(fe, fence_z));
    fence(out, Vec2::new(fw, fence_z), Vec2::new(fw, n - 60.0));
    fence(out, Vec2::new(fe, fence_z), Vec2::new(fe, n - 60.0));

    // A patio off the back of the house, a shed in the corner, and a line.
    place(
        out,
        Vec3::new(ft(18.0), -10.0, n - 460.0),
        Vec3::new(ft(31.0), -2.0, n - 20.0),
        Stuff::Stone,
        Color::srgb(0.60, 0.58, 0.55),
    );
    let shed = Vec2::new(fw + 420.0, fence_z + 300.0);
    place(
        out,
        Vec3::new(shed.x - 170.0, -12.0, shed.y - 130.0),
        Vec3::new(shed.x + 170.0, 190.0, shed.y + 130.0),
        Stuff::Wood,
        Color::srgb(0.38, 0.34, 0.28),
    );
    place(
        out,
        Vec3::new(shed.x - 186.0, 186.0, shed.y - 146.0),
        Vec3::new(shed.x + 186.0, 204.0, shed.y + 30.0),
        Stuff::Wood,
        Color::srgb(0.28, 0.26, 0.24),
    );
    place(
        out,
        Vec3::new(shed.x - 186.0, 168.0, shed.y + 20.0),
        Vec3::new(shed.x + 186.0, 190.0, shed.y + 150.0),
        Stuff::Wood,
        Color::srgb(0.30, 0.28, 0.26),
    );
    place(
        out,
        Vec3::new(shed.x - 48.0, -10.0, shed.y + 126.0),
        Vec3::new(shed.x + 48.0, 178.0, shed.y + 134.0),
        Stuff::Wood,
        Color::srgb(0.30, 0.26, 0.21),
    );
    for side in [-1.0f32, 1.0] {
        let x = ft(24.0) + side * 420.0;
        place(
            out,
            Vec3::new(x - 8.0, -12.0, n - 700.0),
            Vec3::new(x + 8.0, 200.0, n - 684.0),
            Stuff::Metal,
            Color::srgb(0.62, 0.62, 0.60),
        );
    }
    for k in 0..3 {
        let y = 168.0 + k as f32 * 14.0;
        place(
            out,
            Vec3::new(ft(24.0) - 420.0, y, n - 695.0),
            Vec3::new(ft(24.0) + 420.0, y + 2.0, n - 693.0),
            Stuff::Metal,
            Color::srgb(0.78, 0.78, 0.74),
        );
    }
    // A mailbox at the kerb, which is the one thing every one of these streets
    // has and the only reason to walk to the end of the drive.
    let post = Vec2::new(ft(57.0) - 380.0, walk_near + 100.0);
    place(
        out,
        Vec3::new(post.x - 6.0, -10.0, post.y - 6.0),
        Vec3::new(post.x + 6.0, 108.0, post.y + 6.0),
        Stuff::Wood,
        Color::srgb(0.38, 0.30, 0.22),
    );
    place(
        out,
        Vec3::new(post.x - 17.0, 106.0, post.y - 26.0),
        Vec3::new(post.x + 17.0, 148.0, post.y + 26.0),
        Stuff::Metal,
        Color::srgb(0.30, 0.34, 0.38),
    );
    place(
        out,
        Vec3::new(post.x + 17.0, 128.0, post.y - 4.0),
        Vec3::new(post.x + 25.0, 150.0, post.y + 4.0),
        Stuff::Metal,
        Color::srgb(0.66, 0.20, 0.16),
    );
}

// ---------------------------------------------------------------------------
// The roof
// ---------------------------------------------------------------------------

/// Four in twelve, which is a ranch's pitch and shallow enough that the ridge
/// does not tower over a single-storey plan.
const PITCH: f32 = 4.0 / 12.0;
/// How far the roof reaches past the wall. The shadow this throws is the whole
/// reason a house reads as built rather than extruded, and its underside is a
/// sheltered ledge a fly can sit on out of the weather.
const OVERHANG: f32 = 46.0;
const ROOF_THICK: f32 = 16.0;
const FASCIA_DEEP: f32 = 20.0;

const SHINGLE: Color = Color::srgb(0.27, 0.25, 0.24);
const TRIM: Color = Color::srgb(0.90, 0.89, 0.85);
const SIDING: Color = Color::srgb(0.74, 0.73, 0.69);

/// A gabled roof over one rectangle, ridge along whichever axis is asked for.
///
/// Everything here is still a box: the two slopes are boxes with a turn on
/// them, and the triangular gable ends are courses of box, each stopping where
/// the slope above it would cut through. Stepping a triangle out of rectangles
/// leaves a staircase along the rake, which is what the barge boards are for —
/// they are a real piece of a real roof and they cover the one artifact this
/// construction cannot avoid.
fn gable_roof(out: &mut Vec<Solid>, lo: Vec2, hi: Vec2, along_x: bool, eave: f32) {
    // `a` runs along the ridge, `c` across it.
    let (a0, a1, c0, c1) = if along_x {
        (lo.x, hi.x, lo.y, hi.y)
    } else {
        (lo.y, hi.y, lo.x, hi.x)
    };
    let place = |a: f32, y: f32, c: f32| {
        if along_x {
            Vec3::new(a, y, c)
        } else {
            Vec3::new(c, y, a)
        }
    };
    let size = |along: f32, y: f32, across: f32| {
        if along_x {
            Vec3::new(along, y, across)
        } else {
            Vec3::new(across, y, along)
        }
    };

    let (ea0, ea1) = (a0 - OVERHANG, a1 + OVERHANG);
    let (ec0, ec1) = (c0 - OVERHANG, c1 + OVERHANG);
    let ridge_c = (ec0 + ec1) * 0.5;
    let half = (ec1 - ec0) * 0.5;
    let rise = half * PITCH;
    let ridge_y = eave + rise;
    let slope = (half * half + rise * rise).sqrt();
    let theta = rise.atan2(half);
    let along_len = ea1 - ea0;
    let along_mid = (ea0 + ea1) * 0.5;

    // The two slopes.
    for side in [-1.0f32, 1.0] {
        let turn = if along_x { side * theta } else { -side * theta };
        let rot = if along_x {
            Quat::from_rotation_x(turn)
        } else {
            Quat::from_rotation_z(turn)
        };
        let mut plane = Solid::between(
            -size(along_len, ROOF_THICK, slope) * 0.5,
            size(along_len, ROOF_THICK, slope) * 0.5,
            Stuff::Stone,
        );
        plane.center = place(
            along_mid,
            (eave + ridge_y) * 0.5 + ROOF_THICK * 0.5,
            ridge_c + side * half * 0.5,
        );
        plane.rot = rot;
        plane.paint = Some(SHINGLE);
        plane.roof = true;
        out.push(plane);

        // Fascia, hung off the eave, and the soffit that closes the gap back to
        // the wall.
        let edge = ridge_c + side * half;
        let mut fascia = Solid::between(
            place(ea0, eave - FASCIA_DEEP, edge - side * 9.0).min(place(ea1, eave, edge)),
            place(ea0, eave - FASCIA_DEEP, edge - side * 9.0).max(place(ea1, eave, edge)),
            Stuff::Wood,
        );
        fascia.paint = Some(TRIM);
        fascia.roof = true;
        out.push(fascia);

        // A gutter hung on the fascia. It is the last thing missing from an
        // eave, and at fly scale it is a hundred-and-forty-foot trough with a
        // lip on it, out of the weather and out of sight.
        let gutter_c = edge + side * 5.0;
        let mut gutter = Solid::between(
            place(ea0, eave - FASCIA_DEEP - 9.0, gutter_c - 7.0).min(place(
                ea1,
                eave - FASCIA_DEEP + 3.0,
                gutter_c + 7.0,
            )),
            place(ea0, eave - FASCIA_DEEP - 9.0, gutter_c - 7.0).max(place(
                ea1,
                eave - FASCIA_DEEP + 3.0,
                gutter_c + 7.0,
            )),
            Stuff::Metal,
        );
        gutter.paint = Some(TRIM);
        gutter.roof = true;
        out.push(gutter);

        let wall_line = if side < 0.0 { c0 } else { c1 };
        let mut soffit = Solid::between(
            place(ea0, eave - FASCIA_DEEP, edge).min(place(
                ea1,
                eave - FASCIA_DEEP + 7.0,
                wall_line,
            )),
            place(ea0, eave - FASCIA_DEEP, edge).max(place(
                ea1,
                eave - FASCIA_DEEP + 7.0,
                wall_line,
            )),
            Stuff::Wood,
        );
        soffit.paint = Some(TRIM);
        soffit.roof = true;
        out.push(soffit);
    }

    // The gable ends: courses of siding, each cut off under the slope, and a
    // barge board down each rake to hide the steps.
    const COURSES: usize = 26;
    for end in [-1.0f32, 1.0] {
        let wall = if end < 0.0 { a0 } else { a1 };
        // The gable stands on the wall it caps, and never wider than that wall:
        // the roof's half-span includes the overhang, and a gable built out to
        // *that* would hang in mid-air past the corner of the house.
        let base = CEILING;
        let wall_half = half - OVERHANG;
        for k in 0..COURSES {
            let y0 = base + (ridge_y - ROOF_THICK - base) * k as f32 / COURSES as f32;
            let y1 = base + (ridge_y - ROOF_THICK - base) * (k + 1) as f32 / COURSES as f32;
            // Measured at the course's *bottom*, so each one runs a little way
            // up into the slab above it rather than stopping under it. Cutting
            // at the top edge instead leaves a triangular gap per course, and
            // eighteen of those in a row is a row of shark's teeth along the
            // rake — which is exactly how the first capture came back.
            let reach = ((ridge_y - ROOF_THICK - y0) / PITCH).clamp(0.0, wall_half);
            if reach < 1.0 {
                continue;
            }
            let mut course = Solid::between(
                place(wall - OUTER * 0.5, y0, ridge_c - reach),
                place(wall + OUTER * 0.5, y1, ridge_c + reach),
                Stuff::Plaster,
            );
            course.paint = Some(SIDING);
            course.roof = true;
            out.push(course);
        }

        for side in [-1.0f32, 1.0] {
            let turn = if along_x { side * theta } else { -side * theta };
            let rot = if along_x {
                Quat::from_rotation_x(turn)
            } else {
                Quat::from_rotation_z(turn)
            };
            let mut barge = Solid::between(
                -size(14.0, 22.0, slope) * 0.5,
                size(14.0, 22.0, slope) * 0.5,
                Stuff::Wood,
            );
            barge.center = place(
                wall + end * (OUTER * 0.5 + 7.0),
                (eave + ridge_y) * 0.5 - 6.0,
                ridge_c + side * half * 0.5,
            );
            barge.rot = rot;
            barge.paint = Some(TRIM);
            barge.roof = true;
            out.push(barge);
        }
    }
}

/// The whole roof: a long gable down the house, and a second one turned
/// ninety degrees over the garage so its end faces the drive, which is what an
/// attached garage on a plan this shape actually gets built with.
fn roof(out: &mut Vec<Solid>) {
    let (w, e, n, so, garage_south, house_east) = envelope();
    let eave = CEILING + SLAB + 8.0;
    gable_roof(out, Vec2::new(w, n), Vec2::new(house_east, so), true, eave);
    gable_roof(
        out,
        Vec2::new(house_east, n),
        Vec2::new(e, garage_south),
        false,
        eave,
    );

    // Downpipes at the outside corners, from the gutter to a shoe at the
    // ground. Three brackets each, because a pipe drawn as one box reads as a
    // pipe drawn as one box.
    let drop = eave - FASCIA_DEEP - 6.0;
    for (x, z) in [
        (w - OVERHANG + 8.0, n - OVERHANG + 8.0),
        (w - OVERHANG + 8.0, so + OVERHANG - 8.0),
        (house_east - 10.0, so + OVERHANG - 8.0),
        (e + OVERHANG - 8.0, n - OVERHANG + 8.0),
        (e + OVERHANG - 8.0, garage_south + OVERHANG - 8.0),
    ] {
        let mut pipe = Solid::between(
            Vec3::new(x - 5.0, -8.0, z - 5.0),
            Vec3::new(x + 5.0, drop, z + 5.0),
            Stuff::Metal,
        );
        pipe.paint = Some(TRIM);
        pipe.roof = true;
        out.push(pipe);
        for k in 0..3 {
            let y = 40.0 + k as f32 * (drop - 60.0) / 3.0;
            let mut band = Solid::between(
                Vec3::new(x - 7.0, y, z - 7.0),
                Vec3::new(x + 7.0, y + 4.0, z + 7.0),
                Stuff::Metal,
            );
            band.paint = Some(Color::srgb(0.82, 0.81, 0.78));
            band.roof = true;
            out.push(band);
        }
        let mut shoe = Solid::between(
            Vec3::new(x - 7.0, -9.0, z - 7.0),
            Vec3::new(x + 7.0, 14.0, z + 26.0),
            Stuff::Metal,
        );
        shoe.paint = Some(TRIM);
        shoe.roof = true;
        out.push(shoe);
    }
}

pub fn build() -> Home {
    let mut s: Vec<Solid> = Vec::new();
    let top = CEILING;
    let (w, e, n, so, garage_south, house_east) = envelope();

    // -- The ground the house stands on ------------------------------------
    //
    // Not scenery. Every window was a black rectangle hung on a wall until
    // there was something on the other side of it, because a window shows you
    // what is outside and outside was nothing at all. A lawn and a sky are the
    // cheapest believable daylight in the build.
    let mut ground = Solid::between(
        Vec3::new(w - 9000.0, -60.0, n - 9000.0),
        Vec3::new(e + 9000.0, -10.0, so + 9000.0),
        Stuff::Grass,
    );
    ground.outdoors = true;
    s.push(ground);

    // -- Floors, one per room ---------------------------------------------
    //
    // Laid per room and by what the room is for, because a floor is one of the
    // loudest things a room says about itself and a bathroom with floorboards
    // in it says the wrong one. Each is run slightly into the surrounding walls
    // so no seam can open at a threshold.
    for r in rooms() {
        lay_floor(&mut s, &r);
    }

    // -- Exterior ----------------------------------------------------------
    let windows = |list: &[f32], from: f32| -> Vec<Opening> {
        list.iter()
            .map(|&p| Opening::window(ft(p) - from))
            .collect()
    };

    // North, the back of the house, running past the garage too.
    wall_run(
        &mut s,
        Vec2::new(w, n),
        Vec2::new(e, n),
        OUTER,
        top,
        Stuff::Plaster,
        &windows(&NORTH_WINDOWS, w),
    );
    // South, the front. The front door opens into the great room.
    let mut front = windows(&SOUTH_WINDOWS, w);
    front.push(Opening::door(ft(25.0) - w));
    wall_run(
        &mut s,
        Vec2::new(w, so),
        Vec2::new(house_east, so),
        OUTER,
        top,
        Stuff::Plaster,
        &front,
    );
    // West.
    wall_run(
        &mut s,
        Vec2::new(w, n),
        Vec2::new(w, so),
        OUTER,
        top,
        Stuff::Plaster,
        &windows(&WEST_WINDOWS, n),
    );
    // East: the garage's far wall, then the main bedroom's wall south of it.
    wall_run(
        &mut s,
        Vec2::new(e, n),
        Vec2::new(e, garage_south),
        OUTER,
        top,
        Stuff::Plaster,
        &[],
    );
    wall_run(
        &mut s,
        Vec2::new(house_east, garage_south),
        Vec2::new(house_east, so),
        OUTER,
        top,
        Stuff::Plaster,
        &[Opening::window(ft(EAST_WINDOW) - garage_south)],
    );
    // The garage's south wall and its vehicle door — at fly scale the house's
    // widest and least reliable connection to outdoors.
    wall_run(
        &mut s,
        Vec2::new(house_east, garage_south),
        Vec2::new(e, garage_south),
        OUTER,
        top,
        Stuff::Plaster,
        &[Opening::cased(ft(57.0) - house_east, ft(16.0))],
    );

    // -- Interior ----------------------------------------------------------
    // Left rooms | hall: three doors.
    wall_run(
        &mut s,
        Vec2::new(ft(12.0), n),
        Vec2::new(ft(12.0), so),
        INNER,
        top,
        Stuff::Plaster,
        &[
            Opening::door(ft(6.0) - n),
            Opening::door(ft(17.0) - n),
            Opening::door(ft(29.0) - n),
        ],
    );
    // Hall | the open middle: two wide cased openings, so the hall reads as part
    // of the same volume rather than a tunnel with holes in it.
    wall_run(
        &mut s,
        Vec2::new(ft(16.0), n),
        Vec2::new(ft(16.0), so),
        INNER,
        top,
        Stuff::Plaster,
        &[
            Opening::cased(ft(7.0) - n, ft(6.0)),
            Opening::cased(ft(25.0) - n, ft(8.0)),
        ],
    );
    // The middle | the right column.
    wall_run(
        &mut s,
        Vec2::new(ft(33.67), n),
        Vec2::new(ft(33.67), so),
        INNER,
        top,
        Stuff::Plaster,
        &[
            Opening::door(ft(3.3) - n),
            Opening::door(ft(14.0) - n),
            Opening::door(ft(28.0) - n),
        ],
    );
    // Bedroom three | bathroom | bedroom two.
    for z in [11.5f32, 23.0] {
        wall_run(
            &mut s,
            Vec2::new(w, ft(z)),
            Vec2::new(ft(12.0), ft(z)),
            INNER,
            top,
            Stuff::Plaster,
            &[],
        );
    }
    // Laundry | main bath | main bedroom.
    for z in [6.67f32, 22.0] {
        wall_run(
            &mut s,
            Vec2::new(ft(33.67), ft(z)),
            Vec2::new(house_east, ft(z)),
            INNER,
            top,
            Stuff::Plaster,
            &[],
        );
    }
    // House | garage, with the door out of the laundry.
    wall_run(
        &mut s,
        Vec2::new(house_east, n),
        Vec2::new(house_east, garage_south),
        OUTER,
        top,
        Stuff::Plaster,
        &[Opening::door(ft(3.3) - n)],
    );

    // -- Ceilings, one per room -------------------------------------------
    for r in rooms() {
        let mut slab = Solid::between(
            Vec3::new(r.min.x - INNER, top, r.min.y - INNER),
            Vec3::new(r.max.x + INNER, top + SLAB, r.max.y + INNER),
            Stuff::Plaster,
        );
        // Overhead, so `FLY_PLAN` drops it — otherwise the one view that can
        // show a floor plan shows a sheet of plaster instead.
        slab.roof = true;
        s.push(slab);
    }

    roof(&mut s);
    grounds(&mut s);
    neighbourhood(&mut s);
    window_trim(&mut s);
    glaze(&mut s);
    fixtures(&mut s);
    crate::furniture::furnish(&mut s);

    let great = room("great room");
    Home {
        solids: s,
        door: None,
        spawn: Vec3::new(great.middle().x, top - 8.0, great.middle().y),
    }
}

/// A pane in every window opening.
///
/// Real glass: a fly can land on it, walk it, and be fooled by it, which is the
/// most recognisable thing a housefly does indoors. `world` keeps it out of the
/// shadow pass, or every window would be a hole that let no light through.
/// Double-hung sashes, all of them shut.
///
/// Every window was a single sheet of glass filling its hole; a real one is two
/// sashes with a meeting rail between them.
///
/// One was briefly left up, on the theory that a raised sash is how a fly gets
/// into a house. Brett's call is that the fly does not go outside — the house
/// is the whole world — so every sash is down. That is the better game anyway:
/// the street is right there through the glass and can never be reached, which
/// is exactly what a window means to a fly.
fn glaze(s: &mut Vec<Solid>) {
    let (w, _e, n, so, _gs, house_east) = envelope();
    let half = WINDOW_WIDE * 0.5;
    let frame = Color::srgb(0.93, 0.92, 0.89);

    // `at` is the centre of the opening in the plane of the wall.
    let sash = |out: &mut Vec<Solid>, at: Vec3, along_x: bool, lift: f32| {
        let meet = (SILL + HEAD) * 0.5;
        let rail = 5.0;
        let put = |out: &mut Vec<Solid>,
                   along: f32,
                   wide: f32,
                   y0: f32,
                   y1: f32,
                   thick: f32,
                   paint: Option<Color>| {
            let (min, max) = if along_x {
                (
                    Vec3::new(at.x + along - wide * 0.5, y0, at.z - thick * 0.5),
                    Vec3::new(at.x + along + wide * 0.5, y1, at.z + thick * 0.5),
                )
            } else {
                (
                    Vec3::new(at.x - thick * 0.5, y0, at.z + along - wide * 0.5),
                    Vec3::new(at.x + thick * 0.5, y1, at.z + along + wide * 0.5),
                )
            };
            let mut solid = Solid::between(
                min,
                max,
                if paint.is_some() {
                    Stuff::Wood
                } else {
                    Stuff::Glass
                },
            );
            solid.paint = paint;
            out.push(solid);
        };

        // Lower sash, raised by `lift`. Upper sash, always shut.
        for (y0, y1, up) in [(SILL, meet, lift), (meet - rail, HEAD, 0.0)] {
            let (a, b) = (y0 + up, y1 + up);
            put(
                out,
                0.0,
                WINDOW_WIDE - rail * 2.0,
                a + rail,
                b - rail,
                2.0,
                None,
            );
            for side in [-1.0f32, 1.0] {
                put(
                    out,
                    side * (half - rail * 0.5),
                    rail,
                    a,
                    b,
                    5.0,
                    Some(frame),
                );
            }
            put(out, 0.0, WINDOW_WIDE, a, a + rail, 5.0, Some(frame));
            put(out, 0.0, WINDOW_WIDE, b - rail, b, 5.0, Some(frame));
        }
    };

    for &x in &NORTH_WINDOWS {
        sash(s, Vec3::new(ft(x), 0.0, n), true, 0.0);
    }
    for &x in &SOUTH_WINDOWS {
        sash(s, Vec3::new(ft(x), 0.0, so), true, 0.0);
    }
    for &z in &WEST_WINDOWS {
        sash(s, Vec3::new(w, 0.0, ft(z)), false, 0.0);
    }
    sash(s, Vec3::new(house_east, 0.0, ft(EAST_WINDOW)), false, 0.0);
}

// ---------------------------------------------------------------------------
// Checking the laws
// ---------------------------------------------------------------------------

/// Every window's clear opening, as a box in world space.
///
/// Shared by the audit so that "nothing stands in a window" can be *checked*
/// rather than remembered. Twice in one pass a kitchen fitting was placed over
/// glass — first a run of wall cabinets, then a cooker hood — and both times it
/// was invisible in the plan and unmissable from inside the room.
/// Every hinged interior doorway, in world centimetres.
///
/// All of them are in walls that run north to south, so all of them hinge about
/// a vertical axis at one end of the opening.
pub fn interior_doors() -> Vec<(Vec3, Vec3)> {
    let (_w, _e, _n, _so, _gs, house_east) = envelope();
    let half = DOOR_WIDE * 0.5;
    let mut out = Vec::new();
    for (x, zs, t) in [
        (ft(12.0), [6.0f32, 17.0, 29.0].as_slice(), INNER),
        (ft(33.67), [3.3f32, 14.0, 28.0].as_slice(), INNER),
        (house_east, [3.3f32].as_slice(), OUTER),
    ] {
        for &z in zs {
            out.push((
                Vec3::new(x - t * 0.5, 0.0, ft(z) - half),
                Vec3::new(x + t * 0.5, DOOR_HIGH, ft(z) + half),
            ));
        }
    }
    out
}

/// The wide cased openings between the hall and the middle of the house. No
/// leaves — but they are still holes in a wall, and a hole in a wall in a
/// finished house has a lining and an architrave round it.
pub fn cased_openings() -> Vec<(Vec3, Vec3)> {
    [(7.0f32, 6.0f32), (25.0, 8.0)]
        .into_iter()
        .map(|(z, wide)| {
            let half = ft(wide) * 0.5;
            (
                Vec3::new(ft(16.0) - INNER * 0.5, 0.0, ft(z) - half),
                Vec3::new(ft(16.0) + INNER * 0.5, 230.0, ft(z) + half),
            )
        })
        .collect()
}

/// The front door's opening, in world centimetres. The only way in or out of
/// this house that is not glazed shut.
pub fn front_door() -> (Vec3, Vec3) {
    let (w, _, _, so, _, _) = envelope();
    let _ = w;
    let half = DOOR_WIDE * 0.5;
    (
        Vec3::new(ft(25.0) - half, 0.0, so - OUTER * 0.5),
        Vec3::new(ft(25.0) + half, DOOR_HIGH, so + OUTER * 0.5),
    )
}

/// The vehicle door's opening, in world centimetres.
///
/// It is the one hole in the house big enough to drive through, and the only
/// one that needs a leaf built to fit it rather than a curtain hung beside it.
pub fn vehicle_door() -> (Vec3, Vec3) {
    let (_, _, _, _, garage_south, _) = envelope();
    let half = ft(16.0) * 0.5;
    (
        Vec3::new(ft(57.0) - half, 0.0, garage_south - OUTER * 0.5),
        Vec3::new(ft(57.0) + half, 230.0, garage_south + OUTER * 0.5),
    )
}

/// The thickness of an exterior wall, which anything hung on the inside of one
/// needs to know to stand clear of it.
pub const WALL_OUTER: f32 = OUTER;

/// The middle of the footprint. Every window in the house is in an exterior
/// wall, so "which side is the room on" is answered by "the side the middle of
/// the house is on" — which is how curtains work out where to hang.
pub fn centre() -> Vec2 {
    let (lo, hi) = bounds();
    (lo + hi) * 0.5
}

/// The footprint's corners, between finished interior surfaces.
pub fn bounds() -> (Vec2, Vec2) {
    let rooms = rooms();
    let mut lo = rooms[0].min;
    let mut hi = rooms[0].max;
    for r in &rooms {
        lo = lo.min(r.min);
        hi = hi.max(r.max);
    }
    (lo, hi)
}

pub fn window_openings() -> Vec<(Vec3, Vec3)> {
    let (w, _e, n, so, _gs, house_east) = envelope();
    let half = WINDOW_WIDE * 0.5;
    let t = OUTER;
    let mut out = Vec::new();
    for &x in &NORTH_WINDOWS {
        out.push((
            Vec3::new(ft(x) - half, SILL, n - t),
            Vec3::new(ft(x) + half, HEAD, n + t),
        ));
    }
    for &x in &SOUTH_WINDOWS {
        out.push((
            Vec3::new(ft(x) - half, SILL, so - t),
            Vec3::new(ft(x) + half, HEAD, so + t),
        ));
    }
    for &z in &WEST_WINDOWS {
        out.push((
            Vec3::new(w - t, SILL, ft(z) - half),
            Vec3::new(w + t, HEAD, ft(z) + half),
        ));
    }
    out.push((
        Vec3::new(house_east - t, SILL, ft(EAST_WINDOW) - half),
        Vec3::new(house_east + t, HEAD, ft(EAST_WINDOW) + half),
    ));
    out
}

/// Measure what was actually built, and complain if it breaks a law.
///
/// The plan's numbers are not evidence. A room is only fifteen feet if it
/// *measures* fifteen feet once every wall around it has taken its thickness,
/// and the cheapest way to be sure of that forever is to check on every run
/// rather than to be careful once.
/// Is this point within the building's outline?
///
/// The plan is an L: a rectangle with the corner south of the garage bitten
/// out of it.
fn inside_envelope(p: Vec2) -> bool {
    let (w, e, n, so, garage_south, house_east) = envelope();
    // Measured to the outer face of the wall *plus its trim*, not the
    // centreline. Floors are laid a little way into the wall on purpose so no
    // seam can open at a threshold, and the skirting and cornice stand proud of
    // the plaster on both faces; a line drawn down the middle of the wall calls
    // every floorboard and every length of moulding in the house an escapee.
    let m = OUTER * 0.5 + TRIM_PROUD + HAIR;
    if p.x < w - m || p.x > e + m || p.y < n - m || p.y > so + m {
        return false;
    }
    !(p.x > house_east + m && p.y > garage_south + m)
}

pub fn audit(home: &Home) {
    let mut faults = 0;
    let all = rooms();
    for r in &all {
        if r.use_for.habitable() && (r.wide() < MIN_ROOM - HAIR || r.deep() < MIN_ROOM - HAIR) {
            error!(
                "{} is {:.1} x {:.1} cm — under the {:.1} cm minimum",
                r.name,
                r.wide(),
                r.deep(),
                MIN_ROOM
            );
            faults += 1;
        }

        // The ceiling really is at nine feet, sampled across the room rather
        // than at one point, so a missing patch is caught rather than averaged
        // away.
        //
        // Probed from just *under* the ceiling, not up from the floor. The first
        // version cast from ankle height and started failing the moment the
        // house was furnished — it was measuring the top of a bed, an island and
        // a coffee table and calling each of them a low ceiling. Clear height is
        // a property of the room; what is standing on the floor is not the
        // ceiling's business.
        for gx in 1..=3 {
            for gz in 1..=3 {
                let at = Vec2::new(
                    r.min.x + r.wide() * gx as f32 * 0.25,
                    r.min.y + r.deep() * gz as f32 * 0.25,
                );
                let from = Vec3::new(at.x, CEILING - 5.0, at.y);
                match home.raycast(from, Vec3::Y, 40.0) {
                    Some(hit) if (hit.point.y - CEILING).abs() > 1.0 => {
                        error!(
                            "{}: ceiling is {:.1} cm up at ({:.0},{:.0}), not {:.1}",
                            r.name, hit.point.y, at.x, at.y, CEILING
                        );
                        faults += 1;
                    }
                    None => {
                        error!("{}: no ceiling over ({:.0},{:.0})", r.name, at.x, at.y);
                        faults += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    // Nothing stands in *front of* a window.
    //
    // Glass was the only exemption while a window was a single sheet of it.
    // Real ones have sashes, and a sash is not an obstruction — it is the
    // window. What separates the two is which side of the plaster they sit on:
    // a stile lies inside the reveal, and a wall cabinet, a cooker hood or a
    // picture stands proud of it. Everything this law has ever caught was proud
    // of the wall by eight centimetres or more.
    for (lo, hi) in window_openings() {
        let mid = (lo + hi) * 0.5;
        let half = (hi - lo) * 0.5;
        // The opening is much thicker than the wall on purpose; the wall's own
        // plane is its middle, and the thin axis is whichever it is thinnest in.
        let thin_z = (hi.x - lo.x) > (hi.z - lo.z);
        for solid in &home.solids {
            if solid.stuff == Stuff::Glass {
                continue;
            }
            let inside_reveal = if thin_z {
                (solid.center.z - mid.z).abs() + solid.half.z < 4.5
            } else {
                (solid.center.x - mid.x).abs() + solid.half.x < 4.5
            };
            if inside_reveal {
                continue;
            }
            let gap = (solid.center - mid).abs() - (solid.half + half);
            // Overlapping on all three axes, by more than a hair — a wall
            // *around* an opening touches it exactly and is not an obstruction.
            if gap.x < -1.0 && gap.y < -1.0 && gap.z < -1.0 {
                error!(
                    "something is standing in the window at ({:.0},{:.0},{:.0}): \
                     a {:?} box at ({:.0},{:.0},{:.0})",
                    mid.x,
                    mid.y,
                    mid.z,
                    solid.stuff,
                    solid.center.x,
                    solid.center.y,
                    solid.center.z,
                );
                faults += 1;
            }
        }
    }

    // Nothing built indoors may stand outdoors.
    //
    // An exterior view found a shelf unit sticking a metre through the east
    // wall, which no interior capture in the house had been able to show: from
    // inside, furniture that has escaped just looks small. The roof is exempt —
    // hanging past the walls is its whole job.
    for solid in &home.solids {
        // Every solid, not just the ones whose middle is still indoors: a test
        // that skips anything already outside excuses exactly the pieces that
        // got furthest out, and the shelf unit used to prove it walked straight
        // through this check.
        // Things that belong outside are allowed to be outside. That used to
        // be inferred — from the material for planting, and from being no
        // taller than a step for paving — which held right up until there was
        // a house across the road that was neither.
        if solid.roof || solid.outdoors {
            continue;
        }
        let (lo, hi) = (solid.center - solid.half, solid.center + solid.half);
        let escaped = [
            Vec2::new(lo.x, lo.z),
            Vec2::new(hi.x, lo.z),
            Vec2::new(lo.x, hi.z),
            Vec2::new(hi.x, hi.z),
        ]
        .into_iter()
        .find(|c| !inside_envelope(*c));
        if let Some(c) = escaped {
            error!(
                "house fault: something at {:.0},{:.0} reaches outside the walls at {:.0},{:.0}",
                solid.center.x, solid.center.z, c.x, c.y
            );
            faults += 1;
        }
    }

    faults += unreachable(home, &all);

    let living = all.iter().filter(|r| r.use_for.habitable()).count();
    if faults == 0 {
        info!(
            "house: {:.0} x {:.0} cm, {} rooms ({} habitable, all >= {:.0} cm), ceilings {:.2} cm",
            ft(68.0),
            ft(34.5),
            all.len(),
            living,
            MIN_ROOM,
            CEILING
        );
    }
}

/// Can a fly actually get everywhere?
///
/// A blocked doorway and a room walled off by accident look identical to a
/// correct house in every screenshot ever taken of it, because a screenshot only
/// shows the room the camera is in. This flood-fills the open air of the whole
/// house from the great room and reports any room it could not reach.
///
/// Filled on one horizontal slice at 150 cm — above the furniture that stands on
/// the floor, below the head of every door — because that is the height at which
/// the house is a single connected volume if it is connected at all. A fill in
/// three dimensions would be a hundred times the work to answer the same
/// question.
///
/// **Confined to the footprint.** The first version let the fill run outdoors,
/// and it promptly proved a bricked-up laundry door was fine — because a fly can
/// leave by the front door, fly round the house and come back in through the
/// open garage. True, and useless: a law that cannot fail is not a law. Going
/// outside is a route, not a corridor, so the question this asks is whether the
/// house is connected *to itself*.
fn unreachable(home: &Home, all: &[Room]) -> usize {
    const CELL: f32 = 16.0;
    const AT: f32 = 150.0;

    let (lo, hi) = (
        Vec2::new(ft(0.0) - OUTER, ft(0.0) - OUTER),
        Vec2::new(ft(68.0) + OUTER, ft(34.5) + OUTER),
    );
    let wide = (((hi.x - lo.x) / CELL).ceil() as usize).max(1);
    let deep = (((hi.y - lo.y) / CELL).ceil() as usize).max(1);
    let point = |ix: usize, iz: usize| {
        Vec3::new(
            lo.x + (ix as f32 + 0.5) * CELL,
            AT,
            lo.y + (iz as f32 + 0.5) * CELL,
        )
    };

    // Inside the house proper, or inside the garage. Anything else is outdoors.
    let indoors = |p: Vec3| {
        let house = p.x > ft(0.0) && p.x < ft(46.0) && p.z > ft(0.0) && p.z < ft(34.5);
        let garage = p.x > ft(46.0) && p.x < ft(68.0) && p.z > ft(0.0) && p.z < ft(24.0);
        house || garage
    };

    let mut open = vec![false; wide * deep];
    for iz in 0..deep {
        for ix in 0..wide {
            let p = point(ix, iz);
            open[iz * wide + ix] = indoors(p)
                && !home
                    .solids
                    .iter()
                    .any(|solid| solid.nearest(p).distance < 0.0);
        }
    }

    // Start in the great room, which is where a fly hatches.
    let start = room("great room").middle();
    let sx = (((start.x - lo.x) / CELL) as usize).min(wide - 1);
    let sz = (((start.y - lo.y) / CELL) as usize).min(deep - 1);
    let mut seen = vec![false; wide * deep];
    let mut stack = vec![sz * wide + sx];
    seen[sz * wide + sx] = true;
    while let Some(cell) = stack.pop() {
        let (ix, iz) = (cell % wide, cell / wide);
        let mut push = |nx: usize, nz: usize, stack: &mut Vec<usize>| {
            let n = nz * wide + nx;
            if open[n] && !seen[n] {
                seen[n] = true;
                stack.push(n);
            }
        };
        if ix > 0 {
            push(ix - 1, iz, &mut stack);
        }
        if iz > 0 {
            push(ix, iz - 1, &mut stack);
        }
        if ix + 1 < wide {
            push(ix + 1, iz, &mut stack);
        }
        if iz + 1 < deep {
            push(ix, iz + 1, &mut stack);
        }
    }

    let mut faults = 0;
    for r in all {
        // Sample the room rather than trusting its middle, which may have a bed
        // in it.
        let mut found = false;
        for gx in 1..=4 {
            for gz in 1..=4 {
                let at = Vec2::new(
                    r.min.x + r.wide() * gx as f32 * 0.2,
                    r.min.y + r.deep() * gz as f32 * 0.2,
                );
                let ix = (((at.x - lo.x) / CELL) as usize).min(wide - 1);
                let iz = (((at.y - lo.y) / CELL) as usize).min(deep - 1);
                if seen[iz * wide + ix] {
                    found = true;
                }
            }
        }
        if !found {
            error!("{}: a fly cannot reach it from the great room", r.name);
            faults += 1;
        }
    }
    faults
}

// ---------------------------------------------------------------------------
// Light
// ---------------------------------------------------------------------------

/// Authored lights: one fixture per room, plus the sun.
///
/// Authored rather than found. The flood fill in `rooms` exists because a
/// *drawn* house does not say where its rooms are; this one does.
/// A regular octagon lying flat, out of four crossed bars.
///
/// Not two squares at forty-five degrees — that was the first attempt and it
/// unions into an eight-pointed star, because the turned square's corners stand
/// out past the straight one's edges. Four bars the width of the octagon and
/// `2r tan(22.5°)` across, turned forty-five degrees apart, put all eight
/// corners on the same circle. It is the wheel construction from the car, laid
/// on its side.
fn octagon(out: &mut Vec<Solid>, at: Vec3, across: f32, thick: f32, paint: Color, glow: f32) {
    const SIDES: usize = 4;
    let bar = across * (std::f32::consts::PI / (2.0 * SIDES as f32)).tan();
    for k in 0..SIDES {
        let mut s = Solid::between(
            Vec3::new(-across * 0.5, -thick * 0.5, -bar * 0.5),
            Vec3::new(across * 0.5, thick * 0.5, bar * 0.5),
            Stuff::Plaster,
        );
        s.center = at;
        s.rot = Quat::from_rotation_y(k as f32 * std::f32::consts::PI / SIDES as f32);
        s.paint = Some(paint);
        s.glow = glow;
        s.roof = true;
        out.push(s);
    }
}

/// The fixtures themselves: a ceiling rose and a glowing diffuser under it, one
/// per room, at the same place as that room's lamp.
///
/// A room whose light comes from nowhere reads as a room with no light in it,
/// however well lit the floor is — the eye looks for the source. This is also
/// the only thing in the house that emits, which is why `Solid` needed a glow
/// at all.
pub fn fixtures(out: &mut Vec<Solid>) {
    for r in rooms() {
        // The great room has a fan, and the fan carries its own light.
        if r.use_for == Use::Living {
            continue;
        }
        let at = r.middle();
        let wide = if r.use_for == Use::Garage { 52.0 } else { 42.0 };
        octagon(
            out,
            Vec3::new(at.x, CEILING - 2.5, at.y),
            wide + 8.0,
            5.0,
            Color::srgb(0.86, 0.86, 0.84),
            0.0,
        );
        octagon(
            out,
            Vec3::new(at.x, CEILING - 8.0, at.y),
            wide,
            8.0,
            Color::srgb(1.0, 0.97, 0.90),
            11.0,
        );
    }
}

pub fn light_it(commands: &mut Commands) {
    use crate::world::UNITS_PER_METRE;
    let scale = UNITS_PER_METRE * UNITS_PER_METRE;

    commands.insert_resource(bevy::light::DirectionalLightShadowMap { size: 4096 });
    // The sky, which is what a window has to have behind it.
    commands.insert_resource(ClearColor(Color::srgb(0.55, 0.68, 0.84)));

    // Ambient is standing in for bounce light, and it has to stand in for a lot
    // of it. There is no global illumination here, so every photon that would
    // have come off a wall and lit the floor has to arrive as fill or not at
    // all — which is why this is high for a "fill", and why the lamps below are
    // far brighter than household bulbs.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.76, 0.77, 0.80),
        brightness: 105.0,
        ..default()
    });

    // Afternoon, from the south-west and low, so it comes through the front
    // windows and lies along the floor rather than pooling under them.
    commands.spawn((
        Name::new("Afternoon"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.94, 0.84),
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-1200.0, 900.0, ft(52.0))
            .looking_at(Vec3::new(ft(24.0), 40.0, ft(20.0)), Vec3::Y),
        bevy::light::CascadeShadowConfigBuilder {
            num_cascades: 4,
            maximum_distance: 4200.0,
            first_cascade_far_bound: 300.0,
            ..default()
        }
        .build(),
    ));

    for r in rooms() {
        // Not household lumens. A real 1500-lumen bulb at 2.6 m puts about
        // eighteen lux on the floor — which is genuinely what a bare bulb does,
        // and looks like almost nothing here, because a real room is mostly lit
        // by light that has already bounced and nothing bounces in this
        // renderer.
        let (lumens, colour) = match r.use_for {
            Use::Kitchen => (5600.0, Color::srgb(0.98, 0.99, 1.0)),
            Use::Living => (5200.0, Color::srgb(1.0, 0.93, 0.82)),
            Use::Bed => (3000.0, Color::srgb(1.0, 0.90, 0.78)),
            Use::Bath => (3200.0, Color::srgb(0.97, 0.98, 1.0)),
            Use::Utility => (2600.0, Color::srgb(0.96, 0.98, 1.0)),
            Use::Hall => (2400.0, Color::srgb(1.0, 0.93, 0.84)),
            Use::Garage => (2800.0, Color::srgb(0.93, 0.96, 1.0)),
        };
        // A recessed downlight, not a bare bulb.
        //
        // The first pass hung a point light fourteen centimetres under the
        // ceiling, and a point light that close to a surface puts something like
        // ten million lux on it: the ceiling of every room came out pure white.
        // Winding the lamp down would have fixed the ceiling and unlit the floor,
        // because the floor is nearly three metres further away and the falloff
        // is the square of that.
        //
        // A real ceiling fixture solves this by not pointing up. A spot aimed
        // straight down with a wide cone lights the floor and the lower walls,
        // leaves the plaster it is set into alone, and is what a downlight
        // physically is.
        let at = r.middle();
        commands.spawn((
            Name::new(r.name),
            SpotLight {
                color: colour,
                intensity: lumens * scale,
                range: 1100.0,
                radius: 3.0,
                // Wide enough to reach the walls of a fifteen-foot room from
                // nine feet up, and softened over the last fifteen degrees so
                // the pool has an edge rather than a rim.
                outer_angle: 1.25,
                inner_angle: 0.95,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_xyz(at.x, CEILING - 6.0, at.y)
                .looking_at(Vec3::new(at.x, 0.0, at.y), Vec3::Z),
        ));

        // And the light that comes back up off the floor.
        //
        // Nothing bounces in this renderer, so every ceiling in the house was
        // lit by ambient alone: the same value from every angle, over the whole
        // plane, which is exactly what "no shading" looks like and is why the
        // ceilings read as flat grey fields in every interior capture. A wide,
        // dim spot from about table height aimed straight up is a cheap stand-in
        // for the one bounce that matters — it comes from where the real light
        // would come from, and it falls off toward the corners the way real
        // bounce does. No shadow map: bounce light does not have a sharp one.
        commands.spawn((
            Name::new(format!("{} bounce", r.name)),
            SpotLight {
                color: colour,
                intensity: lumens * scale * 0.85,
                range: 700.0,
                radius: 40.0,
                outer_angle: 1.45,
                inner_angle: 0.30,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(at.x, 92.0, at.y)
                .looking_at(Vec3::new(at.x, CEILING, at.y), Vec3::Z),
        ));
    }
}
