use bevy::prelude::*;

use crate::resolution;

// Plugin for the parallax background system
pub struct ParallaxPlugin;

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParallaxSettings>()
            .add_systems(Startup, setup_parallax_background)
            .add_systems(Update, (update_parallax_background, camera_follow_player));
    }
}

// Define the parallax background components
#[derive(Component)]
pub struct ParallaxLayer {
    pub speed_factor: f32,
    pub base_position: f32, // Original position to maintain reference
}

#[derive(Component)]
pub struct ParallaxBackground;

// Resource to store the background state
#[derive(Resource)]
pub struct ParallaxSettings {
    pub camera_move_threshold: f32, // Percentage of screen where camera starts moving
    pub player_move_boundary: f32,  // Boundary distance from edges where player stops moving
}

impl Default for ParallaxSettings {
    fn default() -> Self {
        Self {
            camera_move_threshold: 0.25, // 25% from edge
            player_move_boundary: 0.,    // Calculated in setup
        }
    }
}

// Function to set up the parallax background
fn setup_parallax_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    mut parallax_settings: ResMut<ParallaxSettings>,
    resolution: Res<resolution::Resolution>,
) {
    // Get window dimensions
    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();

    // commands.spawn((
    //     Sprite {
    //         image: asset_server.load("world/levels/1/0.png"),
    //         ..default()
    //     },
    //     Transform::from_xyz(0.0, 0.0, -100.0).with_scale(Vec3::new(
    //         resolution.pixel_ratio,
    //         resolution.pixel_ratio,
    //         1.0,
    //     )),
    // ));

    // Calculate the player move boundary in pixels
    parallax_settings.player_move_boundary = window_width * parallax_settings.camera_move_threshold;

    // Create a parent entity for all parallax layers
    let parallax_parent = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            ParallaxBackground,
        ))
        .id();

    // Layer configuration - define each layer with its image and speed factor
    // Higher speed factor means the layer moves faster (closer to foreground)
    let layers = [
        // Far mountains/moon - moves a bit faster
        ("world/levels/1/1.png", 0.2, -40.0),
        // Mid-distance elements
        ("world/levels/1/2.png", 0.3, -30.0),
        // Closer elements
        ("world/levels/1/3.png", 0.4, -20.0),
        // Foreground elements - moves fastest
        ("world/levels/1/4.png", 0.5, -10.0),
        ("world/levels/1/5.png", 0.7, -5.0),
    ];

    // Spawn each layer
    for (path, speed_factor, z_value) in layers {
        // Load the texture
        let texture = asset_server.load(path);

        // Each layer is a child of the parallax parent
        commands.entity(parallax_parent).with_children(|parent| {
            // Spawn the main instance of this layer
            parent.spawn((
                Sprite {
                    image: texture.clone(),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, z_value).with_scale(Vec3::new(
                    resolution.pixel_ratio,
                    resolution.pixel_ratio,
                    1.0,
                )),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                ParallaxLayer {
                    speed_factor,
                    base_position: 0.0,
                },
            ));

            // Spawn duplicate to the right for seamless scrolling
            parent.spawn((
                Sprite {
                    image: texture.clone(),
                    ..default()
                },
                Transform::from_xyz(window_width, 0.0, z_value).with_scale(Vec3::new(
                    resolution.pixel_ratio,
                    resolution.pixel_ratio,
                    1.0,
                )),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                ParallaxLayer {
                    speed_factor,
                    base_position: window_width,
                },
            ));

            // Spawn duplicate to the left for seamless scrolling
            parent.spawn((
                Sprite {
                    image: texture,
                    ..default()
                },
                Transform::from_xyz(-window_width, 0.0, z_value).with_scale(Vec3::new(
                    resolution.pixel_ratio,
                    resolution.pixel_ratio,
                    1.0,
                )),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                ParallaxLayer {
                    speed_factor,
                    base_position: -window_width,
                },
            ));
        });
    }
}

