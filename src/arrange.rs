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
//! | `↑` `↓` | raise and lower it — how a mug gets onto a shelf |
//! | wheel | resize it, between half and double |
//! | `Ctrl` `S` or `Cmd` `S` | save the arrangement |
//! | `Backspace` | put everything back where the generator had it |
//!
//! A save goes to `~/.fly-on-the-wall/arrangement.txt` and is read back on the
//! next run, so a layout worked out by hand outlives the build it was made in.
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
    /// How far every moved piece has been shifted, turned and resized from
    /// where the generator put it. The Vec3 carries height as well as plan
    /// position: half the point of arranging is getting something off the floor
    /// and onto a shelf.
    ///
    /// A *total*, accumulated as the piece is dragged, rather than a memory of
    /// where it started. Storing the starting position instead meant a save had
    /// to recover the offset by measuring the piece now and subtracting — and a
    /// measurement is exactly what cannot be trusted here. At load time a
    /// model's hull does not exist yet, so `bounds` reports the four-centimetre
    /// stub sitting at floor level rather than the couch. The subtraction then
    /// produced the couch's own half-height as though somebody had lifted it,
    /// and it compounded on every save: the sofa rose forty-two centimetres,
    /// then eighty-eight, and floated over the rug.
    ///
    /// The file holds offsets. Holding offsets here too means the round trip is
    /// a copy rather than an arithmetic reconstruction.
    moved: std::collections::HashMap<u32, (Vec3, f32, f32)>,
}

/// One box of the ghost. There is a pool of them, and a piece borrows as many
/// as it has boxes.
#[derive(Component)]
struct Marker;

/// How many boxes the biggest piece in the house has. The car is about sixty.
const GHOST_POOL: usize = 96;

/// The last thing worth telling the player, and when it was said.
#[derive(Resource, Default, Deref, DerefMut)]
struct Said(Option<(String, f32)>);

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
        .init_resource::<Said>()
        .add_systems(Startup, spawn_marker)
        // *After* the renderer has spawned its entities, not alongside it.
        // `dress_the_set` spawns through deferred commands, so anything in
        // `Startup` either mutates solids the renderer has already read or
        // updates a query that is still empty — and which of the two happens is
        // a scheduling race. `PostStartup` is after the flush, so the query is
        // real and the furniture actually moves with its solids.
        // After the people are standing. They add their own solids in
        // `PostStartup` too, and a saved arrangement that loads first would be
        // looking for a piece that does not exist yet.
        .add_systems(
            PostStartup,
            (load_arrangement, check_the_round_trip)
                .chain()
                .after(crate::folk::raise_the_father),
        )
        .add_systems(Update, (toggle, aim, carry, save_or_reset, show).chain());
    }
}

