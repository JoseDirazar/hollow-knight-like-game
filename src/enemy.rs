use crate::animations::{
    AnimationController, AnimationData, CharacterAnimations, CharacterState, CurrentAnimation,
};
use crate::game::GameState;
use crate::ground::ground_collision;
use crate::physics::Physics;
use crate::player::Player;
use crate::resolution;
use crate::utils;
use bevy::prelude::*;
use bevy::sprite::Anchor;

// Constants
const ENEMY_INITIAL_HEALTH: f32 = 200.0;
const ENEMY_MAX_HEALTH: f32 = 50.0;
const ENEMY_ATTACK: f32 = 10.0;
const ENEMY_DEFENSE: f32 = 5.0;
const ENEMY_SPEED: f32 = 150.0;
const ENEMY_ATTACK_RANGE: f32 = 146.0;
const ENEMY_DETECTION_RANGE: f32 = 400.0;
const ENEMY_COLLISION_SIZE: Vec2 = Vec2::new(32.0, 32.0);
const ENEMY_ATTACK_HITBOX_SIZE: Vec2 = Vec2::new(73.0, 30.0);
const ENEMY_CHARGE_ATTACK_HITBOX_SIZE: Vec2 = Vec2::new(78.0, 30.0);
const ENEMY_ATTACK_HITBOX_DURATION: f32 = 0.1;
const ENEMY_ATTACK_HITBOX_OFFSET: f32 = 0.6;
const ENEMY_DEATH_TIMER: f32 = 3.0;
const ENEMY_HURT_TIMER: f32 = 0.3;
const ENEMY_DESIRED_COUNT: usize = 2;
const ENEMY_SPAWN_OFFSET_X: f32 = 20.0;
const ENEMY_SPAWN_OFFSET_Y: f32 = 90.0;
const ENEMY_SCALE_FACTOR: f32 = 2.0;
const ENEMY_FEET_OFFSET: f32 = 0.5;

// Animation Constants
const ENEMY_IDLE_FRAMES: usize = 8;
const ENEMY_ATTACK_FRAMES: usize = 23;
const ENEMY_MOVE_FRAMES: usize = 10;
const ENEMY_HURT_FRAMES: usize = 3;
const ENEMY_DIE_FRAMES: usize = 24;

const ENEMY_IDLE_FPS: f32 = 14.0;
const ENEMY_ATTACK_FPS: f32 = 14.0;
const ENEMY_MOVE_FPS: f32 = 14.0;
const ENEMY_HURT_FPS: f32 = 10.0;
const ENEMY_DIE_FPS: f32 = 14.0;

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
    pub hurt_timer: Timer,
}

// Componente para la hitbox de ataque
#[derive(Component)]
pub struct AttackHitbox {
    pub damage: f32,
    pub active: bool,
    pub size: Vec2,
    pub timer: Timer,
}

#[derive(Component)]
pub struct CollisionHitbox {
    pub active: bool,
    pub size: Vec2,
}

#[derive(Resource, Default)]
struct PlayerPosition {
    position: Vec3,
}

#[derive(Resource)]
pub struct EnemyCounter {
    pub current_count: usize,
    pub desired_count: usize,
}

