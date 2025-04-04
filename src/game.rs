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

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resolution: Res<resolution::Resolution>,
) {
    commands.spawn((
        Sprite {
            image: asset_server.load("world/levels/1/0.png"),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0).with_scale(Vec3::new(
            resolution.pixel_ratio * 2.,
            resolution.pixel_ratio * 2.,
            -1.,
        )),
    ));
    commands.spawn(Camera2d { ..default() });
}