// System to update the parallax background based on camera movement
fn update_parallax_background(
    mut parallax_query: Query<(&mut Transform, &ParallaxLayer)>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<ParallaxLayer>)>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();

    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_x = camera_transform.translation.x;

        for (mut transform, layer) in parallax_query.iter_mut() {
            // Calculate the relative movement based on speed factor
            // Higher speed factor = moves more with camera (foreground)
            let relative_pos = layer.base_position - (camera_x * layer.speed_factor);

            // Wrap around for seamless scrolling
            let mut wrapped_pos = relative_pos % (window_width * 3.0);
            if wrapped_pos < -window_width {
                wrapped_pos += window_width * 3.0;
            } else if wrapped_pos > window_width * 2.0 {
                wrapped_pos -= window_width * 3.0;
            }

            transform.translation.x = wrapped_pos;
        }
    }
}

// System to make the camera follow the player when they get close to the edge
fn camera_follow_player(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<Camera2d>)>,
    time: Res<Time>,
    parallax_settings: Res<ParallaxSettings>,
    windows: Query<&Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if let (Ok(mut camera_transform), Ok(player_transform)) =
        (camera_query.get_single_mut(), player_query.get_single())
    {
        let window = windows.single();
        let window_width = window.width();
        let half_window = window_width / 2.0;

        // Calculate the threshold positions (25% from each edge)
        let left_threshold =
            camera_transform.translation.x - half_window + parallax_settings.player_move_boundary;
        let right_threshold =
            camera_transform.translation.x + half_window - parallax_settings.player_move_boundary;

        // Camera movement speed based on player's speed
        let camera_speed = 250.0 * time.delta_secs();

        // Check if player is beyond the threshold and move camera accordingly
        if player_transform.translation.x < left_threshold && keyboard.pressed(KeyCode::ArrowLeft) {
            camera_transform.translation.x -= camera_speed;
        } else if player_transform.translation.x > right_threshold
            && keyboard.pressed(KeyCode::ArrowRight)
        {
            camera_transform.translation.x += camera_speed;
        }
    }
}

pub fn extend_world(
    player_position: Vec3,
    current_world_bounds: (f32, f32),
    chunk_width: f32,
) -> Option<Vec3> {
    let (min_x, max_x) = current_world_bounds;

    // If player is getting close to right boundary, extend to the right
    if player_position.x > max_x - chunk_width {
        return Some(Vec3::new(max_x + chunk_width / 2.0, 0.0, 0.0));
    }

    // If player is getting close to left boundary, extend to the left
    if player_position.x < min_x + chunk_width {
        return Some(Vec3::new(min_x - chunk_width / 2.0, 0.0, 0.0));
    }

    None
}

// Monitoring system to track performance and debug issues
#[derive(Default, Resource)]
pub struct ParallaxMonitor {
    pub player_position: Vec3,
    pub camera_position: Vec3,
    pub fps: f32,
    pub frame_time: f32,
    pub active_layers: usize,
    pub visible_sprites: usize,
    pub last_update: f64,
}

// Add this system to your Update schedule
pub fn monitor_performance(
    time: Res<Time>,
    mut monitor: ResMut<ParallaxMonitor>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    parallax_query: Query<&ParallaxLayer>,
    sprite_query: Query<&Visibility>,
) {
    // Update once per second
    if time.elapsed_secs_f64() - monitor.last_update < 1.0 {
        return;
    }

    // Update monitoring data
    if let Ok(player_transform) = player_query.get_single() {
        monitor.player_position = player_transform.translation;
    }

    if let Ok(camera_transform) = camera_query.get_single() {
        monitor.camera_position = camera_transform.translation;
    }

    monitor.active_layers = parallax_query.iter().count();
    monitor.visible_sprites = sprite_query
        .iter()
        .filter(|v| **v == Visibility::Visible)
        .count();
    monitor.fps = 1.0 / time.delta_secs();
    monitor.frame_time = time.delta_secs() * 1000.0; // Convert to milliseconds
    monitor.last_update = time.elapsed_secs_f64();

    // Print debug info if needed
    println!(
        "FPS: {:.2}, Active layers: {}, Player pos: {:.2}, camera_position: {:.2}",
        monitor.fps, monitor.active_layers, monitor.player_position, monitor.camera_position,
    );
}