impl Default for EnemyCounter {
    fn default() -> Self {
        Self {
            current_count: 0,
            desired_count: ENEMY_DESIRED_COUNT,
        }
    }
}

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
                    update_enemy_states,
                    update_attack_hitbox,
                )
                    .after(ground_collision)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn update_attack_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &AnimationController,
        &Transform,
        &Enemy,
        &CurrentAnimation,
    )>,
    mut hitbox_query: Query<(Entity, &Parent, &mut AttackHitbox), Without<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Primero actualizamos los timers y removemos hitboxes expiradas
    for (hitbox_entity, _parent, mut hitbox) in &mut hitbox_query {
        hitbox.timer.tick(time.delta());

        if hitbox.timer.finished() {
            hitbox.active = false;
            commands.entity(hitbox_entity).despawn_recursive();
        }
    }

    for (entity, animation_controller, _transform, player, current_animation) in &mut query {
        let current_state = animation_controller.get_current_state();

        let is_attacking = matches!(
            current_state,
            CharacterState::Attacking | CharacterState::ChargeAttacking
        );

        // Verificar si ya existe un hitbox activo
        let has_active_hitbox = hitbox_query
            .iter()
            .any(|(_, parent, hitbox)| parent.get() == entity && hitbox.active);

        // Eliminar hitboxes antiguas si ya no está atacando
        if !is_attacking {
            for (hitbox_entity, parent, _) in hitbox_query.iter() {
                if parent.get() == entity {
                    commands.entity(hitbox_entity).despawn_recursive();
                }
            }
            continue;
        }

        // Solo crear nuevo hitbox si no hay uno activo y es el inicio del ataque
        if is_attacking && !has_active_hitbox {
            let should_create_hitbox = match current_animation.current_frame {
                4 => true,  // Primer ataque
                13..16 => true, // Segundo ataque (cargado)
                _ => false,
            };

            if should_create_hitbox {
                let damage = if current_state == CharacterState::Attacking {
                    player.attack
                } else {
                    player.attack * 2.0
                };

                let hitbox_size = if current_state == CharacterState::Attacking {
                    ENEMY_ATTACK_HITBOX_SIZE
                } else {
                    ENEMY_CHARGE_ATTACK_HITBOX_SIZE
                };
                let offset_x = hitbox_size.x * ENEMY_ATTACK_HITBOX_OFFSET;

                // Crear entidad hija para la hitbox
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        AttackHitbox {
                            damage,
                            active: true,
                            size: hitbox_size,
                            timer: Timer::from_seconds(ENEMY_ATTACK_HITBOX_DURATION, TimerMode::Once),
                        },
                        Transform::from_translation(Vec3::new(-offset_x, 0., 0.)), //why is offset negative in order to work? on player it is positive LUL
                        Mesh2d(meshes.add(Rectangle::from_size(hitbox_size))),
                        MeshMaterial2d(materials.add(Color::Srgba(Srgba {
                            red: 200.,
                            green: 200.,
                            blue: 0.,
                            alpha: 0.1,
                        }))),
                    ));
                });
            }
        }
    }
}

fn update_enemy_states(
    time: Res<Time>,
    mut enemies: Query<(&mut Enemy, &mut AnimationController)>,
) {
    for (mut enemy, mut animation_controller) in &mut enemies {
        if animation_controller.get_current_state() == CharacterState::Hurt {
            enemy.hurt_timer.tick(time.delta());

            if enemy.hurt_timer.finished() {
                // Si el enemigo sigue vivo, volver a Idle
                if !enemy.is_dead {
                    animation_controller.change_state(CharacterState::Idle);
                    enemy.hurt_timer.reset();
                }
            }
        }
    }
}

fn update_player_position(
    player: Query<&Transform, With<Player>>,
    mut player_position: ResMut<PlayerPosition>,
) {
    if let Ok(transform) = player.get_single() {
        // Solo actualiza, no modifica las coordenadas
        player_position.position = transform.translation;
    }
}

fn can_enemy_move(state: &CharacterState) -> bool {
    match state {
        CharacterState::Attacking | CharacterState::ChargeAttacking | CharacterState::Hurt => false,
        _ => true,
    }
}

fn update_enemy_movement(
    mut query: Query<(
        Entity,
        &mut Enemy,
        &mut Transform,
        &mut Physics,
        &mut AnimationController,
        &mut CharacterAnimations,
    )>,
    player_position: Res<PlayerPosition>,
) {
    for (
        _entity,
        mut enemy,
        mut transform,
        mut physics,
        mut animation_controller,
        mut _animations,
    ) in &mut query
    {
        if enemy.is_dead || animation_controller.get_current_state() == CharacterState::Dead {
            physics.velocity = Vec2::ZERO;
            continue;
        }

        let enemy_pos = transform.translation.truncate();
        let player_pos = player_position.position.truncate();
        let distance = utils::distance_between_points(enemy_pos, player_pos);
        let current_state = animation_controller.get_current_state();

        // Si el jugador está dentro del rango de detección
        if distance < enemy.detection_range {
            // Determinar la dirección a la que debe mirar el enemigo
            let old_facing = enemy.facing_right;
            enemy.facing_right = player_position.position.x > transform.translation.x;

            // Solo actualizar la escala si cambió la dirección
            if old_facing != enemy.facing_right {
                let scale_magnitude = transform.scale.x.abs();
                transform.scale.x = if enemy.facing_right {
                    -scale_magnitude
                } else {
                    scale_magnitude
                };
            }

            // Si está dentro del rango de ataque
            if distance < enemy.attack_range {
                // Detener el movimiento y atacar
                physics.velocity.x = 0.0;
                if can_enemy_move(&current_state) {
                    animation_controller.change_state(CharacterState::Attacking);
                }
            } else if can_enemy_move(&current_state) {
                // Moverse hacia el jugador solo si puede moverse
                let direction = utils::direction_vector(enemy_pos, player_pos);
                physics.velocity.x = direction.x * enemy.speed;
                animation_controller.change_state(CharacterState::Running);
            } else {
                // Si no puede moverse, detener el movimiento horizontal
                physics.velocity.x = 0.0;
            }
        } else {
            // Si el jugador está fuera del rango de detección, quedarse quieto
            physics.velocity.x = 0.0;
            if can_enemy_move(&current_state) {
                animation_controller.change_state(CharacterState::Idle);
            }
        }
    }
}

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
            // if current_state == CharacterState::Attacking {
            //     //TODO chequear que hacer al respecto del offset de la animacion de ataque, avtualemnte se utiliza el cropped version del ataque para acomodar el sprite pero recorta el sprite de la bola, si esta animacion de ataque y alguna otra puede haber se ejecuta donde no hay suelo se vera que esta recortado
            //     transform.translation.y = transform.translation.y;
            // }
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

