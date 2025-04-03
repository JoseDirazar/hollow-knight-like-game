use bevy::prelude::*;

use crate::animations;
use crate::player;
use crate::resolution;
use crate::world;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            resolution::ResolutionPlugin,
            animations::AnimationPlugin,
            world::WorldPlugin,
            player::PlayerPlugin,
        ))
        .add_systems(Startup, setup_scene);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d { ..default() });
}
