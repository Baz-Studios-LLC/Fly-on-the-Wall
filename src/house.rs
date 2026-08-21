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

/// The tallest ceiling in the house, ten feet, for anything that needs one
/// number rather than a room's own.
///
/// Heights are **per room** now. The plan is deliberately mixed — ten feet
/// through the living areas, master suite, kitchen, dining and garage, nine in
/// the secondary bedrooms, baths, office and rear porch — and the variation is
/// worth having: a fly crossing from a nine-foot bedroom into the great room
/// should feel the volume change.
pub const CEILING: f32 = 10.0 * FOOT;

/// A tolerance for the laws: half a millimetre.
///
/// Far below anything a fly could find and far above `f32` noise.
const HAIR: f32 = 0.05;

/// Centimetres per foot.
const FOOT: f32 = 30.48;

/// **One.** The house is built at the dimensions printed on the drawing.
///
/// This was `15 / 11.5`, stretching the whole plan so its tightest bedroom
/// reached a fifteen-foot minimum. That minimum is gone: the house is built to
/// `assets/FloorPlan.jpg` now, where only the great room would have passed it.
/// Kept as a named constant rather than deleted, because a scale factor
/// silently applied to a drawing is exactly the kind of thing that wants to be
/// visible when somebody wonders why a room measures what it measures.
const SCALE: f32 = 1.0;

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
    ///
    /// These *are* the dimensions printed on the drawing. The plan gives clear
    /// interior sizes, so the printed figure is the room and the walls go
    /// outside it — which is what makes "built exactly to the plan" true in the
    /// sense the plan means it.
    pub min: Vec2,
    pub max: Vec2,
    /// Finished floor to finished ceiling, in centimetres.
    pub tall: f32,
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

/// Interior partition thickness, in feet, for chaining the plan.
const PARTITION: f32 = INNER / FOOT;

/// The drawing, as nine columns of stacked rooms.
///
/// `(clear width, where the stack starts, [(name, use, ceiling, clear depth)])`,
/// all in **feet**, `y` running from the back of the house to the front.
///
/// Every printed figure is here. Nothing is a position: widths and depths are
/// chained with a partition between each, so adjacent rooms share a wall
/// exactly and **two rooms cannot overlap by construction**. Sixteen
/// hand-entered rectangles could, and did — by between an inch and seven.
/// A wall built across a nine-centimetre overlap has a nine-centimetre seam in
/// it, and at fly scale that is a doorway.
///
/// The printed figures tile, which is the evidence the columns are real:
/// **garage 23'0" = master bath 8'8" + partition + master bedroom 14'0"**, and
/// **rear porch 30'4" = kitchen 12'0" + partition + great room 18'0"**, and the
/// whole chain lands the east face at 65.24 ft — exactly where the drawing's
/// east wall measures. (At 16.82 px/ft; the 17.087 used before this was
/// calibrated against the *previous* drawing and put every middle room a foot
/// and a half too far south.)
///
/// Where the drawing is narrower than a column, the narrow figure went in and
/// the printed room became a `SPANS` entry across several columns: the laundry
/// is 10'-0" across columns 1–2, the master bedroom 14'-0" across 1–3. The two
/// interior halls have no printed figures and are measured off the drawing —
/// the mudroom hall (garage → laundry → pantry → master bedroom → kitchen) and
/// the wing hall (great room → both bedrooms, bath two, linen).
type Stack = &'static [(&'static str, Use, f32, f32)];
const COLUMNS: [(f32, f32, Stack); 9] = [
    // 0: the master suite's outer column.
    (
        8.667,
        0.0,
        &[
            ("master bath", Use::Bath, 10.0, 13.0),
            ("master closet", Use::Utility, 10.0, 8.5),
        ],
    ),
    // 1: nothing of its own — the west part of the laundry, between the
    // storage bay and the pantry. Everything on it is a span.
    (4.02, 0.0, &[]),
    // 2: the pantry, its printed 5'-7" width.
    (5.58, 22.11, &[("pantry", Use::Utility, 10.0, 4.5)]),
    // 3: the mudroom hall, measured. It is what the second BARN DOOR on the
    // drawing opens from, and the only way from the garage into the house.
    (3.61, 15.39, &[("mud hall", Use::Hall, 10.0, 11.57)]),
    (
        12.0,
        18.0,
        &[
            ("kitchen", Use::Kitchen, 10.0, 16.833),
            ("dining", Use::Living, 10.0, 10.333),
        ],
    ),
    (
        18.0,
        18.0,
        &[
            ("great room", Use::Living, 10.0, 19.167),
            ("front porch", Use::Hall, 10.0, 8.0),
        ],
    ),
    // 6: the wing hall, measured. Off the great room's north-east corner,
    // serving everything in the wing.
    (3.4, 9.8, &[("wing hall", Use::Hall, 9.0, 12.49)]),
    // 7: the band the plan labels `LIN.`, measured.
    (2.0, 12.16, &[("linen", Use::Utility, 9.0, 2.29)]),
    // 8: bath two's tub stands in its own nook east of the linen closet,
    // open to the rest of the bath.
    (4.81, 12.16, &[("bath two tub", Use::Bath, 9.0, 2.29)]),
];

/// Rooms that cover more than one column: `(name, use, ceiling, first column,
/// last column, y, depth)`.
///
/// The wing's stacks chain in `y` exactly as the columns do in `x`: bedroom
/// three, its closet, the linen band, bath two, bedroom two's closet, bedroom
/// two, the office — and the chain lands the office's south face at 43.40 ft,
/// where the drawing measures it. The wing also starts 1.66 ft north of the
/// main block, which the drawing shows as bedroom three projecting past the
/// master suite's rear wall.
const SPANS: [(&str, Use, f32, usize, usize, f32, f32); 11] = [
    ("master bedroom", Use::Bed, 10.0, 1, 3, 0.0, 15.0),
    ("laundry", Use::Utility, 10.0, 1, 2, 15.39, 6.333),
    // Its printed 13'-1" is the width of the garage's north wall that has no
    // wall in it: a storage bay open to the garage, not a room off the house.
    ("open storage", Use::Utility, 10.0, 0, 1, 22.29, 4.667),
    ("garage", Use::Garage, 10.0, 0, 3, 27.35, 23.0),
    ("rear porch", Use::Hall, 9.0, 4, 5, 7.61, 10.0),
    ("bedroom three", Use::Bed, 9.0, 6, 8, -1.66, 11.07),
    ("closet three", Use::Utility, 9.0, 7, 8, 9.8, 1.97),
    ("bath two", Use::Bath, 9.0, 7, 8, 14.84, 4.9),
    ("closet two", Use::Utility, 9.0, 7, 8, 20.13, 2.16),
    ("bedroom two", Use::Bed, 9.0, 6, 8, 22.68, 11.0),
    ("office", Use::Bed, 9.0, 6, 8, 34.07, 9.33),
];

/// What a hole in a wall is.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Cut {
    /// Sill and head heights above the floor, in centimetres.
    Pane { sill: f32, head: f32 },
    /// A doorway, open to the floor: hinged, sliding, bifold or overhead.
    Way { head: f32 },
    /// A cased opening with no leaf in it.
    Arch { head: f32 },
    /// Open plan: the wall simply is not there over this stretch.
    Wide,
}

/// A standard window: sill and head, in centimetres. The photographs of the
/// built house have low sills and tall glass — window heads aligned with the
/// door heads right across a facade.
const PANE: Cut = Cut::Pane {
    sill: 70.0,
    head: 213.0,
};
/// A high-silled window: over the kitchen counter, over a bath.
const HIGH_PANE: Cut = Cut::Pane {
    sill: 130.0,
    head: 213.0,
};
const WAY: Cut = Cut::Way { head: 205.0 };

/// Every opening in every wall, measured off the drawing, in feet.
///
/// `(room, edge, other, from, to, what)` — `edge` is which side of `room` the
/// opening is in (`n`/`s`/`e`/`w` in drawing orientation, north at the top);
/// `other` is the room on the far side, or `"outside"`; `from`/`to` are
/// absolute plan coordinates along the shared edge — x for a north or south
/// wall, y for an east or west one — so every figure here can be checked
/// against the drawing directly.
///
/// This table is the *single* authority: the walls cut these holes, the
/// glazing fills exactly the holes the walls cut, and the audit checks the
/// same boxes. The last window bug in this house was two authorities — a wall
/// system cutting holes where the runs were, and a glazing table hanging glass
/// where the old envelope used to be — agreeing on nothing.
const PORTALS: [(&str, char, &str, f32, f32, Cut); 32] = [
    // -- Windows, exterior ---------------------------------------------
    // The master bath's tub window: the drawing's only west opening, and the
    // photos' near-square pair of sashes over the tub.
    (
        "master bath",
        'w',
        "outside",
        3.98,
        7.97,
        Cut::Pane {
            sill: 106.0,
            head: 213.0,
        },
    ),
    // The pair flanking the master bedroom's rear gable centreline.
    ("master bedroom", 'n', "outside", 9.69, 12.37, PANE),
    ("master bedroom", 'n', "outside", 19.74, 22.41, PANE),
    ("bedroom three", 'n', "outside", 58.44, 60.94, PANE),
    ("bath two", 'e', "outside", 15.70, 17.72, HIGH_PANE),
    ("bedroom two", 'e', "outside", 23.25, 25.92, PANE),
    ("office", 's', "outside", 58.44, 60.94, PANE),
    // The dining room's front: one wide unit the glazing mullions into the
    // photographs' three equal sashes.
    ("dining", 's', "outside", 25.10, 33.50, PANE),
    (
        "great room",
        's',
        "front porch",
        47.62,
        51.13,
        Cut::Pane {
            sill: 25.0,
            head: 213.0,
        },
    ),
    // Onto the rear porch: the kitchen's high window over the sink, and the
    // great room's three.
    ("kitchen", 'n', "rear porch", 26.22, 29.25, HIGH_PANE),
    (
        "great room",
        'n',
        "rear porch",
        41.50,
        44.47,
        Cut::Pane {
            sill: 25.0,
            head: 213.0,
        },
    ),
    (
        "great room",
        'n',
        "rear porch",
        45.01,
        47.98,
        Cut::Pane {
            sill: 25.0,
            head: 213.0,
        },
    ),
    (
        "great room",
        'n',
        "rear porch",
        48.51,
        51.49,
        Cut::Pane {
            sill: 25.0,
            head: 213.0,
        },
    ),
    // -- Doors to the outside ------------------------------------------
    // The double front door, off the front porch.
    ("great room", 's', "front porch", 37.57, 42.57, WAY),
    // The single glass door onto the rear porch.
    ("great room", 'n', "rear porch", 36.21, 39.18, WAY),
    // The sixteen-foot overhead door, centred on the garage.
    (
        "garage",
        's',
        "outside",
        3.53,
        19.53,
        Cut::Way { head: 213.0 },
    ),
    // -- The master suite ----------------------------------------------
    // Barn door on the drawing; a walk-through suite: bedroom, bath, closet.
    ("master bedroom", 'w', "master bath", 5.71, 8.38, WAY),
    ("master bath", 's', "master closet", 3.98, 6.42, WAY),
    // -- The mudroom hall ----------------------------------------------
    ("master bedroom", 's', "mud hall", 19.92, 22.59, WAY),
    ("laundry", 'e', "mud hall", 15.64, 18.67, WAY),
    ("pantry", 'e', "mud hall", 23.25, 25.27, WAY),
    ("garage", 'n', "mud hall", 19.74, 22.77, WAY),
    // The drawing's second barn door, into the kitchen.
    ("kitchen", 'w', "mud hall", 20.81, 23.78, WAY),
    // -- The open plan ---------------------------------------------------
    // Kitchen and great room are one space south of the refrigerator stub.
    ("kitchen", 'e', "great room", 20.57, 34.83, Cut::Wide),
    // -- The wing --------------------------------------------------------
    (
        "great room",
        'e',
        "wing hall",
        18.79,
        21.82,
        Cut::Arch { head: 244.0 },
    ),
    ("great room", 'e', "office", 34.28, 36.95, WAY),
    ("bedroom three", 's', "wing hall", 54.58, 57.25, WAY),
    ("bedroom two", 'n', "wing hall", 54.58, 57.25, WAY),
    ("bath two", 'w', "wing hall", 15.16, 17.48, WAY),
    ("linen", 'w', "wing hall", 12.54, 14.03, WAY),
    // The bedroom closets' five-foot bifold openings, mirrored.
    (
        "bedroom three",
        's',
        "closet three",
        59.39,
        64.39,
        Cut::Arch { head: 205.0 },
    ),
    (
        "bedroom two",
        'n',
        "closet two",
        59.39,
        64.39,
        Cut::Arch { head: 205.0 },
    ),
];

