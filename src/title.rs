//! The title screen.
//!
//! There is no separate menu scene and no still image behind it: the house is
//! already loaded, already lit, and already the best thing this build has to
//! look at, so the title screen simply *is* the game, viewed from a corner of
//! the great room with the sign hung over it.
//!
//! Pressing New Game does not cut. It flies the camera down to the fly over a
//! few seconds — the same fly that has been sitting on the ceiling the whole
//! time you were reading the menu — and hands you the controls when it arrives.
//! A cut would tell you the menu and the game are two different places. They
//! are not, and the dive is the cheapest possible way to say so.

use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

/// How long the camera takes to reach the fly. Long enough to read as a move
/// rather than a cut, short enough that nobody sits through it twice.
const DIVE: f32 = 3.4;

#[derive(Resource, Clone, Copy, PartialEq, Debug, Default)]
pub enum Stage {
    #[default]
    Title,
    /// Seconds into the dive.
    Diving(f32),
    Playing,
}

impl Stage {
    pub fn playing(&self) -> bool {
        matches!(self, Stage::Playing)
    }
}

pub fn playing(stage: Res<Stage>) -> bool {
    stage.playing()
}

#[derive(Component)]
struct TitleUi;

#[derive(Component)]
struct StartButton;

/// Part of the sign, and therefore something the dive fades out.
///
/// The fade used to take `Query<&mut TextColor>` unfiltered — every piece of
/// text in the game, including the F3 readout and the arrange HUD. It ends the
/// dive at zero alpha and never puts it back, so after one dive the readout was
/// invisible for the rest of the session and nobody noticed because every
/// capture switch skips the title.
#[derive(Component)]
struct TitleFade;

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        // Every viewpoint switch is a diagnostic looking at the house, not a
        // person starting a game. They skip the menu.
        let posed = crate::camera::room_view().is_some()
            || crate::camera::plan_view()
            || crate::camera::outside_view().is_some()
            || crate::camera::inspect_azimuth().is_some()
            || crate::camera::folk_view().is_some()
            || crate::studio::studio().is_some();
        // `FLY_DIVE=<seconds>` starts part-way through the dive, so the move
        // itself can be captured at a chosen moment instead of guessed at.
        let start = match std::env::var("FLY_DIVE").ok().and_then(|v| v.parse().ok()) {
            Some(t) => Stage::Diving(t),
            None if posed => Stage::Playing,
            None => Stage::Title,
        };
        app.insert_resource(start)
            .add_systems(Startup, raise_the_sign)
            .add_systems(Update, (raise_the_sign, start_the_game, dive).chain());
    }
}

/// Put the sign up, and put it back up.
///
/// Runs every frame rather than once at startup, because the pause menu can
/// send the game back to the title and the sign has to be there when it
/// arrives. The guard is the existing sign: one is enough.
fn raise_the_sign(
    mut commands: Commands,
    assets: Res<AssetServer>,
    stage: Res<Stage>,
    already: Query<(), With<TitleUi>>,
) {
    if !matches!(*stage, Stage::Title) || !already.is_empty() {
        return;
    }
    commands
        .spawn((
            TitleUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Vh(4.0),
                ..default()
            },
            // A scrim, not a curtain. The room behind is the point; this is
            // only here so cream lettering has something to sit against when
            // the sun happens to be on the wall behind it.
            BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.42)),
        ))
        .with_children(|screen| {
            screen.spawn((
                TitleFade,
                ImageNode::new(assets.load("concepts/fly-on-the-wall-logo-transparent.png")),
                Node {
                    width: Val::Vw(52.0),
                    ..default()
                },
            ));
            screen
                .spawn((
                    StartButton,
                    TitleFade,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Vw(2.4), Val::Vh(1.4)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgba(0.86, 0.84, 0.78, 0.5)),
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.06, 0.55)),
                ))
                .with_child((
                    TitleFade,
                    Text::new("New Game"),
                    TextFont {
                        font_size: FontSize::Px(30.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.90, 0.84)),
                ));
            screen.spawn((
                TitleFade,
                // Plain ASCII: the default font carries a mono subset and a
                // middot renders as a tofu box in it.
                Text::new("hold right mouse to land   -   Q first person   -   Esc release mouse"),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::srgba(0.86, 0.84, 0.78, 0.55)),
            ));
        });
}

/// Anything that means "yes": the button, a click, a key.
fn start_the_game(
    mut stage: ResMut<Stage>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut ui: Query<&mut BackgroundColor, With<TitleUi>>,
) {
    if *stage != Stage::Title {
        return;
    }
    let pressed = buttons.iter().any(|i| *i == Interaction::Pressed)
        || mouse.just_pressed(MouseButton::Left)
        || keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::Space);
    if pressed {
        *stage = Stage::Diving(0.0);
        // The scrim goes immediately; the sign fades over the dive.
        for mut colour in &mut ui {
            colour.0 = Color::NONE;
        }
    }
}

fn dive(
    mut commands: Commands,
    mut stage: ResMut<Stage>,
    time: Res<Time>,
    mut ui: Query<(Entity, &mut Node), With<TitleUi>>,
    mut images: Query<&mut ImageNode, With<TitleFade>>,
    mut texts: Query<&mut TextColor, With<TitleFade>>,
    mut borders: Query<&mut BorderColor, With<TitleFade>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Stage::Diving(t) = *stage else {
        return;
    };
    let t = t + time.delta_secs();
    // The sign goes out over the first half, so it is gone well before the
    // camera arrives and the last second of the move is unobstructed.
    let fade = (1.0 - t / (DIVE * 0.5)).clamp(0.0, 1.0);
    for mut image in &mut images {
        image.color = Color::srgba(1.0, 1.0, 1.0, fade);
    }
    for mut text in &mut texts {
        text.0 = text.0.with_alpha(fade * 0.9);
    }
    for mut border in &mut borders {
        *border = BorderColor::all(Color::srgba(0.86, 0.84, 0.78, fade * 0.5));
    }

    if t >= DIVE {
        *stage = Stage::Playing;
        for (entity, _) in &mut ui {
            commands.entity(entity).despawn();
        }
        // Take the mouse the moment control is handed over, so the first
        // movement anybody makes is the fly's and not the pointer's.
        if let Ok(mut cursor) = cursors.single_mut() {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
    } else {
        *stage = Stage::Diving(t);
    }
}

/// How far through the dive, eased. Zero at the title viewpoint, one at the
/// fly.
pub fn dive_progress(stage: &Stage) -> Option<f32> {
    match stage {
        Stage::Title => Some(0.0),
        Stage::Diving(t) => {
            let x = (t / DIVE).clamp(0.0, 1.0);
            // Smootherstep: it has to leave slowly or the first frame reads as
            // a jump cut, and arrive slowly or the handover is a lurch.
            Some(x * x * x * (x * (x * 6.0 - 15.0) + 10.0))
        }
        Stage::Playing => None,
    }
}
