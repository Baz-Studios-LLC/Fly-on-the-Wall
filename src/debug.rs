//! The readout, on F3.
//!
//! There is an obvious joke about the game with no HUD shipping with nothing but
//! HUD, and it is worth stating the actual position: this is instrumentation for
//! tuning, not an interface, and it does not survive the spike. Every number on
//! it is one the movement constants are being fitted against — speed against
//! `CRUISE`, the surface normal against the edge-wrap cases, the door gap against
//! whether threading it is a thrill or a chore. When those numbers stop changing,
//! this file is deleted, and the layer that replaces it is Ordo drawing a field
//! note over a dead fly.
//!
//! It starts hidden. The first thing anyone should do with this build is fly
//! around without it.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::camera::{Roll, View};
use crate::fly::{Fly, Stance, perch_normal};
use crate::world::{Door, DoorWant, Home};

const INK: Color = Color::srgb(0.86, 0.90, 0.95);
const BACKING: Color = Color::srgba(0.02, 0.03, 0.05, 0.72);

#[derive(Component)]
struct Readout;

#[derive(Resource, Default)]
struct Showing(bool);

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<Showing>()
            .add_systems(Startup, spawn_readout)
            .add_systems(Update, (toggle_readout, write_readout).chain());
    }
}

fn spawn_readout(mut commands: Commands) {
    commands.spawn((
        Readout,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(INK),
        BackgroundColor(BACKING),
        Node {
            position_type: PositionType::Absolute,
            top: px(10),
            left: px(10),
            padding: UiRect::axes(px(10), px(8)),
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn toggle_readout(
    keys: Res<ButtonInput<KeyCode>>,
    mut showing: ResMut<Showing>,
    mut readouts: Query<&mut Visibility, With<Readout>>,
) {
    if !keys.just_pressed(KeyCode::F3) {
        return;
    }
    showing.0 = !showing.0;
    for mut visibility in &mut readouts {
        *visibility = if showing.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn write_readout(
    showing: Res<Showing>,
    diagnostics: Res<DiagnosticsStore>,
    home: Res<Home>,
    door: Res<Door>,
    view: Res<View>,
    roll: Res<Roll>,
    flies: Query<&Fly>,
    mut readouts: Query<&mut Text, With<Readout>>,
) {
    if !showing.0 {
        return;
    }
    let Ok(fly) = flies.single() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let stance = match fly.stance {
        Stance::Flying => "flying".to_string(),
        Stance::Perched(perch) => {
            let normal = perch_normal(fly, &home).unwrap_or(Vec3::Y);
            let face = if normal.y > 0.7 {
                "floor"
            } else if normal.y < -0.7 {
                "ceiling"
            } else {
                "wall"
            };
            let riding = if home.door == Some(perch.solid) {
                " (on the door)"
            } else {
                ""
            };
            format!(
                "perched on a {face}{riding}  n = {:.2} {:.2} {:.2}",
                normal.x, normal.y, normal.z
            )
        }
    };

    let door_state = match door.want {
        DoorWant::Open => "open",
        DoorWant::Ajar => "ajar",
        DoorWant::Closed => "closed",
    };

    **readouts.single_mut().unwrap() = format!(
        "{stance}\n\
         speed   {:5.1} cm/s   effort {:.2}\n\
         at      {:.0} {:.0} {:.0}\n\
         door    {door_state}, {:.1}° open, slot {:.2} cm   ([ / ] to nudge)\n\
         camera  {:?} / {:?}\n\
         {fps:.0} fps",
        fly.vel.length(),
        fly.effort(),
        fly.pos.x,
        fly.pos.y,
        fly.pos.z,
        door.angle.to_degrees(),
        door.gap(),
        *view,
        *roll,
    );
}