fn spawn_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The ghost is the piece's own shape, not a box round it.
    //
    // A single translucent bounding box says where a thing is and nothing about
    // which way it is facing, which is useless precisely when you are turning
    // something. Every solid in this house is a unit cube with a transform, so
    // a faithful ghost is just those transforms again, a shade larger — a pool
    // of cubes, and a piece borrows as many as it has boxes.
    let cube = meshes.add(Cuboid::from_length(1.0));
    let glow = materials.add(StandardMaterial {
        base_color: Color::srgba(0.34, 0.94, 0.80, 0.30),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });
    for _ in 0..GHOST_POOL {
        commands.spawn((
            Marker,
            Mesh3d(cube.clone()),
            MeshMaterial3d(glow.clone()),
            Transform::from_scale(Vec3::ZERO),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
        ));
    }

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
/// How big a piece is, counting its mesh if it has one.
///
/// A made model's solid is a four-centimetre stub carrying an asset path — the
/// real extent is in its hull. Measuring the stub gives a thumbnail ghost and a
/// nonsense size in the readout, and picking a couch up by a box the size of a
/// matchbox is not picking up a couch.
fn bounds(home: &Home, piece: u32) -> Option<(Vec3, Vec3)> {
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for solid in home.solids.iter().filter(|s| s.piece == piece) {
        if solid.model.is_some() {
            // Its stub is a four-centimetre box standing in for a couch, so it
            // contributes its position and not its size. Skipping it outright —
            // which is what this did first — makes `bounds` return nothing at
            // all for a piece that is only a model, and everything downstream
            // gives up: at load time the hull does not exist yet, so a saved
            // couch simply refused to move.
            lo = lo.min(solid.center);
            hi = hi.max(solid.center);
            continue;
        }
        let reach = (solid.rot * solid.half).abs().max(solid.half);
        lo = lo.min(solid.center - reach);
        hi = hi.max(solid.center + reach);
    }
    for hull in &home.hulls {
        if home.solids[hull.solid].piece == piece {
            let (a, b) = hull.bounds();
            lo = lo.min(a);
            hi = hi.max(b);
        }
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
    if !arranging.on {
        for (_, mut seen) in &mut markers {
            *seen = Visibility::Hidden;
        }
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

    // Lay the pool over the piece's own boxes, one for one.
    let showing = arranging.held.map(|(p, _)| p).or(arranging.looking_at);
    let mut ghosts = markers.iter_mut();
    if let Some(piece) = showing {
        for solid in home.solids.iter().filter(|s| s.piece == piece) {
            let Some((mut transform, mut seen)) = ghosts.next() else {
                break;
            };
            transform.translation = solid.center;
            transform.rotation = solid.rot;
            // A shade larger, so it reads as a skin over the piece rather than
            // z-fighting with every face of it.
            transform.scale = solid.half * 2.0 + Vec3::splat(1.2);
            *seen = Visibility::Inherited;
        }
    }
    for (_, mut seen) in ghosts {
        *seen = Visibility::Hidden;
    }
}

/// Move every box in a piece, and the entity drawn for it.
fn shift(
    home: &mut Home,
    parts: &mut Query<(&Part, &mut Transform), Without<Marker>>,
    piece: u32,
    by: Vec3,
    turn: f32,
    factor: f32,
) {
    let Some((lo, hi)) = bounds(home, piece) else {
        return;
    };
    // Turn and resize about the middle of the footprint, at floor level: a
    // chair scaled about its own centre grows down through the floor.
    let heart = Vec3::new((lo.x + hi.x) * 0.5, lo.y, (lo.z + hi.z) * 0.5);
    let spin = Quat::from_rotation_y(turn);
    for solid in home.solids.iter_mut().filter(|s| s.piece == piece) {
        let local = solid.center - heart;
        solid.center = heart + spin * (local * factor) + by;
        solid.rot = spin * solid.rot;
        if solid.model.is_some() {
            solid.scale *= factor;
        } else {
            solid.half *= factor;
        }
    }
    // The mesh has to come too. Its triangles are world-space, so a model that
    // moved without this left its collision behind — you could walk through the
    // couch where it now is and bump into where it used to be.
    for hull in &mut home.hulls {
        if home.solids[hull.solid].piece == piece {
            hull.place(heart, by, spin, factor);
        }
    }
    for (part, mut transform) in parts.iter_mut() {
        let solid = &home.solids[part.solid];
        if solid.piece == piece {
            transform.translation = solid.center;
            transform.rotation = solid.rot;
            transform.scale = match solid.model {
                Some(_) => Vec3::splat(crate::world::UNITS_PER_METRE * solid.scale),
                None => solid.half * 2.0,
            };
        }
    }
}

fn carry(
    mut arranging: ResMut<Arranging>,
    mut home: ResMut<Home>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
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
                arranging
                    .moved
                    .entry(piece)
                    .or_insert((Vec3::ZERO, 0.0, 1.0));
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
    // Height is on its own keys rather than following the fly. Tying it to
    // where you are hovering makes it impossible to slide something along a
    // shelf without also lifting it off.
    let lift = if keys.pressed(KeyCode::ArrowUp) {
        1.0
    } else if keys.pressed(KeyCode::ArrowDown) {
        -1.0
    } else {
        0.0
    };
    by.y = lift * 55.0 * time.delta_secs();

    let turn = if keys.just_pressed(KeyCode::ArrowLeft) {
        -TURN
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        TURN
    } else {
        0.0
    };

    // The wheel resizes. A made model arrives at whatever size its maker gave
    // it, and being able to say "smaller than that" in the room, against the
    // furniture standing next to it, is worth more than any number typed into
    // an exporter.
    let notches: f32 = wheel.read().map(|w| w.y.clamp(-3.0, 3.0)).sum();
    let factor = if notches != 0.0 {
        (1.0 + notches * 0.04).clamp(0.5, 2.0)
    } else {
        1.0
    };

    if by.length_squared() > 0.01 || turn != 0.0 || factor != 1.0 {
        shift(&mut home, &mut parts, piece, by, turn, factor);
        if let Some(record) = arranging.moved.get_mut(&piece) {
            record.0 += by;
            record.1 += turn;
            record.2 *= factor;
        }
    }
}

/// The arrangement file's contents: every piece that stands somewhere other
/// than where the generator put it.
///
/// Pulled out of the save so the round trip can be exercised without a
/// keyboard — `FLY_SAVE=1` writes it on startup, straight after the file has
/// been read, and the two should agree to the last decimal.
fn written(home: &Home, moved: &std::collections::HashMap<u32, (Vec3, f32, f32)>) -> String {
    let mut text = String::from("# piece  x  y  z  yaw  scale — written by arrange mode\n");
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
        let Some(&(by, turn, grew)) = moved.get(&piece) else {
            continue;
        };
        if by.length() > 0.5 || turn.abs() > 0.001 || (grew - 1.0).abs() > 0.001 {
            text.push_str(&format!(
                "{piece} {:.2} {:.2} {:.2} {turn:.4} {grew:.4}\n",
                by.x, by.y, by.z
            ));
        }
    }
    text
}

/// `FLY_SAVE=1` writes the arrangement out on startup, without touching
/// anything. What comes out should be what went in; anything else means a
/// save has quietly lost somebody's afternoon.
fn check_the_round_trip(home: Res<Home>, arranging: Res<Arranging>) {
    if std::env::var("FLY_SAVE").as_deref() != Ok("1") {
        return;
    }
    let text = written(&home, &arranging.moved);
    let was = load_paths()
        .into_iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    if text.trim() == was.trim() {
        info!("arrangement: reads back exactly as written");
    } else {
        warn!("arrangement: would be written differently\n--- was ---\n{was}--- now ---\n{text}");
    }
}

fn save_or_reset(
    mut arranging: ResMut<Arranging>,
    mut home: ResMut<Home>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut parts: Query<(&Part, &mut Transform), Without<Marker>>,
    mut said: ResMut<Said>,
) {
    if !arranging.on {
        return;
    }
    let holding = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::SuperLeft);
    if holding && keys.just_pressed(KeyCode::KeyS) {
        let text = written(&home, &arranging.moved);
        let path = save_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        // Say so on screen. There is no console when the game is started from
        // the launcher, so a save that only logs is a save nobody can tell
        // happened — and "did that work?" is the one question a save has to
        // answer.
        **said = Some(match std::fs::write(&path, &text) {
            Ok(()) => {
                let moved = text.lines().filter(|l| !l.starts_with('#')).count();
                info!("arrangement saved to {}", path.display());
                (
                    format!("saved {moved} pieces to {}", path.display()),
                    time.elapsed_secs(),
                )
            }
            Err(e) => {
                error!("could not save the arrangement: {e}");
                (format!("COULD NOT SAVE: {e}"), time.elapsed_secs())
            }
        });
    }

    if keys.just_pressed(KeyCode::Backspace) {
        **said = Some(("everything put back".into(), time.elapsed_secs()));
        let moves: Vec<_> = arranging
            .moved
            .iter()
            .map(|(p, (was, turn, grew))| (*p, *was, *turn, *grew))
            .collect();
        for (piece, by, turn, grew) in moves {
            shift(&mut home, &mut parts, piece, -by, -turn, 1.0 / grew);
        }
        arranging.moved.clear();
        info!("everything back where the generator had it");
    }
}

