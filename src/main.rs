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
                        resolution: Vec2::new(1024., 900.).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            game::GamePlugin,
        ))
        .run();
}
