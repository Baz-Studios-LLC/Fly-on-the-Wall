//! Everything in the house that is not the house.
//!
//! Built from a handful of mathematical constructors — a painted box, a legged
//! top, a run of cabinets, a bed — composed into an authored arrangement per
//! room. Not scattered: a sofa faces a television because someone put it there,
//! and a scatter that happens to look plausible in one room will look wrong in
//! the next.
//!
//! **What a fly wants from furniture is not what a person wants.** A person
//! sees a worktop; a fly sees the eight-centimetre overhang under its front
//! edge, which is shelter with a view of the room. So the constructors here are
//! deliberately built out of *parts* — a top, a carcass, a plinth — rather than
//! as single blocks, because every seam between two parts is a ledge, and the
//! ledges are the content. The gap behind the fridge is not a detail, it is a
//! room.
//!
//! Any variation is derived from position by [`wobble`], never from a random
//! number generator, so the house is identical on every run and two captures can
//! actually be compared.

use bevy::prelude::*;

use crate::house::{self, Room, Use};
use crate::world::{Solid, Stuff};

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

const OAK: Color = Color::srgb(0.52, 0.38, 0.24);
const DARK_OAK: Color = Color::srgb(0.34, 0.24, 0.16);
const PAINTED: Color = Color::srgb(0.88, 0.86, 0.82);
const CARCASS: Color = Color::srgb(0.80, 0.78, 0.74);
const WORKTOP: Color = Color::srgb(0.26, 0.26, 0.28);
const STEEL: Color = Color::srgb(0.72, 0.74, 0.76);
const WOOL: Color = Color::srgb(0.42, 0.44, 0.50);
const WOOL_WARM: Color = Color::srgb(0.55, 0.45, 0.38);
const LINEN: Color = Color::srgb(0.86, 0.85, 0.80);
const SLATE: Color = Color::srgb(0.20, 0.21, 0.24);

/// The car.
const GLASS: Color = Color::srgba(0.78, 0.86, 0.90, 0.34);
const PAINTWORK: Color = Color::srgb(0.17, 0.28, 0.24);
const TYRE: Color = Color::srgb(0.07, 0.07, 0.08);
const HUB: Color = Color::srgb(0.62, 0.64, 0.66);
const BUMPER: Color = Color::srgb(0.20, 0.21, 0.23);
const LAMP: Color = Color::srgb(0.96, 0.94, 0.84);
const TAIL: Color = Color::srgb(0.54, 0.10, 0.09);
const PLATE: Color = Color::srgb(0.88, 0.88, 0.85);
const CABIN: Color = Color::srgb(0.20, 0.20, 0.21);
const SEAT: Color = Color::srgb(0.32, 0.30, 0.28);
const DOOR_SKIN: Color = Color::srgb(0.80, 0.80, 0.78);
const PORCELAIN: Color = Color::srgb(0.93, 0.94, 0.94);

