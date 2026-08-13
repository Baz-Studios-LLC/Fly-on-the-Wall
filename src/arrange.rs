//! Moving the furniture.
//!
//! The house is generated, which normally means the arrangement is whatever the
//! code says and the only way to disagree with it is to edit Rust. This is the
//! other way in: fly up to a thing, take hold of it, put it somewhere else, and
//! save. What comes out is a small text file the generator reads on the next
//! run, so a layout worked out by hand survives a rebuild and becomes the
//! house's actual arrangement.
//!
//! **The fly is the cursor.** There is no editor camera and no orbit rig: you
//! are still the fly, you still have to fly over to the sofa to move it, and
//! what you are pointing at is whatever is in front of you. It is the control
//! scheme the game already has, and it makes rearranging a room feel like being
//! in the room rather than looking at a plan of it.
//!
//! | | |
//! |---|---|
//! | `Tab` or `F4` | arrange mode on and off |
//! | look | whatever the crosshair is on is highlighted |
//! | left mouse or `G` | take hold of it, and let go of it |
//! | `←` `→` | turn it, twelve degrees a press |
//! | `Ctrl` `S` | save the arrangement |
//! | `Backspace` | put everything back where the generator had it |
//!
//! None of those are the game's. `Q` is the first-person toggle, `R` rolls the
//! camera and `E` cycles the ajar door, so the first pass at this fought the
//! game for three keys out of four — and it had no crosshair, which meant
//! there was no way to tell what you were pointing at in the first place.

use bevy::prelude::*;
use bevy::text::FontSize;

use crate::world::{Home, Part};

/// How far the fly can reach. A room's diagonal: far enough to stand in a
/// doorway and point at the far corner, short enough to not reach into a room
/// you are not in. Five metres was not enough — a look across the great room
/// ran out of ray before it reached anything.
const REACH: f32 = 950.0;
const TURN: f32 = std::f32::consts::FRAC_PI_8 / 1.5;

#[derive(Resource, Default)]
pub struct Arranging {
    pub on: bool,
    /// The piece under the crosshair, if any.
    looking_at: Option<u32>,
    /// The piece being carried, and how far in front of the fly it was taken.
    held: Option<(u32, f32)>,
    /// Where every moved piece started, so it can all be put back.
    moved: std::collections::HashMap<u32, (Vec3, f32)>,
}

#[derive(Component)]
struct Marker;

#[derive(Component)]
struct Crosshair;

#[derive(Component)]
struct Readout;

pub struct ArrangePlugin;

impl Plugin for ArrangePlugin {
    fn build(&self, app: &mut App) {
        // `FLY_ARRANGE=1` opens in arrange mode, which is the only way to get
        // a capture of it — a screenshot cannot press Tab.
        app.insert_resource(Arranging {
            on: std::env::var("FLY_ARRANGE").is_ok(),
            ..default()
        })
        .add_systems(Startup, (spawn_marker, load_arrangement))
        .add_systems(Update, (toggle, aim, carry, save_or_reset, show).chain());
    }
}

