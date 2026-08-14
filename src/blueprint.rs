//! Reading a house that somebody drew.
//!
//! Opificium — the bench from Divus Factus — authors buildings by hand and
//! bakes them to a list of plain boxes. This file takes one of those in. The
//! greybox in [`crate::world`] was always scaffolding for the movement spike;
//! a real floor plan is drawn, not typed, and this is the door it comes through.
//!
//! **The two formats are very nearly the same object.** A baked box is
//! `{at, size, turn}` in metres; a [`Solid`] is `{center, half, rot}` in
//! centimetres. The import is a multiply by a hundred, a halve, and a
//! quaternion copy. That is not luck — both are "an oriented box and nothing
//! else", which is the representation a house wants and the reason no physics
//! engine was taken in the first place.
//!
//! **The baked file, not the blueprint.** The bench also writes `.baz`
//! blueprints, which are a list of *parts* — `wall-6`, `prop:doorway` — and
//! importing those would mean reimplementing how a wall frames itself. It is
//! not a thin job: a small cottage bakes to seventy-six boxes, eleven of them
//! framing, because the bench puts real studs, plates, lintels and sills in its
//! walls. Baked output is the contract worth depending on; the parts are the
//! bench's own business.
//!
//! What is deliberately not handled yet: the five per cent of boxes whose
//! `form` is a roof shape (`wedge`, `ridge`, `cut:`, `hip:`, and the retired
//! `mitre`) are taken as their bounding boxes. For a fly that is wrong only
//! above the ceiling, and it is honest about it — [`load`] reports the count.

use bevy::prelude::*;
use serde::Deserialize;

use crate::world::{Home, Solid, Stuff, UNITS_PER_METRE};

/// What the command line asked for, if anything.
///
/// Nothing is the ordinary case and means the procedural house — the one this
/// game is actually building. `FLY_HOUSE=greybox` asks for the two-room movement
/// test, and `FLY_HOUSE=<name-or-path>` for a house drawn in Opificium, which is
/// kept as reference behaviour rather than as the goal.
pub fn requested() -> Option<String> {
    std::env::var("FLY_HOUSE")
        .ok()
        .filter(|said| !said.is_empty())
}

/// Where Opificium's `install` carries finished work, relative to the game.
///
/// The bench's project lives at `opificium/` **beside** this folder rather than
/// inside it, because the asset root ships verbatim in the bundle and the
/// running game watches it for changes — a bench autosaving in here would post
/// the workshop to every player and reload the game mid-write. `install` in
/// `opificium/opificium.json` is the deliberate act of carrying a finished
/// building across.
const INSTALLED: &str = "assets/buildings";

/// Turn whatever was asked for into a path that exists.
///
/// `FLY_HOUSE=ranch` should find the ranch. Since `install` puts baked work in
/// one known place, a bare name is looked for there, which makes the loop from
/// the bench to the game one word instead of a path.
fn find(named: &str) -> Option<std::path::PathBuf> {
    let given = std::path::PathBuf::from(named);
    if given.is_file() {
        return Some(given);
    }
    for guess in [
        format!("{INSTALLED}/{named}"),
        format!("{INSTALLED}/{named}.json"),
    ] {
        let path = std::path::PathBuf::from(guess);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Everything sitting in the installed folder, for an error message worth
/// reading. A missing house is nearly always a typo, and the cure is the list.
fn installed() -> Vec<String> {
    let Ok(dir) = std::fs::read_dir(INSTALLED) else {
        return Vec::new();
    };
    let mut found: Vec<String> = dir
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.strip_suffix(".json").map(str::to_owned)
        })
        .collect();
    found.sort();
    found
}

// ---------------------------------------------------------------------------
// The file
// ---------------------------------------------------------------------------