fn handle_damage(
    mut enemies: Query<(
        &mut Enemy,
        &mut AnimationController,
        &Children,
        &mut Transform,
        &mut Physics,
    )>,
    enemy_hitboxes: Query<(&CollisionHitbox, &GlobalTransform)>,
    attack_hitboxes: Query<(&AttackHitbox, &GlobalTransform, &Parent)>,
    player_query: Query<Entity, With<Player>>,
) {
    for (mut enemy, mut animation_controller, children, mut _transform, mut physics) in &mut enemies
    {
        if enemy.is_dead {
            continue;
        }

        // Encuentra el hitbox del enemigo
        let mut enemy_hitbox_data = None;
        for &child in children.iter() {
            if let Ok((hitbox, transform)) = enemy_hitboxes.get(child) {
                if hitbox.active {
                    enemy_hitbox_data = Some((hitbox.size, transform.translation().truncate()));
                    break;
                }
            }
        }

        let (enemy_size, enemy_pos) = match enemy_hitbox_data {
            Some(data) => data,
            None => continue,
        };

        // Obtener la entidad del jugador
        if let Ok(player_entity) = player_query.get_single() {
            for (attack_hitbox, attack_transform, parent) in &attack_hitboxes {
                if !attack_hitbox.active || parent.get() != player_entity {
                    continue;
                }

                let attack_pos = attack_transform.translation().truncate();

                // Usar la función de utilidad para verificar la colisión
                if utils::check_rect_collision(
                    enemy_pos,
                    enemy_size,
                    attack_pos,
                    attack_hitbox.size,
                ) {
                    let damage = attack_hitbox.damage - enemy.defense;
                    if damage > 0.0 {
                        enemy.health -= damage;
                        animation_controller.change_state(CharacterState::Hurt);

                        // Aplicar impulso físico constante basado en la dirección del ataque
                        let direction = if attack_pos.x > enemy_pos.x { -1.0 } else { 1.0 };
                        physics.velocity = Vec2::new(direction * 2150.0, direction * 120.0);
                        physics.on_ground = false;
                    }
                    break; // solo un golpe por frame
                }
            }
        }
    }
}

