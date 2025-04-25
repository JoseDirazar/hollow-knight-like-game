use crate::animations::{
    AnimationController, AnimationData, CharacterAnimations, CharacterState, CurrentAnimation,
};
use crate::physics::Physics;
use crate::player::Player;
use bevy::prelude::*;

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

// Plugin principal del enemigo
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerPosition>()
            .add_systems(Startup, setup_enemy)
            .add_systems(
                Update,
                (
                    update_player_position,
                    update_enemy_movement,
                    update_enemy_animations,
                    handle_damage,
                    check_death,
                    cleanup_dead_enemies,
                )
                    .chain(),
            );
    }
}

// Sistema para actualizar la posición del jugador
fn update_player_position(
    player: Query<&Transform, With<Player>>,
    mut player_position: ResMut<PlayerPosition>,
) {
    if let Ok(transform) = player.get_single() {
        player_position.position = transform.translation;
    }
}

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
            enemy.facing_right = distance < 0.0;
            transform.scale.x = if enemy.facing_right { 1.0 } else { -1.0 };

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
                physics.velocity.x = if distance > 0.0 {
                    enemy.speed
                } else {
                    -enemy.speed
                };
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
fn update_enemy_animations(mut enemies: Query<(&mut AnimationController, &Physics, &Enemy)>) {
    for (mut animation_controller, physics, enemy) in &mut enemies {
        if enemy.is_dead {
            continue;
        }

        let current_state = animation_controller.get_current_state();

        // No cambiar las animaciones si está atacando o herido
        if current_state == CharacterState::Attacking || current_state == CharacterState::Hurt {
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
                if distance < hitbox.size.x {
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
            enemy.death_timer = Timer::from_seconds(1.0, TimerMode::Once);
        }
    }
}

// Sistema para limpiar enemigos muertos
fn cleanup_dead_enemies(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Enemy)>,
    time: Res<Time>,
) {
    for (entity, mut enemy) in &mut query {
        if enemy.is_dead {
            enemy.death_timer.tick(time.delta());
            if enemy.death_timer.finished() {
                commands.entity(entity).despawn();
            }
        }
    }
}

// Configuración inicial del enemigo
fn setup_enemy(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Cargar texturas del esqueleto
    let idle_texture = asset_server.load("enemy/skeleton/skeletonIdle-Sheet64x64.png");
    let attack_texture = asset_server.load("enemy/skeleton/skeletonAttack-Sheet146x64.png");
    let move_texture = asset_server.load("enemy/skeleton/skeletonMove-Sheet64x64.png");
    let hurt_texture = asset_server.load("enemy/skeleton/skeletonHurt-Sheet64x64.png");
    let die_texture = asset_server.load("enemy/skeleton/skeletonDie-Sheet118x64_all.png");

    // Crear layouts de atlas
    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 8, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::new(146, 64), 4, 1, None, None);
    let move_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 10, 1, None, None);
    let hurt_layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 3, 1, None, None);
    let die_layout = TextureAtlasLayout::from_grid(UVec2::new(118, 64), 7, 1, None, None);

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
                frames: 4,
                fps: 15.0,
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
                frames: 7,
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

    // Crear entidad del enemigo
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
            attack_range: 50.0,
            detection_range: 200.0,
            facing_right: true,
            is_dead: false,
            death_timer: Timer::from_seconds(1.0, TimerMode::Once),
        },
        Physics {
            velocity: Vec2::ZERO,
            acceleration: Vec2::new(0.0, -500.0),
            on_ground: false,
            gravity_scale: 1.0,
        },
        Transform::from_xyz(400.0, 0.0, 0.0),
        GlobalTransform::default(),
        AnimationController::default(),
        animations,
        initial_animation,
    ));
}
