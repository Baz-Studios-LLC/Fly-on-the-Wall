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
const WOOL_WARM: Color = Color::srgb(0.64, 0.55, 0.47);
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
const FAN_METAL: Color = Color::srgb(0.30, 0.30, 0.32);
const FAN_BLADE: Color = Color::srgb(0.36, 0.26, 0.18);
const DUVET: Color = Color::srgb(0.72, 0.74, 0.78);
const THROW: Color = Color::srgb(0.46, 0.40, 0.36);
const SHADE: Color = Color::srgb(0.90, 0.86, 0.76);
const SEAT_RING: Color = Color::srgb(0.93, 0.93, 0.91);
const CHROME: Color = Color::srgb(0.78, 0.80, 0.82);
const TOWEL_A: Color = Color::srgb(0.66, 0.72, 0.74);
const TOWEL_B: Color = Color::srgb(0.82, 0.80, 0.74);
const DOOR_FACE: Color = Color::srgb(0.30, 0.22, 0.16);
const DOOR_PANEL: Color = Color::srgb(0.26, 0.19, 0.14);
const TRIMWORK: Color = Color::srgb(0.92, 0.91, 0.88);
const THRESHOLD: Color = Color::srgb(0.42, 0.32, 0.22);
const BRASS: Color = Color::srgb(0.72, 0.60, 0.30);
const PAVING: Color = Color::srgb(0.62, 0.61, 0.58);
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
    // Doors on the front, with a shadow gap between them and a handle under
    // each. A run of wall units with no fronts is a shelf with a lid on it.
    let face = Vec2::new(along.y, -along.x);
    let doors = ((run / 46.0).round() as usize).max(1);
    for k in 0..doors {
        let t = (k as f32 + 0.5) / doors as f32 - 0.5;
        let at = mid + along * (run * t) + face * (depth * 0.5 + 1.5);
        let leaf = run / doors as f32 - 3.0;
        let (dx, dz) = if along.x.abs() > 0.5 {
            (leaf, 3.0)
        } else {
            (3.0, leaf)
        };
        slab(
            out,
            Vec3::new(at.x, UNDER + TALL * 0.5, at.y),
            Vec3::new(dx, TALL - 5.0, dz),
            Stuff::Wood,
            Color::srgb(0.88, 0.88, 0.86),
        );
        let pull = at + face * 2.0;
        let (hx, hz) = if along.x.abs() > 0.5 {
            (leaf * 0.5, 3.0)
        } else {
            (3.0, leaf * 0.5)
        };
        slab(
            out,
            Vec3::new(pull.x, UNDER + 8.0, pull.y),
            Vec3::new(hx, 3.0, hz),
            Stuff::Metal,
            CHROME,
        );
    }
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
    // Bedding. A bare mattress with two pillows on it is a showroom bed; what
    // makes one look slept in is the duvet stopping short of the pillows and
    // the sheet turned back over its edge.
    let top = CLEAR + FRAME + MATTRESS;
    let long = size.y.max(size.x);
    let duvet = long * 0.66;
    let foot = at + facing * (long * 0.5 - duvet * 0.5);
    let along = Vec2::new(facing.x.abs(), facing.y.abs());
    let bed_size = |a: f32, c: f32| {
        if along.y > 0.5 {
            Vec3::new(c, 0.0, a)
        } else {
            Vec3::new(a, 0.0, c)
        }
    };
    let d = bed_size(duvet, size.x.min(size.y) + 12.0);
    soft(
        out,
        Vec3::new(foot.x, top + 1.0, foot.y),
        Vec3::new(d.x, 11.0, d.z),
        3.5,
        Stuff::Fabric,
        DUVET,
    );
    let fold = at + facing * (long * 0.5 - duvet - 8.0);
    let f = bed_size(20.0, size.x.min(size.y) + 12.0);
    slab(
        out,
        Vec3::new(fold.x, top + 2.0, fold.y),
        Vec3::new(f.x, 8.0, f.z),
        Stuff::Fabric,
        LINEN,
    );
    // A throw folded across the foot.
    let throw_at = at + facing * (long * 0.5 - 26.0);
    let t = bed_size(44.0, size.x.min(size.y) + 13.0);
    soft(
        out,
        Vec3::new(throw_at.x, top + 9.0, throw_at.y),
        Vec3::new(t.x, 6.0, t.z),
        2.0,
        Stuff::Fabric,
        THROW,
    );

    // Pillows.
    for s in [-1.0f32, 1.0] {
        let p = at - facing * (size.y.max(size.x) * 0.34)
            + Vec2::new(across.x, across.y) * s * size.x.min(size.y) * 0.24;
        soft(
            out,
            Vec3::new(p.x, CLEAR + FRAME + MATTRESS + 7.0, p.y),
            Vec3::new(48.0, 14.0, 32.0),
            4.0,
            Stuff::Fabric,
            PORCELAIN,
        );
    }
}

