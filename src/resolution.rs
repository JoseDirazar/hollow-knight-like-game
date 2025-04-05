use bevy::prelude::*;

pub struct ResolutionPlugin;

pub const SCREEN_DIMENSIONS: Vec2 = Vec2::new(1024., 768.);

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
        pixel_ratio: 2.0, // Base pixel ratio for pixel art
    });
}
