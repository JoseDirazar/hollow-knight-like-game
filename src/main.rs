use bevy::prelude::*;

pub mod animations;
pub mod game;
pub mod paralax_background;
pub mod player;
pub mod resolution;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Solid Knight"),
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        resolution: resolution::SCREEN_DIMENSIONS.into(),
                        mode: bevy::window::WindowMode::Windowed,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            game::GamePlugin,
        ))
        .run();
}