/// A chair: a legged seat with a back on the side `away` points to.
///
/// The kitchen had four of these written out inline, which is exactly the kind
/// of repeated recognisable form that wants a constructor — the bedrooms needed
/// one the moment they got a desk.
fn chair(out: &mut Vec<Solid>, at: Vec2, away: Vec2) {
    legged(
        out,
        at,
        Vec2::new(42.0, 42.0),
        45.0,
        4.0,
        5.0,
        OAK,
        DARK_OAK,
    );
    let b = at + away * 18.0;
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

/// A wardrobe: plinth, carcass, two doors with a shadow gap between them, two
/// handles and a cornice. It was a single painted box, which is the one piece
/// of furniture in a bedroom nobody can mistake for anything else and so the
/// one least worth leaving as a box.
fn wardrobe(out: &mut Vec<Solid>, at: Vec2, size: Vec3, face: Vec2) {
    let across = Vec2::new(face.y.abs(), face.x.abs());
    let wide = if across.x > 0.5 { size.x } else { size.z };
    slab(
        out,
        Vec3::new(at.x, 5.0, at.y),
        Vec3::new(size.x - 6.0, 10.0, size.z - 6.0),
        Stuff::Wood,
        DARK_OAK,
    );
    slab(
        out,
        Vec3::new(at.x, size.y * 0.5 + 8.0, at.y),
        Vec3::new(size.x, size.y, size.z),
        Stuff::Wood,
        PAINTED,
    );
    slab(
        out,
        Vec3::new(at.x, size.y + 12.0, at.y),
        Vec3::new(size.x + 8.0, 8.0, size.z + 8.0),
        Stuff::Wood,
        PAINTED,
    );
    for side in [-1.0f32, 1.0] {
        let off = across * side * wide * 0.25;
        let front = face * 2.0;
        let leaf = if across.x > 0.5 {
            Vec3::new(wide * 0.47, size.y - 18.0, 4.0)
        } else {
            Vec3::new(4.0, size.y - 18.0, wide * 0.47)
        };
        slab(
            out,
            Vec3::new(
                at.x + off.x + front.x * (size.x * 0.5),
                size.y * 0.5 + 8.0,
                at.y + off.y + front.y * (size.z * 0.5),
            ),
            leaf,
            Stuff::Wood,
            Color::srgb(0.86, 0.85, 0.82),
        );
        let knob_at = across * side * 7.0;
        slab(
            out,
            Vec3::new(
                at.x + knob_at.x + face.x * (size.x * 0.5 + 3.0),
                size.y * 0.55,
                at.y + knob_at.y + face.y * (size.z * 0.5 + 3.0),
            ),
            Vec3::new(5.0, 22.0, 5.0),
            Stuff::Metal,
            BRASS,
        );
    }
}

/// A chest of drawers: carcass, plinth, drawer fronts with a shadow gap between
/// them, and a handle on each. A chest without fronts is a cube, and a bedroom
/// with a cube in it is a bedroom nobody has finished.
fn drawers(out: &mut Vec<Solid>, at: Vec2, size: Vec3, face: Vec2, rows: usize) {
    slab(
        out,
        Vec3::new(at.x, 5.0, at.y),
        Vec3::new(size.x - 8.0, 10.0, size.z - 8.0),
        Stuff::Wood,
        DARK_OAK,
    );
    slab(
        out,
        Vec3::new(at.x, size.y * 0.5 + 8.0, at.y),
        Vec3::new(size.x, size.y, size.z),
        Stuff::Wood,
        OAK,
    );
    slab(
        out,
        Vec3::new(at.x, size.y + 10.0, at.y),
        Vec3::new(size.x + 6.0, 5.0, size.z + 6.0),
        Stuff::Wood,
        DARK_OAK,
    );
    let across = Vec2::new(face.y.abs(), face.x.abs());
    let wide = if across.x > 0.5 { size.x } else { size.z };
    let front = Vec3::new(
        face.x * (size.x * 0.5 + 1.5),
        0.0,
        face.y * (size.z * 0.5 + 1.5),
    );
    let leaf = if across.x > 0.5 {
        Vec3::new(wide - 12.0, 0.0, 4.0)
    } else {
        Vec3::new(4.0, 0.0, wide - 12.0)
    };
    for k in 0..rows {
        let high = (size.y - 12.0) / rows as f32;
        let y = 14.0 + high * (k as f32 + 0.5);
        slab(
            out,
            Vec3::new(at.x, y, at.y) + front,
            Vec3::new(leaf.x, high - 5.0, leaf.z),
            Stuff::Wood,
            Color::srgb(0.50, 0.38, 0.26),
        );
        let pull = if across.x > 0.5 {
            Vec3::new(wide * 0.32, 4.0, 4.0)
        } else {
            Vec3::new(4.0, 4.0, wide * 0.32)
        };
        slab(
            out,
            Vec3::new(at.x, y, at.y) + front * 2.0,
            pull,
            Stuff::Metal,
            BRASS,
        );
    }
}

/// An eight-sided disc lying flat: four crossed bars, all eight corners on the
/// same circle. The same construction as the car's wheels and the ceiling
/// roses, which is the third place it has earned its keep.
fn disc(out: &mut Vec<Solid>, at: Vec3, across: f32, thick: f32, paint: Color, glow: f32) {
    const SIDES: usize = 4;
    let bar = across * (std::f32::consts::PI / (2.0 * SIDES as f32)).tan();
    for k in 0..SIDES {
        let mut solid = Solid::between(
            Vec3::new(-across * 0.5, -thick * 0.5, -bar * 0.5),
            Vec3::new(across * 0.5, thick * 0.5, bar * 0.5),
            Stuff::Metal,
        );
        solid.center = at;
        solid.rot = Quat::from_rotation_y(k as f32 * std::f32::consts::PI / SIDES as f32);
        solid.paint = Some(paint);
        solid.glow = glow;
        out.push(solid);
    }
}

/// A cluster of frames on a wall — a gallery, not a single picture centred on
/// nothing. Real walls have five of them at four sizes, not quite aligned.
fn frames(out: &mut Vec<Solid>, at: Vec3, along_x: bool, spread: f32, seed: f32) {
    // Frame, mount, image. The first version painted the frame and the picture
    // in it the same darkness, and five of those on a wall read as five brown
    // blocks — it is the pale mount between the two that says "picture".
    let art = [
        Color::srgb(0.62, 0.52, 0.40),
        Color::srgb(0.34, 0.44, 0.52),
        Color::srgb(0.70, 0.62, 0.44),
        Color::srgb(0.40, 0.46, 0.38),
        Color::srgb(0.58, 0.44, 0.46),
    ];
    for k in 0..5 {
        let n = wobble(seed + k as f32 * 17.0, at.y);
        let m = wobble(at.y + k as f32 * 29.0, seed);
        let along = (k as f32 - 2.0) * spread * 0.21 + n * spread * 0.035;
        let up = m * 16.0 + if k % 2 == 0 { 10.0 } else { -10.0 };
        let wide = 30.0 + n.abs() * 24.0;
        let high = wide * (0.72 + m * 0.26);
        let put = |o: &mut Vec<Solid>, w: f32, h: f32, out_by: f32, paint: Color| {
            let centre = if along_x {
                Vec3::new(at.x + along, at.y + up, at.z + out_by)
            } else {
                Vec3::new(at.x + out_by, at.y + up, at.z + along)
            };
            let size = if along_x {
                Vec3::new(w, h, 2.0)
            } else {
                Vec3::new(2.0, h, w)
            };
            slab(o, centre, size, Stuff::Wood, paint);
        };
        let face = if along_x { 1.0 } else { -1.0 };
        put(out, wide, high, 0.0, DARK_OAK);
        put(
            out,
            wide - 7.0,
            high - 7.0,
            face * 1.4,
            Color::srgb(0.92, 0.91, 0.88),
        );
        put(out, wide - 19.0, high - 19.0, face * 2.2, art[k]);
    }
}

/// A standard lamp: foot, stem, and a shade you can see the underside of.
fn floor_lamp(out: &mut Vec<Solid>, at: Vec2) {
    disc(
        out,
        Vec3::new(at.x, 2.0, at.y),
        34.0,
        4.0,
        Color::srgb(0.26, 0.26, 0.27),
        0.0,
    );
    slab(
        out,
        Vec3::new(at.x, 66.0, at.y),
        Vec3::new(4.0, 130.0, 4.0),
        Stuff::Metal,
        BRASS,
    );
    const SIDES: usize = 4;
    let across = 40.0;
    let bar = across * (std::f32::consts::PI / (2.0 * SIDES as f32)).tan();
    for k in 0..SIDES {
        turned(
            out,
            Vec3::new(at.x, 146.0, at.y),
            Vec3::new(across, 30.0, bar),
            Quat::from_rotation_y(k as f32 * std::f32::consts::PI / SIDES as f32),
            Stuff::Fabric,
            SHADE,
        );
    }
}

/// A basket, with whatever has been dropped in it.
fn basket(out: &mut Vec<Solid>, at: Vec2, wide: f32, tall: f32, paint: Color) {
    disc(
        out,
        Vec3::new(at.x, tall * 0.5, at.y),
        wide,
        tall,
        paint,
        0.0,
    );
    disc(
        out,
        Vec3::new(at.x, tall, at.y),
        wide * 1.06,
        tall * 0.16,
        Color::srgb(0.52, 0.44, 0.34),
        0.0,
    );
}

/// A stack of books, each one a shade and a size off the one under it, with
/// the top one nudged out of square.
fn books(out: &mut Vec<Solid>, at: Vec3, how_many: usize, seed: f32) {
    let mut y = at.y;
    for k in 0..how_many {
        let n = wobble(at.x + seed, at.z + k as f32 * 13.0);
        let thick = 3.5 + n.abs() * 2.5;
        let wide = 20.0 + n * 3.0;
        let deep = 15.0 + n.abs() * 2.0;
        turned(
            out,
            Vec3::new(at.x + n * 2.4, y + thick * 0.5, at.z + n * 1.8),
            Vec3::new(wide, thick, deep),
            Quat::from_rotation_y(n * 0.22),
            Stuff::Wood,
            Color::srgb(
                0.34 + (k as f32 * 0.13 + seed * 0.7).sin().abs() * 0.36,
                0.28 + (k as f32 * 0.31 + seed).sin().abs() * 0.30,
                0.26 + (k as f32 * 0.47 + seed * 1.3).sin().abs() * 0.34,
            ),
        );
        y += thick;
    }
}

/// A mug: a body and a handle, both small enough that a fly could stand on the
/// rim and look in.
fn mug(out: &mut Vec<Solid>, at: Vec3, paint: Color) {
    disc(out, at + Vec3::new(0.0, 5.0, 0.0), 9.0, 10.0, paint, 0.0);
    slab(
        out,
        at + Vec3::new(6.0, 5.5, 0.0),
        Vec3::new(4.0, 5.0, 2.5),
        Stuff::Stone,
        paint,
    );
}

/// A pot plant: a tapered pot, a little soil, and a mass of leaves.
fn pot_plant(out: &mut Vec<Solid>, at: Vec2, tall: f32) {
    let pot = tall * 0.28;
    disc(
        out,
        Vec3::new(at.x, pot * 0.5, at.y),
        pot * 0.86,
        pot,
        Color::srgb(0.44, 0.30, 0.22),
        0.0,
    );
    disc(
        out,
        Vec3::new(at.x, pot + 1.0, at.y),
        pot * 0.94,
        6.0,
        Color::srgb(0.48, 0.33, 0.24),
        0.0,
    );
    // A stem, then leaves. Six fat boxes at half the plant's height across read
    // as a hedge in a pot; nine thin ones on a stem read as a plant.
    slab(
        out,
        Vec3::new(at.x, pot + tall * 0.22, at.y),
        Vec3::new(4.0, tall * 0.44, 4.0),
        Stuff::Wood,
        Color::srgb(0.34, 0.29, 0.20),
    );
    for k in 0..9 {
        let n = wobble(at.x + k as f32 * 19.0, at.y + k as f32 * 7.0);
        let lift = 0.40 + k as f32 * 0.062;
        let wide = tall * (0.30 - k as f32 * 0.022) * (1.0 + n * 0.22);
        let mut leaf = Solid::between(
            Vec3::new(-wide * 0.5, -tall * 0.035, -wide * 0.22),
            Vec3::new(wide * 0.5, tall * 0.035, wide * 0.22),
            Stuff::Grass,
        );
        leaf.center = Vec3::new(at.x + n * 6.0, tall * lift, at.y + n * 5.0);
        leaf.rot = Quat::from_rotation_y(k as f32 * 0.71 + n * 0.6)
            * Quat::from_rotation_z(0.18 + n * 0.22);
        leaf.paint = Some(Color::srgb(
            0.15 + k as f32 * 0.009,
            0.29 + k as f32 * 0.017,
            0.16 + k as f32 * 0.008,
        ));
        out.push(leaf);
    }
}

/// A ceiling fan: downrod, motor, five pitched blades and a light under it.
///
/// The ceiling had one flush fixture in it and was otherwise a featureless
/// plane — which matters more here than in most games, because it is where the
/// player starts and where a fly spends its time. A fan is the right answer
/// twice over: it is what a ranch house of this period has, and at fly scale it
/// is five landing strips and a set of edges to walk round.
fn ceiling_fan(out: &mut Vec<Solid>, at: Vec2, ceiling: f32) {
    let hub = ceiling - 44.0;
    slab(
        out,
        Vec3::new(at.x, ceiling - 14.0, at.y),
        Vec3::new(6.0, 28.0, 6.0),
        Stuff::Metal,
        FAN_METAL,
    );
    disc(
        out,
        Vec3::new(at.x, ceiling - 4.0, at.y),
        26.0,
        6.0,
        FAN_METAL,
        0.0,
    );
    disc(out, Vec3::new(at.x, hub, at.y), 34.0, 18.0, FAN_METAL, 0.0);
    for k in 0..5 {
        let yaw = k as f32 * std::f32::consts::TAU / 5.0;
        let turn = Quat::from_rotation_y(yaw);
        turned(
            out,
            Vec3::new(at.x, hub - 2.0, at.y) + turn * Vec3::new(66.0, 0.0, 0.0),
            Vec3::new(92.0, 3.0, 24.0),
            turn * Quat::from_rotation_x(0.22),
            Stuff::Wood,
            FAN_BLADE,
        );
    }
    // The light kit, which is where this room's lamp already is.
    disc(
        out,
        Vec3::new(at.x, hub - 18.0, at.y),
        30.0,
        14.0,
        Color::srgb(1.0, 0.97, 0.90),
        11.0,
    );
}

/// A table lamp: foot, stem, and an eight-sided shade.
fn lamp(out: &mut Vec<Solid>, at: Vec3) {
    slab(
        out,
        at + Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(18.0, 4.0, 18.0),
        Stuff::Metal,
        BRASS,
    );
    slab(
        out,
        at + Vec3::new(0.0, 17.0, 0.0),
        Vec3::new(5.0, 30.0, 5.0),
        Stuff::Metal,
        BRASS,
    );
    const SIDES: usize = 4;
    let across = 32.0;
    let bar = across * (std::f32::consts::PI / (2.0 * SIDES as f32)).tan();
    for k in 0..SIDES {
        turned(
            out,
            at + Vec3::new(0.0, 44.0, 0.0),
            Vec3::new(across, 26.0, bar),
            Quat::from_rotation_y(k as f32 * std::f32::consts::PI / SIDES as f32),
            Stuff::Fabric,
            SHADE,
        );
    }
}

/// A box with its corners taken off: three crossed slabs, each full length on
/// one axis and inset on the other two.
///
/// Everything soft in this house has been a hard-edged box, and a cushion with
/// eight sharp corners is the one shape upholstery never has. Three boxes
/// instead of one is the cheapest thing that fixes it — the corners go, the
/// silhouette softens, and at the size a cushion is drawn nobody counts faces.
fn soft(out: &mut Vec<Solid>, at: Vec3, size: Vec3, round: f32, stuff: Stuff, paint: Color) {
    let r = round.min(size.min_element() * 0.34);
    for cut in [
        Vec3::new(0.0, r, r),
        Vec3::new(r, 0.0, r),
        Vec3::new(r, r, 0.0),
    ] {
        slab(out, at, size - cut * 2.0, stuff, paint);
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
    soft(
        out,
        Vec3::new(at.x, 34.0, at.y),
        Vec3::new(s.x, 20.0, s.z),
        5.0,
        Stuff::Fabric,
        WOOL,
    );
    // Back.
    let b = at + back * (deep * 0.5 - 8.0);
    let s = dim(long, 16.0);
    soft(
        out,
        Vec3::new(b.x, 56.0, b.y),
        Vec3::new(s.x, 64.0, s.z),
        4.0,
        Stuff::Fabric,
        WOOL,
    );
    // Arms, with a roll along the top. A square arm is the giveaway that a
    // sofa was made of boxes; the roll is eight bars round a cylinder and it
    // is the first thing the eye reads on the silhouette.
    for side in [-1.0f32, 1.0] {
        let a = at + across * side * (long * 0.5 - 8.0);
        let s = dim(16.0, deep);
        slab(
            out,
            Vec3::new(a.x, 34.0, a.y),
            Vec3::new(s.x, 20.0, s.z),
            Stuff::Fabric,
            WOOL,
        );
        const SIDES: usize = 4;
        let across_bar = 16.0 * (std::f32::consts::PI / (2.0 * SIDES as f32)).tan();
        for k in 0..SIDES {
            let turn = k as f32 * std::f32::consts::PI / SIDES as f32;
            let (rot, bar) = if across.x > 0.5 {
                (
                    Quat::from_rotation_x(turn),
                    Vec3::new(16.0, 32.0, across_bar),
                )
            } else {
                (
                    Quat::from_rotation_z(turn),
                    Vec3::new(across_bar, 32.0, 16.0),
                )
            };
            let long_bar = if across.x > 0.5 {
                Vec3::new(bar.x, bar.y, bar.z)
            } else {
                Vec3::new(bar.x, bar.y, bar.z)
            };
            turned(
                out,
                Vec3::new(a.x, 50.0, a.y),
                long_bar,
                rot,
                Stuff::Fabric,
                WOOL,
            );
        }
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
        // A row of books. Five evenly spaced blocks in two colours is a comb;
        // what makes a shelf read is uneven widths, a couple of leaners, a
        // stack lying flat, and a gap where somebody took something out.
        let mut along = -(w * 0.5) + 12.0;
        let stop = w * 0.5 - 12.0;
        let mut b = 0usize;
        while along < stop {
            let n = wobble(at.x + along, at.y + i as f32 * 13.0);
            let m = wobble(at.y + i as f32 * 31.0, along * 1.7);
            b += 1;

            // A gap, now and then.
            if m > 0.62 {
                along += 14.0 + m * 10.0;
                continue;
            }
            let p = if along_x {
                Vec2::new(at.x + along, at.y)
            } else {
                Vec2::new(at.x, at.y + along)
            };

            // A stack lying on its side.
            if n > 0.55 {
                books(out, Vec3::new(p.x, y + 1.5, p.y), 2 + (b % 2), along);
                along += 26.0;
                continue;
            }

            let thick = 6.0 + (n + 1.0) * 5.0;
            let h = 19.0 + m.abs() * 9.0;
            let s = dim(thick, d * 0.66);
            let lean = if n < -0.72 { 0.16 } else { 0.0 };
            let paint = [
                Color::srgb(0.42, 0.24, 0.20),
                Color::srgb(0.24, 0.30, 0.36),
                Color::srgb(0.52, 0.46, 0.30),
                Color::srgb(0.30, 0.36, 0.28),
                Color::srgb(0.46, 0.36, 0.44),
            ][b % 5];
            turned(
                out,
                Vec3::new(p.x, y + 1.5 + h * 0.5, p.y),
                Vec3::new(s.x.max(3.0), h, s.z.max(3.0)),
                if along_x {
                    Quat::from_rotation_z(lean)
                } else {
                    Quat::from_rotation_x(lean)
                },
                Stuff::Wood,
                paint,
            );
            along += thick + 1.5;
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
    // Which way is "into the room"? The comment here used to claim it worked
    // this out and the code just used +z, so every picture on a south or east
    // wall had its mount and image buried inside the plaster and rendered as a
    // dark block. Ask, the same way the paint and the cladding ask: probe
    // twenty centimetres either side and see which one is in a room.
    let axis = if along_x { Vec3::Z } else { Vec3::X };
    // Face the middle of the room the picture is *in*. Probing a fixed distance
    // either side does not work: a partition is twelve centimetres thick and a
    // picture hangs eight off the face, so a twenty-centimetre probe crosses
    // the wall and finds the room next door, and a short one is inside this
    // room on both sides. The room's own middle is unambiguous.
    let side = crate::house::room_at(Vec2::new(at.x, at.z))
        .map(|room| {
            let heart = room.middle();
            let toward = if along_x {
                heart.y - at.z
            } else {
                heart.x - at.x
            };
            if toward < 0.0 { -1.0 } else { 1.0 }
        })
        .unwrap_or(1.0);
    let out_of = axis * side * 1.4;
    let face = Vec3::ONE;
    // Mount, then image. Frame and image alone, both dark, read as one dark
    // block on a wall — it is the pale mount between them that makes the eye
    // call the thing a picture. Every framed thing in the house goes through
    // here, so this is the one place worth fixing it.
    let s = d(1.6);
    let inset = |by: f32| {
        Vec3::new(
            (s.x - by).max(1.6),
            (s.y - by).max(1.6),
            (s.z - by).max(1.6),
        )
    };
    let _ = face;
    slab(
        out,
        at + out_of,
        inset(7.0),
        Stuff::Fabric,
        Color::srgb(0.92, 0.91, 0.88),
    );
    slab(out, at + out_of * 1.6, inset(20.0), Stuff::Fabric, tone);
}

/// Curtains: a pair of panels either side of a window, and a pole across it.
///
/// Hung on every window in the house from one pass, because the windows already
/// know where they are and asking them is the only way this stays right when one
/// moves.
/// A curtain, in folds.
///
/// One slab of fabric is a plank, and there is a pair of them at every window
/// in the house. Five narrow strips with every other one pushed forward is a
/// gather: the front faces catch the light and the ones behind fall into
/// shadow, which is the whole of what a fold looks like from across a room.
fn pleat(out: &mut Vec<Solid>, at: Vec3, size: Vec3, along_x: bool) {
    const FOLDS: usize = 5;
    let wide = if along_x { size.x } else { size.z };
    let strip = wide / FOLDS as f32;
    for k in 0..FOLDS {
        let off = (k as f32 + 0.5) / FOLDS as f32 - 0.5;
        // Alternate strips stand proud. Every strip runs the full drop: the
        // first version tapered the outer two and the curtain came out with a
        // staircase down its edge, which is worse than the plank it replaced.
        let forward = if k % 2 == 0 { 2.6 } else { -1.4 };
        let centre = if along_x {
            Vec3::new(at.x + off * wide, at.y, at.z + forward)
        } else {
            Vec3::new(at.x + forward, at.y, at.z + off * wide)
        };
        let s = if along_x {
            Vec3::new(strip * 1.06, size.y, size.z * 0.74)
        } else {
            Vec3::new(size.x * 0.74, size.y, strip * 1.06)
        };
        // A shade between the folds, not a stripe: the light does most of it.
        let shade = if k % 2 == 0 { 0.0 } else { -0.03 };
        slab(
            out,
            centre,
            s,
            Stuff::Fabric,
            Color::srgb(0.42 + shade, 0.32 + shade, 0.25 + shade),
        );
    }
}

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
                pleat(out, at + inward, full, along_x);
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
                pleat(out, raised, size, along_x);
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

    // Sill and shoulder: the car's mass, in two steps rather than one slab,
    // with the corners taken off both.
    soft(
        out,
        Vec3::new(x, 53.0, z),
        Vec3::new(178.0, 54.0, 430.0),
        7.0,
        Stuff::Metal,
        PAINTWORK,
    );
    soft(
        out,
        Vec3::new(x, 97.0, z),
        Vec3::new(172.0, 34.0, 418.0),
        6.0,
        Stuff::Metal,
        PAINTWORK,
    );
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
    //
    // The roof is softened: a car has no square corner anywhere on it, and the
    // roofline is the edge the eye checks first.
    soft(
        out,
        Vec3::new(x, 157.0, z - 8.0),
        Vec3::new(150.0, 14.0, 184.0),
        5.0,
        Stuff::Metal,
        PAINTWORK,
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
        //
        // Eight centimetres tall, not three: the panels stop three short of the
        // joint on each side, so a three-centimetre reveal left a pair of
        // one-and-a-half-centimetre slots per section — daylight and grass in a
        // line right across a shut garage door.
        slab(
            out,
            Vec3::new(mid_x, low, z),
            Vec3::new(wide, 8.0, 6.0),
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
                Vec3::new(wide, h + 2.0, 9.0),
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
            Vec3::new(wide, 8.0, 6.0),
            Stuff::Metal,
            BUMPER,
        );
    }
}

/// A panelled door, hung on its hinge edge and left standing at `ajar`.
///
/// Built in the door's own frame — x across the leaf from the hinge, y up from
/// the threshold, z through its thickness — and then swung, so the geometry
/// does not have to be re-derived for every angle. Stiles, three rails and two
/// recessed panels: the same stile-and-rail construction a real door has, which
/// is why the shadow lines fall where a person expects them to.
fn door_leaf(out: &mut Vec<Solid>, hinge: Vec3, wide: f32, high: f32, swing: Quat) {
    let leaf = 4.6;
    let mut put = |x0: f32, x1: f32, y0: f32, y1: f32, thick: f32, paint: Color| {
        let local = Vec3::new((x0 + x1) * 0.5, (y0 + y1) * 0.5, 0.0);
        turned(
            out,
            hinge + swing * local,
            Vec3::new(x1 - x0, y1 - y0, thick),
            swing,
            Stuff::Wood,
            paint,
        );
    };
    let stile = 13.0;
    let rail = 16.0;
    put(0.0, stile, 0.0, high, leaf, DOOR_FACE);
    put(wide - stile, wide, 0.0, high, leaf, DOOR_FACE);
    put(stile, wide - stile, 0.0, rail + 6.0, leaf, DOOR_FACE);
    put(
        stile,
        wide - stile,
        high * 0.47,
        high * 0.47 + rail,
        leaf,
        DOOR_FACE,
    );
    put(stile, wide - stile, high - rail, high, leaf, DOOR_FACE);
    // The two panels, set back inside the frame they sit in.
    put(
        stile,
        wide - stile,
        rail + 6.0,
        high * 0.47,
        leaf * 0.55,
        DOOR_PANEL,
    );
    put(
        stile,
        wide - stile,
        high * 0.47 + rail,
        high - rail,
        leaf * 0.55,
        DOOR_PANEL,
    );
    // A knob each side of the leaf, on the swinging edge.
    for face in [-1.0f32, 1.0] {
        let local = Vec3::new(wide - 20.0, high * 0.44, face * (leaf * 0.5 + 2.5));
        turned(
            out,
            hinge + swing * local,
            Vec3::new(7.0, 7.0, 5.0),
            swing,
            Stuff::Metal,
            BRASS,
        );
    }
}

/// Lining and architrave round an opening in a wall that runs north to south.
///
/// A hole with square plaster edges reads as unfinished from every angle; the
/// lining and the band of casing round it are most of what makes a doorway look
/// like part of a built house.
fn case_opening(out: &mut Vec<Solid>, lo: Vec3, hi: Vec3) {
    let x = (lo.x + hi.x) * 0.5;
    let thick = hi.x - lo.x;
    let mid_z = (lo.z + hi.z) * 0.5;
    let wide = hi.z - lo.z;
    // Lining: the reveal itself, boarded out.
    for side in [-1.0f32, 1.0] {
        slab(
            out,
            Vec3::new(x, hi.y * 0.5, mid_z + side * (wide * 0.5 - 1.5)),
            Vec3::new(thick, hi.y, 3.0),
            Stuff::Wood,
            TRIMWORK,
        );
    }
    slab(
        out,
        Vec3::new(x, hi.y - 1.5, mid_z),
        Vec3::new(thick, 3.0, wide),
        Stuff::Wood,
        TRIMWORK,
    );
    // Architrave, both faces.
    for face in [-1.0f32, 1.0] {
        let fx = x + face * (thick * 0.5 + 1.5);
        for side in [-1.0f32, 1.0] {
            slab(
                out,
                Vec3::new(fx, hi.y * 0.5, mid_z + side * (wide * 0.5 + 5.0)),
                Vec3::new(3.0, hi.y + 10.0, 10.0),
                Stuff::Wood,
                TRIMWORK,
            );
        }
        slab(
            out,
            Vec3::new(fx, hi.y + 5.0, mid_z),
            Vec3::new(3.0, 10.0, wide + 20.0),
            Stuff::Wood,
            TRIMWORK,
        );
    }
}

/// Cases every interior opening, and hangs a leaf in the ones that have one.
///
/// The leaves never shut. Partly because the traversal law would fail the
/// moment they did — a fly cannot work a handle — and partly because a house
/// where every internal door is closed is a house nobody is living in. Each one
/// stands somewhere between wide open against its wall and just ajar, picked
/// off its own position so the same door is at the same angle every run.
fn interior_doors(out: &mut Vec<Solid>) {
    for (lo, hi) in crate::house::cased_openings() {
        case_opening(out, lo, hi);
    }
    for (lo, hi) in crate::house::interior_doors() {
        case_opening(out, lo, hi);
        let x = (lo.x + hi.x) * 0.5;
        let wide = hi.z - lo.z;
        // Hinged at the +z jamb, swinging back toward -x. How far it stands
        // open is picked off its own position, so the same door is at the same
        // angle every run — and then walked back until the leaf is not standing
        // in a bed or a wardrobe. A door has to open into a room that is
        // already furnished, so it is the door that gives way.
        let hinge = Vec3::new(x, 2.0, hi.z - 3.0);
        let len = wide - 6.0;
        let want = 0.45 + (wobble(lo.z, hi.y) * 0.5 + 0.5) * 1.05;
        let mut open = 0.30;
        for step in 0..7 {
            let a = want - step as f32 * 0.18;
            if a < 0.30 {
                break;
            }
            let swing = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 + a);
            // Only the outer part of the sweep: the hinge end always overlaps
            // the lining it is hung in.
            let near = hinge + swing * Vec3::new(len * 0.45, 0.0, 0.0);
            let far = hinge + swing * Vec3::new(len, 0.0, 0.0);
            let middle = (near + far) * 0.5 + Vec3::new(0.0, hi.y * 0.5, 0.0);
            let span = (far - near).abs() + Vec3::new(10.0, hi.y, 10.0);
            if !clashes(out, middle, span) {
                open = a;
                break;
            }
        }
        door_leaf(
            out,
            hinge,
            len,
            hi.y - 4.0,
            Quat::from_rotation_y(std::f32::consts::FRAC_PI_2 + open),
        );
    }
}

/// Switch plates and sockets.
///
/// The smallest thing in the house that says a wall was built rather than
/// extruded — and at fly scale they are landmarks: a switch is four body
/// lengths across, standing a centimetre off an otherwise featureless plain.
///
/// A switch goes beside every internal doorway, on the side away from the
/// hinge, which is where a hand reaches. Sockets go round the skirting, and any
/// that would end up inside a wardrobe or behind a bath are dropped — the room
/// is furnished by the time this runs, so it can simply ask.
fn switches_and_sockets(out: &mut Vec<Solid>) {
    let plate = Color::srgb(0.94, 0.93, 0.90);

    // Is there room to stand in front of this plate? Probed *off* the wall
    // rather than at it — the wall is a solid too, and testing at the plate
    // rejects every plate in the house for touching the thing it is screwed to.
    let has_room = |out: &Vec<Solid>, at: Vec3, normal: Vec3| {
        !clashes(out, at + normal * 16.0, Vec3::splat(26.0))
    };

    for (lo, hi) in crate::house::interior_doors() {
        let x = (lo.x + hi.x) * 0.5;
        let thick = hi.x - lo.x;
        for face in [-1.0f32, 1.0] {
            let normal = Vec3::new(face, 0.0, 0.0);
            let at = Vec3::new(x + face * (thick * 0.5 + 1.5), 122.0, lo.z - 26.0);
            if !has_room(out, at, normal) {
                continue;
            }
            slab(out, at, Vec3::new(3.0, 13.0, 9.0), Stuff::Wood, plate);
            slab(
                out,
                at + normal * 1.6 + Vec3::Y,
                Vec3::new(2.0, 6.0, 5.0),
                Stuff::Wood,
                Color::srgb(0.85, 0.84, 0.81),
            );
        }
    }

    for r in house::rooms() {
        for (at, normal) in [
            (
                Vec3::new(r.min.x + r.wide() * 0.33, 28.0, r.min.y + 2.0),
                Vec3::Z,
            ),
            (
                Vec3::new(r.min.x + r.wide() * 0.68, 28.0, r.max.y - 2.0),
                -Vec3::Z,
            ),
            (
                Vec3::new(r.min.x + 2.0, 28.0, r.min.y + r.deep() * 0.62),
                Vec3::X,
            ),
            (
                Vec3::new(r.max.x - 2.0, 28.0, r.min.y + r.deep() * 0.28),
                -Vec3::X,
            ),
        ] {
            if !has_room(out, at, normal) {
                continue;
            }
            let size = if normal.z.abs() > 0.5 {
                Vec3::new(11.0, 8.0, 3.0)
            } else {
                Vec3::new(3.0, 8.0, 11.0)
            };
            slab(out, at, size, Stuff::Wood, plate);
        }
    }
}

/// The front door: cased both sides, shut, with a stoop to stand on.
///
/// It hung ajar for a while, because it was the only opening in the house that
/// was not glazed shut and leaving it open kept a way out. The fly does not go
/// outside — that is the game — so it is shut, and the house is sealed.
fn front_door(out: &mut Vec<Solid>) {
    let (lo, hi) = crate::house::front_door();
    let wide = hi.x - lo.x;
    let high = hi.y;
    let z = (lo.z + hi.z) * 0.5;
    let thick = hi.z - lo.z;

    // Threshold, and the casing on both faces.
    slab(
        out,
        Vec3::new((lo.x + hi.x) * 0.5, 1.5, z),
        Vec3::new(wide, 3.0, thick),
        Stuff::Wood,
        THRESHOLD,
    );
    for face in [-1.0f32, 1.0] {
        let fz = z + face * (thick * 0.5 + 1.5);
        for side in [-1.0f32, 1.0] {
            slab(
                out,
                Vec3::new(
                    (lo.x + hi.x) * 0.5 + side * (wide * 0.5 + 5.0),
                    high * 0.5,
                    fz,
                ),
                Vec3::new(10.0, high + 10.0, 3.0),
                Stuff::Wood,
                TRIMWORK,
            );
        }
        slab(
            out,
            Vec3::new((lo.x + hi.x) * 0.5, high + 5.0, fz),
            Vec3::new(wide + 20.0, 10.0, 3.0),
            Stuff::Wood,
            TRIMWORK,
        );
    }

    // Hung on the left jamb, opening inward — north, into the great room. The
    // envelope law caught the first version of this swinging out into the
    // garden, which is not how a front door on this continent is hung.
    let swing = Quat::from_rotation_y(0.0);
    door_leaf(
        out,
        Vec3::new(lo.x + 2.0, 3.0, z),
        wide - 4.0,
        high - 4.0,
        swing,
    );

    // A stoop outside it, and a step down to the grass. Both are outdoors, and
    // say so: the envelope law used to infer that from being no taller than a
    // step, which stopped being a safe guess once there were neighbours.
    let mid = (lo.x + hi.x) * 0.5;
    for (y, size) in [
        (-4.0f32, Vec3::new(wide + 90.0, 14.0, 116.0)),
        (-9.0, Vec3::new(wide + 130.0, 10.0, 46.0)),
    ] {
        let at = Vec3::new(
            mid,
            y,
            z + thick * 0.5 + if size.z > 100.0 { 58.0 } else { 138.0 },
        );
        let mut s = Solid::between(at - size * 0.5, at + size * 0.5, Stuff::Stone);
        s.paint = Some(PAVING);
        s.outdoors = true;
        out.push(s);
    }
}

// ---------------------------------------------------------------------------
// Sanitaryware
// ---------------------------------------------------------------------------

/// A lavatory, facing `out_of` — the way somebody sitting on it would look.
///
/// It was two white boxes, which is what a bathroom looks like when nobody has
/// built one: a pedestal, a bowl, a seat, a raised lid and a cistern are five
/// silhouettes people read instantly, and the difference between them and a
/// stack of cubes is the difference between a room and a placeholder.
fn toilet(out: &mut Vec<Solid>, at: Vec2, out_of: Vec2) {
    // Local frame: +z is the way it faces, +x across.
    let turn = Quat::from_rotation_y(out_of.x.atan2(out_of.y));
    let put = |o: &mut Vec<Solid>, local: Vec3, size: Vec3, paint: Color, stuff: Stuff| {
        turned(
            o,
            Vec3::new(at.x, 0.0, at.y) + turn * local,
            size,
            turn,
            stuff,
            paint,
        );
    };
    // Foot, waisted pedestal, bowl, seat.
    put(
        out,
        Vec3::new(0.0, 6.0, 2.0),
        Vec3::new(26.0, 12.0, 32.0),
        PORCELAIN,
        Stuff::Stone,
    );
    put(
        out,
        Vec3::new(0.0, 26.0, 0.0),
        Vec3::new(20.0, 28.0, 24.0),
        PORCELAIN,
        Stuff::Stone,
    );
    put(
        out,
        Vec3::new(0.0, 46.0, 4.0),
        Vec3::new(36.0, 14.0, 46.0),
        PORCELAIN,
        Stuff::Stone,
    );
    put(
        out,
        Vec3::new(0.0, 55.0, 5.0),
        Vec3::new(38.0, 4.0, 46.0),
        SEAT_RING,
        Stuff::Fabric,
    );
    // Cistern behind it, and the lid standing up against the wall.
    put(
        out,
        Vec3::new(0.0, 76.0, -22.0),
        Vec3::new(44.0, 48.0, 20.0),
        PORCELAIN,
        Stuff::Stone,
    );
    put(
        out,
        Vec3::new(0.0, 101.0, -22.0),
        Vec3::new(48.0, 4.0, 24.0),
        PORCELAIN,
        Stuff::Stone,
    );
    put(
        out,
        Vec3::new(0.0, 14.0, -3.0),
        Vec3::new(12.0, 6.0, 12.0),
        CHROME,
        Stuff::Metal,
    );
    turned(
        out,
        Vec3::new(at.x, 0.0, at.y) + turn * Vec3::new(0.0, 79.0, -9.0),
        Vec3::new(38.0, 46.0, 4.0),
        turn * Quat::from_rotation_x(-0.16),
        Stuff::Fabric,
        SEAT_RING,
    );
}

/// A mixer tap: a body and a spout reaching out over whatever it fills.
fn tap(out: &mut Vec<Solid>, at: Vec3, reach: Vec2) {
    slab(
        out,
        at + Vec3::new(0.0, 7.0, 0.0),
        Vec3::new(6.0, 14.0, 6.0),
        Stuff::Metal,
        CHROME,
    );
    slab(
        out,
        at + Vec3::new(reach.x * 0.5, 15.0, reach.y * 0.5),
        Vec3::new(4.0 + reach.x.abs(), 4.0, 4.0 + reach.y.abs()),
        Stuff::Metal,
        CHROME,
    );
    for side in [-1.0f32, 1.0] {
        slab(
            out,
            at + Vec3::new(
                side * 11.0 * reach.y.signum().abs().max(0.0),
                12.0,
                side * 11.0,
            ),
            Vec3::new(9.0, 4.0, 4.0),
            Stuff::Metal,
            CHROME,
        );
    }
}

/// A basin sunk into a counter: a rim, four thin sides and a floor, so it holds
/// a shape rather than being a lump on a worktop.
fn basin(out: &mut Vec<Solid>, at: Vec3, wide: f32, deep: f32) {
    let (w, d, high, side) = (wide, deep, 15.0, 4.0);
    slab(
        out,
        Vec3::new(at.x, at.y + 2.0, at.z),
        Vec3::new(w, 4.0, d),
        Stuff::Stone,
        PORCELAIN,
    );
    for s in [-1.0f32, 1.0] {
        slab(
            out,
            Vec3::new(at.x + s * (w * 0.5 - side * 0.5), at.y + high * 0.5, at.z),
            Vec3::new(side, high, d),
            Stuff::Stone,
            PORCELAIN,
        );
        slab(
            out,
            Vec3::new(at.x, at.y + high * 0.5, at.z + s * (d * 0.5 - side * 0.5)),
            Vec3::new(w, high, side),
            Stuff::Stone,
            PORCELAIN,
        );
    }
}

/// A tiled panel: a dark backing with courses of tile standing proud of it, so
/// the joints read as joints rather than as painted lines.
///
/// Courses only, no vertical joints. Six boxes a panel instead of fifty, and at
/// the distance a wall is looked at from, the horizontal bands are what the eye
/// picks up — the floor already has its cross-cut for the close read.
fn tiling(out: &mut Vec<Solid>, at: Vec3, size: Vec3, along_x: bool) {
    slab(out, at, size, Stuff::Stone, Color::srgb(0.55, 0.56, 0.55));
    let course = 23.0;
    let rows = (size.y / course).floor().max(1.0) as usize;
    let (wide, deep) = if along_x {
        (size.x - 2.0, size.z + 2.0)
    } else {
        (size.x + 2.0, size.z - 2.0)
    };
    for k in 0..rows {
        let y = at.y - size.y * 0.5 + course * (k as f32 + 0.5);
        let shade = 0.83 + wobble(at.x + k as f32 * 31.0, y) * 0.025;
        slab(
            out,
            Vec3::new(at.x, y, at.z),
            Vec3::new(wide, course - 2.5, deep),
            Stuff::Stone,
            Color::srgb(shade, shade + 0.015, shade + 0.01),
        );
    }
}

/// A towel rail with two towels over it. The towels are the only soft thing in
/// a tiled room and they are what stops it reading as a showroom.
fn towel_rail(out: &mut Vec<Solid>, at: Vec3, wide: f32, along_x: bool, face: f32) {
    let bar = if along_x {
        Vec3::new(wide, 3.0, 3.0)
    } else {
        Vec3::new(3.0, 3.0, wide)
    };
    slab(out, at, bar, Stuff::Metal, CHROME);
    for (i, side) in [-1.0f32, 1.0].into_iter().enumerate() {
        let along = side * wide * 0.28;
        let hang = 52.0 - i as f32 * 9.0;
        let centre = if along_x {
            Vec3::new(at.x + along, at.y - hang * 0.5, at.z + face * 3.0)
        } else {
            Vec3::new(at.x + face * 3.0, at.y - hang * 0.5, at.z + along)
        };
        let size = if along_x {
            Vec3::new(wide * 0.36, hang, 5.0)
        } else {
            Vec3::new(5.0, hang, wide * 0.36)
        };
        soft(
            out,
            centre,
            size,
            2.0,
            Stuff::Fabric,
            if i == 0 { TOWEL_A } else { TOWEL_B },
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
    front_door(out);
    interior_doors(out);
    switches_and_sockets(out);
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
    ceiling_fan(out, m, crate::house::CEILING);
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
    // Cushions along the back of it, and a blanket over one arm.
    for k in 0..3 {
        let z = m.y + (k as f32 - 1.0) * 62.0;
        turned(
            out,
            Vec3::new(r.min.x + 62.0, 62.0, z),
            Vec3::new(14.0, 40.0, 44.0),
            Quat::from_rotation_z(0.22 + wobble(z, 3.0) * 0.08),
            Stuff::Fabric,
            if k == 1 { WOOL_WARM } else { THROW },
        );
    }
    // A throw over the near arm. Flat on top of it, it read as a plank; what
    // makes a blanket a blanket is the part hanging down the outside.
    soft(
        out,
        Vec3::new(r.min.x + 66.0, 70.0, m.y + 102.0),
        Vec3::new(62.0, 9.0, 48.0),
        3.0,
        Stuff::Fabric,
        DUVET,
    );
    soft(
        out,
        Vec3::new(r.min.x + 66.0, 48.0, m.y + 124.0),
        Vec3::new(58.0, 46.0, 8.0),
        3.0,
        Stuff::Fabric,
        DUVET,
    );

    // Coffee table on the rug, with the things that live on one.
    let table = Vec2::new(m.x - 40.0, m.y);
    legged(
        out,
        table,
        Vec2::new(110.0, 62.0),
        42.0,
        5.0,
        7.0,
        OAK,
        DARK_OAK,
    );
    books(out, Vec3::new(table.x - 28.0, 42.0, table.y + 8.0), 3, 1.0);
    mug(
        out,
        Vec3::new(table.x + 22.0, 42.0, table.y - 12.0),
        Color::srgb(0.80, 0.82, 0.84),
    );
    slab(
        out,
        Vec3::new(table.x + 34.0, 44.0, table.y + 16.0),
        Vec3::new(6.0, 3.0, 20.0),
        Stuff::Metal,
        Color::srgb(0.18, 0.18, 0.19),
    );

    // The front door opens straight into this room, so this room has to be the
    // hall as well: a mat to wipe on, hooks for coats, and shoes kicked off
    // beside them. The hooks go in the stretch of wall between the door and the
    // next window, which is seventy-eight centimetres and the only place they
    // fit — the window law would have said so otherwise.
    let (door_lo, door_hi) = crate::house::front_door();
    let door_x = (door_lo.x + door_hi.x) * 0.5;
    let inner = door_lo.z - 4.0;
    rug(
        out,
        Vec2::new(door_x, inner - 46.0),
        Vec2::new(104.0, 66.0),
        Color::srgb(0.34, 0.30, 0.26),
    );
    let hooks = door_x + 78.0;
    slab(
        out,
        Vec3::new(hooks, 168.0, inner - 3.0),
        Vec3::new(62.0, 9.0, 4.0),
        Stuff::Wood,
        DARK_OAK,
    );
    for k in 0..3 {
        let x = hooks - 20.0 + k as f32 * 20.0;
        slab(
            out,
            Vec3::new(x, 162.0, inner - 8.0),
            Vec3::new(3.0, 4.0, 10.0),
            Stuff::Metal,
            BRASS,
        );
        if k == 1 {
            continue;
        }
        // A coat on two of the three.
        turned(
            out,
            Vec3::new(x, 122.0, inner - 12.0),
            Vec3::new(34.0, 84.0, 12.0),
            Quat::from_rotation_z(wobble(x, 2.0) * 0.05),
            Stuff::Fabric,
            if k == 0 {
                THROW
            } else {
                Color::srgb(0.26, 0.32, 0.38)
            },
        );
    }
    for (i, off) in [-26.0f32, 4.0].into_iter().enumerate() {
        slab(
            out,
            Vec3::new(hooks + off, 5.0, inner - 34.0 - i as f32 * 6.0),
            Vec3::new(11.0, 10.0, 27.0),
            Stuff::Fabric,
            Color::srgb(0.20, 0.18, 0.17),
        );
    }

    // Clutter. A room with one picture centred on each wall and nothing on any
    // surface is a show home; what makes it somebody's is the number of small
    // things that are where they were left.
    //
    // A gallery over the media unit, in whatever stretch of that wall has no
    // window; a lamp in the corner; a basket of magazines by the sofa; things
    // on the media unit and on the side table.
    // Above the media unit, on the long blank wall the room is usually looked
    // at across — a gallery on the wall behind the camera is a gallery nobody
    // sees, which is where the first version put it.
    frames(
        out,
        Vec3::new(r.max.x - 7.0, 182.0, m.y - 20.0),
        false,
        340.0,
        3.0,
    );
    let (gal_lo, gal_hi) = clear_of_windows_on(r, Wall::North);
    frames(
        out,
        Vec3::new((gal_lo + gal_hi) * 0.5, 176.0, r.min.y + 7.0),
        true,
        (gal_hi - gal_lo).min(320.0),
        19.0,
    );
    floor_lamp(out, Vec2::new(r.min.x + 62.0, m.y - 168.0));
    basket(
        out,
        Vec2::new(r.min.x + 148.0, m.y - 136.0),
        34.0,
        30.0,
        Color::srgb(0.58, 0.48, 0.34),
    );
    books(out, Vec3::new(r.min.x + 148.0, 26.0, m.y - 136.0), 3, 11.0);
    // On the media unit: a soundbar, a photograph, and something living.
    let media = Vec2::new(r.max.x - 34.0, m.y);
    slab(
        out,
        Vec3::new(media.x - 6.0, 60.0, media.y),
        Vec3::new(14.0, 8.0, 96.0),
        Stuff::Metal,
        Color::srgb(0.20, 0.20, 0.22),
    );
    picture(
        out,
        Vec3::new(media.x - 10.0, 74.0, media.y - 66.0),
        26.0,
        20.0,
        false,
        Color::srgb(0.46, 0.40, 0.34),
    );
    pot_plant(out, Vec2::new(media.x - 12.0, media.y + 72.0), 52.0);
    // On the coffee table: a dish and a candle beside the books.
    disc(
        out,
        Vec3::new(table.x - 4.0, 45.0, table.y - 18.0),
        22.0,
        5.0,
        Color::srgb(0.66, 0.62, 0.56),
        0.0,
    );
    disc(
        out,
        Vec3::new(table.x + 44.0, 50.0, table.y - 4.0),
        11.0,
        14.0,
        Color::srgb(0.86, 0.84, 0.78),
        0.0,
    );
    // And a couple of things on the floor that nobody has put away.
    disc(
        out,
        Vec3::new(m.x - 130.0, 9.0, m.y + 122.0),
        18.0,
        18.0,
        Color::srgb(0.60, 0.26, 0.22),
        0.0,
    );
    books(out, Vec3::new(m.x + 96.0, 0.6, m.y - 150.0), 2, 27.0);

    // A plant in the corner, and a stack of books beside the sofa.
    pot_plant(out, Vec2::new(r.min.x + 82.0, r.max.y - 96.0), 138.0);
    books(out, Vec3::new(r.min.x + 62.0, 0.0, m.y - 148.0), 5, 4.0);
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

/// Which wall of a room, for the purpose of asking what is in it.
#[derive(Clone, Copy, PartialEq)]
enum Wall {
    North,
    South,
    West,
}

/// The widest windowless run along one wall of a room.
///
/// Anything tall goes here. A cooker hood, a run of wall cabinets, a picture
/// over a bed and — the last time this was north-only — a wardrobe and two more
/// pictures all ended up over glass by being placed at a hand-picked offset.
/// The fix that does not need repeating is to ask where the windows are rather
/// than to remember, on whichever wall is being used.
fn clear_of_windows_on(r: &Room, wall: Wall) -> (f32, f32) {
    let along_x = matches!(wall, Wall::North | Wall::South);
    let line = match wall {
        Wall::North => r.min.y,
        Wall::South => r.max.y,
        Wall::West => r.min.x,
    };
    let (from, to) = if along_x {
        (r.min.x, r.max.x)
    } else {
        (r.min.y, r.max.y)
    };

    let mut spans: Vec<(f32, f32)> = crate::house::window_openings()
        .into_iter()
        .filter(|(lo, hi)| {
            let mid = (*lo + *hi) * 0.5;
            let (on, across) = if along_x {
                (mid.x, mid.z)
            } else {
                (mid.z, mid.x)
            };
            on > from && on < to && (across - line).abs() < 60.0
        })
        .map(|(lo, hi)| if along_x { (lo.x, hi.x) } else { (lo.z, hi.z) })
        .collect();
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut best = (from + 30.0, from + 130.0);
    let mut widest = 0.0;
    let mut walk = from + 30.0;
    for (lo, hi) in spans.iter().chain(std::iter::once(&(to - 30.0, 0.0))) {
        let gap = lo - walk;
        if gap > widest {
            widest = gap;
            best = (walk, *lo);
        }
        walk = walk.max(*hi);
    }
    best
}

fn clear_of_windows(r: &Room) -> (f32, f32) {
    clear_of_windows_on(r, Wall::North)
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
    let fridge = Vec2::new(r.max.x - 100.0, r.min.y + 56.0);
    appliance(out, fridge, Vec3::new(78.0, 178.0, 68.0), STEEL);
    // Fridge over freezer, a shadow gap between them, and a handle down the
    // same side of each. Without them it is a wardrobe in a kitchen.
    for (y, high) in [(64.0f32, 116.0f32), (2.0, 56.0)] {
        slab(
            out,
            Vec3::new(fridge.x, y + high * 0.5, fridge.y + 35.0),
            Vec3::new(72.0, high - 4.0, 3.0),
            Stuff::Metal,
            Color::srgb(0.80, 0.81, 0.83),
        );
        slab(
            out,
            Vec3::new(fridge.x - 28.0, y + high * 0.5, fridge.y + 39.0),
            Vec3::new(4.0, high * 0.62, 5.0),
            Stuff::Metal,
            CHROME,
        );
    }
    // The cooker goes in the widest stretch of wall with no window in it, so
    // its extractor has somewhere to be that is not over glass.
    let cooker = Vec2::new(cooker_x, north);
    appliance(out, cooker, Vec3::new(76.0, 90.0, 62.0), SLATE);
    // A hob with four rings, a control fascia, and an oven door with a window
    // and a bar handle.
    slab(
        out,
        Vec3::new(cooker_x, 91.0, north),
        Vec3::new(74.0, 3.0, 60.0),
        Stuff::Metal,
        Color::srgb(0.14, 0.14, 0.15),
    );
    for k in 0..4 {
        let (dx, dz) = ((k % 2) as f32 - 0.5, (k / 2) as f32 - 0.5);
        disc(
            out,
            Vec3::new(cooker_x + dx * 36.0, 93.0, north + dz * 28.0),
            22.0,
            2.0,
            Color::srgb(0.26, 0.26, 0.28),
            0.0,
        );
    }
    let front = north + 31.0;
    slab(
        out,
        Vec3::new(cooker_x, 82.0, front),
        Vec3::new(74.0, 14.0, 3.0),
        Stuff::Metal,
        Color::srgb(0.20, 0.20, 0.22),
    );
    for k in 0..4 {
        disc(
            out,
            Vec3::new(cooker_x - 27.0 + k as f32 * 18.0, 82.0, front + 2.0),
            7.0,
            4.0,
            CHROME,
            0.0,
        );
    }
    slab(
        out,
        Vec3::new(cooker_x, 40.0, front + 1.0),
        Vec3::new(70.0, 62.0, 3.0),
        Stuff::Metal,
        Color::srgb(0.22, 0.22, 0.24),
    );
    slab(
        out,
        Vec3::new(cooker_x, 44.0, front + 3.0),
        Vec3::new(52.0, 34.0, 2.0),
        Stuff::Glass,
        Color::srgba(0.10, 0.11, 0.12, 0.82),
    );
    slab(
        out,
        Vec3::new(cooker_x, 68.0, front + 5.0),
        Vec3::new(64.0, 4.0, 5.0),
        Stuff::Metal,
        CHROME,
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

    // What lives on a worktop. A kitchen with nothing on its counters is a
    // showroom; a bowl, a board and a kettle are what say somebody cooks here.
    disc(
        out,
        Vec3::new(island.x - 46.0, 96.0, island.y),
        30.0,
        10.0,
        Color::srgb(0.72, 0.70, 0.64),
        0.0,
    );
    for k in 0..4 {
        let n = wobble(island.x + k as f32 * 11.0, island.y);
        slab(
            out,
            Vec3::new(
                island.x - 46.0 + n * 8.0,
                100.0 + (k % 2) as f32 * 5.0,
                island.y + n * 7.0,
            ),
            Vec3::splat(9.0),
            Stuff::Fabric,
            [
                Color::srgb(0.72, 0.24, 0.16),
                Color::srgb(0.80, 0.62, 0.16),
                Color::srgb(0.42, 0.56, 0.22),
                Color::srgb(0.66, 0.30, 0.14),
            ][k],
        );
    }
    turned(
        out,
        Vec3::new(island.x + 44.0, 94.0, island.y - 6.0),
        Vec3::new(46.0, 3.0, 30.0),
        Quat::from_rotation_y(0.22),
        Stuff::Wood,
        OAK,
    );
    // A kettle on the run under the window, and two jars beside it.
    disc(
        out,
        Vec3::new(r.min.x + 150.0, 104.0, north),
        20.0,
        26.0,
        Color::srgb(0.74, 0.76, 0.78),
        0.0,
    );
    slab(
        out,
        Vec3::new(r.min.x + 150.0, 122.0, north + 12.0),
        Vec3::new(4.0, 14.0, 12.0),
        Stuff::Metal,
        Color::srgb(0.24, 0.24, 0.26),
    );
    for (i, tall) in [22.0f32, 16.0].into_iter().enumerate() {
        disc(
            out,
            Vec3::new(r.min.x + 186.0 + i as f32 * 26.0, 92.0 + tall * 0.5, north),
            15.0 - i as f32 * 2.0,
            tall,
            Color::srgb(0.80, 0.78, 0.70),
            0.0,
        );
    }

    // A knife block, a tea towel on the cooker, and a notice board by the door
    // — the things a kitchen accumulates that are not appliances.
    turned(
        out,
        Vec3::new(cooker_x - 62.0, 100.0, north + 4.0),
        Vec3::new(14.0, 22.0, 12.0),
        Quat::from_rotation_x(-0.12),
        Stuff::Wood,
        DARK_OAK,
    );
    for k in 0..4 {
        slab(
            out,
            Vec3::new(cooker_x - 66.0 + k as f32 * 3.0, 116.0, north + 2.0),
            Vec3::new(2.0, 14.0, 2.0),
            Stuff::Metal,
            CHROME,
        );
    }
    soft(
        out,
        Vec3::new(cooker_x + 20.0, 52.0, north + 34.0),
        Vec3::new(26.0, 40.0, 4.0),
        2.0,
        Stuff::Fabric,
        TOWEL_A,
    );
    picture(
        out,
        Vec3::new(r.min.x + 8.0, 150.0, m.y - 40.0),
        60.0,
        44.0,
        false,
        Color::srgb(0.52, 0.48, 0.36),
    );

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
        chair(out, c, (c - table).normalize_or_zero());
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
    drawers(
        out,
        Vec2::new(r.max.x - 40.0, m.y + 60.0),
        Vec3::new(52.0, 82.0, 130.0),
        Vec2::new(-1.0, 0.0),
        4,
    );
    // And a wardrobe in the corner, which is the tallest thing in the room and
    // the only place with a top surface nobody ever dusts.
    // Against the west wall, in whatever stretch of it has no window — the
    // first version stood it half over the glass in two bedrooms out of three.
    let (west_lo, west_hi) = clear_of_windows_on(r, Wall::West);
    wardrobe(
        out,
        Vec2::new(r.min.x + 38.0, (west_lo + west_hi) * 0.5),
        Vec3::new(64.0, 196.0, (west_hi - west_lo - 24.0).clamp(72.0, 150.0)),
        Vec2::new(1.0, 0.0),
    );

    // A lamp on the nightstand nearest the door side, and the things that
    // collect on the other one.
    lamp(
        out,
        Vec3::new(head_x + size.x * 0.5 + 32.0, 54.0, r.min.y + 60.0),
    );
    let far = head_x - size.x * 0.5 - 32.0;
    books(out, Vec3::new(far - 6.0, 54.0, r.min.y + 54.0), 2, head_x);
    disc(
        out,
        Vec3::new(far + 12.0, 59.0, r.min.y + 66.0),
        13.0,
        10.0,
        Color::srgb(0.30, 0.32, 0.34),
        0.0,
    );
    pot_plant(out, Vec2::new(r.max.x - 40.0, m.y + 116.0), 46.0);

    // The side walls were bare in every bedroom capture. A desk under the far
    // window in the children's rooms, a chair pushed under it, and a pair of
    // small pictures on the wall the bed does not use.
    if !double {
        let (dl, dh) = clear_of_windows_on(r, Wall::South);
        let desk = Vec2::new((dl + dh) * 0.5, r.max.y - 44.0);
        legged(
            out,
            desk,
            Vec2::new(132.0, 62.0),
            74.0,
            5.0,
            6.0,
            OAK,
            DARK_OAK,
        );
        chair(out, desk + Vec2::new(0.0, -56.0), Vec2::new(0.0, -1.0));
        // Two small pictures on the south wall, in the run of it with no
        // window — which is also the wall the bed and the wardrobe leave alone.
        let art_at = (dl + dh) * 0.5;
        for (i, off) in [-38.0f32, 38.0].into_iter().enumerate() {
            picture(
                out,
                Vec3::new(art_at + off, 168.0 + i as f32 * 10.0, r.max.y - 8.0),
                50.0,
                38.0,
                true,
                if i == 0 {
                    Color::srgb(0.52, 0.44, 0.34)
                } else {
                    Color::srgb(0.34, 0.42, 0.46)
                },
            );
        }
        // Toys, because a child's room with nothing on the floor is not one.
        for k in 0..5 {
            let n = wobble(desk.x + k as f32 * 23.0, desk.y);
            let at = Vec2::new(
                m.x - 90.0 + n * 110.0 + k as f32 * 26.0,
                m.y + 46.0 + n * 80.0,
            );
            let size = 9.0 + n.abs() * 7.0;
            turned(
                out,
                Vec3::new(at.x, size * 0.5, at.y),
                Vec3::splat(size),
                Quat::from_rotation_y(n * 1.1),
                Stuff::Wood,
                [
                    Color::srgb(0.72, 0.26, 0.20),
                    Color::srgb(0.26, 0.44, 0.62),
                    Color::srgb(0.82, 0.66, 0.22),
                    Color::srgb(0.34, 0.56, 0.34),
                    Color::srgb(0.62, 0.34, 0.56),
                ][k],
            );
        }
        books(out, Vec3::new(desk.x - 40.0, 74.0, desk.y - 4.0), 3, desk.y);
    } else {
        // The main bedroom gets a chair in the corner instead, which is where
        // clothes actually live — with clothes on it.
        let seat = Vec2::new(r.max.x - 70.0, r.max.y - 70.0);
        chair(out, seat, Vec2::new(-1.0, 0.0));
        soft(
            out,
            Vec3::new(seat.x, 52.0, seat.y),
            Vec3::new(44.0, 16.0, 44.0),
            5.0,
            Stuff::Fabric,
            Color::srgb(0.34, 0.38, 0.44),
        );
        soft(
            out,
            Vec3::new(seat.x - 14.0, 40.0, seat.y + 18.0),
            Vec3::new(20.0, 34.0, 14.0),
            4.0,
            Stuff::Fabric,
            Color::srgb(0.52, 0.44, 0.40),
        );
    }
}

fn bathroom(out: &mut Vec<Solid>, r: &Room) {
    // Everything stands against a wall, which is what bathrooms do and also
    // what keeps the middle clear — the main bath was a tiled hall with one
    // small tub adrift in it.
    //
    // North wall: the vanity run, with the basin sunk into it and the mirror
    // over. South wall: the bath, in the corner, under a shower. East wall: the
    // towels. The lavatory takes the north-east corner, facing into the room.
    counter_run(
        out,
        Vec2::new(r.min.x + 24.0, r.min.y + 34.0),
        Vec2::new(r.min.x + 200.0, r.min.y + 34.0),
        58.0,
        Vec2::new(0.0, 1.0),
        &[],
    );
    basin(
        out,
        Vec3::new(r.min.x + 106.0, 90.0, r.min.y + 32.0),
        56.0,
        42.0,
    );
    tap(
        out,
        Vec3::new(r.min.x + 106.0, 92.0, r.min.y + 8.0),
        Vec2::new(0.0, 16.0),
    );
    // Mirror, and a shelf under it for the things that live on one.
    slab(
        out,
        Vec3::new(r.min.x + 106.0, 158.0, r.min.y + 7.0),
        Vec3::new(96.0, 84.0, 3.0),
        Stuff::Glass,
        Color::srgb(0.80, 0.85, 0.88),
    );
    slab(
        out,
        Vec3::new(r.min.x + 106.0, 113.0, r.min.y + 11.0),
        Vec3::new(100.0, 4.0, 12.0),
        Stuff::Stone,
        PORCELAIN,
    );
    for (i, tall) in [13.0f32, 9.0, 16.0, 11.0].into_iter().enumerate() {
        let x = r.min.x + 74.0 + i as f32 * 21.0;
        slab(
            out,
            Vec3::new(x, 115.0 + tall * 0.5, r.min.y + 11.0),
            Vec3::new(6.0, tall, 6.0),
            Stuff::Glass,
            if i % 2 == 0 { TOWEL_A } else { TOWEL_B },
        );
    }

    // Cistern against the wall, not thirty centimetres off it.
    toilet(
        out,
        Vec2::new(r.max.x - 64.0, r.min.y + 44.0),
        Vec2::new(0.0, 1.0),
    );

    // The bath, in the south-west corner: four sides and a floor, so it holds a
    // shape and a fly can get down inside it.
    let (bw, bd, bh, wall) = (176.0, 82.0, 58.0, 6.0);
    let b = Vec2::new(r.min.x + 8.0 + bw * 0.5, r.max.y - 6.0 - bd * 0.5);
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
    // Bath tap at the far end, a riser and a head above it, and a glass screen
    // at the open end so the shower has somewhere for the water to stop.
    tap(
        out,
        Vec3::new(b.x + bw * 0.5 - 16.0, bh, b.y),
        Vec2::new(-22.0, 0.0),
    );
    slab(
        out,
        Vec3::new(b.x + bw * 0.5 - 5.0, 130.0, b.y),
        Vec3::new(5.0, 150.0, 5.0),
        Stuff::Metal,
        CHROME,
    );
    slab(
        out,
        Vec3::new(b.x + bw * 0.5 - 22.0, 202.0, b.y),
        Vec3::new(34.0, 5.0, 22.0),
        Stuff::Metal,
        CHROME,
    );
    slab(
        out,
        Vec3::new(b.x + bw * 0.5 - 3.0, 128.0, b.y),
        Vec3::new(6.0, 132.0, bd - 10.0),
        Stuff::Glass,
        Color::srgba(0.84, 0.90, 0.92, 0.30),
    );
    // A mat to step out onto.
    rug(
        out,
        Vec2::new(b.x, b.y - bd * 0.5 - 34.0),
        Vec2::new(bw * 0.62, 52.0),
        TOWEL_A,
    );

    // Tiling where a bathroom is actually tiled: behind the basins, and around
    // the bath, which is the wall the shower is aimed at.
    tiling(
        out,
        Vec3::new(r.min.x + 112.0, 122.0, r.min.y + 4.0),
        Vec3::new(198.0, 68.0, 4.0),
        true,
    );
    tiling(
        out,
        Vec3::new(b.x, 128.0, r.max.y - 4.0),
        Vec3::new(bw + 16.0, 140.0, 4.0),
        true,
    );
    tiling(
        out,
        Vec3::new(r.min.x + 4.0, 128.0, b.y),
        Vec3::new(4.0, 140.0, bd + 12.0),
        false,
    );

    // The small things. A bathroom without a roll on a holder and a bin beside
    // the lavatory is a showroom.
    slab(
        out,
        Vec3::new(r.max.x - 18.0, 74.0, r.min.y + 96.0),
        Vec3::new(4.0, 4.0, 16.0),
        Stuff::Metal,
        CHROME,
    );
    disc(
        out,
        Vec3::new(r.max.x - 22.0, 74.0, r.min.y + 96.0),
        11.0,
        11.0,
        Color::srgb(0.94, 0.93, 0.90),
        0.0,
    );
    disc(
        out,
        Vec3::new(r.max.x - 34.0, 15.0, r.min.y + 132.0),
        26.0,
        30.0,
        Color::srgb(0.72, 0.73, 0.72),
        0.0,
    );
    disc(
        out,
        Vec3::new(r.min.x + 132.0, 97.0, r.min.y + 22.0),
        9.0,
        14.0,
        Color::srgb(0.66, 0.74, 0.76),
        0.0,
    );

    towel_rail(
        out,
        Vec3::new(r.max.x - 8.0, 148.0, r.max.y - 120.0),
        84.0,
        false,
        -1.0,
    );

    // A bathroom this size wants more than three fixtures against two walls.
    // The plan's scale makes the main bath sixteen feet by twenty, and a bath,
    // a basin and a lavatory in a room that big read as a tiled hall with
    // plumbing in the corners.
    if r.deep() < 520.0 {
        return;
    }

    // A second basin, because the run is long enough to have wanted one.
    basin(
        out,
        Vec3::new(r.min.x + 172.0, 90.0, r.min.y + 32.0),
        50.0,
        42.0,
    );
    tap(
        out,
        Vec3::new(r.min.x + 172.0, 92.0, r.min.y + 8.0),
        Vec2::new(0.0, 16.0),
    );

    // A linen press against the east wall: two doors, two handles, and a plinth
    // it stands on.
    let press = Vec2::new(r.max.x - 34.0, r.min.y + 210.0);
    slab(
        out,
        Vec3::new(press.x, 6.0, press.y),
        Vec3::new(56.0, 12.0, 122.0),
        Stuff::Wood,
        Color::srgb(0.30, 0.30, 0.31),
    );
    slab(
        out,
        Vec3::new(press.x, 112.0, press.y),
        Vec3::new(60.0, 200.0, 128.0),
        Stuff::Wood,
        PORCELAIN,
    );
    for side in [-1.0f32, 1.0] {
        slab(
            out,
            Vec3::new(press.x - 31.0, 108.0, press.y + side * 32.0),
            Vec3::new(4.0, 176.0, 56.0),
            Stuff::Wood,
            Color::srgb(0.88, 0.88, 0.86),
        );
        slab(
            out,
            Vec3::new(press.x - 34.0, 108.0, press.y + side * 8.0),
            Vec3::new(4.0, 20.0, 4.0),
            Stuff::Metal,
            CHROME,
        );
    }

    // A shower in the far corner: a tray, two glass sides, and a riser.
    let tray = Vec2::new(r.max.x - 66.0, r.max.y - 72.0);
    let (sw, sd) = (118.0, 130.0);
    slab(
        out,
        Vec3::new(tray.x, 5.0, tray.y),
        Vec3::new(sw, 10.0, sd),
        Stuff::Stone,
        PORCELAIN,
    );
    slab(
        out,
        Vec3::new(tray.x - sw * 0.5 + 3.0, 106.0, tray.y),
        Vec3::new(6.0, 192.0, sd),
        Stuff::Glass,
        Color::srgba(0.84, 0.90, 0.92, 0.26),
    );
    slab(
        out,
        Vec3::new(tray.x, 106.0, tray.y - sd * 0.5 + 3.0),
        Vec3::new(sw, 192.0, 6.0),
        Stuff::Glass,
        Color::srgba(0.84, 0.90, 0.92, 0.26),
    );
    slab(
        out,
        Vec3::new(r.max.x - 12.0, 130.0, tray.y),
        Vec3::new(5.0, 150.0, 5.0),
        Stuff::Metal,
        CHROME,
    );
    slab(
        out,
        Vec3::new(r.max.x - 30.0, 202.0, tray.y),
        Vec3::new(36.0, 5.0, 22.0),
        Stuff::Metal,
        CHROME,
    );

    // And a laundry basket, because clothes come off somewhere.
    slab(
        out,
        Vec3::new(r.min.x + 46.0, 26.0, r.max.y - 60.0),
        Vec3::new(46.0, 52.0, 40.0),
        Stuff::Fabric,
        Color::srgb(0.68, 0.64, 0.56),
    );
    slab(
        out,
        Vec3::new(r.min.x + 46.0, 54.0, r.max.y - 60.0),
        Vec3::new(50.0, 6.0, 44.0),
        Stuff::Fabric,
        Color::srgb(0.76, 0.72, 0.64),
    );
}

fn laundry(out: &mut Vec<Solid>, r: &Room) {
    // Washer and dryer side by side. The dryer is the warmest surface in the
    // house, which will matter a great deal to a fly the moment warmth is a
    // thing the game models.
    for (i, paint) in [STEEL, PORCELAIN].into_iter().enumerate() {
        let at = Vec2::new(r.min.x + 70.0 + i as f32 * 74.0, r.min.y + 46.0);
        appliance(out, at, Vec3::new(66.0, 88.0, 64.0), paint);
        // A carcass and nothing else is a white cupboard: side by side and both
        // pale, the pair read as one box. A door, a fascia and a dial is what
        // tells you which one is which.
        let front = at.y + 33.0;
        slab(
            out,
            Vec3::new(at.x, 78.0, front),
            Vec3::new(62.0, 16.0, 3.0),
            Stuff::Metal,
            Color::srgb(0.24, 0.25, 0.27),
        );
        disc(
            out,
            Vec3::new(at.x + 20.0, 78.0, front + 2.0),
            9.0,
            4.0,
            CHROME,
            0.0,
        );
        if i == 0 {
            // The washer's drum door, with a glass port in it.
            disc(
                out,
                Vec3::new(at.x, 42.0, front + 1.0),
                44.0,
                4.0,
                Color::srgb(0.86, 0.87, 0.88),
                0.0,
            );
            disc(
                out,
                Vec3::new(at.x, 42.0, front + 3.0),
                26.0,
                3.0,
                Color::srgba(0.40, 0.44, 0.46, 0.65),
                0.0,
            );
        } else {
            // The dryer's is a plain hinged door with a handle down one side.
            slab(
                out,
                Vec3::new(at.x, 42.0, front + 1.5),
                Vec3::new(56.0, 52.0, 3.0),
                Stuff::Metal,
                Color::srgb(0.88, 0.88, 0.86),
            );
            slab(
                out,
                Vec3::new(at.x - 26.0, 42.0, front + 4.0),
                Vec3::new(4.0, 34.0, 4.0),
                Stuff::Metal,
                CHROME,
            );
        }
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

    // The south wall was blank the full width of the room. A laundry is where a
    // house keeps the things it has nowhere else for.
    //
    // A hanging rail with a few things on it, an ironing board leaning where
    // one always leans, a broom and a mop in the corner, and a basket.
    let south = r.max.y - 16.0;
    for side in [-1.0f32, 1.0] {
        slab(
            out,
            Vec3::new(r.middle().x + side * 78.0, 172.0, south + 6.0),
            Vec3::new(5.0, 26.0, 16.0),
            Stuff::Metal,
            CHROME,
        );
    }
    slab(
        out,
        Vec3::new(r.middle().x, 160.0, south),
        Vec3::new(170.0, 3.0, 3.0),
        Stuff::Metal,
        CHROME,
    );
    for k in 0..4 {
        let n = wobble(r.middle().x + k as f32 * 23.0, south);
        let x = r.middle().x - 60.0 + k as f32 * 40.0 + n * 6.0;
        slab(
            out,
            Vec3::new(x, 158.0, south + 2.0),
            Vec3::new(26.0, 5.0, 10.0),
            Stuff::Wood,
            OAK,
        );
        slab(
            out,
            Vec3::new(x, 132.0, south + 2.0),
            Vec3::new(34.0, 48.0, 6.0),
            Stuff::Fabric,
            [TOWEL_A, TOWEL_B, DUVET, WOOL_WARM][k],
        );
    }
    // The ironing board, leaning.
    turned(
        out,
        Vec3::new(r.min.x + 62.0, 74.0, south - 14.0),
        Vec3::new(38.0, 130.0, 8.0),
        Quat::from_rotation_x(-0.16),
        Stuff::Fabric,
        Color::srgb(0.70, 0.72, 0.68),
    );
    // Broom and mop in the corner.
    for (i, (paint, head)) in [
        (Color::srgb(0.46, 0.34, 0.20), Color::srgb(0.62, 0.50, 0.24)),
        (Color::srgb(0.36, 0.40, 0.44), Color::srgb(0.74, 0.74, 0.70)),
    ]
    .into_iter()
    .enumerate()
    {
        let at = Vec3::new(r.min.x + 24.0 + i as f32 * 22.0, 0.0, south - 6.0);
        let lean = Quat::from_rotation_z(0.10 + i as f32 * 0.05);
        turned(
            out,
            at + Vec3::new(0.0, 74.0, 0.0),
            Vec3::new(4.0, 148.0, 4.0),
            lean,
            Stuff::Wood,
            paint,
        );
        turned(
            out,
            at + Vec3::new(-6.0, 8.0, 0.0),
            Vec3::new(26.0, 16.0, 10.0),
            lean,
            Stuff::Fabric,
            head,
        );
    }
    slab(
        out,
        Vec3::new(r.max.x - 60.0, 24.0, south - 26.0),
        Vec3::new(48.0, 48.0, 40.0),
        Stuff::Fabric,
        Color::srgb(0.68, 0.64, 0.56),
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

    // A bowl for keys on the console, and an umbrella leaning in the corner.
    disc(
        out,
        Vec3::new(r.min.x + 34.0, 79.0, m.y - 260.0),
        20.0,
        7.0,
        Color::srgb(0.44, 0.40, 0.34),
        0.0,
    );
    turned(
        out,
        Vec3::new(r.min.x + 22.0, 44.0, r.min.y + 42.0),
        Vec3::new(6.0, 88.0, 6.0),
        Quat::from_rotation_z(0.13),
        Stuff::Fabric,
        Color::srgb(0.26, 0.30, 0.38),
    );

    // Photographs down the long wall, hung in the stretches between the bedroom
    // doors rather than at guessed offsets — the doorways are where they are,
    // and a frame over an architrave is the same mistake as one over a window.
    let doors: Vec<f32> = crate::house::interior_doors()
        .into_iter()
        .filter(|(lo, hi)| ((lo.x + hi.x) * 0.5 - r.min.x).abs() < 40.0)
        .map(|(lo, hi)| (lo.z + hi.z) * 0.5)
        .collect();
    let mut edges = vec![r.min.y + 40.0];
    for z in &doors {
        edges.push(z - 70.0);
        edges.push(z + 70.0);
    }
    edges.push(r.max.y - 40.0);
    edges.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut hung = 0usize;
    for pair in edges.chunks(2) {
        let (lo, hi) = (pair[0], pair[1]);
        if hi - lo < 120.0 {
            continue;
        }
        // Two or three to a stretch, stepped in height the way a family hangs
        // them: not level, and not random either.
        let how_many = if hi - lo > 300.0 { 3 } else { 2 };
        for k in 0..how_many {
            let t = (k as f32 + 1.0) / (how_many as f32 + 1.0);
            let z = lo + (hi - lo) * t;
            let n = wobble(z, hung as f32 * 5.0);
            let wide = 40.0 + n.abs() * 22.0;
            picture(
                out,
                Vec3::new(r.min.x + 7.0, 156.0 + n * 16.0, z),
                wide,
                wide * (0.72 + n * 0.16),
                false,
                [
                    Color::srgb(0.44, 0.40, 0.34),
                    Color::srgb(0.30, 0.36, 0.38),
                    Color::srgb(0.50, 0.44, 0.30),
                    Color::srgb(0.36, 0.32, 0.36),
                ][hung % 4],
            );
            hung += 1;
        }
    }
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
    // A pegboard over the bench with the tools on it. The wall above a
    // workbench is the one wall in a house that is never blank.
    let board_x = r.min.x + 180.0;
    let board_y = 150.0;
    let wall = r.min.y + 6.0;
    slab(
        out,
        Vec3::new(board_x, board_y, wall),
        Vec3::new(272.0, 96.0, 4.0),
        Stuff::Wood,
        Color::srgb(0.52, 0.40, 0.26),
    );
    for k in 0..9 {
        let n = wobble(board_x + k as f32 * 17.0, board_y);
        let x = board_x - 118.0 + k as f32 * 29.0;
        let hang = wall - 5.0;
        match k % 4 {
            // A hammer: handle and head.
            0 => {
                slab(
                    out,
                    Vec3::new(x, board_y - 8.0, hang),
                    Vec3::new(4.0, 40.0, 4.0),
                    Stuff::Wood,
                    Color::srgb(0.54, 0.36, 0.20),
                );
                slab(
                    out,
                    Vec3::new(x, board_y + 16.0, hang),
                    Vec3::new(18.0, 8.0, 7.0),
                    Stuff::Metal,
                    Color::srgb(0.30, 0.30, 0.32),
                );
            }
            // A saw: blade and grip.
            1 => {
                turned(
                    out,
                    Vec3::new(x + 6.0, board_y - 6.0, hang),
                    Vec3::new(46.0, 14.0, 2.0),
                    Quat::from_rotation_z(-0.34),
                    Stuff::Metal,
                    Color::srgb(0.70, 0.71, 0.73),
                );
                slab(
                    out,
                    Vec3::new(x - 12.0, board_y + 8.0, hang),
                    Vec3::new(10.0, 16.0, 5.0),
                    Stuff::Wood,
                    Color::srgb(0.44, 0.28, 0.18),
                );
            }
            // A spanner or two.
            2 => {
                for j in 0..2 {
                    slab(
                        out,
                        Vec3::new(
                            x + j as f32 * 9.0 - 4.0,
                            board_y + 2.0 - j as f32 * 5.0,
                            hang,
                        ),
                        Vec3::new(4.0, 30.0 - j as f32 * 7.0, 3.0),
                        Stuff::Metal,
                        Color::srgb(0.62, 0.63, 0.66),
                    );
                }
            }
            // A level, hung crooked.
            _ => {
                turned(
                    out,
                    Vec3::new(x, board_y + n * 6.0, hang),
                    Vec3::new(58.0, 8.0, 4.0),
                    Quat::from_rotation_z(0.06 + n * 0.05),
                    Stuff::Metal,
                    Color::srgb(0.72, 0.56, 0.16),
                );
            }
        }
    }
    // Paint tins under the bench.
    for k in 0..3 {
        let n = wobble(r.min.x + k as f32 * 41.0, r.min.y);
        disc(
            out,
            Vec3::new(
                r.min.x + 70.0 + k as f32 * 34.0 + n * 6.0,
                13.0,
                r.min.y + 44.0 + n * 8.0,
            ),
            26.0 - k as f32 * 3.0,
            26.0,
            [
                Color::srgb(0.78, 0.78, 0.76),
                Color::srgb(0.36, 0.42, 0.50),
                Color::srgb(0.62, 0.60, 0.52),
            ][k],
            0.0,
        );
    }

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
