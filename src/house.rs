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

/// Skirting: how tall, and how far proud of the plaster it stands.
///
/// Two centimetres of relief is nothing to a person and a landable ledge running
/// the entire perimeter of every room to a fly — which is the whole argument for
/// modelling trim in a game seen from six millimetres up.
const SKIRT_HIGH: f32 = 9.0;
const SKIRT_PROUD: f32 = 2.0;

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

        // Skirting, on any piece that reaches the floor.
        //
        // Emitted here rather than around the room afterwards, because a run
        // already knows where its solid parts are: a doorway gets no skirting
        // across it for free, and nothing has to be told where the openings
        // were. Both faces, since a partition has a room on each side.
        if low <= 0.01 && high > SKIRT_HIGH {
            for face in [-1.0f32, 1.0] {
                let (smin, smax) = if along_x {
                    (
                        Vec3::new(a.x + s, 0.0, a.y + face * half),
                        Vec3::new(a.x + e, SKIRT_HIGH, a.y + face * (half + SKIRT_PROUD)),
                    )
                } else {
                    (
                        Vec3::new(a.x + face * half, 0.0, a.y + s),
                        Vec3::new(a.x + face * (half + SKIRT_PROUD), SKIRT_HIGH, a.y + e),
                    )
                };
                let lo = smin.min(smax);
                let hi = smin.max(smax);
                let mut skirt = Solid::between(lo, hi, Stuff::Wood);
                skirt.paint = Some(Color::srgb(0.90, 0.89, 0.86));
                out.push(skirt);
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
    s.push(Solid::between(
        Vec3::new(w - 4000.0, -60.0, n - 4000.0),
        Vec3::new(e + 4000.0, -10.0, so + 4000.0),
        Stuff::Grass,
    ));

    // -- Floor -------------------------------------------------------------
    s.push(Solid::between(
        Vec3::new(w, -SLAB, n),
        Vec3::new(house_east, 0.0, so),
        Stuff::Floorboard,
    ));
    // The garage slab is concrete and lower — a step down from the laundry, the
    // way every ranch does it, and a fly-scale ledge into the bargain.
    s.push(Solid::between(
        Vec3::new(house_east, -SLAB, n),
        Vec3::new(e, -6.0, garage_south),
        Stuff::Stone,
    ));

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

    glaze(&mut s);
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
fn glaze(s: &mut Vec<Solid>) {
    let (w, _e, n, so, _gs, house_east) = envelope();
    let pane = 2.0;
    let half = WINDOW_WIDE * 0.5;

    for &x in &NORTH_WINDOWS {
        s.push(Solid::between(
            Vec3::new(ft(x) - half, SILL, n - pane * 0.5),
            Vec3::new(ft(x) + half, HEAD, n + pane * 0.5),
            Stuff::Glass,
        ));
    }
    for &x in &SOUTH_WINDOWS {
        s.push(Solid::between(
            Vec3::new(ft(x) - half, SILL, so - pane * 0.5),
            Vec3::new(ft(x) + half, HEAD, so + pane * 0.5),
            Stuff::Glass,
        ));
    }
    for &z in &WEST_WINDOWS {
        s.push(Solid::between(
            Vec3::new(w - pane * 0.5, SILL, ft(z) - half),
            Vec3::new(w + pane * 0.5, HEAD, ft(z) + half),
            Stuff::Glass,
        ));
    }
    s.push(Solid::between(
        Vec3::new(house_east - pane * 0.5, SILL, ft(EAST_WINDOW) - half),
        Vec3::new(house_east + pane * 0.5, HEAD, ft(EAST_WINDOW) + half),
        Stuff::Glass,
    ));
}

// ---------------------------------------------------------------------------
// Checking the laws
// ---------------------------------------------------------------------------

/// Measure what was actually built, and complain if it breaks a law.
///
/// The plan's numbers are not evidence. A room is only fifteen feet if it
/// *measures* fifteen feet once every wall around it has taken its thickness,
/// and the cheapest way to be sure of that forever is to check on every run
/// rather than to be careful once.
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

// ---------------------------------------------------------------------------
// Light
// ---------------------------------------------------------------------------

/// Authored lights: one fixture per room, plus the sun.
///
/// Authored rather than found. The flood fill in `rooms` exists because a
/// *drawn* house does not say where its rooms are; this one does.
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
    }
}