/// A baked building, as `assets/buildings/<name>.json` in Divus Factus.
///
/// Only the fields this game has a use for are named; serde ignores the rest,
/// which is what lets the bench's format grow without breaking the import.
#[derive(Deserialize)]
struct Baked {
    #[serde(default)]
    name: String,
    /// Footprint half-extents and total height, in metres. Used to find a
    /// sensible place to put a fly.
    #[serde(default)]
    half_w: f32,
    #[serde(default)]
    half_d: f32,
    #[serde(default)]
    high: f32,
    boxes: Vec<BakedBox>,
    #[serde(default)]
    marks: Vec<Mark>,
}

#[derive(Deserialize)]
struct BakedBox {
    at: [f32; 3],
    size: [f32; 3],
    /// `[x, y, z, w]`, which is glam's own order.
    #[serde(default = "identity")]
    turn: [f32; 4],
    #[serde(default = "white")]
    rgb: [u8; 3],
    #[serde(default = "opaque")]
    alpha: f32,
    #[serde(default)]
    form: String,
    /// `footing | frame | walls | roof | furnishing`. The plan view drops the
    /// roof with it; telling furniture from structure is the next thing it is
    /// for.
    #[serde(default)]
    stage: String,
}

/// A semantic anchor: `door`, `sleep`, `fire`, and whatever else the bench
/// learns to say.
///
/// Nothing reads these yet, and they are the most valuable thing in the file.
/// A fly's world is not made of walls, it is made of *what things are* — this
/// is warm, that is food, someone is asleep there — and marks are already the
/// shape of that answer.
#[derive(Deserialize, Clone, Debug)]
pub struct Mark {
    pub mark: String,
    pub at: [f32; 3],
    #[serde(default)]
    pub yaw: f32,
}

fn identity() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}
fn white() -> [u8; 3] {
    [200, 200, 200]
}
fn opaque() -> f32 {
    1.0
}

// ---------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------

/// What came in besides the geometry.
pub struct Imported {
    pub home: Home,
    pub marks: Vec<Mark>,
}

pub fn load(named: &str) -> Result<Imported, String> {
    let path = find(named).ok_or_else(|| {
        let known = installed();
        if known.is_empty() {
            format!("no house called '{named}', and nothing is installed in {INSTALLED}/")
        } else {
            format!(
                "no house called '{named}' — {INSTALLED}/ has: {}",
                known.join(", ")
            )
        }
    })?;
    let path = path.display().to_string();
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
    let baked: Baked = serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))?;

    let mut solids = Vec::with_capacity(baked.boxes.len());
    let mut approximated = 0usize;

    for b in &baked.boxes {
        // The bench draws the ground plane's clutter too. Nothing below the
        // footing is anything a fly indoors can reach, but it is cheap to keep
        // and wrong to guess at, so everything comes in.
        if !b.form.is_empty() && b.form != "box" {
            approximated += 1;
        }
        solids.push(Solid {
            center: Vec3::from(b.at) * UNITS_PER_METRE,
            half: Vec3::from(b.size) * 0.5 * UNITS_PER_METRE,
            rot: Quat::from_xyzw(b.turn[0], b.turn[1], b.turn[2], b.turn[3]).normalize(),
            unseen: false,
            model: None,
            piece: u32::MAX,
            outdoors: false,
            glow: 0.0,
            stuff: Stuff::Plaster,
            paint: Some(Color::srgb_u8(b.rgb[0], b.rgb[1], b.rgb[2])),
            sheer: b.alpha < 0.99,
            roof: b.stage == "roof",
        });
    }

    if solids.is_empty() {
        return Err(format!("{path}: no boxes in it"));
    }

    let mut home = Home {
        hulls: Vec::new(),
        solids,
        door: None,
        spawn: Vec3::ZERO,
    };

    ceil_the_rooms(&mut home);

    // Somewhere to put a fly: hanging from the ceiling of the biggest room in
    // the house.
    //
    // The first attempt stood in the middle of the footprint and looked up,
    // which is a fine rule and put the fly under a dining table in the dark —
    // the middle of a house is as likely to be furniture as air. Now that the
    // rooms are known, the largest one is the great room in every plan anybody
    // draws, and its ceiling is the best seat in the building.
    let (_, mut rooms) = crate::rooms::find(&home);
    rooms.sort_by_key(|room| std::cmp::Reverse(room.cells));
    home.spawn = match rooms.first() {
        Some(room) => {
            let from = Vec3::new(room.at.x, 15.0, room.at.z);
            let reach = baked.high.max(1.0) * UNITS_PER_METRE;
            match home.raycast(from, Vec3::Y, reach) {
                Some(hit) => hit.point - Vec3::Y * 8.0,
                None => Vec3::new(room.at.x, 75.0, room.at.z),
            }
        }
        None => Vec3::new(0.0, 75.0, 0.0),
    };

    info!(
        "house '{}' — {} boxes ({} roof shapes taken as boxes), {} marks, {:.1} x {:.1} x {:.1} m",
        baked.name,
        home.solids.len(),
        approximated,
        baked.marks.len(),
        baked.half_w * 2.0,
        baked.half_d * 2.0,
        baked.high,
    );

    Ok(Imported {
        home,
        marks: baked.marks,
    })
}

