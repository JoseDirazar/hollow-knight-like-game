use crate::animations::{
    AnimationController, AnimationData, CharacterAnimations, CharacterState, CurrentAnimation,
};
use crate::ground::ground_collision;
use crate::physics::Physics;
use crate::player::Player;
use crate::resolution; // Importar el sistema de física
use bevy::prelude::*;
use bevy::sprite::Anchor;
// Componente para el enemigo
#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
    pub attack_range: f32,
    pub detection_range: f32,
    pub facing_right: bool,
    pub is_dead: bool,
    pub death_timer: Timer,
}

// Componente para la hitbox de ataque
#[derive(Component)]
pub struct AttackHitbox {
    pub damage: f32,
    pub active: bool,
    pub size: Vec2,
}

// Recurso para almacenar la posición del jugador
#[derive(Resource, Default)]
struct PlayerPosition {
    position: Vec3,
}

// Agregar este recurso para rastrear enemigos activos
#[derive(Resource)]
pub struct EnemyCounter {
    pub current_count: usize,
    pub desired_count: usize,
}

impl Default for EnemyCounter {
    fn default() -> Self {
        Self {
            current_count: 0,
            desired_count: 2, // Queremos mantener 2 enemigos activos
        }
    }
}

// Plugin principal del enemigo
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerPosition>()
            .init_resource::<EnemyCounter>() // Inicializar el contador de enemigos
            .add_systems(Startup, setup_enemies) // Cambiar a setup_enemies (plural)
            .add_systems(
                Update,
                (
                    update_player_position,
                    update_enemy_movement,
                    update_enemy_animations,
                    handle_damage,
                    check_death,
                    cleanup_dead_enemies,
                    respawn_enemies, // Añadir el nuevo sistema de respawn
                )
                    .after(ground_collision),
            );
    }
}

// Sistema para actualizar la posición del jugador
fn update_player_position(
    player: Query<&Transform, With<Player>>,
    mut player_position: ResMut<PlayerPosition>,
) {
    if let Ok(transform) = player.get_single() {
        // Solo actualiza, no modifica las coordenadas
        player_position.position = transform.translation;
    }
}

// Sistema para actualizar el movimiento del enemigo
// Sistema para actualizar el movimiento del enemigo
fn update_enemy_movement(
    mut enemies: Query<(
        &mut Enemy,
        &mut Transform,
        &mut Physics,
        &mut AnimationController,
    )>,
    player_position: Res<PlayerPosition>,
) {
    for (mut enemy, mut transform, mut physics, mut animation_controller) in &mut enemies {
        if enemy.is_dead {
            continue;
        }

        let distance = player_position.position.x - transform.translation.x;
        let abs_distance = distance.abs();

        // Si el jugador está dentro del rango de detección
        if abs_distance < enemy.detection_range {
            // Determinar la dirección a la que debe mirar el enemigo
            let old_facing = enemy.facing_right;
            enemy.facing_right = distance > 0.0;

            // Solo actualizar la escala si cambió la dirección
            if old_facing != enemy.facing_right {
                // Mantener el valor absoluto de la escala actual y solo cambiar el signo
                let scale_magnitude = transform.scale.x.abs();
                transform.scale.x = if enemy.facing_right {
                    -scale_magnitude
                } else {
                    scale_magnitude
                };
            }

            // Si está dentro del rango de ataque
            if abs_distance < enemy.attack_range {
                // Detener el movimiento y atacar
                physics.velocity.x = 0.0;
                if animation_controller.get_current_state() != CharacterState::Attacking
                    && animation_controller.get_current_state() != CharacterState::Hurt
                {
                    animation_controller.change_state(CharacterState::Attacking);
                }
            } else {
                // Moverse hacia el jugador
                physics.velocity.x = if distance > 0.0
                    && animation_controller.get_current_state() != CharacterState::Attacking
                {
                    enemy.speed
                } else {
                    -enemy.speed
                };
                if animation_controller.get_current_state() == CharacterState::Attacking {
                    physics.velocity.x = 0.0;
                }
                if animation_controller.get_current_state() != CharacterState::Attacking
                    && animation_controller.get_current_state() != CharacterState::Hurt
                {
                    animation_controller.change_state(CharacterState::Running);
                }
            }
        } else {
            // Si el jugador está fuera del rango de detección, quedarse quieto
            physics.velocity.x = 0.0;
            if animation_controller.get_current_state() != CharacterState::Attacking
                && animation_controller.get_current_state() != CharacterState::Hurt
            {
                animation_controller.change_state(CharacterState::Idle);
            }
        }
    }
}