/// Deterministic variation from a position: never a generator.
///
/// Two captures of the same house have to be comparable, so nothing here may
/// differ between runs. This returns roughly -1..1 and is stable for a given
/// point, which is enough to keep a row of books from being a comb.
fn wobble(x: f32, z: f32) -> f32 {
    let v = (x * 12.9898 + z * 78.233).sin() * 43758.547;
    (v - v.floor()) * 2.0 - 1.0
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

/// A painted box, by centre and size.
fn slab(out: &mut Vec<Solid>, at: Vec3, size: Vec3, stuff: Stuff, paint: Color) {
    let mut s = Solid::between(at - size * 0.5, at + size * 0.5, stuff);
    s.paint = Some(paint);
    out.push(s);
}

/// A top on four legs: table, desk, nightstand.
///
/// The top overhangs its legs, which is both what furniture does and what makes
/// the underside somewhere to stand.
fn legged(
    out: &mut Vec<Solid>,
    at: Vec2,
    top: Vec2,
    high: f32,
    thick: f32,
    leg: f32,
    top_paint: Color,
    leg_paint: Color,
) {
    slab(
        out,
        Vec3::new(at.x, high - thick * 0.5, at.y),
        Vec3::new(top.x, thick, top.y),
        Stuff::Wood,
        top_paint,
    );
    let inset = leg * 0.5 + 4.0;
    for sx in [-1.0f32, 1.0] {
        for sz in [-1.0f32, 1.0] {
            slab(
                out,
                Vec3::new(
                    at.x + sx * (top.x * 0.5 - inset),
                    (high - thick) * 0.5,
                    at.y + sz * (top.y * 0.5 - inset),
                ),
                Vec3::new(leg, high - thick, leg),
                Stuff::Wood,
                leg_paint,
            );
        }
    }
}

/// A run of base cabinets along a wall: plinth, carcass, and a worktop that
/// overhangs both.
///
/// `along` is the wall's axis. The overhang is the point of the whole thing at
/// this scale — it is the sheltered ledge that runs the length of a kitchen.
fn counter_run(
    out: &mut Vec<Solid>,
    from: Vec2,
    to: Vec2,
    depth: f32,
    face: Vec2,
    gaps: &[(f32, f32)],
) {
    const PLINTH: f32 = 12.0;
    const HIGH: f32 = 91.0;
    const TOP: f32 = 4.0;
    const OVER: f32 = 4.0;

    let along = (to - from).normalize_or_zero();
    let along_x = along.x.abs() > 0.5;
    // Where along the run a coordinate sits, and the point at a given distance.
    let coord = |p: Vec2| if along_x { p.x } else { p.y };
    let point = |d: f32| from + along * d;
    let size = |long: f32, across: f32| {
        if along_x {
            Vec3::new(long, 0.0, across)
        } else {
            Vec3::new(across, 0.0, long)
        }
    };

    // The run, broken around whatever stands in it.
    //
    // A cooker is not a thing that sits on top of a counter, it is a thing the
    // counter stops for — and burying one inside the carcass, which is what
    // happened first, leaves an appliance entirely inside a cupboard.
    let start = coord(from);
    let end = coord(to);
    let (lo, hi) = (start.min(end), start.max(end));
    let mut cuts: Vec<(f32, f32)> = gaps
        .iter()
        .map(|&(a, b)| (a.min(b).max(lo), a.max(b).min(hi)))
        .filter(|(a, b)| b > a)
        .collect();
    cuts.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut walked = lo;
    let mut runs: Vec<(f32, f32)> = Vec::new();
    for (a, b) in cuts {
        if a > walked {
            runs.push((walked, a));
        }
        walked = walked.max(b);
    }
    if walked < hi {
        runs.push((walked, hi));
    }

    let kick = 6.0;
    for (a, b) in runs {
        let length = b - a;
        if length < 12.0 {
            continue;
        }
        let mid = point((a + b) * 0.5 - start.min(end)) + along * 0.0;
        let mid = Vec2::new(
            if along_x { (a + b) * 0.5 } else { mid.x },
            if along_x { mid.y } else { (a + b) * 0.5 },
        );

        let plinth_at = mid + face * kick * 0.5;
        let s = size(length, depth - kick);
        slab(
            out,
            Vec3::new(plinth_at.x, PLINTH * 0.5, plinth_at.y),
            Vec3::new(s.x, PLINTH, s.z),
            Stuff::Wood,
            SLATE,
        );
        let s = size(length, depth);
        slab(
            out,
            Vec3::new(mid.x, PLINTH + (HIGH - TOP - PLINTH) * 0.5, mid.y),
            Vec3::new(s.x, HIGH - TOP - PLINTH, s.z),
            Stuff::Wood,
            CARCASS,
        );
        let top_at = mid - face * OVER * 0.5;
        let s = size(length, depth + OVER);
        slab(
            out,
            Vec3::new(top_at.x, HIGH - TOP * 0.5, top_at.y),
            Vec3::new(s.x, TOP, s.z),
            Stuff::Stone,
            WORKTOP,
        );
    }
}

/// Wall cabinets: a box with its underside at head height, which is the best
/// inverted perch in any house.
fn wall_cabinets(out: &mut Vec<Solid>, from: Vec2, to: Vec2, depth: f32) {
    const UNDER: f32 = 148.0;
    const TALL: f32 = 78.0;
    let mid = (from + to) * 0.5;
    let run = (to - from).length();
    let along = (to - from).normalize_or_zero();
    let (sx, sz) = if along.x.abs() > 0.5 {
        (run, depth)
    } else {
        (depth, run)
    };
    slab(
        out,
        Vec3::new(mid.x, UNDER + TALL * 0.5, mid.y),
        Vec3::new(sx, TALL, sz),
        Stuff::Wood,
        PAINTED,
    );
}

/// A bed: frame, mattress, and two pillows. The frame is inset from the
/// mattress so there is a rail to stand on, and it stands off the floor.
fn bed(out: &mut Vec<Solid>, at: Vec2, size: Vec2, facing: Vec2) {
    const CLEAR: f32 = 22.0;
    const FRAME: f32 = 14.0;
    const MATTRESS: f32 = 22.0;

    slab(
        out,
        Vec3::new(at.x, CLEAR + FRAME * 0.5, at.y),
        Vec3::new(size.x, FRAME, size.y),
        Stuff::Wood,
        DARK_OAK,
    );
    slab(
        out,
        Vec3::new(at.x, CLEAR + FRAME + MATTRESS * 0.5, at.y),
        Vec3::new(size.x - 8.0, MATTRESS, size.y - 8.0),
        Stuff::Fabric,
        LINEN,
    );
    // Headboard, at the end the bed faces away from.
    let head = at - facing * (size.y.max(size.x) * 0.5);
    let across = Vec2::new(facing.y.abs(), facing.x.abs());
    slab(
        out,
        Vec3::new(head.x, CLEAR + 46.0, head.y),
        Vec3::new(
            if across.x > 0.5 { size.x } else { 8.0 },
            92.0,
            if across.x > 0.5 { 8.0 } else { size.y },
        ),
        Stuff::Wood,
        DARK_OAK,
    );
    // Pillows.
    for s in [-1.0f32, 1.0] {
        let p = at - facing * (size.y.max(size.x) * 0.34)
            + Vec2::new(across.x, across.y) * s * size.x.min(size.y) * 0.24;
        slab(
            out,
            Vec3::new(p.x, CLEAR + FRAME + MATTRESS + 7.0, p.y),
            Vec3::new(48.0, 14.0, 32.0),
            Stuff::Fabric,
            PORCELAIN,
        );
    }
}

/// A sofa: plinth, seat cushion, back, and two arms — five parts, so it has
/// seams, an underside and a gap behind the cushion.
fn sofa(out: &mut Vec<Solid>, at: Vec2, size: Vec2, back: Vec2) {
    let across = Vec2::new(back.y.abs(), back.x.abs());
    let long = if across.x > 0.5 { size.x } else { size.y };
    let deep = if across.x > 0.5 { size.y } else { size.x };
    let dim = |l: f32, d: f32| {
        if across.x > 0.5 {
            Vec3::new(l, 0.0, d)
        } else {
            Vec3::new(d, 0.0, l)
        }
    };

    let s = dim(long, deep);
    slab(
        out,
        Vec3::new(at.x, 12.0, at.y),
        Vec3::new(s.x, 24.0, s.z),
        Stuff::Wood,
        DARK_OAK,
    );
    let s = dim(long - 28.0, deep - 12.0);
    slab(
        out,
        Vec3::new(at.x, 34.0, at.y),
        Vec3::new(s.x, 20.0, s.z),
        Stuff::Fabric,
        WOOL,
    );
    // Back.
    let b = at + back * (deep * 0.5 - 8.0);
    let s = dim(long, 16.0);
    slab(
        out,
        Vec3::new(b.x, 56.0, b.y),
        Vec3::new(s.x, 64.0, s.z),
        Stuff::Fabric,
        WOOL,
    );
    // Arms.
    for side in [-1.0f32, 1.0] {
        let a = at + across * side * (long * 0.5 - 8.0);
        let s = dim(16.0, deep);
        slab(
            out,
            Vec3::new(a.x, 40.0, a.y),
            Vec3::new(s.x, 32.0, s.z),
            Stuff::Fabric,
            WOOL,
        );
    }
}

/// A tall box standing off the wall, with a real gap behind it.
///
/// The gap is the point. Fifteen centimetres behind a fridge is unreachable in
/// flight, trivial on foot, warm, and dark — which is four things a fly cares
/// about in one place.
fn appliance(out: &mut Vec<Solid>, at: Vec2, size: Vec3, paint: Color) {
    slab(
        out,
        Vec3::new(at.x, size.y * 0.5, at.y),
        size,
        Stuff::Metal,
        paint,
    );
}

/// Open shelving: uprights and boards, so every board is a landing.
fn shelves(out: &mut Vec<Solid>, at: Vec2, size: Vec2, high: f32, boards: usize, along_x: bool) {
    // `size` is (length, depth) and `along_x` says which way the length runs;
    // `dim` already does the axis mapping, so swapping the pair here as well
    // undid it. The garage's shelving came out three metres wide across the
    // wrong axis and stood a metre outside the east wall of the house.
    let (w, d) = (size.x, size.y);
    let dim = |a: f32, b: f32| {
        if along_x {
            Vec3::new(a, 0.0, b)
        } else {
            Vec3::new(b, 0.0, a)
        }
    };
    for side in [-1.0f32, 1.0] {
        let u = if along_x {
            Vec2::new(at.x + side * (w * 0.5 - 2.0), at.y)
        } else {
            Vec2::new(at.x, at.y + side * (w * 0.5 - 2.0))
        };
        let s = dim(4.0, d);
        slab(
            out,
            Vec3::new(u.x, high * 0.5, u.y),
            Vec3::new(s.x, high, s.z),
            Stuff::Wood,
            OAK,
        );
    }
    for i in 0..boards {
        let y = high * (i as f32 + 0.6) / boards as f32;
        let s = dim(w, d);
        slab(
            out,
            Vec3::new(at.x, y, at.y),
            Vec3::new(s.x, 3.0, s.z),
            Stuff::Wood,
            OAK,
        );
        // A few books, sized off their own position so the row is never a comb.
        let n = 5;
        for b in 0..n {
            let t = (b as f32 + 0.5) / n as f32 - 0.5;
            let p = if along_x {
                Vec2::new(at.x + t * (w - 20.0), at.y)
            } else {
                Vec2::new(at.x, at.y + t * (w - 20.0))
            };
            let h = 20.0 + wobble(p.x, p.y + i as f32 * 13.0) * 5.0;
            let s = dim(12.0, d * 0.7);
            slab(
                out,
                Vec3::new(p.x, y + 1.5 + h * 0.5, p.y),
                Vec3::new(s.x, h, s.z),
                Stuff::Wood,
                if b % 2 == 0 { WOOL_WARM } else { SLATE },
            );
        }
    }
}

/// A rug: thin, wide, and the one thing on the floor a fly can feel the edge of.
fn rug(out: &mut Vec<Solid>, at: Vec2, size: Vec2, paint: Color) {
    slab(
        out,
        Vec3::new(at.x, 0.6, at.y),
        Vec3::new(size.x, 1.2, size.y),
        Stuff::Fabric,
        paint,
    );
}

/// A framed picture on a wall: a frame and a canvas set inside it.
///
/// Two boxes rather than one, because the frame standing proud of the canvas is
/// what makes it read as a picture instead of a painted rectangle — and because
/// the rebate between them is a two-centimetre shelf a fly can stand on.
fn picture(out: &mut Vec<Solid>, at: Vec3, wide: f32, tall: f32, along_x: bool, tone: Color) {
    let d = |t: f32| {
        if along_x {
            Vec3::new(wide, tall, t)
        } else {
            Vec3::new(t, tall, wide)
        }
    };
    // Frame first, then the canvas standing PROUD of it rather than buried
    // inside it — the first version put a two-centimetre canvas inside a
    // three-centimetre frame, so every picture in the house rendered as its own
    // dark surround and nothing else.
    slab(out, at, d(2.0), Stuff::Wood, DARK_OAK);
    let out_of = if along_x {
        Vec3::new(0.0, 0.0, 1.4)
    } else {
        Vec3::new(1.4, 0.0, 0.0)
    };
    // Which way is "into the room"? Whichever side of the wall the frame's own
    // centre is not: the canvas goes toward the viewer.
    let face = if along_x {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let s = d(1.6);
    slab(
        out,
        at + out_of * face.length(),
        Vec3::new(
            (s.x - 9.0).max(1.6),
            (s.y - 9.0).max(1.6),
            (s.z - 9.0).max(1.6),
        ),
        Stuff::Fabric,
        tone,
    );
}

/// Curtains: a pair of panels either side of a window, and a pole across it.
///
/// Hung on every window in the house from one pass, because the windows already
/// know where they are and asking them is the only way this stays right when one
/// moves.
/// Curtains, on the room side of the glass.
///
/// The first version of this hung every panel and pole on the wall's own
/// centreline, which is where the *opening* is. An eight-centimetre curtain
/// inside a twenty-centimetre wall is a curtain nobody can see: the whole house
/// had them, and not one showed. They now stand proud of the inside face, which
/// is also the only place a fly could ever land on one.
fn dress_the_windows(out: &mut Vec<Solid>) {
    /// Where a window's sill sits, and so where a short curtain stops.
    const SILL_CLEAR: f32 = 100.0;
    let heart = crate::house::centre();
    // Clear of the plaster by a centimetre, so the fabric reads as hanging in
    // front of the wall rather than growing out of it.
    let proud = crate::house::WALL_OUTER * 0.5 + 5.0;

    for (lo, hi) in crate::house::window_openings() {
        let mid = (lo + hi) * 0.5;
        let span = hi - lo;
        // The thin axis is the wall's own; the wide one is the window's.
        let along_x = span.x > span.z;
        let wide = if along_x { span.x } else { span.z };
        let head = hi.y;
        // Inward along the wall's thin axis.
        let inward = if along_x {
            Vec3::Z * (heart.y - mid.z).signum()
        } else {
            Vec3::X * (heart.x - mid.x).signum()
        } * proud;

        // The pole, a little wider than the opening.
        let pole = if along_x {
            Vec3::new(wide + 44.0, 3.0, 3.0)
        } else {
            Vec3::new(3.0, 3.0, wide + 44.0)
        };
        slab(
            out,
            Vec3::new(mid.x, head + 12.0, mid.z) + inward,
            pole,
            Stuff::Metal,
            SLATE,
        );

        // A panel each side, hanging nearly to the floor, and standing fully
        // clear of the opening rather than lapping over its edge — a real
        // curtain laps the reveal, but the law says nothing stands in a window
        // and a law with exceptions in it is not worth having.
        let drop = head + 10.0 - 12.0;
        for side in [-1.0f32, 1.0] {
            let off = (wide * 0.5 + 26.0) * side;
            let at = if along_x {
                Vec3::new(mid.x + off, 12.0 + drop * 0.5, mid.z)
            } else {
                Vec3::new(mid.x, 12.0 + drop * 0.5, mid.z + off)
            };
            let full = if along_x {
                Vec3::new(34.0, drop, 8.0)
            } else {
                Vec3::new(8.0, drop, 34.0)
            };
            // Full length unless a counter, a vanity or a bed head is already
            // standing there. A kitchen curtain drawn through a worktop was the
            // giveaway that this was being hung blind; the same question the
            // wall cabinets already ask about windows, asked the other way
            // round.
            if !clashes(out, at + inward, full) {
                slab(out, at + inward, full, Stuff::Fabric, WOOL_WARM);
                continue;
            }
            let short = head + 10.0 - (SILL_CLEAR - 6.0);
            let size = if along_x {
                Vec3::new(34.0, short, 8.0)
            } else {
                Vec3::new(8.0, short, 34.0)
            };
            let raised = Vec3::new(at.x, SILL_CLEAR - 6.0 + short * 0.5, at.z) + inward;
            if !clashes(out, raised, size) {
                slab(out, raised, size, Stuff::Fabric, WOOL_WARM);
            }
        }
    }
}

/// A box with a turn on it. The collision is already oriented — `Solid` carries
/// a quaternion — so a raked windscreen costs exactly as much as a flat one.
fn turned(out: &mut Vec<Solid>, at: Vec3, size: Vec3, rot: Quat, stuff: Stuff, paint: Color) {
    let mut s = Solid::between(-size * 0.5, size * 0.5, stuff);
    s.center = at;
    s.rot = rot;
    s.paint = Some(paint);
    out.push(s);
}

/// A wheel, out of the only primitive there is.
///
/// Four rectangles of the same length crossed at forty-five degrees union into
/// an octagon, which at the size of a road wheel is round enough to read and
/// round enough to walk on. The width across a rectangle is `2r tan(22.5°)`,
/// which is what puts all eight corners on the same circle. The axle runs along
/// x, because every wheel in this house is on a car pointing down the garage.
fn wheel(out: &mut Vec<Solid>, at: Vec3, radius: f32, width: f32) {
    const SIDES: usize = 4;
    let across = 2.0 * radius * (std::f32::consts::PI / (2.0 * SIDES as f32)).tan();
    for k in 0..SIDES {
        let a = k as f32 * std::f32::consts::PI / SIDES as f32;
        turned(
            out,
            at,
            Vec3::new(width, radius * 2.0, across),
            Quat::from_rotation_x(a),
            Stuff::Metal,
            TYRE,
        );
    }
    // The hub, proud of the tyre on both faces so one pair of boxes reads from
    // either side of the car.
    for k in 0..2 {
        let a = k as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
        turned(
            out,
            at,
            Vec3::new(width + 2.0, radius * 0.60, radius * 0.42),
            Quat::from_rotation_x(a),
            Stuff::Metal,
            HUB,
        );
    }
}

/// The family car: nose toward the door, down the length of the garage.
///
/// It was a single slate box, which is a shipping container in a room whose
/// whole identity is the thing parked in it. Built the way the house is built —
/// a sill, a shoulder, decks front and rear, a glasshouse on top — it reads at
/// human scale, and at fly scale it becomes the most interesting furniture in
/// the building: warm metal, a dozen ledges, and a sheltered underside.
fn car(out: &mut Vec<Solid>, at: Vec2) {
    let (x, z) = (at.x, at.y);
    let body = |o: &mut Vec<Solid>, c: Vec3, s: Vec3| slab(o, c, s, Stuff::Metal, PAINTWORK);

    // Sill and shoulder: the car's mass, in two steps rather than one slab.
    body(out, Vec3::new(x, 53.0, z), Vec3::new(178.0, 54.0, 430.0));
    body(out, Vec3::new(x, 97.0, z), Vec3::new(172.0, 34.0, 418.0));
    // Rocker panels, darker, between the wheels.
    for side in [-1.0f32, 1.0] {
        slab(
            out,
            Vec3::new(x + 89.0 * side, 36.0, z),
            Vec3::new(6.0, 18.0, 250.0),
            Stuff::Metal,
            BUMPER,
        );
    }
    // Bonnet and boot decks.
    body(
        out,
        Vec3::new(x, 118.0, z + 150.0),
        Vec3::new(168.0, 8.0, 118.0),
    );
    body(
        out,
        Vec3::new(x, 118.0, z - 152.0),
        Vec3::new(168.0, 8.0, 100.0),
    );

    // The cabin, before the glass goes in. You could see clean through the car
    // to the wall behind it, which reads as a shell rather than a vehicle, and
    // a lit interior is most of what tells you a car is a car from outside.
    slab(
        out,
        Vec3::new(x, 116.0, z - 8.0),
        Vec3::new(152.0, 6.0, 186.0),
        Stuff::Fabric,
        CABIN,
    );
    for (sz, back) in [(52.0f32, 86.0f32), (-46.0, -12.0)] {
        slab(
            out,
            Vec3::new(x, 130.0, z + sz),
            Vec3::new(140.0, 22.0, 46.0),
            Stuff::Fabric,
            SEAT,
        );
        slab(
            out,
            Vec3::new(x, 140.0, z + back),
            Vec3::new(140.0, 44.0, 12.0),
            Stuff::Fabric,
            SEAT,
        );
    }
    slab(
        out,
        Vec3::new(x, 128.0, z + 76.0),
        Vec3::new(146.0, 26.0, 26.0),
        Stuff::Metal,
        CABIN,
    );
    turned(
        out,
        Vec3::new(x - 44.0, 134.0, z + 60.0),
        Vec3::new(4.0, 30.0, 30.0),
        Quat::from_rotation_z(std::f32::consts::FRAC_PI_2 * 0.72),
        Stuff::Metal,
        BUMPER,
    );

    // The glasshouse. Roof, pillars, and glass between them.
    body(
        out,
        Vec3::new(x, 157.0, z - 8.0),
        Vec3::new(150.0, 14.0, 184.0),
    );
    for side in [-1.0f32, 1.0] {
        for (pz, pw) in [(84.0, 9.0), (-6.0, 7.0), (-96.0, 10.0)] {
            body(
                out,
                Vec3::new(x + 72.0 * side, 133.0, z + pz),
                Vec3::new(6.0, 42.0, pw),
            );
        }
        // Side glass, in two lights split by the middle pillar.
        for pz in [42.0, -50.0] {
            slab(
                out,
                Vec3::new(x + 72.0 * side, 133.0, z + pz),
                Vec3::new(3.0, 36.0, 78.0),
                Stuff::Glass,
                GLASS,
            );
        }
        // Mirror, on a stalk at the front pillar.
        body(
            out,
            Vec3::new(x + 84.0 * side, 130.0, z + 80.0),
            Vec3::new(20.0, 12.0, 7.0),
        );
        // Door handles.
        for hz in [46.0, -42.0] {
            slab(
                out,
                Vec3::new(x + 88.0 * side, 104.0, z + hz),
                Vec3::new(5.0, 6.0, 22.0),
                Stuff::Metal,
                HUB,
            );
        }
    }
    // Windscreen and backlight, raked the way glass is: the top of each leans
    // toward the middle of the car.
    turned(
        out,
        Vec3::new(x, 134.0, z + 90.0),
        Vec3::new(144.0, 52.0, 4.0),
        Quat::from_rotation_x(-0.52),
        Stuff::Glass,
        GLASS,
    );
    turned(
        out,
        Vec3::new(x, 134.0, z - 100.0),
        Vec3::new(144.0, 48.0, 4.0),
        Quat::from_rotation_x(0.46),
        Stuff::Glass,
        GLASS,
    );

    // Wheels, tucked just inside the flanks.
    for sx in [-1.0f32, 1.0] {
        for sz in [-1.0f32, 1.0] {
            wheel(
                out,
                Vec3::new(x + 76.0 * sx, 34.0, z + 142.0 * sz),
                34.0,
                26.0,
            );
        }
    }

    // The ends: bumpers, lamps, a grille and a plate.
    for (sz, lamp) in [(1.0f32, LAMP), (-1.0f32, TAIL)] {
        slab(
            out,
            Vec3::new(x, 54.0, z + 221.0 * sz),
            Vec3::new(176.0, 26.0, 14.0),
            Stuff::Metal,
            BUMPER,
        );
        for sx in [-1.0f32, 1.0] {
            slab(
                out,
                Vec3::new(x + 60.0 * sx, 78.0, z + 217.0 * sz),
                Vec3::new(36.0, 15.0, 8.0),
                Stuff::Glass,
                lamp,
            );
        }
    }
    slab(
        out,
        Vec3::new(x, 78.0, z + 217.0),
        Vec3::new(76.0, 13.0, 8.0),
        Stuff::Metal,
        BUMPER,
    );
    slab(
        out,
        Vec3::new(x, 40.0, z + 229.0),
        Vec3::new(62.0, 15.0, 3.0),
        Stuff::Metal,
        PLATE,
    );
}

/// The sectional door, shut.
///
/// A garage with a hole in the wall is a carport. Four panels with a reveal
/// between them and a row of lights in the top one, which is what keeps the
/// room from going pitch dark with the door closed — and gives the fly a two
/// metre climb with a view.
fn garage_door(out: &mut Vec<Solid>) {
    let (lo, hi) = crate::house::vehicle_door();
    let wide = hi.x - lo.x;
    let mid_x = (lo.x + hi.x) * 0.5;
    let z = (lo.z + hi.z) * 0.5;
    let top = hi.y;
    const PANELS: usize = 4;
    let panel = top / PANELS as f32;

    for i in 0..PANELS {
        let low = i as f32 * panel;
        let centre = low + panel * 0.5;
        // The reveal between sections, set back so the joint reads as a shadow.
        slab(
            out,
            Vec3::new(mid_x, low, z),
            Vec3::new(wide, 3.0, 6.0),
            Stuff::Metal,
            BUMPER,
        );
        if i + 1 < PANELS {
            slab(
                out,
                Vec3::new(mid_x, centre, z),
                Vec3::new(wide, panel - 6.0, 9.0),
                Stuff::Metal,
                DOOR_SKIN,
            );
            continue;
        }
        // The top section is glazed: a frame, four lights, three mullions.
        const LIGHTS: usize = 4;
        let bay = wide / LIGHTS as f32;
        let glass_high = panel * 0.52;
        let rail = (panel - glass_high) * 0.5;
        for (y, h) in [
            (centre - (glass_high + rail) * 0.5, rail),
            (centre + (glass_high + rail) * 0.5, rail),
        ] {
            slab(
                out,
                Vec3::new(mid_x, y, z),
                Vec3::new(wide, h - 3.0, 9.0),
                Stuff::Metal,
                DOOR_SKIN,
            );
        }
        for k in 0..LIGHTS {
            let cx = lo.x + bay * (k as f32 + 0.5);
            slab(
                out,
                Vec3::new(cx, centre, z),
                Vec3::new(bay - 10.0, glass_high, 3.0),
                Stuff::Glass,
                GLASS,
            );
            slab(
                out,
                Vec3::new(cx - bay * 0.5, centre, z),
                Vec3::new(10.0, glass_high, 9.0),
                Stuff::Metal,
                DOOR_SKIN,
            );
        }
        slab(
            out,
            Vec3::new(hi.x, centre, z),
            Vec3::new(10.0, glass_high, 9.0),
            Stuff::Metal,
            DOOR_SKIN,
        );
        slab(
            out,
            Vec3::new(mid_x, top, z),
            Vec3::new(wide, 3.0, 6.0),
            Stuff::Metal,
            BUMPER,
        );
    }
}

// ---------------------------------------------------------------------------
// The rooms
// ---------------------------------------------------------------------------

/// Furnish every room. Called once, after the shell.
pub fn furnish(out: &mut Vec<Solid>) {
    for r in house::rooms() {
        match r.use_for {
            Use::Living => living(out, &r),
            Use::Kitchen => kitchen(out, &r),
            Use::Bed => bedroom(out, &r),
            Use::Bath => bathroom(out, &r),
            Use::Utility => laundry(out, &r),
            Use::Hall => hall(out, &r),
            Use::Garage => garage(out, &r),
        }
    }
    // Last, so a curtain can see what is already standing under its window.
    dress_the_windows(out);
}

/// Does this box share space with anything already built?
fn clashes(out: &[Solid], at: Vec3, size: Vec3) -> bool {
    let (lo, hi) = (at - size * 0.5, at + size * 0.5);
    out.iter().any(|s| {
        let (slo, shi) = (s.center - s.half, s.center + s.half);
        lo.x < shi.x - 1.0
            && hi.x > slo.x + 1.0
            && lo.y < shi.y - 1.0
            && hi.y > slo.y + 1.0
            && lo.z < shi.z - 1.0
            && hi.z > slo.z + 1.0
    })
}

fn living(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();
    rug(
        out,
        m,
        Vec2::new(r.wide() * 0.55, r.deep() * 0.40),
        WOOL_WARM,
    );

    // Sofa against the west side of the room, facing east across the rug.
    sofa(
        out,
        Vec2::new(r.min.x + 70.0, m.y),
        Vec2::new(96.0, 220.0),
        Vec2::new(-1.0, 0.0),
    );
    // Coffee table on the rug.
    legged(
        out,
        Vec2::new(m.x - 40.0, m.y),
        Vec2::new(110.0, 62.0),
        42.0,
        5.0,
        7.0,
        OAK,
        DARK_OAK,
    );
    // Media unit opposite the sofa, with the television standing on it.
    let east = Vec2::new(r.max.x - 34.0, m.y);
    slab(
        out,
        Vec3::new(east.x, 28.0, east.y),
        Vec3::new(52.0, 56.0, 190.0),
        Stuff::Wood,
        DARK_OAK,
    );
    slab(
        out,
        Vec3::new(east.x - 4.0, 56.0 + 34.0, east.y),
        Vec3::new(8.0, 68.0, 122.0),
        Stuff::Metal,
        SLATE,
    );
    // Bookshelves in the far corner.
    shelves(
        out,
        Vec2::new(r.min.x + 120.0, r.min.y + 30.0),
        Vec2::new(180.0, 32.0),
        190.0,
        4,
        true,
    );
    // A picture over the sofa, which is where a house puts one.
    picture(
        out,
        Vec3::new(r.min.x + 6.0, 168.0, m.y),
        130.0,
        84.0,
        false,
        Color::srgb(0.62, 0.58, 0.50),
    );

    // A side table by the sofa's arm.
    legged(
        out,
        Vec2::new(r.min.x + 74.0, m.y - 140.0),
        Vec2::new(48.0, 48.0),
        56.0,
        4.0,
        6.0,
        OAK,
        DARK_OAK,
    );
}

/// Where along a room's north wall there is no window, and the widest such run.
///
/// Anything tall goes here. A cooker hood and a run of wall cabinets both ended
/// up over glass by being placed at a hand-picked offset, and the fix that does
/// not need repeating is to ask where the windows are rather than to remember.
fn clear_of_windows(r: &Room) -> (f32, f32) {
    let mut spans: Vec<(f32, f32)> = crate::house::window_openings()
        .into_iter()
        // Only the ones in this room's north wall.
        .filter(|(lo, hi)| {
            let mid = (*lo + *hi) * 0.5;
            mid.x > r.min.x && mid.x < r.max.x && (mid.z - r.min.y).abs() < 60.0
        })
        .map(|(lo, hi)| (lo.x, hi.x))
        .collect();
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut best = (r.min.x + 30.0, r.min.x + 130.0);
    let mut widest = 0.0;
    let mut walk = r.min.x + 30.0;
    for (lo, hi) in spans.iter().chain(std::iter::once(&(r.max.x - 30.0, 0.0))) {
        let gap = lo - walk;
        if gap > widest {
            widest = gap;
            best = (walk, *lo);
        }
        walk = walk.max(*hi);
    }
    best
}

fn kitchen(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();

    // The sink run goes under the north windows, and gets NO wall cabinets.
    //
    // The first pass ran cabinets the length of that wall and put one straight
    // over a window, which is the sort of overlap that is invisible in a plan
    // and unmissable from inside the room. Cabinets live on the east wall
    // instead, which is the one wall of this kitchen with nothing in it —
    // which is also why a real kitchen puts them there.
    let north = r.min.y + 34.0;
    let (gap_lo, gap_hi) = clear_of_windows(r);
    let cooker_x = (gap_lo + gap_hi) * 0.5;
    counter_run(
        out,
        Vec2::new(r.min.x + 30.0, north),
        Vec2::new(r.max.x - 150.0, north),
        64.0,
        Vec2::new(0.0, 1.0),
        // The counter stops for the cooker rather than swallowing it.
        &[(cooker_x - 40.0, cooker_x + 40.0)],
    );
    // The sink goes UNDER a window, which is where every kitchen puts one. Its
    // rim is below the sill, so it is not an obstruction.
    let windows = crate::house::window_openings();
    let sink_x = windows
        .iter()
        .map(|(lo, hi)| (lo.x + hi.x) * 0.5)
        .filter(|x| *x > r.min.x + 60.0 && *x < r.max.x - 60.0)
        .min_by(|a, b| (a - m.x).abs().partial_cmp(&(b - m.x).abs()).unwrap())
        .unwrap_or(m.x);
    slab(
        out,
        Vec3::new(sink_x, 88.0, north - 4.0),
        Vec3::new(84.0, 10.0, 46.0),
        Stuff::Metal,
        STEEL,
    );

    // Tall run down the east wall: counter, and cabinets over it.
    let east = r.max.x - 36.0;
    counter_run(
        out,
        Vec2::new(east, r.min.y + 90.0),
        Vec2::new(east, r.min.y + 330.0),
        66.0,
        Vec2::new(-1.0, 0.0),
        &[],
    );
    wall_cabinets(
        out,
        Vec2::new(r.max.x - 20.0, r.min.y + 100.0),
        Vec2::new(r.max.x - 20.0, r.min.y + 320.0),
        36.0,
    );

    // The fridge, standing off the north wall with a gap behind it. The gap is
    // the point: unreachable in flight, trivial on foot, warm and dark.
    appliance(
        out,
        Vec2::new(r.max.x - 100.0, r.min.y + 56.0),
        Vec3::new(78.0, 178.0, 68.0),
        STEEL,
    );
    // The cooker goes in the widest stretch of wall with no window in it, so
    // its extractor has somewhere to be that is not over glass.
    appliance(
        out,
        Vec2::new(cooker_x, north),
        Vec3::new(76.0, 90.0, 62.0),
        SLATE,
    );
    slab(
        out,
        Vec3::new(cooker_x, 186.0, north - 6.0),
        Vec3::new(80.0, 26.0, 54.0),
        Stuff::Metal,
        STEEL,
    );

    // An island with a worktop proud on every side.
    let island = Vec2::new(m.x + 30.0, m.y + 40.0);
    slab(
        out,
        Vec3::new(island.x, 45.0, island.y),
        Vec3::new(190.0, 78.0, 86.0),
        Stuff::Wood,
        CARCASS,
    );
    slab(
        out,
        Vec3::new(island.x, 89.0, island.y),
        Vec3::new(206.0, 6.0, 102.0),
        Stuff::Stone,
        WORKTOP,
    );
    // Two stools tucked under the island's overhang.
    for s in [-1.0f32, 1.0] {
        let p = island + Vec2::new(s * 52.0, 74.0);
        slab(
            out,
            Vec3::new(p.x, 32.0, p.y),
            Vec3::new(34.0, 64.0, 34.0),
            Stuff::Wood,
            DARK_OAK,
        );
    }

    // Table and four chairs at the open end, toward the great room.
    let table = Vec2::new(m.x, r.max.y - 130.0);
    legged(
        out,
        table,
        Vec2::new(150.0, 92.0),
        74.0,
        5.0,
        8.0,
        OAK,
        DARK_OAK,
    );
    for (dx, dz) in [(-95.0, 0.0), (95.0, 0.0), (0.0, -72.0), (0.0, 72.0)] {
        let c = table + Vec2::new(dx, dz);
        legged(out, c, Vec2::new(42.0, 42.0), 45.0, 4.0, 5.0, OAK, DARK_OAK);
        let away = (c - table).normalize_or_zero();
        let b = c + away * 18.0;
        let across = Vec2::new(away.y.abs(), away.x.abs());
        slab(
            out,
            Vec3::new(b.x, 68.0, b.y),
            Vec3::new(
                if across.x > 0.5 { 42.0 } else { 5.0 },
                46.0,
                if across.x > 0.5 { 5.0 } else { 42.0 },
            ),
            Stuff::Wood,
            DARK_OAK,
        );
    }

    // The bin. The single strongest fly attractor a house has, and the reason a
    // kitchen is worth flying into at all.
    slab(
        out,
        Vec3::new(r.min.x + 40.0, 32.0, r.max.y - 70.0),
        Vec3::new(40.0, 64.0, 40.0),
        Stuff::Metal,
        STEEL,
    );
}

fn bedroom(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();
    // The headboard wall has windows in it, so the bed and the picture over it
    // both go where there are none. Hanging art over glass is exactly the fault
    // the window law exists to catch, and it caught this one.
    let (gap_lo, gap_hi) = clear_of_windows(r);
    let head_x = (gap_lo + gap_hi) * 0.5;
    // Art is sized to the wall it has, not to a fixed width that then has to be
    // clamped somewhere — clamping is what pushed the last picture a centimetre
    // into a window.
    let art_wide = (gap_hi - gap_lo - 24.0).clamp(50.0, 130.0);
    // Which bed by which room it is, not by floor area: the main bedroom of a
    // house has a double in it whether or not it happens to be the largest.
    let double = r.name == "main bedroom";
    let size = if double {
        Vec2::new(150.0, 200.0)
    } else {
        Vec2::new(100.0, 195.0)
    };

    rug(out, m, Vec2::new(r.wide() * 0.5, r.deep() * 0.35), WOOL);
    bed(
        out,
        Vec2::new(head_x, r.min.y + size.y * 0.5 + 40.0),
        size,
        Vec2::new(0.0, 1.0),
    );
    // Nightstands either side of the head.
    for s in [-1.0f32, 1.0] {
        legged(
            out,
            Vec2::new(head_x + s * (size.x * 0.5 + 32.0), r.min.y + 60.0),
            Vec2::new(44.0, 40.0),
            54.0,
            4.0,
            5.0,
            OAK,
            DARK_OAK,
        );
    }
    // A picture over the headboard.
    picture(
        out,
        Vec3::new(head_x, 186.0, r.min.y + 8.0),
        art_wide,
        art_wide * 0.66,
        true,
        Color::srgb(0.38, 0.42, 0.40),
    );

    // A chest of drawers against the far wall.
    slab(
        out,
        Vec3::new(r.max.x - 40.0, 44.0, m.y + 60.0),
        Vec3::new(52.0, 88.0, 130.0),
        Stuff::Wood,
        OAK,
    );
    // And a wardrobe in the corner, which is the tallest thing in the room and
    // the only place with a top surface nobody ever dusts.
    slab(
        out,
        Vec3::new(r.min.x + 38.0, 100.0, r.max.y - 70.0),
        Vec3::new(64.0, 200.0, 130.0),
        Stuff::Wood,
        PAINTED,
    );
}

fn bathroom(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();
    // A vanity run with a basin sunk into it.
    counter_run(
        out,
        Vec2::new(r.min.x + 24.0, r.min.y + 34.0),
        Vec2::new(r.min.x + 190.0, r.min.y + 34.0),
        58.0,
        Vec2::new(0.0, 1.0),
        &[],
    );
    slab(
        out,
        Vec3::new(r.min.x + 100.0, 96.0, r.min.y + 30.0),
        Vec3::new(52.0, 12.0, 40.0),
        Stuff::Stone,
        PORCELAIN,
    );
    // Mirror over it.
    slab(
        out,
        Vec3::new(r.min.x + 100.0, 155.0, r.min.y + 8.0),
        Vec3::new(90.0, 80.0, 3.0),
        Stuff::Glass,
        Color::srgb(0.80, 0.85, 0.88),
    );
    // The lavatory.
    slab(
        out,
        Vec3::new(r.max.x - 50.0, 20.0, r.min.y + 46.0),
        Vec3::new(40.0, 40.0, 60.0),
        Stuff::Stone,
        PORCELAIN,
    );
    slab(
        out,
        Vec3::new(r.max.x - 50.0, 52.0, r.min.y + 24.0),
        Vec3::new(42.0, 64.0, 20.0),
        Stuff::Stone,
        PORCELAIN,
    );
    // A bath along the far wall: four sides and a floor, so it is a basin rather
    // than a block, and a fly can get down inside it.
    let b = Vec2::new(m.x, r.max.y - 46.0);
    let (bw, bd, bh, wall) = (170.0, 76.0, 56.0, 6.0);
    slab(
        out,
        Vec3::new(b.x, wall * 0.5, b.y),
        Vec3::new(bw, wall, bd),
        Stuff::Stone,
        PORCELAIN,
    );
    for s in [-1.0f32, 1.0] {
        slab(
            out,
            Vec3::new(b.x + s * (bw * 0.5 - wall * 0.5), bh * 0.5, b.y),
            Vec3::new(wall, bh, bd),
            Stuff::Stone,
            PORCELAIN,
        );
        slab(
            out,
            Vec3::new(b.x, bh * 0.5, b.y + s * (bd * 0.5 - wall * 0.5)),
            Vec3::new(bw, bh, wall),
            Stuff::Stone,
            PORCELAIN,
        );
    }
}

fn laundry(out: &mut Vec<Solid>, r: &Room) {
    // Washer and dryer side by side. The dryer is the warmest surface in the
    // house, which will matter a great deal to a fly the moment warmth is a
    // thing the game models.
    for (i, paint) in [STEEL, PORCELAIN].into_iter().enumerate() {
        appliance(
            out,
            Vec2::new(r.min.x + 70.0 + i as f32 * 74.0, r.min.y + 46.0),
            Vec3::new(66.0, 88.0, 64.0),
            paint,
        );
    }
    // A folding counter over them, and a shelf above that.
    slab(
        out,
        Vec3::new(r.min.x + 107.0, 92.0, r.min.y + 44.0),
        Vec3::new(156.0, 5.0, 70.0),
        Stuff::Wood,
        OAK,
    );
    shelves(
        out,
        Vec2::new(r.max.x - 90.0, r.min.y + 30.0),
        Vec2::new(150.0, 30.0),
        180.0,
        3,
        true,
    );
}

fn hall(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();
    // A console table against the wall, and a runner down the floor.
    rug(
        out,
        m,
        Vec2::new(r.wide() * 0.55, r.deep() * 0.7),
        WOOL_WARM,
    );
    legged(
        out,
        Vec2::new(r.min.x + 34.0, m.y - 260.0),
        Vec2::new(44.0, 130.0),
        76.0,
        4.0,
        5.0,
        DARK_OAK,
        DARK_OAK,
    );
}

fn garage(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();
    // A workbench along the back.
    counter_run(
        out,
        Vec2::new(r.min.x + 40.0, r.min.y + 40.0),
        Vec2::new(r.min.x + 320.0, r.min.y + 40.0),
        70.0,
        Vec2::new(0.0, 1.0),
        &[],
    );
    // Steel shelving along the east wall, stacked with boxes.
    shelves(
        out,
        Vec2::new(r.max.x - 50.0, m.y),
        Vec2::new(300.0, 46.0),
        210.0,
        4,
        false,
    );
    car(out, Vec2::new(m.x + 40.0, m.y + 40.0));
    garage_door(out);
    // And a stack of boxes in the corner, each sized off its own position.
    // A stack of boxes in the corner. Each sits on the one below rather than on
    // a guessed pitch: sizing them off their own position and then spacing them
    // by a *different* size left the stack floating in steps.
    let mut top = 0.0;
    for i in 0..4 {
        let p = Vec2::new(r.min.x + 96.0, r.max.y - 96.0);
        let s = 46.0 - i as f32 * 4.0 + wobble(p.x + i as f32 * 37.0, p.y) * 7.0;
        let lean = wobble(p.y, p.x + i as f32 * 61.0) * 9.0;
        slab(
            out,
            Vec3::new(p.x + lean, top + s * 0.5, p.y - lean * 0.6),
            Vec3::new(s, s, s * 0.86),
            Stuff::Wood,
            // A shade apart each, or four cardboard boxes stack into one
            // cardboard-coloured panel.
            Color::srgb(
                0.52 + i as f32 * 0.035,
                0.38 + i as f32 * 0.028,
                0.25 + i as f32 * 0.018,
            ),
        );
        top += s;
    }
}
