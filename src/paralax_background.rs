use bevy::prelude::*;

// Plugin for the parallax background system
pub struct ParallaxPlugin;

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParallaxSettings>()
            .add_systems(Startup, setup_parallax_background)
            .configure_sets(
                Update,
                (
                    ParallaxSystems::CameraMovement,
                    ParallaxSystems::BackgroundUpdate.after(ParallaxSystems::CameraMovement),
                ),
            )
            .add_systems(
                Update,
                camera_follow_player.in_set(ParallaxSystems::CameraMovement),
            )
            .add_systems(
                Update,
                (
                    update_parallax_background_optimized,
                    update_static_background,
                )
                    .in_set(ParallaxSystems::BackgroundUpdate),
            );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum ParallaxSystems {
    CameraMovement,
    BackgroundUpdate,
}

// Define the parallax background components
#[derive(Component)]
pub struct ParallaxLayer {
    pub speed_factor: f32,
    pub sprite_width: f32,       // Width of the sprite
    pub original_position: Vec3, // Original spawn position
}

#[derive(Component)]
pub struct ParallaxBackground;

#[derive(Component)]
pub struct StaticBackground;

// Resource to store the background state
#[derive(Resource)]
pub struct ParallaxSettings {
    pub camera_move_threshold: f32,
    pub player_move_boundary: f32,
    pub layer_configurations: Vec<LayerConfig>,
}

// Configuration for each parallax layer
#[derive(Clone)]
pub struct LayerConfig {
    pub path: String,
    pub speed_factor: f32,
    pub z_value: f32,
    pub dimensions: Vec2,
}

impl Default for ParallaxSettings {
    fn default() -> Self {
        Self {
            camera_move_threshold: 0.25,
            player_move_boundary: 0.0,
            layer_configurations: vec![
                LayerConfig {
                    path: "world/levels/1/1.png".to_string(),
                    speed_factor: 0.2,
                    z_value: -40.0,
                    dimensions: Vec2::new(128., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/2.png".to_string(),
                    speed_factor: 0.3,
                    z_value: -30.0,
                    dimensions: Vec2::new(144., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/3.png".to_string(),
                    speed_factor: 0.4,
                    z_value: -20.0,
                    dimensions: Vec2::new(160., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/4.png".to_string(),
                    speed_factor: 0.5,
                    z_value: -10.0,
                    dimensions: Vec2::new(320., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/5.png".to_string(),
                    speed_factor: 0.7,
                    z_value: -5.0,
                    dimensions: Vec2::new(240., 240.),
                },
            ],
        }
    }
}

fn scale_factor(window_width: f32, sprite_dimensions: Vec2) -> f32 {
    window_width / sprite_dimensions.x
}

// Function to set up the parallax background
fn setup_parallax_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<&Window>,
    mut parallax_settings: ResMut<ParallaxSettings>,
) {
    // Get window dimensions
    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();
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

    // Static background
    let static_background_scale_factor = scale_factor(window_width, Vec2::new(320., 240.));
    println!(
        "Static background scale factor: {}",
        static_background_scale_factor
    );
    commands.spawn((
        Sprite {
            image: asset_server.load("world/levels/1/0.png"),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0).with_scale(Vec3::new(
            static_background_scale_factor,
            static_background_scale_factor,
            1.0,
        )),
        StaticBackground,
    ));

    // Spawn each layer with multiple instances for seamless scrolling
    for layer_config in parallax_settings.layer_configurations.iter() {
        // Load the texture
        let texture = asset_server.load(&layer_config.path);
        let parallax_scale_factor = scale_factor(window_width, layer_config.dimensions);

        // Calculate how many instances we need to cover viewport and some extra for scrolling
        // We'll cover 3x the screen width to ensure smooth scrolling in both directions
        let instances_needed = (window_width * 3.0 / layer_config.dimensions.x).ceil() as i32;

        println!(
            "Layer: {}, Scale: {}, Instances: {}",
            layer_config.path, parallax_scale_factor, instances_needed
        );

        commands.entity(parallax_parent).with_children(|parent| {
            // Spawn multiple instances of each layer to cover the screen width and then some
            for i in -instances_needed..=instances_needed {
                let x_pos = i as f32 * layer_config.dimensions.x;

                parent.spawn((
                    Sprite {
                        image: texture.clone(),
                        ..default()
                    },
                    Transform::from_xyz(x_pos, 0.0, layer_config.z_value).with_scale(Vec3::new(
                        parallax_scale_factor,
                        parallax_scale_factor,
                        1.0,
                    )),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    ParallaxLayer {
                        speed_factor: layer_config.speed_factor,
                        sprite_width: layer_config.dimensions.x,
                        original_position: Vec3::new(x_pos, 0.0, layer_config.z_value),
                    },
                ));
            }
        });
    }
}

// System to update the static background position
fn update_static_background(
    mut static_bg_query: Query<&mut Transform, With<StaticBackground>>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<StaticBackground>)>,
) {
    if let (Ok(mut bg_transform), Ok(camera_transform)) =
        (static_bg_query.get_single_mut(), camera_query.get_single())
    {
        bg_transform.translation.x = camera_transform.translation.x;
        bg_transform.translation.y = camera_transform.translation.y;
    }
}

// Optimized system to update parallax layers with sprite recycling
fn update_parallax_background_optimized(
    mut parallax_query: Query<(&mut Transform, &ParallaxLayer)>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<ParallaxLayer>)>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();
    let viewport_width = window_width * 1.5; // Extra width to determine when to recycle sprites

    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_x = camera_transform.translation.x;

        for (mut transform, layer) in parallax_query.iter_mut() {
            // Calculate position based on parallax effect
            let parallax_offset = camera_x * layer.speed_factor;
            let relative_pos = transform.translation.x - parallax_offset;

            // Determine if this sprite is too far to the left or right and needs repositioning
            let distance_from_camera = (transform.translation.x - camera_x).abs();

            if distance_from_camera > viewport_width {
                // Determine which direction to reposition (left or right)
                let direction = if transform.translation.x < camera_x {
                    1.0
                } else {
                    -1.0
                };

                // Calculate number of sprite widths to move (at least 2 to ensure it's offscreen to visible)
                let repositioned_x = camera_x + (direction * viewport_width * 0.8);

                // Update the sprite position
                transform.translation.x = repositioned_x;
            } else {
                // Apply normal parallax movement
                transform.translation.x = layer.original_position.x - parallax_offset;
            }
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

        // Calcular los umbrales (25% desde cada borde)
        let left_threshold =
            camera_transform.translation.x - half_window + parallax_settings.player_move_boundary;
        let right_threshold =
            camera_transform.translation.x + half_window - parallax_settings.player_move_boundary;

        // Velocidad de movimiento de la cámara basada en la velocidad del jugador
        let camera_speed = 250.0 * time.delta_secs();

        // Comprobar si el jugador está más allá del umbral y mover la cámara en consecuencia
        if player_transform.translation.x < left_threshold && keyboard.pressed(KeyCode::ArrowLeft) {
            camera_transform.translation.x -= camera_speed;
        } else if player_transform.translation.x > right_threshold
            && keyboard.pressed(KeyCode::ArrowRight)
        {
            camera_transform.translation.x += camera_speed;
        }

        // Asegurarse de que la cámara se mueva de manera precisa
        camera_transform.translation.z = camera_transform.translation.z.round();
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
