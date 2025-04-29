use bevy::prelude::*;
use crate::animations::AnimationPlugin;
use crate::enemy::EnemyPlugin;
use crate::ground::GroundPlugin;
use crate::hitbox::HitboxPlugin;
use crate::paralax_background;
use crate::physics::GravityPlugin;
use crate::player::PlayerPlugin;
use crate::resolution::ResolutionPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Configure window and camera
            .add_systems(Startup, setup_scene)
            .insert_resource(paralax_background::ParallaxMonitor::default())
            
            // Add all plugins in correct order
            .add_plugins((
                ResolutionPlugin,
                GravityPlugin,
                GroundPlugin,
                AnimationPlugin,
                PlayerPlugin,
                EnemyPlugin,
                HitboxPlugin,
                paralax_background::ParallaxPlugin,
            ))
            
            // Add performance monitoring
            .add_systems(Update, paralax_background::monitor_performance);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.spawn(Camera2d { ..default() });
}
