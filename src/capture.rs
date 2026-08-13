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
//!
//! **Captures go through an offscreen target, not the window.** Reading back a
//! window's swapchain needs the compositor to have actually drawn that window,
//! and macOS does not draw a window that is not in front — which is precisely
//! the situation an unattended run is in. The symptom is a frame that is solid
//! black *including its background*, arriving with no error of any kind, and it
//! looks exactly like a lighting bug: three separate times in one session it was
//! mistaken for one. Rendering to an image instead is compositor-independent and
//! gives the same picture the player would see.

use bevy::camera::RenderTarget;
use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use wgpu_types::TextureFormat;

pub const CAPTURE_VAR: &str = "FLY_CAPTURE";
pub const CAPTURE_DELAY_VAR: &str = "FLY_CAPTURE_DELAY";
pub const CAPTURE_SIZE_VAR: &str = "FLY_CAPTURE_SIZE";

pub fn capture_path() -> Option<String> {
    std::env::var(CAPTURE_VAR).ok().filter(|p| !p.is_empty())
}

/// The offscreen image the camera draws into while capturing.
#[derive(Resource)]
pub struct CaptureTarget {
    pub image: Handle<Image>,
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
        // After the camera exists, so there is something to redirect.
        .add_systems(PostStartup, aim_at_an_image)
        .add_systems(Update, auto_capture);
    }
}

/// Point the camera at an offscreen image rather than at the window.
fn aim_at_an_image(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    let Ok(camera) = cameras.single() else {
        return;
    };
    let size = std::env::var(CAPTURE_SIZE_VAR)
        .ok()
        .and_then(|v| {
            let (w, h) = v.split_once('x')?;
            Some(UVec2::new(w.parse().ok()?, h.parse().ok()?))
        })
        .or_else(|| {
            windows
                .single()
                .ok()
                .map(|w| UVec2::new(w.physical_width(), w.physical_height()))
        })
        .unwrap_or(UVec2::new(1920, 1080));

    let image = images.add(Image::new_target_texture(
        size.x.max(1),
        size.y.max(1),
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    commands
        .entity(camera)
        .insert(RenderTarget::Image(image.clone().into()));
    commands.insert_resource(CaptureTarget { image });
    info!("capture target sized {}x{}", size.x, size.y);
}

fn auto_capture(
    mut commands: Commands,
    // Real time: the delay is a wall-clock promise, not a simulation one.
    time: Res<Time<Real>>,
    mut capture: ResMut<AutoCapture>,
    target: Option<Res<CaptureTarget>>,
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
        info!(
            "CAPTURE keys held: {:?}",
            keys.get_pressed().collect::<Vec<_>>()
        );
    }

    // The offscreen target if there is one; the window only as a fallback, and
    // a window readback on an unfocused macOS window is the black frame this
    // whole arrangement exists to avoid.
    let shot = match &target {
        Some(target) => Screenshot::image(target.image.clone()),
        None => Screenshot::primary_window(),
    };
    commands
        .spawn(shot)
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
