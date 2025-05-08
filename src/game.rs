use bevy::prelude::*;

use crate::animations;
use crate::enemy;
use crate::ground;
use crate::menu;
use crate::paralax_background;
use crate::pause;
use crate::physics;
use crate::player;
use crate::resolution;

// Game state enum to control the flow of the game
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                menu::MenuPlugin,
                resolution::ResolutionPlugin,
                paralax_background::ParallaxPlugin,
                pause::PausePlugin,
            ))
            .add_plugins((
                physics::GravityPlugin,
                animations::AnimationPlugin,
                player::PlayerPlugin,
                ground::GroundPlugin,
                enemy::EnemyPlugin,
            ))
            .add_systems(Startup, setup_camera)
        .add_systems(Update, paralax_background::monitor_performance);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d { ..default() });
}