/// Where each column's clear interior starts and ends, in feet.
fn column(i: usize) -> (f32, f32) {
    let mut x = 0.0;
    for (n, (wide, _, _)) in COLUMNS.iter().enumerate() {
        if n == i {
            return (x, x + wide);
        }
        x += wide + PARTITION;
    }
    (x, x)
}

/// Which room a point is in, if any.
pub fn room_at(p: Vec2) -> Option<Room> {
    rooms()
        .into_iter()
        .find(|r| p.x > r.min.x && p.x < r.max.x && p.y > r.min.y && p.y < r.max.y)
}

/// What colour a room is painted.
///
/// Every wall in this house was the same grey, which is the strongest single
/// reason the rooms all read alike however differently they were furnished. A
/// family does not paint a house one colour: the living rooms go warm, the wet
/// rooms go cool, and the children get to choose.
fn wall_colour(r: &Room) -> Color {
    match (r.use_for, r.name) {
        (Use::Living, _) => Color::srgb(0.86, 0.83, 0.76),
        (Use::Kitchen, _) => Color::srgb(0.88, 0.87, 0.82),
        (Use::Hall, _) => Color::srgb(0.83, 0.81, 0.78),
        // The children's bath goes cool; the master stays warm-neutral. The
        // tub nook is part of bath two and must match it.
        (Use::Bath, "master bath") => Color::srgb(0.82, 0.85, 0.83),
        (Use::Bath, _) => Color::srgb(0.78, 0.84, 0.85),
        (Use::Bed, "master bedroom") => Color::srgb(0.80, 0.80, 0.78),
        (Use::Bed, "bedroom two") => Color::srgb(0.76, 0.81, 0.85),
        (Use::Bed, _) => Color::srgb(0.85, 0.82, 0.74),
        (Use::Utility, _) => Color::srgb(0.87, 0.88, 0.87),
        (Use::Garage, _) => Color::srgb(0.79, 0.79, 0.77),
    }
}

