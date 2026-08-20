//! A turntable for looking at a person.
//!
//! `FLY_STUDIO=<degrees>[:head]` empties the room, lights the body evenly and
//! orbits it. Nothing here is part of the game.
//!
//! It exists because the house kept getting in the way of the work. The
//! in-room viewpoint stands three and a half metres out, and at a good half of
//! the compass that is inside a wall — one capture of the father came back as a
//! flat sheet of magnolia. Even the angles that clear the wall are lit by
//! whatever ceiling fixture happens to be overhead, so the same body reads
//! warm from one side and grey from the other and neither is the body's fault.
//!
//! The people are nearly the whole cast of this game. A body that has to be
//! judged through a doorway with a lamp behind it will be judged badly, and
//! most of what was wrong with the first smooth father — the head two thirds
//! the height it should be, hair that read as sunglasses — was wrong in plain
//! sight and simply could not be seen from in there.

use bevy::prelude::*;

use crate::folk::Person;

/// How the turntable has been asked for.
#[derive(Clone, Copy)]
pub struct Turntable {
    /// Degrees round from dead ahead of the person.
    pub round: f32,
    /// Frame the head rather than the whole body.
    pub head: bool,
    /// Leave the house standing. The camera framing is the useful half of this
    /// tool when the question is about a body *in* a room — where somebody has
    /// walked to, whether their feet are on the floorboards — and the room
    /// viewpoints are no use for that, because at half the compass they stand
    /// inside a wall.
    pub keep: bool,
}

/// `FLY_STUDIO=<degrees>[:head][:keep]`.
pub fn studio() -> Option<Turntable> {
    let raw = std::env::var("FLY_STUDIO").ok()?;
    let mut parts = raw.split(':');
    let round = parts.next()?.trim().parse().ok()?;
    let flags: Vec<&str> = parts.map(str::trim).collect();
    Some(Turntable {
        round,
        head: flags.contains(&"head"),
        keep: flags.contains(&"keep"),
    })
}

pub struct StudioPlugin;

impl Plugin for StudioPlugin {
    fn build(&self, app: &mut App) {
        if studio().is_none() {
            return;
        }
        let asked = studio();
        if asked.is_some_and(|t| t.keep) {
            return;
        }
        app.insert_resource(ClearColor(Color::srgb(0.30, 0.31, 0.33)))
            .add_systems(Startup, light_the_set)
            .add_systems(Update, clear_the_set);
    }
}

/// Three-point lighting, and no shadows from the house because there is no
/// house. Key from the front and one side, fill from the other at a third the
/// strength, and a rim behind to lift the silhouette off the background — a
/// body read against a flat grey with one lamp on it loses its whole outline.
fn light_the_set(mut commands: Commands) {
    for (dir, strength, warmth) in [
        (
            Vec3::new(-0.55, -0.62, -0.56),
            2700.0,
            Color::srgb(1.0, 0.97, 0.92),
        ),
        (
            Vec3::new(0.72, -0.30, -0.62),
            950.0,
            Color::srgb(0.90, 0.94, 1.0),
        ),
        (
            Vec3::new(0.10, -0.28, 0.95),
            1500.0,
            Color::srgb(0.96, 0.96, 1.0),
        ),
    ] {
        commands.spawn((
            StudioLight,
            DirectionalLight {
                illuminance: strength,
                color: warmth,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(Vec3::ZERO).looking_to(dir.normalize(), Vec3::Y),
        ));
    }
}

#[derive(Component)]
struct StudioLight;

/// Hide everything that is not a person.
///
/// Runs every frame rather than once: models and generated furniture arrive
/// over the first several frames, and a set cleared on frame one fills back up
/// by the time a capture is taken two and a half seconds later.
fn clear_the_set(
    folk: Query<Entity, With<Person>>,
    children: Query<&Children>,
    mut drawn: Query<(Entity, &mut Visibility), (With<Mesh3d>, Without<crate::made::Probe>)>,
) {
    let mut keep = bevy::platform::collections::HashSet::new();
    for person in &folk {
        let mut stack = vec![person];
        while let Some(entity) = stack.pop() {
            keep.insert(entity);
            if let Ok(kids) = children.get(entity) {
                stack.extend(kids.iter());
            }
        }
    }
    for (entity, mut visible) in &mut drawn {
        let wanted = if keep.contains(&entity) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visible != wanted {
            *visible = wanted;
        }
    }
}
