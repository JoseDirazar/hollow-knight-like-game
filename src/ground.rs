use crate::physics::Physics;
use crate::resolution::Resolution;
use bevy::prelude::*;
pub struct GroundPlugin;

const PLAYER_HEIGHT: f32 = 160.0;
const GROUND_HEIGHT: f32 = 19.0;
// Ajustado basado en los logs y las diferencias de posición observadas
pub const PLAYER_FEET_OFFSET: f32 = 25.0;
const GROUND_REPEAT: i32 = 28;

// Valor ajustado para el enemigo
pub const ENEMY_FEET_OFFSET: f32 = 32.0;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ground)
            .add_systems(Update, update_ground_position)
            .add_systems(Update, ground_collision);
    }
}

// Component to identify ground sprites
#[derive(Component, Debug)]
pub struct Ground {
    pub sprite_width: f32,
    pub original_position: Vec3,
    pub position_index: i32,
}
fn setup_ground(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<Resolution>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let window_height = window.height();

    // Cargar la imagen del tileset
    let texture_handle = asset_server.load("world/levels/1/ground/ground-320x19.png");

    // Usar 6x6 grilla con tiles de 160x160 px
    let tile_size = UVec2::new(19, 19);
    let ground_atlas = TextureAtlasLayout::from_grid(tile_size, 19, 1, None, None);
    let ground_atlas_layout = texture_atlas_layouts.add(ground_atlas);

    // Escalado y posicionamiento
    let scale_factor = resolution.pixel_ratio * 1.8;
    let scaled_width = tile_size.x as f32 * scale_factor;
    let ground_height = -window_height * 0.45;

    // Entidad padre
    let ground_parent = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ))
        .id();

    // Tile que queremos renderizar, ej: tile 30
    let tile_index = 3;

    // Crear los bloques de suelo
    commands.entity(ground_parent).with_children(|parent| {
        for i in 0..=GROUND_REPEAT {
            let x_pos = i as f32 * scaled_width;

            parent.spawn((
                Sprite::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas {
                        layout: ground_atlas_layout.clone(),
                        index: tile_index,
                    },
                ),
                Transform::from_xyz(x_pos, ground_height, 10.0).with_scale(Vec3::new(
                    scale_factor,
                    scale_factor,
                    1.0,
                )),
                Ground {
                    sprite_width: scaled_width,
                    original_position: Vec3::new(x_pos, ground_height, 10.0),
                    position_index: i as i32 - 14,
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
                transform.translation.x += ground.sprite_width * GROUND_REPEAT as f32;

                // Update position index
                ground.position_index += 28;

                // Update original position
                ground.original_position.x = transform.translation.x;
            } else if transform.translation.x > camera_x + half_window + (ground.sprite_width / 2.0)
            {
                // This ground piece is off-screen to the right, move it to the left
                // Move it 5 positions to the left
                transform.translation.x -= ground.sprite_width * GROUND_REPEAT as f32;

                // Update position index
                ground.position_index -= GROUND_REPEAT;

                // Update original position
                ground.original_position.x = transform.translation.x;
            }
        }
    }
}

// Implement collision detection for the ground
pub fn ground_collision(
    ground_query: Query<(&Transform, &Ground)>,
    mut characters_query: Query<(Entity, &mut Transform, &mut Physics), Without<Ground>>,
) {
    // Procesar cada entidad (jugador o enemigo) individualmente
    for (entity, mut character_transform, mut physics) in characters_query.iter_mut() {
        physics.on_ground = false;
        let character_scale = character_transform.scale.y.abs();
        
        // Check if this entity is the player based on its Z position
        // Player is at Z=0, enemies are at Z=5
        let is_player = character_transform.translation.z == 0.0;
        // Debug information                
        println!("{:?}", is_player);
        // Use the appropriate feet offset based on entity type
        let feet_offset = if is_player { PLAYER_FEET_OFFSET } else { ENEMY_FEET_OFFSET };
        
        // Calculate the feet position using the appropriate offset
        let character_feet = character_transform.translation.y - feet_offset * character_scale;

        for (ground_transform, ground) in ground_query.iter() {
            let ground_scale = ground_transform.scale.y.abs();
            let ground_top = ground_transform.translation.y + (GROUND_HEIGHT / 2.0) * ground_scale;
            if physics.velocity.y <= 0.0
            && character_feet <= ground_top + 10.0
            && character_feet >= ground_top - 15.0
            && (character_transform.translation.x - ground_transform.translation.x).abs()
            < ground.sprite_width / 2.0
            {
                // Adjust character position based on its feet offset
                character_transform.translation.y = ground_top + feet_offset * character_scale;
                
                physics.velocity.y = 0.0;
                physics.on_ground = true;
                break;
            }
        }
    }
}

// fn setup_ground_old(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
//     resolution: Res<Resolution>,
//     windows: Query<&Window>,
// ) {
//     let window = windows.single();
//     let _window_width = window.width(); // Prefix with underscore since it's currently unused
//     let window_height = window.height();

//     // Load the ground sprite
//     let ground_texture = asset_server.load("world/levels/1/ground/GroundTileset.png");
//     let texture_size = UVec2::splat(180);
//     let ground_atlas = TextureAtlasLayout::from_grid(texture_size, 3, 4, None, None);

//     let ground_atlas_layout = texture_atlas_layouts.add(ground_atlas);
//     // Get the sprite dimensions and calculate scale
//     let sprite_width = 180.0;
//     let _sprite_height = 180.0; // Prefix with underscore since it's currently unused

//     // Calculate scale to fit the sprite properly
//     let scale_factor = resolution.pixel_ratio * 2.0;
//     let scaled_width = sprite_width * scale_factor;

//     // Calculate ground height (20% of the bottom of the screen)
//     let ground_height = -window_height * 2.; // Positioning at 30% from the bottom

//     // Ground parent entity
//     let ground_parent = commands
//         .spawn((
//             Transform::default(),
//             Visibility::default(),
//             InheritedVisibility::default(),
//             ViewVisibility::default(),
//         ))
//         .id();

//     // Spawn 5 ground instances to create a continuous ground
//     commands.entity(ground_parent).with_children(|parent| {
//         for i in -10..=10 {
//             let x_pos = i as f32 * scaled_width;

//             parent.spawn((
//                 Sprite {
//                     image: ground_texture.clone(),
//                     ..default()
//                 },
//                 Transform::from_xyz(x_pos, ground_height, 10.0).with_scale(Vec3::new(
//                     scale_factor,
//                     scale_factor,
//                     1.0,
//                 )),
//                 Ground {
//                     sprite_width: scaled_width,
//                     original_position: Vec3::new(x_pos, ground_height, 10.0),
//                     position_index: i,
//                 },
//                 Visibility::default(),
//                 InheritedVisibility::default(),
//                 ViewVisibility::default(),
//             ));
//         }
//     });
