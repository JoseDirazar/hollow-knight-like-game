use bevy::prelude::*;

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
    pub height_ratio: f32, // New field for height-based scaling
}

fn setup_resolution(mut commands: Commands, window_query: Query<&Window>) {
    let window = window_query.single();
    let window_height = window.height();

    // Calculate ratios based on design resolution
    let design_height = 768.0; // Example value - adjust to your target design height
    let height_ratio = window_height / design_height;

    commands.insert_resource(Resolution {
        screen_dimensions: Vec2::new(window.width(), window_height),
        pixel_ratio: 2.0, // Base pixel ratio for pixel art
        height_ratio,     // Scale factor to fit window height
    });
}
