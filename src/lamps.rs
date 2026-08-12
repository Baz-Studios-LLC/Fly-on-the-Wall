//! Lighting a house nobody wrote down the lights for.
//!
//! The greybox could have its bulbs placed by hand because it had two rooms and
//! they never moved. A drawn house cannot: it arrives as two hundred boxes with
//! no idea where its rooms are, and the first one loaded rendered as a black
//! rectangle with a fly in it.
//!
//! Where the rooms *are* is [`crate::rooms`]'s problem. This hangs one bulb in
//! each, at that room's own ceiling.
//!
//! **Shadows are not optional here.** A ranch is a dozen small rooms, and an
//! unshadowed bulb lights every one of them through the walls. So the bulbs cast
//! — which is why there is a ceiling on how many get placed.

use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use crate::world::{Home, UNITS_PER_METRE};

/// How many bulbs at most. Each one casts, and a shadow-casting point light is
/// six shadow passes.
const MOST_BULBS: usize = 14;

/// A household bulb, in lumens, before the scale correction. Bright for a bulb,
/// because it is usually the only thing lighting its room: a drawn house is
/// roofed, so the sun reaches no further than the windows.
const BULB: f32 = 1_600.0;

pub fn light_a_drawn_house(commands: &mut Commands, home: &Home) {
    let Some((low, high)) = crate::rooms::bounds(home) else {
        return;
    };
    let (lit, mut rooms) = crate::rooms::find(home);

    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });

    // Fill, and close to neutral. The first pass used the greybox's cool blue
    // at half again its brightness, which on a house painted in browns — and
    // this one is almost entirely `(69,47,34)` and `(93,66,44)` — turned every
    // floorboard navy. A fill this broad has to stay out of the way of whatever
    // the maker actually painted; the bulbs are what should be coloured.
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.72, 0.74, 0.78),
        brightness: 85.0,
        ..default()
    });

    // Sun, low and across the long axis, so whatever windows the maker drew
    // throw something. Aimed at the middle of the floor from outside the
    // footprint — the house's own size decides where that is.
    let span = high - low;
    let middle = (low + high) * 0.5;
    let from = Vec3::new(
        low.x - span.x * 0.6,
        high.y + span.y * 0.5,
        low.z - span.z * 0.35,
    );
    commands.spawn((
        Name::new("Afternoon"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.945, 0.85),
            illuminance: 9_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_translation(from)
            .looking_at(Vec3::new(middle.x, low.y + 30.0, middle.z), Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            maximum_distance: span.length() * 1.6,
            first_cascade_far_bound: 200.0,
            ..default()
        }
        .build(),
    ));

    // Biggest first, so the ceiling on how many bulbs there are drops cupboards
    // rather than the great room.
    rooms.sort_by_key(|room| std::cmp::Reverse(room.cells));
    rooms.truncate(MOST_BULBS);

    // The slice found the rooms; it is not where a lamp belongs. It sits
    // wherever the walls happened to stop dividing the house, which on the
    // first real ranch was 317 cm — above the median wall top, so every bulb
    // hung in the open air over the walls rather than inside a room. Ask each
    // room for its own ceiling instead.
    let scale = UNITS_PER_METRE * UNITS_PER_METRE;
    for (n, room) in rooms.iter().enumerate() {
        let from = Vec3::new(room.at.x, low.y + 20.0, room.at.z);
        let hung = home
            .raycast(from, Vec3::Y, high.y - from.y)
            .map_or(lit, |hit| hit.point.y - 15.0)
            .min(lit);
        commands.spawn((
            Name::new(format!("Bulb {}", n + 1)),
            PointLight {
                color: Color::srgb(1.0, 0.92, 0.80),
                intensity: BULB * scale,
                range: 800.0,
                radius: 2.5,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_translation(Vec3::new(room.at.x, hung, room.at.z)),
        ));
    }

    info!(
        "lit a drawn house: {} rooms found on the slice at {:.0} cm, {} bulbs hung",
        rooms.len(),
        lit,
        rooms.len().min(MOST_BULBS),
    );
}