fn check_death(mut query: Query<(&mut Enemy, &mut AnimationController, &mut Transform)>) {
    for (mut enemy, mut animation_controller, mut transform) in &mut query {
        if enemy.health <= 0.0 && !enemy.is_dead {
            enemy.is_dead = true;
            animation_controller.change_state(CharacterState::Dead);
            enemy.death_timer = Timer::from_seconds(ENEMY_DEATH_TIMER, TimerMode::Once);
            transform.translation.x -= ENEMY_SPAWN_OFFSET_X;
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
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
                &mut meshes,
                &mut materials,
            );
            enemy_counter.current_count += 1;
        }
    }
}

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
                commands.entity(entity).despawn_recursive();

                enemy_counter.current_count -= 1;
            }
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    resolution: &resolution::Resolution,
    windows: &Query<&Window>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let window_width = window.width();
    let window_height = window.height();
    let ground_height = -window_height * 0.3;

    let spawn_side = if rand::random::<bool>() { 1.0 } else { -1.0 };
    let spawn_x = spawn_side * (window_width * 0.4);

    let enemy_y = ground_height + ENEMY_SPAWN_OFFSET_Y * resolution.pixel_ratio;

    let idle_texture = asset_server.load("enemy/skeleton/skeletonIdle-Sheet64x64.png");
    let attack_texture = asset_server.load("enemy/skeleton/skeletonAttack-cropped.png");
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
                frames: ENEMY_IDLE_FRAMES,
                fps: ENEMY_IDLE_FPS,
                looping: true,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Attacking,
                texture: attack_texture.clone(),
                atlas_layout: attack_atlas_layout.clone(),
                frames: ENEMY_ATTACK_FRAMES,
                fps: ENEMY_ATTACK_FPS,
                looping: false,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Running,
                texture: move_texture.clone(),
                atlas_layout: move_atlas_layout.clone(),
                frames: ENEMY_MOVE_FRAMES,
                fps: ENEMY_MOVE_FPS,
                looping: true,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Hurt,
                texture: hurt_texture.clone(),
                atlas_layout: hurt_atlas_layout.clone(),
                frames: ENEMY_HURT_FRAMES,
                fps: ENEMY_HURT_FPS,
                looping: false,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Dead,
                texture: die_texture.clone(),
                atlas_layout: die_atlas_layout.clone(),
                frames: ENEMY_DIE_FRAMES,
                fps: ENEMY_DIE_FPS,
                looping: false,
                ping_pong: false,
            },
        ],
    };

    // Animación inicial (idle)
    let initial_animation = CurrentAnimation {
        current_frame: 0,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        total_frames: ENEMY_IDLE_FRAMES,
        looping: true,
        reverse_direction: false,
    };

    // Crear entidad del enemigo con escala uniforme
    commands
        .spawn((
            Sprite::from_atlas_image(
                idle_texture,
                TextureAtlas {
                    layout: idle_atlas_layout,
                    index: 0,
                },
            ),
            Enemy {
                health: ENEMY_INITIAL_HEALTH,
                max_health: ENEMY_MAX_HEALTH,
                attack: ENEMY_ATTACK,
                defense: ENEMY_DEFENSE,
                speed: ENEMY_SPEED,
                attack_range: ENEMY_ATTACK_RANGE,
                detection_range: ENEMY_DETECTION_RANGE,
                facing_right: false,
                is_dead: false,
                death_timer: Timer::from_seconds(ENEMY_DEATH_TIMER, TimerMode::Once),
                hurt_timer: Timer::from_seconds(ENEMY_HURT_TIMER, TimerMode::Once),
            },
            Physics {
                velocity: Vec2::ZERO,
                acceleration: Vec2::ZERO,
                on_ground: true,
                gravity_scale: 1.0,
            },
            Transform::from_xyz(spawn_x, enemy_y, 5.0).with_scale(Vec3::new(
                ENEMY_SCALE_FACTOR,
                ENEMY_SCALE_FACTOR,
                1.0,
            )),
            Anchor::Center,
            AnimationController::default(),
            animations,
            initial_animation,
        ))
        .with_children(|parent| {
            parent.spawn((
                CollisionHitbox {
                    active: true,
                    size: ENEMY_COLLISION_SIZE * ENEMY_SCALE_FACTOR,
                },
                Mesh2d(meshes.add(Rectangle::from_size(ENEMY_COLLISION_SIZE))),
                MeshMaterial2d(materials.add(Color::Srgba(Srgba {
                    red: 0.,
                    green: 0.,
                    blue: 255.,
                    alpha: 0.1,
                }))),
                Transform::from_scale(Vec3::new(ENEMY_SCALE_FACTOR, ENEMY_SCALE_FACTOR, 1.0))
                    .with_translation(Vec3::new(0.0, -ENEMY_FEET_OFFSET * 0.5, 0.0)),
                Anchor::Center,
            ));
        });
}

fn setup_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
    windows: Query<&Window>,
    mut enemy_counter: ResMut<EnemyCounter>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Generar 2 enemigos iniciales
    for _ in 0..enemy_counter.desired_count {
        spawn_enemy(
            &mut commands,
            &asset_server,
            &mut texture_atlas_layouts,
            &resolution,
            &windows,
            &mut meshes,
            &mut materials,
        );
        enemy_counter.current_count += 1;
    }
}