fn spawn_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // One translucent box, moved onto whatever is being looked at. Cheaper and
    // clearer than tinting the piece's own materials, which are shared with
    // every other object that happens to look the same.
    commands.spawn((
        Marker,
        Mesh3d(meshes.add(Cuboid::from_length(1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.42, 0.86, 0.78, 0.22),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_scale(Vec3::ZERO),
        Visibility::Hidden,
        bevy::light::NotShadowCaster,
    ));

    // A crosshair. Without one there is no way to know what the ray is on, and
    // "point at the thing" is the whole interface.
    commands.spawn((
        Crosshair,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(7.0),
            height: Val::Px(7.0),
            margin: UiRect::all(Val::Px(-3.5)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.42, 0.90, 0.80, 0.95)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
        Visibility::Hidden,
    ));

    commands.spawn((
        Readout,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgba(0.90, 0.94, 0.92, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Vh(4.0),
            left: Val::Vw(3.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

fn toggle(keys: Res<ButtonInput<KeyCode>>, mut arranging: ResMut<Arranging>) {
    if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::F4) {
        arranging.on = !arranging.on;
        if !arranging.on {
            arranging.held = None;
            arranging.looking_at = None;
        }
    }
}

/// The bounds of a piece, and its middle.
fn bounds(home: &Home, piece: u32) -> Option<(Vec3, Vec3)> {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for solid in home.solids.iter().filter(|s| s.piece == piece) {
        let reach = (solid.rot * solid.half).abs().max(solid.half);
        lo = lo.min(solid.center - reach);
        hi = hi.max(solid.center + reach);
    }
    lo.x.is_finite().then_some((lo, hi))
}

fn aim(
    arranging: ResMut<Arranging>,
    home: Res<Home>,
    eyes: Query<&GlobalTransform, With<Camera3d>>,
    mut markers: Query<(&mut Transform, &mut Visibility), With<Marker>>,
) {
    let mut arranging = arranging;
    let Ok((mut marker, mut seen)) = markers.single_mut() else {
        return;
    };
    if !arranging.on {
        *seen = Visibility::Hidden;
        return;
    }
    let Ok(eye) = eyes.single() else { return };

    // Whatever is in front of the fly. A held piece stays selected — you are
    // carrying it, and looking past it should not put it down.
    if arranging.held.is_none() {
        let from = eye.translation();
        let dir = eye.forward().as_vec3();
        arranging.looking_at = home
            .raycast(from, dir, REACH)
            .map(|hit| home.solids[hit.solid].piece)
            .filter(|p| *p != u32::MAX);
    }

    let showing = arranging.held.map(|(p, _)| p).or(arranging.looking_at);
    match showing.and_then(|p| bounds(&home, p)) {
        Some((lo, hi)) => {
            marker.translation = (lo + hi) * 0.5;
            marker.scale = (hi - lo) + Vec3::splat(1.5);
            marker.rotation = Quat::IDENTITY;
            *seen = Visibility::Inherited;
        }
        None => *seen = Visibility::Hidden,
    }
}

/// Move every box in a piece, and the entity drawn for it.
fn shift(
    home: &mut Home,
    parts: &mut Query<(&Part, &mut Transform), Without<Marker>>,
    piece: u32,
    by: Vec3,
    turn: f32,
) {
    let Some((lo, hi)) = bounds(home, piece) else {
        return;
    };
    let heart = Vec3::new((lo.x + hi.x) * 0.5, 0.0, (lo.z + hi.z) * 0.5);
    let spin = Quat::from_rotation_y(turn);
    for solid in home.solids.iter_mut().filter(|s| s.piece == piece) {
        let local = solid.center - heart;
        solid.center = heart + spin * local + by;
        solid.rot = spin * solid.rot;
    }
    for (part, mut transform) in parts.iter_mut() {
        if home.solids[part.solid].piece == piece {
            transform.translation = home.solids[part.solid].center;
            transform.rotation = home.solids[part.solid].rot;
        }
    }
}

fn carry(
    mut arranging: ResMut<Arranging>,
    mut home: ResMut<Home>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    eyes: Query<&GlobalTransform, With<Camera3d>>,
    mut parts: Query<(&Part, &mut Transform), Without<Marker>>,
) {
    if !arranging.on {
        return;
    }
    let Ok(eye) = eyes.single() else { return };

    let take = keys.just_pressed(KeyCode::KeyG) || mouse.just_pressed(MouseButton::Left);
    if take {
        arranging.held = match arranging.held {
            Some(_) => None,
            None => arranging.looking_at.map(|piece| {
                let held_at = bounds(&home, piece)
                    .map(|(lo, hi)| eye.translation().distance((lo + hi) * 0.5))
                    .unwrap_or(120.0);
                // Remember where it started the first time it is picked up, so
                // Backspace can put the whole room back.
                if let Some((lo, hi)) = bounds(&home, piece) {
                    arranging.moved.entry(piece).or_insert((
                        Vec3::new((lo.x + hi.x) * 0.5, 0.0, (lo.z + hi.z) * 0.5),
                        0.0,
                    ));
                }
                (piece, held_at.clamp(60.0, REACH))
            }),
        };
    }

    let Some((piece, held_at)) = arranging.held else {
        return;
    };

    // The piece rides at a fixed distance in front of the fly, on the floor.
    // Its height is left alone: a lamp that was on a table stays at table
    // height, and the way to put it on the floor is to fly lower.
    let want = eye.translation() + eye.forward().as_vec3() * held_at;
    let Some((lo, hi)) = bounds(&home, piece) else {
        return;
    };
    let heart = Vec3::new((lo.x + hi.x) * 0.5, 0.0, (lo.z + hi.z) * 0.5);
    let mut by = Vec3::new(want.x - heart.x, 0.0, want.z - heart.z);
    // A little damping, or the piece jitters with every twitch of the mouse.
    by *= 0.35;

    let turn = if keys.just_pressed(KeyCode::ArrowLeft) {
        -TURN
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        TURN
    } else {
        0.0
    };

    if by.length_squared() > 0.01 || turn != 0.0 {
        shift(&mut home, &mut parts, piece, by, turn);
        if let Some(record) = arranging.moved.get_mut(&piece) {
            record.1 += turn;
        }
    }
}

fn save_or_reset(
    arranging: Res<Arranging>,
    mut home: ResMut<Home>,
    keys: Res<ButtonInput<KeyCode>>,
    mut parts: Query<(&Part, &mut Transform), Without<Marker>>,
) {
    if !arranging.on {
        return;
    }
    let holding = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::SuperLeft);
    if holding && keys.just_pressed(KeyCode::KeyS) {
        let mut text = String::from("# piece  x  z  yaw — written by arrange mode\n");
        let mut seen = std::collections::HashSet::new();
        for piece in home
            .solids
            .iter()
            .map(|s| s.piece)
            .filter(|p| *p != u32::MAX)
        {
            if !seen.insert(piece) {
                continue;
            }
            if let (Some((lo, hi)), Some(was)) = (bounds(&home, piece), arranging.moved.get(&piece))
            {
                let now = Vec3::new((lo.x + hi.x) * 0.5, 0.0, (lo.z + hi.z) * 0.5);
                let by = now - was.0;
                if by.length() > 0.5 || was.1.abs() > 0.001 {
                    text.push_str(&format!("{piece} {:.2} {:.2} {:.4}\n", by.x, by.z, was.1));
                }
            }
        }
        match std::fs::write(arrangement_path(), &text) {
            Ok(()) => info!("arrangement saved to {}", arrangement_path().display()),
            Err(e) => error!("could not save the arrangement: {e}"),
        }
    }

    if keys.just_pressed(KeyCode::Backspace) {
        let moves: Vec<_> = arranging
            .moved
            .iter()
            .map(|(p, (was, turn))| (*p, *was, *turn))
            .collect();
        for (piece, was, turn) in moves {
            if let Some((lo, hi)) = bounds(&home, piece) {
                let now = Vec3::new((lo.x + hi.x) * 0.5, 0.0, (lo.z + hi.z) * 0.5);
                shift(&mut home, &mut parts, piece, was - now, -turn);
            }
        }
        info!("everything back where the generator had it");
    }
}

fn arrangement_path() -> std::path::PathBuf {
    // Beside the executable, the same place the assets live, so a saved layout
    // travels with the build it was made in.
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("arrangement.txt")))
        .unwrap_or_else(|| "arrangement.txt".into())
}

fn load_arrangement(
    mut home: ResMut<Home>,
    mut parts: Query<(&Part, &mut Transform), Without<Marker>>,
) {
    let Ok(text) = std::fs::read_to_string(arrangement_path()) else {
        return;
    };
    let mut moved = 0;
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let mut bits = line.split_whitespace();
        let (Some(p), Some(x), Some(z), Some(yaw)) =
            (bits.next(), bits.next(), bits.next(), bits.next())
        else {
            continue;
        };
        let (Ok(p), Ok(x), Ok(z), Ok(yaw)) = (
            p.parse::<u32>(),
            x.parse::<f32>(),
            z.parse::<f32>(),
            yaw.parse::<f32>(),
        ) else {
            continue;
        };
        shift(&mut home, &mut parts, p, Vec3::new(x, 0.0, z), yaw);
        moved += 1;
    }
    if moved > 0 {
        info!("arrangement: {moved} pieces moved from where the generator put them");
    }
}

fn show(
    arranging: Res<Arranging>,
    home: Res<Home>,
    mut readouts: Query<(&mut Text, &mut Visibility), With<Readout>>,
    mut crosshairs: Query<&mut Visibility, (With<Crosshair>, Without<Readout>)>,
) {
    for mut seen in &mut crosshairs {
        *seen = if arranging.on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let Ok((mut text, mut seen)) = readouts.single_mut() else {
        return;
    };
    if !arranging.on {
        *seen = Visibility::Hidden;
        return;
    }
    *seen = Visibility::Inherited;
    let what = arranging.held.map(|(p, _)| p).or(arranging.looking_at);
    let size = what
        .and_then(|p| bounds(&home, p).map(|b| (p, b)))
        .map(|(p, (lo, hi))| {
            let s = hi - lo;
            // The id as well as the size: it is what appears in the saved file,
            // and being able to read it off the screen is how you check that a
            // line in that file is the thing you think it is.
            format!("piece {p}   {:.0} x {:.0} x {:.0} cm", s.x, s.y, s.z)
        })
        .unwrap_or_else(|| "nothing in front of you".into());
    text.0 = format!(
        "ARRANGING   -   {}{}\nclick or G take / drop    left right arrows turn    Ctrl+S save    Backspace put it all back    Tab done",
        size,
        if arranging.held.is_some() {
            "   -   carrying"
        } else {
            ""
        }
    );
}
