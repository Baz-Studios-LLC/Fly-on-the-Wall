//! Unattended capture: run for a moment, save a frame, exit.
//!
//! Same reasoning as the other projects here — the world needs to be *looked at*
//! without a human sitting in front of it. In this build it earns its place
//! immediately: the fly's model orientation was derived from the mesh rather
//! than observed, and the lighting is a set of numbers nobody has seen the
//! result of. Both are settled by looking at one picture.
//!
//! `FLY_CAPTURE=shot.png cargo run` writes a frame and quits. `FLY_CAPTURE_DELAY`
//! moves the shutter, which matters because the glTF and its three 4K textures
//! are still decoding for the first second or so and an early shutter catches a
//! fly that has not arrived yet.
//!
//! F12 saves a frame whenever there *is* someone watching.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

pub const CAPTURE_VAR: &str = "FLY_CAPTURE";
pub const CAPTURE_DELAY_VAR: &str = "FLY_CAPTURE_DELAY";

fn capture_path() -> Option<String> {
    std::env::var(CAPTURE_VAR).ok().filter(|p| !p.is_empty())
}

#[derive(Resource)]
struct AutoCapture {
    path: String,
    delay: f32,
    taken: bool,
}

pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, screenshot_on_request);

        let Some(path) = capture_path() else {
            return;
        };
        let delay = std::env::var(CAPTURE_DELAY_VAR)
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(4.0);
        app.insert_resource(AutoCapture {
            path,
            delay,
            taken: false,
        })
        .add_systems(Update, auto_capture);
    }
}

fn auto_capture(
    mut commands: Commands,
    // Real time: the delay is a wall-clock promise, not a simulation one.
    time: Res<Time<Real>>,
    mut capture: ResMut<AutoCapture>,
    flies: Query<(&crate::fly::Fly, &crate::fly::Intent)>,
    meshes: Query<&Mesh3d>,
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.taken {
        // Let the save land before tearing the app down.
        if time.elapsed_secs() > capture.delay + 2.0 {
            exit.write(AppExit::Success);
        }
        return;
    }
    if time.elapsed_secs() < capture.delay {
        return;
    }
    capture.taken = true;

    // Worth logging: the mesh count says whether the glTF actually finished
    // loading, which is the difference between "the model is wrong" and "the
    // model is not there".
    if let Ok((fly, intent)) = flies.single() {
        info!(
            "CAPTURE meshes={} at ({:.1},{:.1},{:.1}) vel=({:.1},{:.1},{:.1}) \
             thrust={:?} land={} stance={:?}",
            meshes.iter().count(),
            fly.pos.x,
            fly.pos.y,
            fly.pos.z,
            fly.vel.x,
            fly.vel.y,
            fly.vel.z,
            intent.thrust,
            intent.land,
            fly.stance,
        );
        info!("CAPTURE keys held: {:?}", keys.get_pressed().collect::<Vec<_>>());
    }

    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(capture.path.clone()));
}

fn screenshot_on_request(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut counter: Local<u32>,
) {
    if keys.just_pressed(KeyCode::F12) {
        let path = format!("fly-{:03}.png", *counter);
        *counter += 1;
        info!("saving screenshot to {path}");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}