/// Where a saved arrangement goes: in the player's own directory, not inside
/// the application.
///
/// Beside the executable was the first answer and it is wrong for anything
/// installed: the launcher replaces the whole bundle on update, so every layout
/// anybody had worked out would go with it, and the bundle may not even be
/// writable. This is the one path that survives a reinstall.
fn save_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home)
        .join(".fly-on-the-wall")
        .join("arrangement.txt")
}

/// Where one is looked for: the player's copy first, then one shipped beside
/// the executable, so a build can carry an authored layout of its own.
fn load_paths() -> Vec<std::path::PathBuf> {
    let mut paths = vec![save_path()];
    if let Some(beside) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("arrangement.txt")))
    {
        paths.push(beside);
    }
    paths
}

/// Put the furniture back where it was left, and remember that it was left
/// there.
///
/// The remembering is the part that was missing. A save is written from the
/// pieces moved *in this session*, so loading a file and then saving without
/// touching anything wrote an empty file and threw the whole arrangement away.
/// Brett moved the sofa, saved, played, opened arrange mode again, pressed the
/// same keys, and got the generator's sofa back.
///
/// Seeding the record with where each piece started before the file was applied
/// makes a save cumulative: the offset written out is always measured from
/// where the generator put the thing, whether it was moved a minute ago or a
/// week ago.
fn load_arrangement(
    mut home: ResMut<Home>,
    mut arranging: ResMut<Arranging>,
    mut parts: Query<(&Part, &mut Transform), Without<Marker>>,
) {
    let Some(text) = load_paths()
        .into_iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
    else {
        return;
    };
    let mut moved = 0;
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let mut bits = line.split_whitespace();
        let (Some(p), Some(x), Some(y), Some(z), Some(yaw)) = (
            bits.next(),
            bits.next(),
            bits.next(),
            bits.next(),
            bits.next(),
        ) else {
            continue;
        };
        let (Ok(p), Ok(x), Ok(y), Ok(z), Ok(yaw)) = (
            p.parse::<u32>(),
            x.parse::<f32>(),
            y.parse::<f32>(),
            z.parse::<f32>(),
            yaw.parse::<f32>(),
        ) else {
            continue;
        };
        // Scale is the newest column, so a file written before it existed
        // still loads — it just does not resize anything.
        let grew = bits
            .next()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0);
        let by = Vec3::new(x, y, z);
        shift(&mut home, &mut parts, p, by, yaw, grew);
        // Copied, not measured. This is the whole reason the record holds a
        // total rather than a starting place.
        arranging.moved.insert(p, (by, yaw, grew));
        moved += 1;
    }
    if moved > 0 {
        info!("arrangement: {moved} pieces moved from where the generator put them");
    }
}

fn show(
    arranging: Res<Arranging>,
    home: Res<Home>,
    time: Res<Time>,
    said: Res<Said>,
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
    // A confirmation outlives the keypress by a few seconds and then goes.
    let note = said
        .0
        .as_ref()
        .filter(|(_, when)| time.elapsed_secs() - *when < 5.0)
        .map(|(what, _)| format!("\n{what}"))
        .unwrap_or_default();
    text.0 = format!(
        "ARRANGING   -   {}{}\n{}{note}",
        size,
        if arranging.held.is_some() {
            "   -   carrying"
        } else {
            ""
        },
        // The keys that do something right now. Turning only works while you
        // are holding something, and a line that says so at all times is a line
        // that gets ignored.
        if arranging.held.is_some() {
            "arrows turn and raise    wheel resizes    click or G to put it down    Ctrl+S save"
        } else {
            "click or G to pick it up    Backspace put it all back    Tab done"
        }
    );
}
