use crate::physics::Physics;
use crate::resolution::Resolution;
use bevy::prelude::*;

pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ground)
            .add_systems(Update, update_ground_position)
            .add_systems(Update, ground_collision);
    }
}

// Component to identify ground sprites
#[derive(Component)]
pub struct Ground {
    pub sprite_width: f32,
    pub original_position: Vec3,
    pub position_index: i32,
}

// Setup the ground with 5 instances
fn setup_ground(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    resolution: Res<Resolution>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let _window_width = window.width(); // Prefix with underscore since it's currently unused
    let window_height = window.height();

    // Load the ground sprite
    let ground_texture = asset_server.load("world/levels/1/ground/ground-230x19.png");

    // Get the sprite dimensions and calculate scale
    let sprite_width = 230.0;
    let _sprite_height = 19.0; // Prefix with underscore since it's currently unused

    // Calculate scale to fit the sprite properly
    let scale_factor = resolution.pixel_ratio * 2.0;
    let scaled_width = sprite_width * scale_factor;

    // Calculate ground height (20% of the bottom of the screen)
    let ground_height = -window_height * 0.3; // Positioning at 30% from the bottom

    // Ground parent entity
    let ground_parent = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Spawn 5 ground instances to create a continuous ground
    commands.entity(ground_parent).with_children(|parent| {
        for i in -2..=2 {
            let x_pos = i as f32 * scaled_width;

            parent.spawn((
                Sprite {
                    image: ground_texture.clone(),
                    ..default()
                },
                Transform::from_xyz(x_pos, ground_height, 10.0).with_scale(Vec3::new(
                    scale_factor,
                    scale_factor,
                    1.0,
                )),
                Ground {
                    sprite_width: scaled_width,
                    original_position: Vec3::new(x_pos, ground_height, 10.0),
                    position_index: i,
                },
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ));
        }
    });
}

// Update ground positions when player moves (similar to parallax but with world position)
fn update_ground_position(
    mut ground_query: Query<(&mut Transform, &mut Ground), Without<Camera2d>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();

    if let Ok(camera_transform) = camera_query.get_single() {
        let camera_x = camera_transform.translation.x;

        for (mut transform, mut ground) in ground_query.iter_mut() {
            // The ground stays fixed to world position (no parallax effect)
            // But we need to reposition the sprites to create an infinite ground

            // Check if ground piece is off-screen
            let half_window = window_width / 2.0;

            if transform.translation.x < camera_x - half_window - (ground.sprite_width / 2.0) {
                // This ground piece is off-screen to the left, move it to the right
                // Move it 5 positions to the right
                transform.translation.x += ground.sprite_width * 5.0;

                // Update position index
                ground.position_index += 5;

                // Update original position
                ground.original_position.x = transform.translation.x;
            } else if transform.translation.x > camera_x + half_window + (ground.sprite_width / 2.0)
            {
                // This ground piece is off-screen to the right, move it to the left
                // Move it 5 positions to the left
                transform.translation.x -= ground.sprite_width * 5.0;

                // Update position index
                ground.position_index -= 5;

                // Update original position
                ground.original_position.x = transform.translation.x;
            }
        }
    }
}

// Implement collision detection for the ground
fn ground_collision(
    ground_query: Query<(&Transform, &Ground)>,
    mut player_query: Query<(&mut Transform, &mut Physics), Without<Ground>>,
) {
    // Obtener los datos del jugador
    if let Ok((mut player_transform, mut physics)) = player_query.get_single_mut() {
        // Reset ground state
        physics.on_ground = false;

        // Calculate player feet position (ajustado para la escala del sprite)
        let player_feet = player_transform.translation.y - 80.0 * player_transform.scale.y.abs();

        // Verificar colisión con cada pieza de suelo
        for (ground_transform, ground) in ground_query.iter() {
            // Calculate ground collision area - posición superior del suelo
            let ground_top = ground_transform.translation.y + 9.5 * ground_transform.scale.y.abs();

            // Check if player is standing on ground - condiciones de colisión mejoradas
            if physics.velocity.y <= 0.0 && // Player is falling or stationary
               player_feet <= ground_top + 10.0 && // Player feet at or below ground top (con margen)
               player_feet >= ground_top - 20.0 && // Not too far below ground
               (player_transform.translation.x - ground_transform.translation.x).abs() < ground.sprite_width / 2.0
            // Dentro del ancho del suelo
            {
                // Colisión detectada - colocar al jugador sobre el suelo
                player_transform.translation.y = ground_top + 80.0 * player_transform.scale.y.abs();

                // Detener caída y marcar que está en el suelo
                physics.velocity.y = 0.0;
                physics.on_ground = true;
                break;
            }
        }
    }
}