// Sistema para actualizar las animaciones del enemigo

fn update_enemy_animations(
    mut enemies: Query<(&mut AnimationController, &Physics, &Enemy, &mut Transform)>,
) {
    for (mut animation_controller, physics, enemy, mut transform) in &mut enemies {
        let current_state = animation_controller.get_current_state();

        if enemy.is_dead {
            transform.translation.y = transform.translation.y - 5.0;
            continue;
        }

        // No cambiar las animaciones si está atacando o herido
        if current_state == CharacterState::Attacking || current_state == CharacterState::Hurt {
            if current_state == CharacterState::Attacking {
                transform.translation.y = transform.translation.y - 10.0;
            }
            continue;
        }

        // Si está en el aire, usar animación de salto
        if !physics.on_ground {
            animation_controller.change_state(CharacterState::Jumping);
        }
        // Si está en el suelo y la velocidad horizontal es cero, usar idle
        else if physics.velocity.x.abs() < 0.1 {
            if current_state != CharacterState::Idle {
                animation_controller.change_state(CharacterState::Idle);
            }
        }
        // Si está en el suelo y se está moviendo, usar animación de correr
        else if physics.on_ground {
            if current_state != CharacterState::Running {
                animation_controller.change_state(CharacterState::Running);
            }
        }
    }
}

// Sistema para manejar el daño
fn handle_damage(
    mut enemies: Query<(&mut Enemy, &Transform, &mut AnimationController)>,
    hitboxes: Query<(&AttackHitbox, &Transform)>,
) {
    for (mut enemy, enemy_transform, mut animation_controller) in &mut enemies {
        if enemy.is_dead {
            continue;
        }

        for (hitbox, hitbox_transform) in &hitboxes {
            if hitbox.active {
                let distance =
                    (hitbox_transform.translation - enemy_transform.translation).length();
                if distance * 0.3 < hitbox.size.x {
                    // Aplicar daño al enemigo
                    let damage = hitbox.damage - enemy.defense;
                    if damage > 0.0 {
                        enemy.health -= damage;
                        animation_controller.change_state(CharacterState::Hurt);
                    }
                }
            }
        }
    }
}

// Sistema para verificar la muerte
fn check_death(mut query: Query<(&mut Enemy, &mut AnimationController)>) {
    for (mut enemy, mut animation_controller) in &mut query {
        if enemy.health <= 0.0 && !enemy.is_dead {
            enemy.is_dead = true;
            animation_controller.change_state(CharacterState::Dead);
            enemy.death_timer = Timer::from_seconds(10.0, TimerMode::Once);
        }
    }
}

fn respawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
    windows: Query<&Window>,
    mut enemy_counter: ResMut<EnemyCounter>,
) {
    // Si tenemos menos enemigos de los deseados, crear nuevos
    if enemy_counter.current_count < enemy_counter.desired_count {
        let to_spawn = enemy_counter.desired_count - enemy_counter.current_count;

        for _ in 0..to_spawn {
            spawn_enemy(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                &resolution,
                &windows,
            );
            enemy_counter.current_count += 1;
        }
    }
}

