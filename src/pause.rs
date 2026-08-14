//! The menu behind `Esc`.
//!
//! Three things and no more: carry on, go back to the title, leave. A pause
//! menu is not a place to put settings — it is the answer to "how do I stop",
//! and every extra line on it is a line between somebody and the door.
//!
//! Pausing genuinely stops the fly. The input systems and the fixed-step
//! simulation are both gated on it, so the wingbeat stops, the sag stops, and
//! a fly left hovering does not quietly sink into the floor while somebody
//! reads the menu.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::title::Stage;

/// Whether the game is stopped behind the menu.
#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub struct Paused(pub bool);

/// A run condition: true while the menu is up.
pub fn paused(paused: Res<Paused>) -> bool {
    paused.0
}

#[derive(Component)]
struct PauseUi;

#[derive(Component, Clone, Copy, PartialEq)]
enum Choice {
    Resume,
    Title,
    Quit,
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Paused>()
            .add_systems(Update, (open_or_close, choose).chain());
    }
}

fn open_or_close(
    mut commands: Commands,
    mut pause: ResMut<Paused>,
    stage: Res<Stage>,
    keys: Res<ButtonInput<KeyCode>>,
    open: Query<Entity, With<PauseUi>>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Only while playing. On the title screen there is nothing to pause, and
    // during the dive it would strand the camera half way across the room.
    if !stage.playing() {
        return;
    }
    // `FLY_PAUSE=1` opens it on the first frame, so the menu can be captured
    // like every other screen in this game — a keypress cannot be.
    let asked = keys.just_pressed(KeyCode::Escape)
        || (std::env::var("FLY_PAUSE").is_ok() && open.is_empty() && !pause.0);
    if !asked {
        return;
    }

    pause.0 = !pause.0;
    if !pause.0 {
        for ui in &open {
            commands.entity(ui).despawn();
        }
        return;
    }

    // The cursor comes back, because a menu you cannot point at is not a menu.
    if let Ok(mut cursor) = cursors.single_mut() {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    commands
        .spawn((
            PauseUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Vh(2.2),
                ..default()
            },
            // Darker than the title's scrim. The title is showing off the
            // room behind it; this is asking a question and wants an answer.
            BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.62)),
            GlobalZIndex(10),
        ))
        .with_children(|screen| {
            screen.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: FontSize::Px(34.0),
                    ..default()
                },
                TextColor(Color::srgba(0.92, 0.90, 0.84, 0.85)),
                Node {
                    margin: UiRect::bottom(Val::Vh(2.0)),
                    ..default()
                },
            ));
            for (choice, label) in [
                (Choice::Resume, "Resume"),
                (Choice::Title, "Title Screen"),
                (Choice::Quit, "Exit Game"),
            ] {
                screen
                    .spawn((
                        choice,
                        Button,
                        Node {
                            width: Val::Px(280.0),
                            padding: UiRect::axes(Val::Vw(2.0), Val::Vh(1.2)),
                            border: UiRect::all(Val::Px(2.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.86, 0.84, 0.78, 0.5)),
                        BackgroundColor(Color::srgba(0.05, 0.05, 0.06, 0.55)),
                    ))
                    .with_child((
                        Text::new(label),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.90, 0.84)),
                    ));
            }
            screen.spawn((
                Text::new("Esc to carry on"),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgba(0.86, 0.84, 0.78, 0.5)),
                Node {
                    margin: UiRect::top(Val::Vh(1.4)),
                    ..default()
                },
            ));
        });
}

fn choose(
    mut commands: Commands,
    mut pause: ResMut<Paused>,
    mut stage: ResMut<Stage>,
    mut leaving: MessageWriter<AppExit>,
    mut buttons: Query<
        (&Interaction, &Choice, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    open: Query<Entity, With<PauseUi>>,
    home: Res<crate::world::Home>,
    mut flies: Query<&mut crate::fly::Fly>,
) {
    for (interaction, choice, mut colour) in &mut buttons {
        match interaction {
            Interaction::Hovered => colour.0 = Color::srgba(0.14, 0.14, 0.16, 0.75),
            Interaction::None => colour.0 = Color::srgba(0.05, 0.05, 0.06, 0.55),
            Interaction::Pressed => {
                match choice {
                    Choice::Resume => {}
                    Choice::Title => {
                        // Back to the title *and* back to the ceiling. "New
                        // Game" that drops you wherever you happened to be
                        // standing is not a new game, and the dive is written
                        // to arrive at the fly's spawn.
                        *stage = Stage::Title;
                        for mut fly in &mut flies {
                            *fly = crate::fly::at_spawn(&home);
                        }
                    }
                    Choice::Quit => {
                        leaving.write(AppExit::Success);
                    }
                }
                pause.0 = false;
                for ui in &open {
                    commands.entity(ui).despawn();
                }
            }
        }
    }
}