pub fn rooms() -> Vec<Room> {
    let mut out = Vec::new();
    for (i, (_, from, stack)) in COLUMNS.iter().enumerate() {
        let (x0, x1) = column(i);
        let mut y = *from;
        for &(name, use_for, tall, deep) in stack.iter() {
            out.push(Room {
                name,
                use_for,
                min: Vec2::new(ft(x0), ft(y)),
                max: Vec2::new(ft(x1), ft(y + deep)),
                tall: ft(tall),
            });
            y += deep + PARTITION;
        }
    }
    for &(name, use_for, tall, first, last, y, deep) in SPANS.iter() {
        out.push(Room {
            name,
            use_for,
            min: Vec2::new(ft(column(first).0), ft(y)),
            max: Vec2::new(ft(column(last).1), ft(y + deep)),
            tall: ft(tall),
        });
    }
    out
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

    let mut cuts: Vec<Opening> = holes.to_vec();
    cuts.sort_by(|p, q| p.at.partial_cmp(&q.at).unwrap());

    // A footing under the run, from below the grass up to the house's floor
    // level — except across an open-plan stretch, where there is no wall to
    // seal under and the saddle below owns the gap.
    //
    // Every wall used to start at zero, which is right for every room whose
    // floor is also at zero — and wrong for the garage, whose slab is a step
    // down. That six-centimetre difference was a six-centimetre slot running
    // right around the garage with daylight and grass showing through it. A
    // stem wall is what a real slab sits against, and above the garage floor it
    // reads as exactly that.
    {
        let foot = |s0: f32, e0: f32, out: &mut Vec<Solid>| {
            if e0 - s0 < 0.5 {
                return;
            }
            let (min, max) = if along_x {
                (
                    Vec3::new(a.x + s0, -FOOTING, a.y - half),
                    Vec3::new(a.x + e0, 0.0, a.y + half),
                )
            } else {
                (
                    Vec3::new(a.x - half, -FOOTING, a.y + s0),
                    Vec3::new(a.x + half, 0.0, a.y + e0),
                )
            };
            out.push(Solid::between(min, max, Stuff::Stone));
        };
        let mut walked = 0.0;
        for hole in &cuts {
            if hole.sill <= 0.01 && hole.head >= height - 0.01 {
                foot(walked, hole.at - hole.wide * 0.5, out);
                walked = hole.at + hole.wide * 0.5;
            }
        }
        foot(walked, length, out);
    }

    // A saddle across the gap in every floor-level opening. The gap between
    // two rooms is covered by both rooms' floors and by the footing, all
    // coplanar at the finished floor — invisible under a wall, and a stripe of
    // z-fighting in a doorway. A threshold is what a real doorway has there.
    for hole in &cuts {
        if hole.sill > 0.01 {
            continue;
        }
        let (s0, e0) = (hole.at - hole.wide * 0.5, hole.at + hole.wide * 0.5);
        let deep = half + 0.6;
        let (min, max) = if along_x {
            (
                Vec3::new(a.x + s0, -SLAB, a.y - deep),
                Vec3::new(a.x + e0, 0.25, a.y + deep),
            )
        } else {
            (
                Vec3::new(a.x - deep, -SLAB, a.y + s0),
                Vec3::new(a.x + deep, 0.25, a.y + e0),
            )
        };
        let stone = thick >= OUTER - 0.01;
        let mut saddle = Solid::between(min, max, if stone { Stuff::Stone } else { Stuff::Wood });
        saddle.paint = Some(if stone {
            Color::srgb(0.66, 0.67, 0.65)
        } else {
            Color::srgb(0.52, 0.40, 0.28)
        });
        out.push(saddle);
    }

    let piece = |s: f32, e: f32, low: f32, high: f32, out: &mut Vec<Solid>| {
        if e - s < 0.01 || high - low < 0.01 {
            return;
        }
        // The wall, in two halves, each painted for the room its face looks
        // into.
        //
        // The first version left the wall whole and stuck a paint skin half a
        // centimetre proud of each side, and those skins z-fought with the wall
        // behind them: fine vertical banding across whole pieces at grazing
        // angles. Splitting the wall removes the coplanar pair entirely and
        // costs nothing — it is the same volume and the same box count.
        //
        // Which room a face looks into comes from the same trick the cladding
        // uses to find the outdoors: a probe thirty centimetres off it.
        for face in [-1.0f32, 1.0] {
            let probe = if along_x {
                Vec2::new((a.x + s + a.x + e) * 0.5, a.y + face * (half + 30.0))
            } else {
                Vec2::new(a.x + face * (half + 30.0), (a.y + s + a.y + e) * 0.5)
            };
            let (hmin, hmax) = if along_x {
                (
                    Vec3::new(a.x + s, low, a.y),
                    Vec3::new(a.x + e, high, a.y + face * half),
                )
            } else {
                (
                    Vec3::new(a.x, low, a.y + s),
                    Vec3::new(a.x + face * half, high, a.y + e),
                )
            };
            let mut leaf = Solid::between(hmin.min(hmax), hmin.max(hmax), stuff);
            // A face looking into a room wears that room's paint; a face
            // looking outdoors — truly outside, or onto a porch — is the
            // photographs' white siding.
            leaf.paint = Some(match room_at(probe) {
                Some(room) if !outdoors(room.name) => wall_colour(&room),
                _ => Color::srgb(0.92, 0.92, 0.93),
            });
            out.push(leaf);
        }

        // Board-and-batten, on whichever face of an exterior wall looks
        // outward — including the walls at the back of each porch.
        //
        // The photographs of the built house are unanimous: white vertical
        // board-and-batten on every elevation, battens on roughly 16-inch
        // centres, siding running to grade. It replaced lap boards, which were
        // an invention. The flat board field is the wall face itself; the
        // battens are what throw the vertical shadow lines that put a scale on
        // an elevation. "Exterior" is not a flag anyone has to remember to set
        // — it is a probe thirty centimetres off each face, asking whether
        // that is a room.
        if thick >= OUTER - 0.01 {
            for face in [-1.0f32, 1.0] {
                let probe = if along_x {
                    Vec2::new((a.x + s + a.x + e) * 0.5, a.y + face * (half + 30.0))
                } else {
                    Vec2::new(a.x + face * (half + 30.0), (a.y + s + a.y + e) * 0.5)
                };
                if room_at(probe).is_some_and(|room| !outdoors(room.name)) {
                    continue;
                }
                // Battens land on a global grid, so they stay aligned across
                // the separate pieces either side of a window and across
                // separate runs of the same facade.
                const SPACE: f32 = 34.0;
                const BATTEN: f32 = 6.5;
                let along0 = if along_x { a.x } else { a.y };
                let mut k = ((along0 + s + BATTEN * 0.5) / SPACE).ceil();
                while k * SPACE < along0 + e - BATTEN * 0.5 {
                    let at = k * SPACE - along0;
                    k += 1.0;
                    let (bmin, bmax) = if along_x {
                        (
                            Vec3::new(a.x + at - BATTEN * 0.5, low, a.y + face * half),
                            Vec3::new(a.x + at + BATTEN * 0.5, high, a.y + face * (half + 2.2)),
                        )
                    } else {
                        (
                            Vec3::new(a.x + face * half, low, a.y + at - BATTEN * 0.5),
                            Vec3::new(a.x + face * (half + 3.0), high, a.y + at + BATTEN * 0.5),
                        )
                    };
                    let mut batten = Solid::between(bmin.min(bmax), bmin.max(bmax), Stuff::Wood);
                    batten.paint = Some(Color::srgb(0.93, 0.93, 0.94));
                    out.push(batten);
                }
            }
        }

        // Cornice, on any face that reaches its own room's ceiling.
        //
        // Per face, off the same probe as the paint: the two rooms either side
        // of one wall can have different ceilings, and a 9-foot room's cornice
        // belongs at 9 feet even when the wall itself carries on up to a
        // 10-foot neighbour. An outdoor face gets none — eaves are the roof's
        // job.
        for face in [-1.0f32, 1.0] {
            let probe = if along_x {
                Vec2::new((a.x + s + a.x + e) * 0.5, a.y + face * (half + 30.0))
            } else {
                Vec2::new(a.x + face * (half + 30.0), (a.y + s + a.y + e) * 0.5)
            };
            let Some(room) = room_at(probe) else {
                continue;
            };
            if outdoors(room.name) {
                continue;
            }
            let crown = room.tall.min(height);
            if high < crown - 0.01 || low >= crown - CORNICE_HIGH {
                continue;
            }
            // Stepped, like the skirting. A cornice run as one square
            // section reads as a stripe of paint where the wall meets the
            // ceiling; two steps and it reads as a moulding, because the
            // lower one is in shadow and the upper one is not.
            for (bottom, top, proud) in [
                (
                    crown - CORNICE_HIGH,
                    crown - CORNICE_HIGH + 2.4,
                    CORNICE_PROUD - 1.4,
                ),
                (crown - CORNICE_HIGH + 2.4, crown, CORNICE_PROUD),
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

        // Skirting, on any piece that reaches the floor.
        //
        // Emitted here rather than around the room afterwards, because a run
        // already knows where its solid parts are: a doorway gets no skirting
        // across it for free, and nothing has to be told where the openings
        // were. Both faces, since a partition has a room on each side.
        if low <= 0.01 && high > SKIRT_HIGH {
            for face in [-1.0f32, 1.0] {
                // Only indoors: outside, the siding runs to grade.
                let probe = if along_x {
                    Vec2::new((a.x + s + a.x + e) * 0.5, a.y + face * (half + 30.0))
                } else {
                    Vec2::new(a.x + face * (half + 30.0), (a.y + s + a.y + e) * 0.5)
                };
                if !room_at(probe).is_some_and(|room| !outdoors(room.name)) {
                    continue;
                }
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

/// A room's floor: boards, tile or concrete, by what the room is for.
///
/// Strips rather than one plane. It costs a few dozen solids per room, tiles the
/// same rectangle so collision is unchanged, and it is the difference between a
/// floor and a colour — the largest single surface in any shot of any room.
fn lay_floor(out: &mut Vec<Solid>, r: &Room) {
    let (stuff, base, run, tone, cross) = match (r.name, r.use_for) {
        // The front porch is brick, per the photographs, in tight courses.
        ("front porch", _) => (
            Stuff::Stone,
            Color::srgb(0.56, 0.39, 0.32),
            20.0,
            0.05,
            true,
        ),
        // The rear porch is a broomed concrete slab.
        ("rear porch", _) => (
            Stuff::Stone,
            Color::srgb(0.62, 0.62, 0.60),
            120.0,
            0.03,
            false,
        ),
        // Tile, in squarer courses and a cooler colour.
        (_, Use::Bath | Use::Utility) => (
            Stuff::Stone,
            Color::srgb(0.72, 0.73, 0.71),
            34.0,
            0.05,
            true,
        ),
        // The garage is a poured slab: no courses at all, just a little
        // mottling, and it sits lower than the house.
        (_, Use::Garage) => (
            Stuff::Stone,
            Color::srgb(0.44, 0.44, 0.46),
            110.0,
            0.03,
            false,
        ),
        // Mid oak, not the near-walnut this was. A dark floor and a dark rug
        // between them made the bottom half of every interior one brown mass,
        // and the floor is the largest surface in any room — it sets the key
        // for everything standing on it.
        _ => (
            Stuff::Floorboard,
            Color::srgb(0.58, 0.44, 0.30),
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
        ft(0.0) - h,   // west
        ft(65.24) + h, // east
        ft(-1.66) - h, // north — the wing projects past the main block
        ft(50.35) + h, // south, the garage front: the furthest the house reaches
        ft(50.35) + h, // the garage's south wall
        ft(23.06) + h, // where the west bar ends
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
            if room_at(probe).is_some_and(|room| !outdoors(room.name)) {
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
            let (sill_y, head_y) = (lo.y, hi.y);
            let high = head_y - sill_y;
            let cy = (sill_y + head_y) * 0.5;
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
                    head_y + board * 0.5,
                    out_face,
                    wide + board * 2.0,
                    board,
                    6.0,
                );
                put(
                    mid_x,
                    sill_y - 4.0,
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
                    head_y + board * 0.5,
                    mid_z,
                    6.0,
                    board,
                    wide + board * 2.0,
                );
                put(
                    out_face + face * 1.5,
                    sill_y - 4.0,
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
    let (_w, _e, _n, so, _gs, _he) = envelope();
    let joint = Color::srgb(0.46, 0.46, 0.44);

    let mut pave = |min: Vec3, max: Vec3, tint: Color| {
        let mut s = Solid::between(min, max, Stuff::Stone);
        s.paint = Some(tint);
        s.outdoors = true;
        out.push(s);
    };

    // The drive, in bays, running out from under the garage door to the
    // sidewalk. Centred on the garage because that is what a drive is for —
    // the old one was authored against the previous plan and missed the
    // garage by ten feet.
    let garage = room("garage");
    let (dx, drive_half) = (garage.middle().x, ft(16.0) * 0.5 + 40.0);
    let drive_end = so + 900.0;
    const BAYS: usize = 6;
    pave(
        Vec3::new(dx - drive_half, -11.0, garage.max.y),
        Vec3::new(dx + drive_half, -2.0, drive_end),
        joint,
    );
    for k in 0..BAYS {
        let run = (drive_end - garage.max.y) / BAYS as f32;
        let z0 = garage.max.y + run * k as f32;
        pave(
            Vec3::new(dx - drive_half + 3.0, -11.0, z0 + 3.0),
            Vec3::new(dx + drive_half - 3.0, -1.2, z0 + run - 3.0),
            Color::srgb(
                0.63 + grain(dx + z0 * 1.7) * 0.02,
                0.62 + grain(z0 + dx * 1.7) * 0.02,
                0.59,
            ),
        );
    }

    // The walk, from the drive across to the porch steps.
    let porch = room("front porch");
    let door_x = (ft(37.57) + ft(42.57)) * 0.5;
    let walk = porch.max.y + 120.0;
    pave(
        Vec3::new(dx + drive_half - 10.0, -11.0, walk - 62.0),
        Vec3::new(door_x + 62.0, -1.4, walk + 62.0),
        joint,
    );
    // The spur up to the steps.
    pave(
        Vec3::new(door_x - 62.0, -11.0, porch.max.y + 24.0),
        Vec3::new(door_x + 62.0, -1.4, walk),
        joint,
    );
    // Two brick steps the full width of the porch mouth, up from the walk to
    // the porch slab — the photographs' brick steps, in the same muted
    // terracotta as the porch floor now wears.
    let brick = Color::srgb(0.56, 0.39, 0.32);
    pave(
        Vec3::new(porch.min.x + 30.0, -11.0, porch.max.y),
        Vec3::new(porch.max.x - 30.0, -4.5, porch.max.y + 24.0),
        brick,
    );

    // Foundation planting: low mulched beds with shrub masses, sited clear of
    // every opening, and a pair of narrow uprights flanking the garage door.
    //
    // Five boxes a shrub rather than three, turned to different angles and
    // overlapping hard, because a plant wants to read as one mass with a
    // ragged edge. They are `Stuff::Grass`, which is both what they are and
    // how they get past the envelope law.
    let mulch = Color::srgb(0.24, 0.18, 0.13);
    let shrub = |x: f32, z_wall: f32, tall: f32, girth: f32, out: &mut Vec<Solid>| {
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
            let wide = girth * spread * (1.0 + n * 0.10);
            let y = tall * lift;
            let at = Vec3::new(x + n * 6.0, y, z_wall + girth * 0.5 + 14.0 + n * 5.0);
            let mut bush = Solid::between(
                at - Vec3::new(wide * 0.5, tall * 0.20, wide * 0.5),
                at + Vec3::new(wide * 0.5, tall * 0.20, wide * 0.5),
                Stuff::Grass,
            );
            bush.outdoors = true;
            bush.rot = Quat::from_rotation_y(k as f32 * 0.55 + n * 0.4);
            bush.paint = Some(Color::srgb(
                0.13 + k as f32 * 0.014,
                0.23 + k as f32 * 0.022,
                0.13 + k as f32 * 0.010,
            ));
            out.push(bush);
        }
    };

    // (bed centre x in feet, which wall face it stands against, height, girth)
    let garage_face = garage.max.y;
    let dining_face = ft(45.56) + OUTER;
    let wing_face = ft(43.40) + OUTER;
    for (x, face, tall, girth) in [
        // Uprights flanking the garage door, like the photographs' cypresses.
        (2.0, garage_face, 190.0, 44.0),
        (21.0, garage_face, 190.0, 44.0),
        // Under the dining triple, kept below its low sill.
        (26.8, dining_face, 52.0, 70.0),
        (31.8, dining_face, 55.0, 74.0),
        // The east wing's front, clear of the office window.
        (55.9, wing_face, 66.0, 78.0),
        (63.4, wing_face, 82.0, 84.0),
    ] {
        let cx = ft(x);
        let mut bed = Solid::between(
            Vec3::new(cx - girth * 0.9, -9.0, face),
            Vec3::new(cx + girth * 0.9, -6.0, face + girth + 34.0),
            Stuff::Stone,
        );
        bed.paint = Some(mulch);
        bed.outdoors = true;
        out.push(bed);
        shrub(cx, face, tall, girth, out);
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

    // Far enough out that a kitchen window shows a garden rather than a fence.
    // At six metres the boards filled the glass from sill to head.
    let fence_z = n - 980.0;

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
        // Their roofs are outdoors too. They are built by the same constructor
        // the real house uses, which marks its work as roof and nothing else —
        // and the plan view frames on everything that is not outdoors, so five
        // neighbours' roofs were pulling the whole drawing out to a thumbnail.
        let before = out.len();
        gable_roof(out, min, max, along_x, top + SLAB + 8.0, PITCH);
        for solid in &mut out[before..] {
            solid.outdoors = true;
        }
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
    // Backing onto the garden. Their roofs over the fence are most of what a
    // north-facing window in a street like this ever shows.
    for (x0, wide, back) in [(w - 300.0, 1700.0, 900.0), (w + 1900.0, 1520.0, 1120.0)] {
        let z1 = fence_z - back;
        neighbour(
            out,
            Vec2::new(x0, z1 - 1020.0),
            Vec2::new(x0 + wide, z1),
            true,
            grain(x0) * 0.4,
        );
    }
    neighbour(
        out,
        Vec2::new(e + 940.0, n + 300.0),
        Vec2::new(e + 2500.0, n + 1860.0),
        false,
        -0.20,
    );

    for (x, z, tall) in [
        // One in the back garden, so the north windows have something in them
        // that is not fence.
        (ft(38.0), n - 620.0, 540.0),
        (w + 260.0, n - 780.0, 470.0),
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

const SHINGLE: Color = Color::srgb(0.16, 0.16, 0.17);
const TRIM: Color = Color::srgb(0.90, 0.89, 0.85);
const SIDING: Color = Color::srgb(0.90, 0.90, 0.91);

/// A gabled roof over one rectangle, ridge along whichever axis is asked for.
///
/// Everything here is still a box: the two slopes are boxes with a turn on
/// them, and the triangular gable ends are courses of box, each stopping where
/// the slope above it would cut through. Stepping a triangle out of rectangles
/// leaves a staircase along the rake, which is what the barge boards are for —
/// they are a real piece of a real roof and they cover the one artifact this
/// construction cannot avoid.
fn gable_roof(out: &mut Vec<Solid>, lo: Vec2, hi: Vec2, along_x: bool, eave: f32, pitch: f32) {
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
    let rise = half * pitch;
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
        let base = eave - SLAB - 8.0;
        let wall_half = half - OVERHANG;
        for k in 0..COURSES {
            let y0 = base + (ridge_y - ROOF_THICK - base) * k as f32 / COURSES as f32;
            let y1 = base + (ridge_y - ROOF_THICK - base) * (k + 1) as f32 / COURSES as f32;
            // Measured at the course's *bottom*, so each one runs a little way
            // up into the slab above it rather than stopping under it. Cutting
            // at the top edge instead leaves a triangular gap per course, and
            // eighteen of those in a row is a row of shark's teeth along the
            // rake — which is exactly how the first capture came back.
            let reach = ((ridge_y - ROOF_THICK - y0) / pitch).clamp(0.0, wall_half);
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

/// The roof, in the three volumes the photographs of the built house show.
///
/// One north-south ridge runs the whole west bar, from the master bedroom's
/// rear gable to the garage's front one — the drawing makes them the same
/// width, which is how the photos can show one clean gable at each end. The
/// main east-west ridge covers the middle block, its rear slope running down
/// over the recessed rear porch and its front slope over the front porch, so
/// both porches stand under the main roof exactly as built. The east wing
/// carries its own lower ridge over its 9-foot walls, which is the step down
/// every photograph shows. The middle ridge stands tallest, the bar next, the
/// wing lowest.
fn roof(out: &mut Vec<Solid>) {
    let h = OUTER;
    let garage = room("garage");
    let porch_front = room("front porch");
    let porch_rear = room("rear porch");
    let wing_n = room("bedroom three");
    let wing_s = room("office");

    // The west bar: master suite through garage, gables north and south,
    // steep like the photographs' garage gable.
    gable_roof(
        out,
        Vec2::new(-h, -h),
        Vec2::new(garage.max.x + INNER, garage.max.y + h),
        false,
        ft(10.0) + SLAB + 8.0,
        9.0 / 12.0,
    );
    // The middle block: ridge east-west, eaves over both porches. Its gable
    // ends rise above the bar and the wing as clad triangles.
    gable_roof(
        out,
        Vec2::new(garage.max.x, porch_rear.min.y),
        Vec2::new(wing_n.min.x, porch_front.max.y + h),
        true,
        ft(10.0) + SLAB + 8.0,
        6.0 / 12.0,
    );
    // The east wing continues the main ridge east at a stepped-down height:
    // ridge east-west, slopes facing front and back, and the wide low gable
    // on the east face that the side photograph shows. Its west gable end is
    // buried inside the middle block's taller one.
    gable_roof(
        out,
        Vec2::new(wing_n.min.x - INNER, wing_n.min.y - h),
        Vec2::new(wing_n.max.x + h, wing_s.max.y + h),
        true,
        ft(9.0) + SLAB + 8.0,
        4.0 / 12.0,
    );
    // The entry cross-gable: the steep front-facing gable over the porch,
    // nearly the garage gable's size, its own ridge running back into the
    // main south slope — and its peak just showing over the main ridge from
    // the back, exactly as the photographs have it.
    gable_roof(
        out,
        Vec2::new(porch_front.min.x, porch_front.max.y - ft(11.0)),
        Vec2::new(porch_front.max.x, porch_front.max.y + h),
        false,
        ft(10.0) + SLAB + 8.0,
        12.0 / 12.0,
    );
    gable_windows(out);

    // The headers the porch posts carry, closing the gap between each porch's
    // ceiling and the main roof's eave: the white-trimmed beam line straight
    // across each porch in the photographs.
    porch_headers(out);

    // Downpipes at outside corners, from the gutter to a shoe at the ground.
    // Three brackets each, because a pipe drawn as one box reads as a pipe
    // drawn as one box.
    let eave = ft(10.0) + SLAB + 8.0;
    let drop = eave - FASCIA_DEEP - 6.0;
    for (x, z) in [
        (-h - OVERHANG + 8.0, -h - OVERHANG + 8.0),
        (-h - OVERHANG + 8.0, garage.max.y + h + OVERHANG - 8.0),
        (
            garage.max.x + INNER - 10.0,
            porch_front.max.y + h + OVERHANG - 8.0,
        ),
        (
            wing_n.max.x + h + OVERHANG - 8.0,
            wing_n.min.y - h - OVERHANG + 8.0,
        ),
        (
            wing_n.max.x + h + OVERHANG - 8.0,
            wing_s.max.y + h + OVERHANG - 8.0,
        ),
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

/// The windows in the gable fields, from the photographs: a mulled triple in
/// the entry cross-gable, and one tall narrow sash high in the garage gable.
/// Applied to the gable face rather than cut through it — there is only attic
/// behind, and what the street reads is the black frame on the white field.
fn gable_windows(out: &mut Vec<Solid>) {
    let black = Color::srgb(0.10, 0.10, 0.11);
    let mut unit = |cx: f32, face_y: f32, y0: f32, y1: f32, wide: f32, lites: usize| {
        let rail = 5.0;
        let each = wide / lites as f32;
        for i in 0..lites {
            let x0 = cx - wide * 0.5 + each * i as f32;
            let mut frame = |fx0: f32, fy0: f32, fx1: f32, fy1: f32, stuff: Stuff, paint| {
                let mut solid = Solid::between(
                    Vec3::new(fx0, fy0, face_y),
                    Vec3::new(fx1, fy1, face_y + 4.0),
                    stuff,
                );
                solid.paint = paint;
                solid.roof = true;
                out.push(solid);
            };
            frame(x0, y0, x0 + rail, y1, Stuff::Wood, Some(black));
            frame(
                x0 + each - rail,
                y0,
                x0 + each,
                y1,
                Stuff::Wood,
                Some(black),
            );
            frame(x0, y0, x0 + each, y0 + rail, Stuff::Wood, Some(black));
            frame(x0, y1 - rail, x0 + each, y1, Stuff::Wood, Some(black));
            let mid = (y0 + y1) * 0.5;
            frame(
                x0,
                mid - 2.0,
                x0 + each,
                mid + 2.0,
                Stuff::Wood,
                Some(black),
            );
            frame(
                x0 + rail,
                y0 + rail,
                x0 + each - rail,
                y1 - rail,
                Stuff::Glass,
                None,
            );
        }
    };
    // The entry gable's triple, centred over the porch.
    let porch = room("front porch");
    let face = porch.max.y + OUTER + OUTER * 0.5 + 1.5;
    unit(porch.middle().x, face, 355.0, 450.0, 200.0, 3);
    // The garage gable's single, tall and narrow.
    let garage = room("garage");
    let face = garage.max.y + OUTER + OUTER * 0.5 + 1.5;
    unit(garage.middle().x, face, 355.0, 462.0, 58.0, 1);
}

/// The header beam across each porch's open side, sitting on the posts,
/// closing the band between the porch ceiling and the roof above it.
fn porch_headers(out: &mut Vec<Solid>) {
    for (name, along_x) in [("rear porch", true), ("front porch", true)] {
        let r = room(name);
        let line = if name == "rear porch" {
            r.min.y
        } else {
            r.max.y
        };
        let _ = along_x;
        let mut beam = Solid::between(
            Vec3::new(r.min.x, r.tall - 2.0, line - 9.0),
            Vec3::new(r.max.x, r.tall + 60.0, line + 9.0),
            Stuff::Wood,
        );
        beam.paint = Some(TRIM);
        beam.roof = true;
        out.push(beam);
    }
}

/// What kind of hole a wall cut, for the record below.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Hole {
    Pane,
    Doorway,
    Arch,
    Front,
    Back,
    Vehicle,
    Wide,
}

/// Every opening the derived walls actually cut, as world-space boxes.
///
/// Recorded rather than tabulated. Trim, glazing and the law that stops a
/// picture being hung over a window all used to read constant lists of
/// positions measured against a hand-authored shell — so the moment the walls
/// came from the plan instead, the frames stayed where the old walls had been
/// and hung in mid-air with daylight around them.
///
/// One source now: `PORTALS` says what to cut, the wall that cut the hole
/// records where the hole is, and everything downstream reads the record.
static CUT_BOXES: std::sync::LazyLock<std::sync::Mutex<Vec<(Hole, Vec3, Vec3)>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

fn cut_boxes(kind: Hole) -> Vec<(Vec3, Vec3)> {
    CUT_BOXES
        .lock()
        .unwrap()
        .iter()
        .filter(|(k, _, _)| *k == kind)
        .map(|&(_, lo, hi)| (lo, hi))
        .collect()
}

/// Which pairs of rooms the plan leaves open to each other.
///
/// The drawing says so by drawing them dashed, and the pixel scan found no wall
/// line between them at all: open plan reads as an absence of ink.
fn open_to_each_other(a: &str, b: &str) -> bool {
    const OPEN: [(&str, &str); 3] = [
        // No wall is drawn between the kitchen and the dining room at all;
        // kitchen to great room keeps its refrigerator stub, so that pair is a
        // `Wide` portal instead.
        ("kitchen", "dining"),
        // The storage bay's printed width is exactly the stretch of the
        // garage's north wall that is not there.
        ("garage", "open storage"),
        // The tub stands in its own nook of bath two.
        ("bath two", "bath two tub"),
    ];
    OPEN.iter()
        .any(|&(x, y)| (a == x && b == y) || (a == y && b == x))
}

/// A porch is outside. The wall between a room and one is an exterior wall with
/// a door in it, not a partition.
fn outdoors(name: &str) -> bool {
    name.ends_with("porch")
}

/// Every wall in the house, derived from the plan rather than authored.
///
/// The old shell was hand-written runs around a simple rectangle: four exterior
/// walls and a handful of partitions between three columns. This plan is sixteen
/// rooms in a U with two recessed porches, and no amount of authoring that by
/// hand stays correct through the next reading of the drawing.
///
/// So each room offers its four edges, and each edge is walked in six-inch
/// steps asking one question: is there another room immediately on the other
/// side? Runs of the same answer become one wall — a partition where two rooms
/// meet, an exterior wall where a room meets the outside. Partial adjacency
/// falls out for free, which matters because the garage's back wall meets two
/// different columns along its length and its neighbours change halfway.
///
/// Each run is emitted once. A partition belongs to both of its rooms and would
/// otherwise be built twice, in the same place, which is invisible until
/// something has to decide which of the two surfaces a fly is standing on.
fn walls_from_plan(s: &mut Vec<Solid>) {
    let all = rooms();
    let mut done: std::collections::HashSet<(i32, i32, i32, i32)> = Default::default();
    let (mut built, mut shared, mut holes_cut) = (0, 0, 0);
    let mut used = [false; PORTALS.len()];
    CUT_BOXES.lock().unwrap().clear();

    for r in &all {
        // (runs along x, the edge's own line, the span, which way is outward)
        let edges = [
            (true, r.min.y, r.min.x, r.max.x, -1.0f32),
            (true, r.max.y, r.min.x, r.max.x, 1.0),
            (false, r.min.x, r.min.y, r.max.y, -1.0),
            (false, r.max.x, r.min.y, r.max.y, 1.0),
        ];
        for (horizontal, line, from, to, out) in edges {
            // Who is on the other side, and over exactly which stretch.
            //
            // This was sampled in six-inch steps and grouped by whose name came
            // back, which quantised every run end to a step boundary. Two rooms
            // sharing a wall then disagreed about where their shared stretch
            // ended — one stopping on its true edge, the other on the nearest
            // step — so the two offers never matched, the duplicate was never
            // dropped, and every partition between columns was built twice.
            // Coplanar walls fight, and their door reveals fight worst.
            //
            // Intersecting the spans outright has no step to land on.
            let mut spans: Vec<(f32, f32, &Room)> = Vec::new();
            for n in &all {
                if n.name == r.name {
                    continue;
                }
                let (near, lo, hi) = if horizontal {
                    (if out > 0.0 { n.min.y } else { n.max.y }, n.min.x, n.max.x)
                } else {
                    (if out > 0.0 { n.min.x } else { n.max.x }, n.min.y, n.max.y)
                };
                if (near - (line + out * INNER)).abs() > 2.0 {
                    continue;
                }
                let (a, b) = (lo.max(from), hi.min(to));
                if b - a > 1.0 {
                    spans.push((a, b, n));
                }
            }
            spans.sort_by(|x, y| x.0.total_cmp(&y.0));

            // Walk the edge, alternating between the gaps and the neighbours.
            let mut runs: Vec<(f32, f32, Option<&Room>)> = Vec::new();
            let mut at = from;
            for (a, b, n) in &spans {
                if a - at > 1.0 {
                    runs.push((at, *a, None));
                }
                runs.push((at.max(*a), *b, Some(*n)));
                at = b.max(at);
            }
            if to - at > 1.0 {
                runs.push((at, to, None));
            }

            for (start, end, neighbour) in runs {
                // The wall goes in the *gap*, not on the room's own face, so
                // both rooms compute the same centreline and the duplicate can
                // be recognised at all. From *both* edges, not from this
                // room's edge plus a constant: the measured figures in the
                // plan are rounded to the centimetre, so gaps are 12 cm give
                // or take a millimetre, and a millimetre is enough for the
                // two offers to land in different dedupe buckets — which is
                // every partition built twice again.
                let mid = match neighbour {
                    Some(n) => {
                        let near = if horizontal {
                            if out > 0.0 { n.min.y } else { n.max.y }
                        } else if out > 0.0 {
                            n.min.x
                        } else {
                            n.max.x
                        };
                        (line + near) * 0.5
                    }
                    None => line + out * OUTER * 0.5,
                };
                let (p, q) = if horizontal {
                    (Vec2::new(start, mid), Vec2::new(end, mid))
                } else {
                    (Vec2::new(mid, start), Vec2::new(mid, end))
                };
                let key = (
                    (p.x * 4.0) as i32,
                    (p.y * 4.0) as i32,
                    (q.x * 4.0) as i32,
                    (q.y * 4.0) as i32,
                );
                if done.contains(&key) || done.contains(&(key.2, key.3, key.0, key.1)) {
                    shared += 1;
                    continue;
                }
                done.insert(key);

                if let Some(n) = neighbour
                    && open_to_each_other(r.name, n.name)
                {
                    // No wall — but the gap between the two rooms' floors is
                    // real, and both floors cover it, coplanar, which is a
                    // stripe of z-fighting right across an open plan. A flush
                    // seam board owns the gap instead, like the transition
                    // strip a real floor gets.
                    let across = if horizontal {
                        Vec2::new(0.0, 1.0)
                    } else {
                        Vec2::new(1.0, 0.0)
                    } * (INNER * 0.5 + 0.6);
                    let (ga, gb) = if horizontal {
                        (Vec2::new(start, mid), Vec2::new(end, mid))
                    } else {
                        (Vec2::new(mid, start), Vec2::new(mid, end))
                    };
                    let (lo, hi) = (ga - across, gb + across);
                    let wet = matches!(r.use_for, Use::Bath | Use::Utility | Use::Garage);
                    let mut seam = Solid::between(
                        Vec3::new(lo.x.min(hi.x), -SLAB, lo.y.min(hi.y)),
                        Vec3::new(lo.x.max(hi.x), 0.25, lo.y.max(hi.y)),
                        if wet { Stuff::Stone } else { Stuff::Wood },
                    );
                    seam.paint = Some(if wet {
                        Color::srgb(0.66, 0.67, 0.65)
                    } else {
                        Color::srgb(0.52, 0.40, 0.28)
                    });
                    s.push(seam);
                    continue;
                }
                // A porch's open sides have no wall at all — posts hold that
                // stretch of roof up instead.
                if outdoors(r.name) && neighbour.is_none_or(|n| outdoors(n.name)) {
                    continue;
                }
                let span = (q - p).length();
                if span <= 1.0 {
                    continue;
                }
                let inside = neighbour.is_some_and(|n| !outdoors(n.name)) && !outdoors(r.name);
                let tall = neighbour.map_or(r.tall, |n| r.tall.max(n.tall));
                let thick = if inside { INNER } else { OUTER };

                // Which way this edge faces, in the drawing's orientation, so
                // the openings measured off the drawing can find their wall.
                let edge = match (horizontal, out > 0.0) {
                    (true, false) => 'n',
                    (true, true) => 's',
                    (false, false) => 'w',
                    (false, true) => 'e',
                };
                let flip = |e: char| match e {
                    'n' => 's',
                    's' => 'n',
                    'w' => 'e',
                    _ => 'w',
                };

                let mut cut: Vec<Opening> = Vec::new();
                for (i, &(a, on, b, lo, hi, what)) in PORTALS.iter().enumerate() {
                    // A portal names one room's edge; the same wall is offered
                    // from both sides, so match it from either.
                    let here = (a == r.name
                        && on == edge
                        && neighbour.map_or(b == "outside", |n| b == n.name))
                        || neighbour
                            .is_some_and(|n| a == n.name && b == r.name && flip(on) == edge);
                    if !here {
                        continue;
                    }
                    let (lo, hi) = (ft(lo).max(start), ft(hi).min(end));
                    if hi - lo < 2.0 {
                        continue;
                    }
                    used[i] = true;
                    let (sill, head) = match what {
                        Cut::Pane { sill, head } => (sill, head),
                        Cut::Way { head } | Cut::Arch { head } => (0.0, head),
                        Cut::Wide => (0.0, tall),
                    };
                    cut.push(Opening {
                        at: (lo + hi) * 0.5 - start,
                        wide: hi - lo,
                        sill,
                        head,
                    });
                    // The record: a world-space box for everything downstream.
                    // Windows are recorded thicker than the wall on purpose —
                    // the audit probes the reveal against them — doorways at
                    // the wall's own thickness.
                    let along = (q - p) / span;
                    let deep = match what {
                        Cut::Pane { .. } => OUTER,
                        _ => thick * 0.5,
                    };
                    let across = Vec2::new(-along.y, along.x) * deep;
                    let (bl, bh) = (
                        p + along * (lo - start) - across,
                        p + along * (hi - start) + across,
                    );
                    let kind = match what {
                        Cut::Pane { .. } => Hole::Pane,
                        Cut::Way { .. } if a == "garage" && b == "outside" => Hole::Vehicle,
                        Cut::Way { .. } if b == "front porch" => Hole::Front,
                        Cut::Way { .. } if b == "rear porch" => Hole::Back,
                        Cut::Way { .. } => Hole::Doorway,
                        Cut::Arch { .. } => Hole::Arch,
                        Cut::Wide => Hole::Wide,
                    };
                    CUT_BOXES.lock().unwrap().push((
                        kind,
                        Vec3::new(bl.x.min(bh.x), sill, bl.y.min(bh.y)),
                        Vec3::new(bl.x.max(bh.x), head, bl.y.max(bh.y)),
                    ));
                }
                holes_cut += cut.len();
                built += 1;
                wall_run(s, p, q, thick, tall, Stuff::Plaster, &cut);
            }
        }
    }
    // Every opening in the table must have found its wall. One that has not is
    // a typo against the drawing — a mismeasured edge, a misnamed room — and
    // it refuses loudly, exactly as a mis-sized room does.
    for (i, p) in PORTALS.iter().enumerate() {
        if !used[i] {
            error!(
                "the {} → {} opening at {:.1}–{:.1} ft matched no wall — the table disagrees with the plan",
                p.0, p.2, p.3, p.4
            );
        }
    }
    // Every partition is offered twice, once by each room it separates, so
    // `shared` counts the duplicates recognised. A figure near zero means the
    // two offers are landing on different lines and every partition is being
    // built twice — invisible in a still frame, and the door reveals fighting
    // the moment anything moves.
    info!(
        "walls: {built} runs from the plan, {shared} duplicate offers dropped, {holes_cut} openings"
    );
}

/// Every door to the outside, shut, as part of the shell. The fly does not go
/// outside — the house is the whole world — so these are never ajar: the white
/// carriage door across the garage, the slate double front doors with their
/// glass lites, and the full-glass rear door. Same law as the sashes.
fn door_leaves(out: &mut Vec<Solid>) {
    let black = Color::srgb(0.10, 0.10, 0.11);
    let white = Color::srgb(0.92, 0.92, 0.93);

    // The garage's sixteen-foot carriage door: a white field, four six-lite
    // window panels across the top, and an X-buck brace on each lower panel.
    {
        let (lo, hi) = vehicle_door();
        let mid = (lo.z + hi.z) * 0.5 - 4.0;
        let mut slab = Solid::between(
            Vec3::new(lo.x, 0.0, mid - 2.5),
            Vec3::new(hi.x, hi.y, mid + 2.5),
            Stuff::Wood,
        );
        // A shade warmer than the siding, and set back into the opening —
        // flush and matching, it read as more wall with windows floating on it.
        slab.paint = Some(Color::srgb(0.88, 0.87, 0.85));
        out.push(slab);
        // Jamb and header casing, in the trim white.
        let trim = Color::srgb(0.94, 0.93, 0.90);
        for (bx0, by0, bx1, by1) in [
            (lo.x - 9.0, 0.0, lo.x, hi.y + 9.0),
            (hi.x, 0.0, hi.x + 9.0, hi.y + 9.0),
            (lo.x - 9.0, hi.y, hi.x + 9.0, hi.y + 9.0),
        ] {
            let mut case = Solid::between(
                Vec3::new(bx0, by0, mid + 3.0),
                Vec3::new(bx1, by1, mid + 9.0),
                Stuff::Wood,
            );
            case.paint = Some(trim);
            out.push(case);
        }
        let face = mid + 2.5;
        let panel = (hi.x - lo.x) / 4.0;
        for i in 0..4 {
            let x0 = lo.x + panel * i as f32 + 9.0;
            let x1 = lo.x + panel * (i as f32 + 1.0) - 9.0;
            // The lite along the top.
            let (y0, y1) = (hi.y - 46.0, hi.y - 12.0);
            let mut pane = Solid::between(
                Vec3::new(x0 + 3.0, y0 + 3.0, face),
                Vec3::new(x1 - 3.0, y1 - 3.0, face + 1.2),
                Stuff::Glass,
            );
            pane.roof = false;
            out.push(pane);
            for (fx0, fy0, fx1, fy1) in [
                (x0, y0, x1, y0 + 3.0),
                (x0, y1 - 3.0, x1, y1),
                (x0, y0, x0 + 3.0, y1),
                (x1 - 3.0, y0, x1, y1),
            ] {
                let mut bar = Solid::between(
                    Vec3::new(fx0, fy0, face),
                    Vec3::new(fx1, fy1, face + 1.6),
                    Stuff::Wood,
                );
                bar.paint = Some(black);
                out.push(bar);
            }
            // The X-buck on the panel below.
            let cy = (hi.y - 46.0) * 0.5;
            let reach = ((x1 - x0) * 0.5).min(cy - 8.0);
            for lean in [-1.0f32, 1.0] {
                let mut board = Solid::between(
                    Vec3::new(-reach, -5.5, 0.0),
                    Vec3::new(reach, 5.5, 1.4),
                    Stuff::Wood,
                );
                board.center = Vec3::new((x0 + x1) * 0.5, cy, face + 0.7);
                board.rot = Quat::from_rotation_z(lean * (cy - 8.0).atan2(reach));
                board.paint = Some(white);
                out.push(board);
            }
        }
    }

    // The double front doors: slate green-grey, four glass lites over two
    // panels on each leaf, lever handles at the meeting stiles.
    {
        let (lo, hi) = front_door();
        let mid = (lo.z + hi.z) * 0.5;
        let slate = Color::srgb(0.30, 0.33, 0.31);
        let trim = Color::srgb(0.94, 0.93, 0.90);
        for (bx0, by0, bx1, by1) in [
            (lo.x - 8.0, 0.0, lo.x, hi.y + 8.0),
            (hi.x, 0.0, hi.x + 8.0, hi.y + 8.0),
            (lo.x - 8.0, hi.y, hi.x + 8.0, hi.y + 8.0),
        ] {
            let mut case = Solid::between(
                Vec3::new(bx0, by0, mid + 2.6),
                Vec3::new(bx1, by1, mid + 8.0),
                Stuff::Wood,
            );
            case.paint = Some(trim);
            out.push(case);
        }
        let leaf_w = (hi.x - lo.x) * 0.5 - 1.0;
        for side in [0.0f32, 1.0] {
            let x0 = lo.x + side * ((hi.x - lo.x) * 0.5 + 1.0);
            let x1 = x0 + leaf_w;
            let mut leaf = Solid::between(
                Vec3::new(x0, 0.0, mid - 2.2),
                Vec3::new(x1, hi.y - 1.0, mid + 2.2),
                Stuff::Wood,
            );
            leaf.paint = Some(slate);
            out.push(leaf);
            let face = mid + 2.2;
            // Four lites in the upper half.
            let (gx0, gx1) = (x0 + 9.0, x1 - 9.0);
            let (gy0, gy1) = (hi.y * 0.52, hi.y - 14.0);
            let mut pane = Solid::between(
                Vec3::new(gx0, gy0, face - 0.6),
                Vec3::new(gx1, gy1, face + 0.6),
                Stuff::Glass,
            );
            pane.roof = false;
            out.push(pane);
            let (mx, my) = ((gx0 + gx1) * 0.5, (gy0 + gy1) * 0.5);
            for (bx0, by0, bx1, by1) in [
                (mx - 1.8, gy0, mx + 1.8, gy1),
                (gx0, my - 1.8, gx1, my + 1.8),
            ] {
                let mut muntin = Solid::between(
                    Vec3::new(bx0, by0, face + 0.6),
                    Vec3::new(bx1, by1, face + 1.4),
                    Stuff::Wood,
                );
                muntin.paint = Some(slate);
                out.push(muntin);
            }
            // Two raised panels below.
            for k in 0..2 {
                let py0 = 14.0 + k as f32 * (hi.y * 0.52 - 34.0) * 0.5;
                let py1 = py0 + (hi.y * 0.52 - 34.0) * 0.5 - 8.0;
                let mut raised = Solid::between(
                    Vec3::new(gx0, py0, face),
                    Vec3::new(gx1, py1, face + 1.0),
                    Stuff::Wood,
                );
                raised.paint = Some(slate);
                out.push(raised);
            }
            // The lever, on the meeting stile.
            let hx = if side < 0.5 { x1 - 6.0 } else { x0 + 6.0 };
            let mut lever = Solid::between(
                Vec3::new(hx - 2.0, 96.0, face),
                Vec3::new(hx + 2.0, 110.0, face + 2.4),
                Stuff::Metal,
            );
            lever.paint = Some(Color::srgb(0.75, 0.75, 0.74));
            out.push(lever);
        }
    }

    // The lanterns: two flanking the garage door, one beside the front
    // doors, one by the rear door. A dark box with a warm pane is all a
    // lantern is at this distance.
    {
        let (glo, ghi) = vehicle_door();
        let (flo, fhi) = front_door();
        let mut spots = vec![
            (glo.x - 40.0, 205.0, (glo.z + ghi.z) * 0.5 + 8.0),
            (ghi.x + 40.0, 205.0, (glo.z + ghi.z) * 0.5 + 8.0),
            (fhi.x + 34.0, 180.0, (flo.z + fhi.z) * 0.5 + 8.0),
        ];
        for (lo, hi) in cut_boxes(Hole::Back) {
            spots.push((hi.x + 34.0, 180.0, (lo.z + hi.z) * 0.5 - 8.0));
        }
        for (x, y, z) in spots {
            let mut lantern = Solid::between(
                Vec3::new(x - 7.0, y - 13.0, z - 7.0),
                Vec3::new(x + 7.0, y + 9.0, z + 7.0),
                Stuff::Metal,
            );
            lantern.paint = Some(black);
            out.push(lantern);
            let mut glow = Solid::between(
                Vec3::new(x - 4.5, y - 9.0, z - 4.5),
                Vec3::new(x + 4.5, y + 4.0, z + 4.5),
                Stuff::Glass,
            );
            glow.paint = Some(Color::srgb(1.0, 0.92, 0.72));
            out.push(glow);
            let mut cap = Solid::between(
                Vec3::new(x - 8.0, y + 9.0, z - 8.0),
                Vec3::new(x + 8.0, y + 13.0, z + 8.0),
                Stuff::Metal,
            );
            cap.paint = Some(black);
            out.push(cap);
        }
    }

    // The rear door: one full-view glass leaf in a dark frame.
    for (lo, hi) in cut_boxes(Hole::Back) {
        let mid = (lo.z + hi.z) * 0.5;
        let (y0, y1) = (0.0, hi.y - 1.0);
        for (fx0, fy0, fx1, fy1) in [
            (lo.x, y0, lo.x + 7.0, y1),
            (hi.x - 7.0, y0, hi.x, y1),
            (lo.x, y0, hi.x, y0 + 10.0),
            (lo.x, y1 - 10.0, hi.x, y1),
        ] {
            let mut bar = Solid::between(
                Vec3::new(fx0, fy0, mid - 2.2),
                Vec3::new(fx1, fy1, mid + 2.2),
                Stuff::Wood,
            );
            bar.paint = Some(black);
            out.push(bar);
        }
        let pane = Solid::between(
            Vec3::new(lo.x + 6.0, y0 + 9.0, mid - 1.0),
            Vec3::new(hi.x - 6.0, y1 - 9.0, mid + 1.0),
            Stuff::Glass,
        );
        out.push(pane);
    }
}

/// The posts that carry the roof across each porch's open side.
///
/// The photographs: the front porch stands on two square white posts at its
/// open corners; the rear porch's thirty-foot opening is divided into four
/// bays by three square cedar-toned posts.
fn porch_posts(out: &mut Vec<Solid>) {
    let fp = room("front porch");
    for x in [fp.min.x + 12.0, fp.max.x - 12.0] {
        let mut post = Solid::between(
            Vec3::new(x - 10.0, 0.0, fp.max.y - 20.0),
            Vec3::new(x + 10.0, fp.tall, fp.max.y),
            Stuff::Wood,
        );
        post.paint = Some(Color::srgb(0.93, 0.93, 0.94));
        out.push(post);
    }
    let rp = room("rear porch");
    for f in [0.25, 0.5, 0.75] {
        let x = rp.min.x + (rp.max.x - rp.min.x) * f;
        let mut post = Solid::between(
            Vec3::new(x - 10.0, 0.0, rp.min.y),
            Vec3::new(x + 10.0, rp.tall, rp.min.y + 20.0),
            Stuff::Wood,
        );
        post.paint = Some(Color::srgb(0.64, 0.45, 0.30));
        out.push(post);
    }
}

pub fn build() -> Home {
    let mut s: Vec<Solid> = Vec::new();
    let top = CEILING;
    let (w, e, n, so, _garage_south, _house_east) = envelope();

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

    // -- Walls, derived from the plan --------------------------------------
    walls_from_plan(&mut s);
    porch_posts(&mut s);

    // -- Ceilings, one per room -------------------------------------------
    for (i, r) in rooms().iter().enumerate() {
        // A hair of per-room height. Ceiling slabs overrun their rooms so no
        // seam can open above a wall — which makes two open-plan neighbours'
        // slabs coplanar where they overlap, and that is a stripe of
        // z-fighting overhead. A third of a millimetre is invisible and ends
        // the tie. Downward, not up: a slab lifted off the wall tops opens a
        // slit into the attic that reads as a dark stripe along the cornice.
        let hair = (i % 5 + 1) as f32 * 0.03;
        let mut slab = Solid::between(
            Vec3::new(r.min.x - INNER, r.tall - hair, r.min.y - INNER),
            Vec3::new(r.max.x + INNER, r.tall + SLAB - hair, r.max.y + INNER),
            Stuff::Plaster,
        );
        // Ceilings are painted white, and brighter than any wall under them.
        // That is not decoration: it is the surface the bounce fill is aimed
        // at, so it is the one place in a room where a lighter colour buys
        // light everywhere else.
        slab.paint = Some(Color::srgb(0.94, 0.93, 0.91));
        // Overhead, so `FLY_PLAN` drops it — otherwise the one view that can
        // show a floor plan shows a sheet of plaster instead.
        slab.roof = true;
        s.push(slab);
    }

    roof(&mut s);

    // The grounds and the neighbours, and then anything of theirs that has
    // ended up indoors is thrown out.
    //
    // Both are laid out against the old rectangle's outline, so on the new
    // footprint they planted a shrub in the great room and ran paving under the
    // dining table. Rather than re-author either against a plan that is still
    // settling, the rule is the one thing that cannot go stale: **nothing
    // outdoors belongs inside a room.**
    let before = s.len();
    grounds(&mut s);
    neighbourhood(&mut s);
    let mut evicted = 0;
    let mut kept = Vec::with_capacity(s.len());
    for (i, solid) in s.drain(..).enumerate() {
        if i >= before && solid.outdoors && inside_envelope(solid.center.xz()) {
            evicted += 1;
            continue;
        }
        kept.push(solid);
    }
    s = kept;
    if evicted > 0 {
        info!("grounds: {evicted} outdoor pieces were standing inside the house");
    }
    window_trim(&mut s);
    glaze(&mut s);
    door_leaves(&mut s);
    fixtures(&mut s);
    // **Off by default while the shell is being rebuilt to the plan.**
    //
    // Every piece of it is authored against the previous ten-room layout — as
    // fractions of room bounds that no longer exist, and hand-tuned offsets
    // against walls that have moved. Left on, it puts beds in the master bath
    // and parks the car across a partition, and the result is unjudgeable: "the
    // house looks like a Picasso painting". A shell cannot be assessed through
    // furniture standing in the wrong rooms.
    //
    // `FLY_FURNISH=1` brings it back, for re-flowing it room by room.
    if std::env::var("FLY_FURNISH").as_deref() == Ok("1") {
        crate::furniture::furnish(&mut s);
    }

    let great = room("great room");
    Home {
        hulls: Vec::new(),
        solids: s,
        door: None,
        spawn: Vec3::new(great.middle().x, top - 8.0, great.middle().y),
    }
}

/// A sash in every window opening the walls actually cut.
///
/// Real glass: a fly can land on it, walk it, and be fooled by it, which is the
/// most recognisable thing a housefly does indoors. `world` keeps it out of the
/// shadow pass, or every window would be a hole that let no light through.
///
/// The photographs of the built house set the style: black sashes, one
/// horizontal meeting rail, no muntin grids, every sash shut. One was briefly
/// left up, on the theory that a raised sash is how a fly gets into a house.
/// Brett's call is that the fly does not go outside — the house is the whole
/// world — so every sash is down. That is the better game anyway: the street
/// is right there through the glass and can never be reached, which is exactly
/// what a window means to a fly.
fn glaze(s: &mut Vec<Solid>) {
    let black = Color::srgb(0.10, 0.10, 0.11);

    for (lo, hi) in window_openings() {
        let along_x = (hi.x - lo.x) > (hi.z - lo.z);
        let (sill, head) = (lo.y, hi.y);
        let wide = if along_x { hi.x - lo.x } else { hi.z - lo.z };
        let mid = (lo + hi) * 0.5;

        // A wide opening is a mulled unit of equal sashes — the dining room's
        // front is the drawing's triple — and the doubled stiles where two
        // sashes meet read as the mullions.
        let lites = (wide / 110.0).ceil().max(1.0);
        let each = wide / lites;

        for i in 0..lites as usize {
            let shift = -wide * 0.5 + each * (i as f32 + 0.5);
            let at = if along_x {
                Vec3::new(mid.x + shift, 0.0, mid.z)
            } else {
                Vec3::new(mid.x, 0.0, mid.z + shift)
            };
            let rail = 4.5;
            let meet = (sill + head) * 0.5;
            let put = |out: &mut Vec<Solid>,
                       along: f32,
                       w: f32,
                       y0: f32,
                       y1: f32,
                       thick: f32,
                       paint: Option<Color>| {
                let (bmin, bmax) = if along_x {
                    (
                        Vec3::new(at.x + along - w * 0.5, y0, at.z - thick * 0.5),
                        Vec3::new(at.x + along + w * 0.5, y1, at.z + thick * 0.5),
                    )
                } else {
                    (
                        Vec3::new(at.x - thick * 0.5, y0, at.z + along - w * 0.5),
                        Vec3::new(at.x + thick * 0.5, y1, at.z + along + w * 0.5),
                    )
                };
                let mut solid = Solid::between(
                    bmin,
                    bmax,
                    if paint.is_some() {
                        Stuff::Wood
                    } else {
                        Stuff::Glass
                    },
                );
                solid.paint = paint;
                out.push(solid);
            };

            // Lower sash and upper sash, both shut, meeting rail between.
            for (y0, y1) in [(sill, meet), (meet - rail, head)] {
                put(s, 0.0, each - rail * 2.0, y0 + rail, y1 - rail, 1.6, None);
                for side in [-1.0f32, 1.0] {
                    put(
                        s,
                        side * (each * 0.5 - rail * 0.5),
                        rail,
                        y0,
                        y1,
                        4.5,
                        Some(black),
                    );
                }
                put(s, 0.0, each, y0, y0 + rail, 4.5, Some(black));
                put(s, 0.0, each, y1 - rail, y1, 4.5, Some(black));
            }
        }
    }
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
/// Every hinged interior doorway, in world centimetres — recorded by the wall
/// that cut it. The rear glass door onto the porch is among them.
pub fn interior_doors() -> Vec<(Vec3, Vec3)> {
    cut_boxes(Hole::Doorway)
}

/// The cased openings: the wing hall's arch off the great room and the two
/// bedroom closets' bifold openings. No leaves — but they are still holes in a
/// wall, and a hole in a wall in a finished house has a lining and an
/// architrave round it.
pub fn cased_openings() -> Vec<(Vec3, Vec3)> {
    cut_boxes(Hole::Arch)
}

/// Every hole in the building: windows, interior doors, the wide cased
/// openings, the front door and the vehicle door.
///
/// `clear_of_windows_on` was named for the only thing it knew about, and that
/// was the bug: a gallery hung itself across a doorway and a notice board
/// inside a cased opening, because neither is a window. Anything looking for a
/// blank stretch of wall wants all of them.
pub fn all_openings() -> Vec<(Vec3, Vec3)> {
    let mut out = window_openings();
    out.extend(interior_doors());
    out.extend(cased_openings());
    out.extend(cut_boxes(Hole::Back));
    out.push(front_door());
    out.push(vehicle_door());
    out
}

/// The front door's opening, in world centimetres: the double door off the
/// front porch into the great room.
pub fn front_door() -> (Vec3, Vec3) {
    cut_boxes(Hole::Front)
        .into_iter()
        .next()
        .expect("the walls cut the front door before anyone asked for it")
}

/// The vehicle door's opening, in world centimetres.
///
/// It is the one hole in the house big enough to drive through, and the only
/// one that needs a leaf built to fit it rather than a curtain hung beside it.
pub fn vehicle_door() -> (Vec3, Vec3) {
    cut_boxes(Hole::Vehicle)
        .into_iter()
        .next()
        .expect("the walls cut the vehicle door before anyone asked for it")
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
    cut_boxes(Hole::Pane)
}

// ---------------------------------------------------------------------------
// Checking the laws
// ---------------------------------------------------------------------------

/// Is this point within the building's outline?
///
/// Derived from the plan rather than described. The old version spelled out an
/// L — a rectangle with the corner south of the garage bitten out — which is
/// exactly the kind of hand-written shape that stops being true the moment the
/// drawing changes, and this plan is a U with two recessed porches.
///
/// Measured to the outer face of the wall *plus its trim*, not the centreline.
/// Floors run a little way into the wall on purpose so no seam can open at a
/// threshold, and skirting and cornice stand proud of the plaster on both faces;
/// a line down the middle of the wall calls every floorboard and every length
/// of moulding in the house an escapee.
fn inside_envelope(p: Vec2) -> bool {
    let m = OUTER * 0.5 + TRIM_PROUD + HAIR;
    rooms().iter().any(|r| {
        p.x >= r.min.x - m && p.x <= r.max.x + m && p.y >= r.min.y - m && p.y <= r.max.y + m
    })
}

pub fn audit(home: &Home) {
    let mut faults = 0;
    let all = rooms();
    for r in &all {
        // No two rooms may overlap.
        //
        // This replaces a fifteen-foot minimum, and it is the stricter law.
        // "At least fifteen feet" only ever caught a room that was small. The
        // plan is sixteen rooms of hand-entered coordinates in a U, and the
        // mistake waiting in that is a transposed digit putting two of them
        // through each other — which a minimum would pass without comment.
        for other in &all {
            if std::ptr::eq(r, other) {
                continue;
            }
            let over = (r.max.x.min(other.max.x) - r.min.x.max(other.min.x))
                .min(r.max.y.min(other.max.y) - r.min.y.max(other.min.y));
            if over > HAIR {
                error!(
                    "{} and {} overlap by {:.1} cm — the plan has them apart",
                    r.name, other.name, over
                );
                faults += 1;
            }
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
                let from = Vec3::new(at.x, r.tall - 5.0, at.y);
                match home.raycast(from, Vec3::Y, 40.0) {
                    Some(hit) if (hit.point.y - r.tall).abs() > 1.0 => {
                        error!(
                            "{}: ceiling is {:.1} cm up at ({:.0},{:.0}), not {:.1}",
                            r.name, hit.point.y, at.x, at.y, r.tall
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
                     a {:?} box {:.0}x{:.0}x{:.0} at ({:.0},{:.0},{:.0})",
                    mid.x,
                    mid.y,
                    mid.z,
                    solid.stuff,
                    solid.half.x * 2.0,
                    solid.half.y * 2.0,
                    solid.half.z * 2.0,
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

    faults += floating(home);
    faults += unreachable(home, &all);

    // The envelope the plan was measured at, in feet, as an independent check
    // on sixteen hand-entered rooms: printed sizes at measured positions have
    // to add back up to the drawing's own outline.
    let wide = all.iter().fold(0.0f32, |m, r| m.max(r.max.x));
    let deep = all.iter().fold(0.0f32, |m, r| m.max(r.max.y));
    // A foot and a half of tolerance, because the two figures are measuring
    // different things and cannot agree exactly. The plan's ink gives wall
    // *centrelines*; the rooms are chained from the printed *clear interior*
    // sizes with a partition between each. Over five columns and five stacks
    // those differ by about a foot, and the chained figure is the trustworthy
    // one — the printed dimensions are the drawing's own statement, and the
    // pixels were only ever scaffolding to find the topology.
    //
    // It still earns its place as a guard: an error large enough to matter
    // trips it, and a room quietly gaining a foot in a later edit would too.
    for (what, got, want) in [("wide", wide, ft(64.44)), ("deep", deep, ft(51.03))] {
        if (got - want).abs() > ft(1.5) {
            error!("the house is {got:.1} cm {what}, and the drawing measures {want:.1}");
            faults += 1;
        }
    }
    if faults == 0 {
        let tall: Vec<String> = {
            let mut heights: Vec<i32> =
                all.iter().map(|r| (r.tall / FOOT).round() as i32).collect();
            heights.sort_unstable();
            heights.dedup();
            heights.iter().map(|h| format!("{h}ft")).collect()
        };
        info!(
            "house: {:.0} x {:.0} cm to the plan, {} rooms, ceilings {}",
            wide,
            deep,
            all.len(),
            tall.join(" and ")
        );
    }
}

/// Nothing floats.
///
/// Every solid in this house must touch another one. That is the whole rule,
/// and it is the one that would have caught what Brett caught by flying: a
/// gallery hanging in mid-air over the dining table, a photograph eighteen
/// centimetres above the media unit, a stack of boxes stepping up through
/// nothing. Fixed viewpoints look *at* rooms from the corners; they are very
/// bad at showing that a thing is a few centimetres off the surface behind it,
/// and that is exactly the error a procedural house makes over and over,
/// because every position here is arithmetic and arithmetic is off by two.
///
/// Bucketed on a metre grid, or it is twenty-two million pairs.
fn floating(home: &Home) -> usize {
    const NEAR: f32 = 3.0;
    const CELL: f32 = 100.0;

    let key = |p: Vec3| {
        (
            (p.x / CELL).floor() as i32,
            (p.y / CELL).floor() as i32,
            (p.z / CELL).floor() as i32,
        )
    };
    let mut grid: std::collections::HashMap<(i32, i32, i32), Vec<usize>> =
        std::collections::HashMap::new();
    for (i, s) in home.solids.iter().enumerate() {
        let lo = s.center - s.half - Vec3::splat(NEAR);
        let hi = s.center + s.half + Vec3::splat(NEAR);
        let (a, b) = (key(lo), key(hi));
        for x in a.0..=b.0 {
            for y in a.1..=b.1 {
                for z in a.2..=b.2 {
                    grid.entry((x, y, z)).or_default().push(i);
                }
            }
        }
    }

    let touches = |a: &Solid, b: &Solid| {
        let gap = (a.center - b.center).abs() - (a.half + b.half);
        gap.x < NEAR && gap.y < NEAR && gap.z < NEAR
    };

    let mut faults = 0;
    for (i, solid) in home.solids.iter().enumerate() {
        // The ground has nothing under it, and a rotated solid's axis-aligned
        // half is a lie — a tilted siding board or a car wheel reports a box
        // bigger than it is, which makes this test meaningless for them.
        if solid.stuff == Stuff::Grass || solid.rot != Quat::IDENTITY {
            continue;
        }
        let lo = solid.center - solid.half - Vec3::splat(NEAR);
        let hi = solid.center + solid.half + Vec3::splat(NEAR);
        let (a, b) = (key(lo), key(hi));
        let mut held = false;
        'search: for x in a.0..=b.0 {
            for y in a.1..=b.1 {
                for z in a.2..=b.2 {
                    for &j in grid.get(&(x, y, z)).into_iter().flatten() {
                        if j != i && touches(solid, &home.solids[j]) {
                            held = true;
                            break 'search;
                        }
                    }
                }
            }
        }
        if !held {
            error!(
                "house fault: a {:?} box {:.0}x{:.0}x{:.0} at ({:.0},{:.0},{:.0}) \
                 touches nothing",
                solid.stuff,
                solid.half.x * 2.0,
                solid.half.y * 2.0,
                solid.half.z * 2.0,
                solid.center.x,
                solid.center.y,
                solid.center.z,
            );
            faults += 1;
        }
    }
    faults
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

    let (blo, bhi) = bounds();
    let (lo, hi) = (blo - OUTER, bhi + OUTER);
    let wide = (((hi.x - lo.x) / CELL).ceil() as usize).max(1);
    let deep = (((hi.y - lo.y) / CELL).ceil() as usize).max(1);
    let point = |ix: usize, iz: usize| {
        Vec3::new(
            lo.x + (ix as f32 + 0.5) * CELL,
            AT,
            lo.y + (iz as f32 + 0.5) * CELL,
        )
    };

    // Inside any indoor room, or inside the wall gap between two of them —
    // each room expanded by the partition width covers the gaps, so a doorway
    // is crossable and it is the wall solids that decide whether it is open.
    // The porches are rooms but not indoors: with every outside door shut and
    // every sash down, the outside is scenery, not floor plan.
    let indoors = |p: Vec3| {
        all.iter().any(|r| {
            !outdoors(r.name)
                && p.x > r.min.x - INNER
                && p.x < r.max.x + INNER
                && p.z > r.min.y - INNER
                && p.z < r.max.y + INNER
        })
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
        if outdoors(r.name) {
            continue;
        }
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
        // From the room's own ceiling — a 9-foot room's light hung at the
        // 10-foot constant floats above its own plaster.
        octagon(
            out,
            Vec3::new(at.x, r.tall - 2.5, at.y),
            wide + 8.0,
            5.0,
            Color::srgb(0.86, 0.86, 0.84),
            0.0,
        );
        octagon(
            out,
            Vec3::new(at.x, r.tall - 8.0, at.y),
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
                // A downlight grazes every wall in the room it is in, which is
                // the exact condition shadow maps produce acne under: fine
                // banding across a patch of wall bounded by the light's own
                // cone. The default biases are tuned for a world measured in
                // metres and this one is in centimetres.
                shadow_depth_bias: 0.06,
                shadow_normal_bias: 5.0,
                ..default()
            },
            Transform::from_xyz(at.x, r.tall - 6.0, at.y)
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
                .looking_at(Vec3::new(at.x, r.tall, at.y), Vec3::Z),
        ));
    }
}
