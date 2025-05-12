use bevy::prelude::*;

use crate::game::GameState;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

// Component to mark the start button
#[derive(Component)]
struct StartButton;

// Component to mark the menu UI
#[derive(Component)]
struct MenuUI;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(
                Update,
                handle_start_button.run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Main menu root node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                display: Display::Flex,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 1.)),
            MenuUI,
        ))
        .with_children(|parent| {
            // Menu container
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        display: Display::Flex,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                ))
                .with_children(|parent| {
                    // Game title
                    parent.spawn((
                        Text::new("My Awesome Bevy Game"),
                        TextFont {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Start button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(65.0),
                                border: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            BorderColor(Color::BLACK),
                            BorderRadius::MAX,
                            BackgroundColor(NORMAL_BUTTON),
                            StartButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Start Game"),
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

// Remove menu UI when exiting Menu state
fn cleanup_menu(mut commands: Commands, menu_query: Query<Entity, With<MenuUI>>) {
    for menu_entity in menu_query.iter() {
        commands.entity(menu_entity).despawn_recursive();
    }
}

// Handle button interactions to transition to the Playing state
fn handle_start_button(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<StartButton>),
    >,
    mut text_query: Query<&mut Text>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // Check for button press
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "Starting...".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::srgb(1.0, 0.0, 0.0);
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                **text = "Start Game".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Start Game".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }

    // Also allow starting with Enter key
    if keyboard.just_pressed(KeyCode::Backspace) || keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}