// Función helper para crear un enemigo
fn spawn_enemy(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    resolution: &resolution::Resolution,
    windows: &Query<&Window>,
) {
    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();
    let ground_height = -window_height * 0.3;

    // Generar posición aleatoria en los bordes de la pantalla
    let spawn_side = if rand::random::<bool>() { 1.0 } else { -1.0 };
    let spawn_x = spawn_side * (window_width * 0.4); // 40% desde el centro hacia los bordes

    let enemy_y = ground_height + 90.0 * resolution.pixel_ratio;

    let idle_texture = asset_server.load("enemy/skeleton/skeletonIdle-Sheet64x64.png");
    let attack_texture = asset_server.load("enemy/skeleton/skeletonAttack-Sheet146x64.png");
    let move_texture = asset_server.load("enemy/skeleton/skeletonMove-Sheet64x64.png");
    let hurt_texture = asset_server.load("enemy/skeleton/skeletonHurt-Sheet64x64.png");
    let die_texture = asset_server.load("enemy/skeleton/skeletonDie-Sheet118x64_all.png");

    // Crear layouts de atlas
    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 1, None, None);
    let attack_layout =
        TextureAtlasLayout::from_grid(UVec2::new(146, 64), 5, 5, Some(UVec2::new(0, 0)), None);
    let move_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 10, 1, None, None);
    let hurt_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 3, 1, None, None);
    let die_layout = TextureAtlasLayout::from_grid(UVec2::new(118, 64), 5, 5, None, None);

    let idle_atlas_layout = texture_atlas_layouts.add(idle_layout);
    let attack_atlas_layout = texture_atlas_layouts.add(attack_layout);
    let move_atlas_layout = texture_atlas_layouts.add(move_layout);
    let hurt_atlas_layout = texture_atlas_layouts.add(hurt_layout);
    let die_atlas_layout = texture_atlas_layouts.add(die_layout);

    // Crear datos de animación
    let animations = CharacterAnimations {
        animations: vec![
            AnimationData {
                state: CharacterState::Idle,
                texture: idle_texture.clone(),
                atlas_layout: idle_atlas_layout.clone(),
                frames: 8,
                fps: 10.0,
                looping: true,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Attacking,
                texture: attack_texture.clone(),
                atlas_layout: attack_atlas_layout.clone(),
                frames: 23,
                fps: 12.0,
                looping: false,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Running,
                texture: move_texture.clone(),
                atlas_layout: move_atlas_layout.clone(),
                frames: 10,
                fps: 12.0,
                looping: true,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Hurt,
                texture: hurt_texture.clone(),
                atlas_layout: hurt_atlas_layout.clone(),
                frames: 3,
                fps: 10.0,
                looping: false,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Dead,
                texture: die_texture.clone(),
                atlas_layout: die_atlas_layout.clone(),
                frames: 24,
                fps: 10.0,
                looping: false,
                ping_pong: false,
            },
        ],
    };

    // Animación inicial (idle)
    let initial_animation = CurrentAnimation {
        current_frame: 0,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        total_frames: 8,
        looping: true,
        reverse_direction: false,
    };

    // Factor de escala para el enemigo
    let scale_factor = 2.0;
    // Ajuste de la posición Y para evitar que los pies estén bajo el suelo
    let adjusted_y = enemy_y + ((scale_factor - 1.0) * 32.0); // 32 es la mitad de la altura original (64)

    // Crear entidad del enemigo con escala uniforme
    commands.spawn((
        Sprite::from_atlas_image(
            idle_texture,
            TextureAtlas {
                layout: idle_atlas_layout,
                index: 0,
            },
        ),
        Enemy {
            health: 50.0,
            max_health: 50.0,
            attack: 10.0,
            defense: 5.0,
            speed: 150.0,
            attack_range: 73.0,
            detection_range: 500.0,
            facing_right: true,
            is_dead: false,
            death_timer: Timer::from_seconds(10.0, TimerMode::Once),
        },
        Physics {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            on_ground: true,
            gravity_scale: 1.0,
        },
        Transform::from_xyz(spawn_x, adjusted_y, 5.0).with_scale(Vec3::new(
            scale_factor,
            scale_factor,
            1.0,
        )),
        AnimationController::default(),
        animations,
        initial_animation,
    ));
}

// Reemplazar el setup_enemy original con esta función que crea 2 enemigos
fn setup_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
    windows: Query<&Window>,
    mut enemy_counter: ResMut<EnemyCounter>,
) {
    // Generar 2 enemigos iniciales
    for _ in 0..enemy_counter.desired_count {
        spawn_enemy(
            &mut commands,
            &asset_server,
            &mut texture_atlas_layouts,
            &resolution,
            &windows,
        );
        enemy_counter.current_count += 1;
    }
}

// Modificar el sistema de limpieza para actualizar el contador
fn cleanup_dead_enemies(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Enemy)>,
    time: Res<Time>,
    mut enemy_counter: ResMut<EnemyCounter>,
) {
    for (entity, mut enemy) in &mut query {
        if enemy.is_dead {
            enemy.death_timer.tick(time.delta());
            if enemy.death_timer.finished() {
                commands.entity(entity).despawn();
                enemy_counter.current_count -= 1;
            }
        }
    }
}
