use bevy::prelude::*;

use crate::animations;
use crate::paralax_background;
use crate::player;
use crate::resolution;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            resolution::ResolutionPlugin,
            paralax_background::ParallaxPlugin,
            animations::AnimationPlugin,
            player::PlayerPlugin,
        ))
        .add_systems(Startup, setup_scene)
        .insert_resource(paralax_background::ParallaxMonitor::default())
        .add_systems(Update, paralax_background::monitor_performance);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d { ..default() });
}
