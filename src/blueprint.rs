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

/// `FLY_HOUSE=<path>` loads a baked building instead of the greybox.
pub fn requested() -> Option<String> {
    std::env::var("FLY_HOUSE").ok().filter(|p| !p.is_empty())
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
    /// `footing | frame | walls | roof | furnishing`. Parsed and not yet used —
    /// it is how an interior-only mode would drop the roof, and how furniture
    /// would be told apart from structure.
    #[serde(default)]
    #[allow(dead_code)]
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

pub fn load(path: &str) -> Result<Imported, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
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
            stuff: Stuff::Plaster,
            paint: Some(Color::srgb_u8(b.rgb[0], b.rgb[1], b.rgb[2])),
            sheer: b.alpha < 0.99,
        });
    }

    if solids.is_empty() {
        return Err(format!("{path}: no boxes in it"));
    }

    let mut home = Home {
        solids,
        door: None,
        spawn: Vec3::ZERO,
    };

    // Somewhere to put a fly. `high` is the roof *peak*, so any fraction of it
    // is as likely to be in the rafters as in a room — the first attempt at this
    // hatched the fly inside the gable and it spent its life falling.
    //
    // Instead, stand in the middle of the footprint and look up: the first thing
    // over your head is the ceiling of whatever room that is, and hanging just
    // under it is where a fly should start.
    let reach = baked.high.max(1.0) * UNITS_PER_METRE * 1.2;
    home.spawn = match home.raycast(Vec3::new(0.0, 15.0, 0.0), Vec3::Y, reach) {
        Some(hit) => hit.point - Vec3::Y * 8.0,
        // No ceiling over the middle of the house is odd but not fatal — start
        // at about the height of a table and let it fly.
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

/// Every mark that came in, for whatever reads them next.
#[derive(Resource, Default)]
pub struct Marks(pub Vec<Mark>);
