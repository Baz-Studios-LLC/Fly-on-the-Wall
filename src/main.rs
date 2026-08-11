//! **Fly on the Wall** — flight test.
//!
//! This binary is not the game. It is the first of the two spikes the design
//! discussion asked for, and it answers exactly one question:
//!
//! > From a rest on the ceiling of the living room, can you take off, thread the
//! > ajar door, and land upside down under the kitchen cabinets in one continuous
//! > motion — without thinking about the controls?
//!
//! Everything here exists to make that question answerable and nothing else does.
//! There is no family, no hunger, no danger, no night, no scent, no death. Those
//! are all downstream of movement feeling right, and none of them can rescue it
//! if it does not.
//!
//! One caveat worth writing down before anyone plays it: a movement model with
//! nothing at stake can feel excellent and still be wrong. Pressure changes how a
//! control scheme reads, and there is no pressure in this build. "Flight is
//! solved" stays provisional until something in the house is trying to kill you.
//!
//! ## Controls
//!
//! | | |
//! |---|---|
//! | Mouse | aim |
//! | `W` `A` `S` `D` | `W` thrust, `S` **brake** (a fly cannot fly backwards), `A` `D` sideslip. All four crawl when perched — including backwards, which flies really do |
//! | `Space` | climb, or take off from a surface |
//! | `Left Ctrl` | descend |
//! | right mouse / `F` | **hold to land.** Contact only sticks while it is held; let go and the fly grazes off what it clips |
//! | `E` | cycle the door: closed → ajar → open |
//! | `[` `]` | narrow or widen the ajar gap, a millimetre a press |
//! | `Q` | chase camera ↔ first person |
//! | `R` | keep the room upright ↔ keep the fly upright |
//! | `F3` | the readout |
//! | `F12` | save a screenshot |
//! | `Esc` | release the mouse |
//!
//! ## Switches
//!
//! | | |
//! |---|---|
//! | `FLY_MODEL=glb` | use `assets/fly.glb` instead of the procedural fly |
//! | `FLY_INSPECT=<deg>` | park the camera close to the fly at that azimuth — 0 behind, 180 head-on — and stand it on the living room floor. For looking at the model, not playing. |
//! | `FLY_CAPTURE=<path>` | render for a moment, save a frame there, exit |
//! | `FLY_CAPTURE_DELAY=<s>` | move the shutter (default 4) |

mod blueprint;
mod body;
mod camera;
mod capture;
mod debug;
mod fly;
mod wingbeat;
mod world;

use bevy::audio::{AudioPlugin, Volume};
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fly on the Wall — flight test".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AudioPlugin {
                    // The wingbeat is the only sound in the build and it plays
                    // continuously, so the global level is set low here rather
                    // than in the synthesis, where it would be tangled up with
                    // the effort curve.
                    global_volume: Volume::Linear(0.7).into(),
                    ..default()
                }),
        )
        .add_plugins((
            world::WorldPlugin,
            fly::FlyPlugin,
            body::BodyPlugin,
            camera::CameraPlugin,
            wingbeat::WingbeatPlugin,
            debug::DebugPlugin,
            capture::CapturePlugin,
        ))
        .add_systems(Startup, light_the_house)
        .run();
}

/// Lighting a house at one centimetre to the unit.
///
/// The first version of this was flat and wrong, for a structural reason worth
/// recording: **a sealed room cannot be lit from outside.** A directional light
/// over a closed box contributes nothing indoors, so every surface in the house
/// was drawing pure ambient — the same value from every angle, which is exactly
/// what "no shading" looks like. No amount of tuning fixes that; the wall needs
/// a hole in it. There is now a window in the living room's west wall, and the
/// sun is aimed through it.
///
/// The second trap is the scale. Bevy's lighting is physical and assumes one
/// unit is one metre, so at one unit to the centimetre a bulb 250 units up reads
/// as 250 *metres* away and its inverse-square falloff annihilates it. Lumens
/// have to be multiplied by the square of the unit ratio — [`LUMEN_SCALE`] —
/// which is why the numbers below look absurd and are in fact ordinary
/// household bulbs. Directional light is unaffected: illuminance does not fall
/// off with distance, so lux is lux.
fn light_the_house(mut commands: Commands) {
    /// Lumens are per square metre; the world is in centimetres. Ten thousand.
    const LUMEN_SCALE: f32 = world::UNITS_PER_METRE * world::UNITS_PER_METRE;

    // Fill, kept low enough that shading reads and high enough that the
    // underside of a kitchen cabinet is not a black rectangle. A fly lives under
    // things, so this floor matters more here than it would in most games — but
    // at 260 it was drowning every other light in the house.
    // Shadow maps at house scale. The default 2048 spread over a cascade sized
    // for a room leaves the sunbeam's edge visibly stair-stepped, and at fly
    // scale a stair is a centimetre wide.
    commands.insert_resource(DirectionalLightShadowMap { size: 4096 });

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.55, 0.64, 0.82),
        brightness: 88.0,
        ..default()
    });

    // Afternoon sun, from the west and low, aimed through the window. Sixteen
    // thousand lux is an overcast-to-bright day indoors; the shaft it throws
    // across the floorboards is the best-looking thing in the build and the main
    // argument for the window existing at all.
    commands.spawn((
        Name::new("Afternoon"),
        DirectionalLight {
            color: Color::srgb(1.0, 0.945, 0.85),
            illuminance: 6_200.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-900.0, 470.0, 40.0)
            .looking_at(Vec3::new(260.0, 30.0, 250.0), Vec3::Y),
        // Cascades sized for a house rather than a landscape. The defaults assume
        // a world measured in kilometres and put the first cascade outside the
        // building, which at this scale means no usable shadow anywhere.
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            maximum_distance: 1600.0,
            first_cascade_far_bound: 160.0,
            ..default()
        }
        .build(),
    ));

    // Two ceiling bulbs, so the rooms read when the sun is not in them — and so
    // the kitchen, which has no window, is lit at all. Warm in the living room,
    // cooler in the kitchen, which is most of what makes two grey boxes feel like
    // two different rooms.
    for (name, position, lumens, colour) in [
        (
            "Living room bulb",
            Vec3::new(250.0, 252.0, 200.0),
            820.0,
            Color::srgb(1.0, 0.90, 0.76),
        ),
        (
            "Kitchen bulb",
            Vec3::new(670.0, 222.0, 110.0),
            640.0,
            Color::srgb(0.94, 0.97, 1.0),
        ),
    ] {
        commands.spawn((
            Name::new(name),
            PointLight {
                color: colour,
                intensity: lumens * LUMEN_SCALE,
                // Eight metres, which covers either room comfortably.
                range: 800.0,
                // A bulb is a few centimetres across, and at this scale that is a
                // few units — enough to soften the shadow edges rather than
                // stamping them.
                radius: 2.5,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_translation(position),
        ));
    }
}
