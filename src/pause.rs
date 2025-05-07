use crate::game::GameState;
use bevy::prelude::*;

// Component to mark pause menu elements
#[derive(Component)]
struct PauseMenu;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(
                Update,
                (
                    handle_resume_button.run_if(in_state(GameState::Paused)),
                    handle_pause_input.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(OnExit(GameState::Paused), cleanup_pause_menu);
    }
}

fn setup_pause_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            PauseMenu,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                ))
                .with_children(|parent| {
                    // Pause title
                    parent.spawn((
                        Text::new("PAUSED"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Resume button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(65.0),
                                border: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BorderRadius::MAX,
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Resume"),
                                TextFont {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

fn cleanup_pause_menu(mut commands: Commands, pause_menu_query: Query<Entity, With<PauseMenu>>) {
    for entity in pause_menu_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_resume_button(
    mut next_state: ResMut<NextState<GameState>>,
    interaction_query: Query<&Interaction, Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Check for button press
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }

    // Also allow resuming with Escape or P key
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyP) {
        next_state.set(GameState::Playing);
    }
}

fn handle_pause_input(
    mut next_state: ResMut<NextState<GameState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::KeyP) {
        next_state.set(GameState::Paused);
    }
}
