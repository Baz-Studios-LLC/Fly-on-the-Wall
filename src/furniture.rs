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
fn counter_run(out: &mut Vec<Solid>, from: Vec2, to: Vec2, depth: f32, face: Vec2) {
    const PLINTH: f32 = 12.0;
    const HIGH: f32 = 91.0;
    const TOP: f32 = 4.0;
    const OVER: f32 = 4.0;

    let mid = (from + to) * 0.5;
    let run = (to - from).length();
    let along = (to - from).normalize_or_zero();
    // Size along the run, and across it.
    let size = |long: f32, across: f32| {
        Vec3::new(
            if along.x.abs() > 0.5 { long } else { across },
            0.0,
            if along.x.abs() > 0.5 { across } else { long },
        )
    };

    // Plinth, set back so the toe kick is a real recess.
    let kick = 6.0;
    let plinth_at = mid + face * kick * 0.5;
    let s = size(run, depth - kick);
    slab(
        out,
        Vec3::new(plinth_at.x, PLINTH * 0.5, plinth_at.y),
        Vec3::new(s.x, PLINTH, s.z),
        Stuff::Wood,
        SLATE,
    );
    // Carcass.
    let s = size(run, depth);
    slab(
        out,
        Vec3::new(mid.x, PLINTH + (HIGH - TOP - PLINTH) * 0.5, mid.y),
        Vec3::new(s.x, HIGH - TOP - PLINTH, s.z),
        Stuff::Wood,
        CARCASS,
    );
    // Worktop, proud of the carcass on the room side.
    let top_at = mid - face * OVER * 0.5;
    let s = size(run, depth + OVER);
    slab(
        out,
        Vec3::new(top_at.x, HIGH - TOP * 0.5, top_at.y),
        Vec3::new(s.x, TOP, s.z),
        Stuff::Stone,
        WORKTOP,
    );
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
    let (w, d) = if along_x {
        (size.x, size.y)
    } else {
        (size.y, size.x)
    };
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

fn kitchen(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();
    // An L of base units along the north and west walls, with cabinets over the
    // north run.
    let north = r.min.y + 32.0;
    counter_run(
        out,
        Vec2::new(r.min.x + 20.0, north),
        Vec2::new(r.max.x - 140.0, north),
        64.0,
        Vec2::new(0.0, 1.0),
    );
    wall_cabinets(
        out,
        Vec2::new(r.min.x + 40.0, r.min.y + 18.0),
        Vec2::new(r.max.x - 200.0, r.min.y + 18.0),
        36.0,
    );
    counter_run(
        out,
        Vec2::new(r.min.x + 32.0, north + 40.0),
        Vec2::new(r.min.x + 32.0, north + 260.0),
        64.0,
        Vec2::new(1.0, 0.0),
    );

    // The fridge, standing off the wall. The gap behind it is the point.
    appliance(
        out,
        Vec2::new(r.max.x - 90.0, r.min.y + 52.0),
        Vec3::new(76.0, 178.0, 70.0),
        STEEL,
    );
    // The cooker, in the run.
    appliance(
        out,
        Vec2::new(r.min.x + 200.0, north),
        Vec3::new(76.0, 90.0, 64.0),
        SLATE,
    );

    // An island with a worktop overhang all round — the best flying obstacle in
    // the house, and the best perch under it.
    let island = Vec2::new(m.x + 40.0, m.y + 30.0);
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

    // A table and four chairs at the front of the room.
    let table = Vec2::new(m.x + 20.0, r.max.y - 150.0);
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
        // Chair back, on whichever side faces the table.
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

    // The bin. The single strongest fly attractor in a house, and the reason a
    // kitchen is worth flying into at all.
    slab(
        out,
        Vec3::new(r.min.x + 34.0, 32.0, r.max.y - 60.0),
        Vec3::new(40.0, 64.0, 40.0),
        Stuff::Metal,
        STEEL,
    );
}

fn bedroom(out: &mut Vec<Solid>, r: &Room) {
    let m = r.middle();
    let double = r.wide() * r.deep() > 250_000.0;
    let size = if double {
        Vec2::new(150.0, 200.0)
    } else {
        Vec2::new(100.0, 195.0)
    };

    rug(out, m, Vec2::new(r.wide() * 0.5, r.deep() * 0.35), WOOL);
    bed(
        out,
        Vec2::new(m.x, r.min.y + size.y * 0.5 + 40.0),
        size,
        Vec2::new(0.0, 1.0),
    );
    // Nightstands either side of the head.
    for s in [-1.0f32, 1.0] {
        legged(
            out,
            Vec2::new(m.x + s * (size.x * 0.5 + 32.0), r.min.y + 60.0),
            Vec2::new(44.0, 40.0),
            54.0,
            4.0,
            5.0,
            OAK,
            DARK_OAK,
        );
    }
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
    // A car-sized box, because a garage without one reads as a hall.
    slab(
        out,
        Vec3::new(m.x + 40.0, 72.0, m.y + 60.0),
        Vec3::new(190.0, 144.0, 450.0),
        Stuff::Metal,
        Color::srgb(0.28, 0.32, 0.38),
    );
    // And a stack of boxes in the corner, each sized off its own position.
    for i in 0..4 {
        let p = Vec2::new(r.min.x + 90.0 + i as f32 * 12.0, r.max.y - 90.0);
        let s = 44.0 + wobble(p.x, p.y) * 8.0;
        slab(
            out,
            Vec3::new(p.x, s * 0.5 + i as f32 * s, p.y),
            Vec3::new(s, s, s),
            Stuff::Wood,
            WOOL_WARM,
        );
    }
}
