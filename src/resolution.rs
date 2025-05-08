use bevy::prelude::*;

// Window Constants
pub const WINDOW_TITLE: &str = "Solid Knight";
pub const SCREEN_WIDTH: f32 = 1024.0;
pub const SCREEN_HEIGHT: f32 = 768.0;
pub const SCREEN_DIMENSIONS: Vec2 = Vec2::new(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const PIXEL_RATIO: f32 = 2.0;

// Ground Constants
pub const GROUND_HEIGHT_RATIO: f32 = 0.45; // 30% from bottom of screen

pub struct ResolutionPlugin;

impl Plugin for ResolutionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_resolution);
    }
}

#[derive(Resource)]
pub struct Resolution {
    pub screen_dimensions: Vec2,
    pub pixel_ratio: f32,
}

fn setup_resolution(mut commands: Commands) {
    commands.insert_resource(Resolution {
        screen_dimensions: SCREEN_DIMENSIONS,
        pixel_ratio: PIXEL_RATIO,
    });
}
