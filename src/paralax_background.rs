use bevy::prelude::*;

use crate::game::GameState;

// Plugin for the parallax background system
pub struct ParallaxPlugin;

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParallaxSettings>()
            .add_systems(Startup, setup_parallax_background)
            // .configure_sets(
            //     Update,
            //     (
            //         ParallaxSystems::CameraMovement,
            //         ParallaxSystems::BackgroundUpdate.after(ParallaxSystems::CameraMovement),
            //     ),
            // )
            .add_systems(
                Update,
                (
                    camera_follow_player.in_set(ParallaxSystems::CameraMovement),
                    update_parallax_background_recycled.in_set(ParallaxSystems::BackgroundUpdate),
                    update_static_background.in_set(ParallaxSystems::BackgroundUpdate),
                )
                    .run_if(in_state(GameState::Playing)),
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
    pub position_index: i32,     // -1 = Left, 0 = Center, 1 = Right
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
                    speed_factor: 0.01, // Farthest background (nubes) moves very little (5% of camera movement)
                    z_value: -40.0,
                    dimensions: Vec2::new(128., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/2.png".to_string(),
                    speed_factor: 0.02, // Distant clouds move slightly (10% of camera movement)
                    z_value: -30.0,
                    dimensions: Vec2::new(144., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/3.png".to_string(),
                    speed_factor: 0.04, // Mountains (30% of camera movement)
                    z_value: -20.0,
                    dimensions: Vec2::new(160., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/4.png".to_string(),
                    speed_factor: 0.1, // Forest (50% of camera movement)
                    z_value: -10.0,
                    dimensions: Vec2::new(320., 240.),
                },
                LayerConfig {
                    path: "world/levels/1/5.png".to_string(),
                    speed_factor: 0.20, // Closest to foreground, moves the most (80% of camera movement)
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
    // Calculate the player move boundary in pixels
    parallax_settings.player_move_boundary = window_width * parallax_settings.camera_move_threshold;

    // Create a parent entity for all parallax layers
    let static_background_scale_factor = scale_factor(window_width, Vec2::new(320., 240.));
    let parallax_parent = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            ParallaxBackground,
        ))
        .id();

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

    // Spawn each layer with exactly 3 instances (left, center, right)
    for (layer_index, layer_config) in parallax_settings.layer_configurations.iter().enumerate() {
        // Load the texture
        let texture = asset_server.load(&layer_config.path);
        let _parallax_scale_factor = scale_factor(window_width, layer_config.dimensions);

        // Width of each sprite after scaling
        let scaled_width = layer_config.dimensions.x * static_background_scale_factor;

        commands.entity(parallax_parent).with_children(|parent| {
            // Para las capas 0 y 1 (índices 0 y 1, que corresponden a las nubes lejanas)
            // usamos 5 instancias en lugar de 3 para cubrir mejor la pantalla
            let instance_range = if layer_index == 0 || layer_index == 1 {
                -5..=5 // 5 instancias para nubes (-2, -1, 0, 1, 2)
            } else {
                -1..=1 // 3 instancias para el resto (-1, 0, 1)
            };

            for i in instance_range {
                let x_pos = i as f32 * scaled_width;

                parent.spawn((
                    Sprite {
                        image: texture.clone(),
                        ..default()
                    },
                    ParallaxLayer {
                        speed_factor: layer_config.speed_factor,
                        sprite_width: scaled_width,
                        original_position: Vec3::new(x_pos, 0.0, layer_config.z_value),
                        position_index: i,
                    },
                    Transform::from_xyz(x_pos, 0., layer_config.z_value).with_scale(Vec3::new(
                        static_background_scale_factor,
                        static_background_scale_factor,
                        1.0,
                    )),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
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

// New system that uses exactly 3 sprites per layer and recycles them
fn update_parallax_background_recycled(
    mut parallax_query: Query<(&mut Transform, &mut ParallaxLayer)>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<ParallaxLayer>)>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();

    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_x = camera_transform.translation.x;

        for (mut transform, mut layer) in parallax_query.iter_mut() {
            // Calculate position based on parallax effect
            // Instead of moving the background by the full camera position,
            // we only move it by a fraction determined by the speed_factor
            let parallax_offset = camera_x * (1.0 - layer.speed_factor);

            // Update position to be centered on camera but offset by parallax factor
            transform.translation.x = layer.original_position.x + parallax_offset;

            // Check if this sprite is now off-screen
            let half_window = window_width / 2.0;

            if transform.translation.x < camera_x - half_window - (layer.sprite_width / 2.0) {
                // This sprite is off-screen to the left, move it to the right
                // Determine how many sprite widths to move based on position index range
                let max_index = if layer.position_index >= -1 && layer.position_index <= 1 {
                    1 // Capas normales (-1, 0, 1)
                } else {
                    2 // Capas especiales con 5 instancias (-2, -1, 0, 1, 2)
                };

                // Move to the rightmost position - convertimos a f32 para evitar error de tipo
                let movement = (2 * max_index + 1) as f32;
                transform.translation.x += layer.sprite_width * movement;

                // Update position index
                // Para las capas con rango -2..=2
                if max_index == 2 {
                    if layer.position_index == -2 {
                        layer.position_index = 2;
                    } else if layer.position_index == -1 {
                        layer.position_index = -2;
                    } else if layer.position_index == 0 {
                        layer.position_index = -1;
                    } else if layer.position_index == 1 {
                        layer.position_index = 0;
                    } else if layer.position_index == 2 {
                        layer.position_index = 1;
                    }
                } else {
                    // Para las capas con rango -1..=1
                    if layer.position_index == -1 {
                        layer.position_index = 1;
                    } else if layer.position_index == 0 {
                        layer.position_index = -1;
                    } else if layer.position_index == 1 {
                        layer.position_index = 0;
                    }
                }

                // Update original position
                layer.original_position.x = transform.translation.x - parallax_offset;
            } else if transform.translation.x > camera_x + half_window + (layer.sprite_width / 2.0)
            {
                // This sprite is off-screen to the right, move it to the left
                // Determine how many sprite widths to move based on position index range
                let max_index = if layer.position_index >= -1 && layer.position_index <= 1 {
                    1 // Capas normales (-1, 0, 1)
                } else {
                    2 // Capas especiales con 5 instancias (-2, -1, 0, 1, 2)
                };

                // Move to the leftmost position - convertimos a f32 para evitar error de tipo
                let movement = (2 * max_index + 1) as f32;
                transform.translation.x -= layer.sprite_width * movement;

                // Update position index
                // Para las capas con rango -2..=2
                if max_index == 2 {
                    if layer.position_index == 2 {
                        layer.position_index = -2;
                    } else if layer.position_index == 1 {
                        layer.position_index = 2;
                    } else if layer.position_index == 0 {
                        layer.position_index = 1;
                    } else if layer.position_index == -1 {
                        layer.position_index = 0;
                    } else if layer.position_index == -2 {
                        layer.position_index = -1;
                    }
                } else {
                    // Para las capas con rango -1..=1
                    if layer.position_index == 1 {
                        layer.position_index = -1;
                    } else if layer.position_index == 0 {
                        layer.position_index = 1;
                    } else if layer.position_index == -1 {
                        layer.position_index = 0;
                    }
                }

                // Update original position
                layer.original_position.x = transform.translation.x - parallax_offset;
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
    // println!(
    //     "FPS: {:.2}, Active layers: {}, Player pos: {:.2}, camera_position: {:.2}",
    //     monitor.fps, monitor.active_layers, monitor.player_position, monitor.camera_position,
    // );
}
