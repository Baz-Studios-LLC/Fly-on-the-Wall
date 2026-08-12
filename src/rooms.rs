//! Finding the rooms in a house that never said where they are.
//!
//! A drawn house arrives as a couple of hundred boxes. Nothing in it says
//! "kitchen"; nothing even says "room". Two things need to know anyway — the
//! lamps, so each room gets one, and the ceiling, because the first real house
//! drawn in the bench turned out not to have one.
//!
//! **The trick is the height.** Flood-filling the open space at head height
//! finds *one* room, because every doorway joins it to the next: a house is one
//! connected volume to anything shorter than a door. Flood it above the door
//! heads and the lintels close every doorway, so each room becomes its own
//! island.
//!
//! **Which height is not something to guess at.** The first attempt cast upward
//! and took the commonest answer as the ceiling — and on a house with no ceiling
//! it found the underside of the roof, floated above the wall tops where every
//! room is one room, and reported a whole ranch as two. So it searches now:
//! every distinct surface is a candidate, it floods at each, and it keeps
//! whichever found the most rooms. That needs to know nothing about ceilings,
//! doors or roofs, and answers the same for a house that has a ceiling as for
//! one that does not.

use bevy::prelude::*;

use crate::world::Home;

/// The flood fill's cell, in centimetres. Finer than any doorway, coarse enough
/// that a whole house is a few thousand cells.
const CELL: f32 = 20.0;

/// A door head is about 2.1 m, so nothing below this can separate one room from
/// the next.
const ABOVE_THE_DOORS: f32 = 220.0;

/// How many heights to try. Each is a flood fill over the whole grid, and past a
/// handful they stop saying anything new.
const TRIES: usize = 8;

/// Islands smaller than this are a cupboard, a chimney void, or the gap behind a
/// fridge.
const SMALLEST_ROOM: usize = 12;

pub struct Room {
    /// The middle of the floor, for hanging something over.
    pub at: Vec3,
    /// How many cells it covered — a stand-in for area, and how rooms are
    /// ranked when there are more than there is budget for.
    pub cells: usize,
    /// The footprint, in world units.
    pub min: Vec2,
    pub max: Vec2,
}

/// Every room in the house, and the height the slice that found them was taken
/// at.
pub fn find(home: &Home) -> (f32, Vec<Room>) {
    let Some((low, high)) = bounds(home) else {
        return (0.0, Vec::new());
    };

    let mut heights: Vec<f32> = Vec::new();
    for solid in &home.solids {
        for face in [solid.center.y - solid.half.y, solid.center.y + solid.half.y] {
            // Just under a surface, which is where a room is still a room.
            let at = face - 8.0;
            if at > low.y + ABOVE_THE_DOORS && at < high.y {
                heights.push(at);
            }
        }
    }
    heights.sort_by(|a, b| a.partial_cmp(b).unwrap());
    heights.dedup_by(|a, b| (*a - *b).abs() < 10.0);

    // Thin to a handful, evenly spread: a house has hundreds of surfaces and
    // most sit at the same few heights as their neighbours.
    if heights.len() > TRIES {
        let step = heights.len() as f32 / TRIES as f32;
        heights = (0..TRIES)
            .map(|i| heights[((i as f32 + 0.5) * step) as usize])
            .collect();
    }
    if heights.is_empty() {
        heights.push(low.y + ABOVE_THE_DOORS);
    }

    let mut best = (heights[0], Vec::new());
    for at in heights {
        let found = on_the_slice(home, low, high, at);
        if found.len() > best.1.len() {
            best = (at, found);
        }
    }
    best
}

pub fn bounds(home: &Home) -> Option<(Vec3, Vec3)> {
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for solid in &home.solids {
        // The rotated case would want the eight corners; nothing in a baked
        // house is turned far enough for the difference to matter, and being
        // generous costs only a slightly larger grid.
        let reach = (solid.rot * solid.half).abs().max(solid.half);
        low = low.min(solid.center - reach);
        high = high.max(solid.center + reach);
    }
    low.x.is_finite().then_some((low, high))
}

fn on_the_slice(home: &Home, low: Vec3, high: Vec3, at: f32) -> Vec<Room> {
    let wide = (((high.x - low.x) / CELL).ceil() as usize).max(1);
    let deep = (((high.z - low.z) / CELL).ceil() as usize).max(1);

    let point = |ix: usize, iz: usize| {
        Vec3::new(
            low.x + (ix as f32 + 0.5) * CELL,
            at,
            low.z + (iz as f32 + 0.5) * CELL,
        )
    };

    // Open means "not inside anything". A cell inside a solid is wall, ceiling,
    // or furniture tall enough to reach up here.
    let mut open = vec![false; wide * deep];
    for iz in 0..deep {
        for ix in 0..wide {
            let p = point(ix, iz);
            open[iz * wide + ix] = !home
                .solids
                .iter()
                .any(|solid| solid.nearest(p).distance < 0.0);
        }
    }

    let mut seen = vec![false; wide * deep];
    let mut rooms = Vec::new();
    for start in 0..wide * deep {
        if seen[start] || !open[start] {
            continue;
        }
        let mut stack = vec![start];
        let mut island = Vec::new();
        // An island touching the outer ring is outdoors — the air around the
        // house is "open" too, and without this the sky is one enormous room.
        let mut outdoors = false;
        seen[start] = true;
        while let Some(cell) = stack.pop() {
            island.push(cell);
            let (ix, iz) = (cell % wide, cell / wide);
            if ix == 0 || iz == 0 || ix + 1 == wide || iz + 1 == deep {
                outdoors = true;
            }
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
        if outdoors || island.len() < SMALLEST_ROOM {
            continue;
        }

        let mut middle = Vec3::ZERO;
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for cell in &island {
            let p = point(cell % wide, cell / wide);
            middle += p;
            min = min.min(Vec2::new(p.x - CELL * 0.5, p.z - CELL * 0.5));
            max = max.max(Vec2::new(p.x + CELL * 0.5, p.z + CELL * 0.5));
        }
        rooms.push(Room {
            at: middle / island.len() as f32,
            cells: island.len(),
            min,
            max,
        });
    }
    rooms
}