/// The commonest wall top in the house, which is where a ceiling belongs.
///
/// The mode rather than the mean or the max: a house has a few walls taller
/// than the rest — a gable end, a lintel course — and the ceiling follows the
/// ordinary ones.
fn wall_top(home: &Home) -> Option<f32> {
    let mut tally: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
    for solid in &home.solids {
        if solid.roof {
            continue;
        }
        let top = solid.center.y + solid.half.y;
        if top > 200.0 {
            *tally.entry((top / 5.0).round() as i32).or_default() += 1;
        }
    }
    tally
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(bucket, _)| bucket as f32 * 5.0)
}

/// Put a ceiling over every room, if the house has not got one already.
///
/// Opificium draws walls, footings and a roof; a ceiling is a thing a maker has
/// to remember, and the first real house did not have one. Without it a fly can
/// climb a wall and out into the roof void, every room shares its air with every
/// other over the top of the walls, and nothing indoors is in shadow because
/// there is nothing overhead to cast one.
///
/// Each room gets a slab across its own footprint, which is why this waits for
/// [`crate::rooms`] rather than laying one sheet over the whole plan: a single
/// sheet would also roof the porch, the void behind the walls, and the gap the
/// stairs would go in.
fn ceil_the_rooms(home: &mut Home) {
    let Some(top) = wall_top(home) else {
        return;
    };
    // Already ceilinged? Something wide and flat sitting at wall-top height is
    // a ceiling whatever the maker called it.
    let existing = home.solids.iter().any(|solid| {
        !solid.roof
            && solid.half.y < 20.0
            && solid.half.x > 100.0
            && solid.half.z > 100.0
            && (solid.center.y - top).abs() < 40.0
    });
    if existing {
        return;
    }

    let (_, rooms) = crate::rooms::find(home);
    if rooms.is_empty() {
        return;
    }

    const THICK: f32 = 12.0;
    // Over the wall tops rather than under them, so the slab meets the wall it
    // sits on instead of hanging in the room below it.
    let middle = top + THICK * 0.5;
    let count = rooms.len();
    for room in &rooms {
        let min = Vec3::new(room.min.x, middle - THICK * 0.5, room.min.y);
        let max = Vec3::new(room.max.x, middle + THICK * 0.5, room.max.y);
        let mut slab = Solid::between(min, max, Stuff::Plaster);
        // Overhead, so the plan view drops it along with the roof — otherwise
        // adding a ceiling is exactly the thing that stops you seeing the rooms.
        slab.roof = true;
        // Plaster, and a touch warmer than the greybox's: it is lit from below
        // by household bulbs and reads cold otherwise.
        slab.paint = Some(Color::srgb_u8(196, 186, 172));
        home.solids.push(slab);
    }
    info!("no ceiling in this house — laid {count} over the rooms at {top:.0} cm");
}

/// Every mark that came in, for whatever reads them next.
#[derive(Resource, Default)]
pub struct Marks(pub Vec<Mark>);
