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
const ENEMY_SPAWN_OFFSET_X: f32 = 450.0; // Increased for better visibility from camera
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

// Enemy component
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

// Attack hitbox component
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
    pub initial_spawn_done: bool, // Track if initial spawn has been done
}

impl Default for EnemyCounter {
    fn default() -> Self {
        Self {
            current_count: 0,
            desired_count: ENEMY_DESIRED_COUNT,
            initial_spawn_done: false,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerPosition>()
            .init_resource::<EnemyCounter>()
            // Remove the startup system and handle initial spawning in first update
            .add_systems(
                Update,
                (
                    initial_enemy_spawn, // Add a new system for initial spawn
                    update_player_position,
                    update_enemy_movement,
                    update_enemy_animations,
                    handle_damage,
                    check_death,
                    cleanup_dead_enemies,
                    respawn_enemies,
                    update_enemy_states,
                    update_attack_hitbox,
                )
                    .after(ground_collision)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// New system for initial enemy spawn that runs only once when camera is available
fn initial_enemy_spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
    windows: Query<&Window>,
    mut enemy_counter: ResMut<EnemyCounter>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    // Only run this system if we haven't spawned initial enemies yet
    if enemy_counter.initial_spawn_done {
        return;
    }

    // Check if camera is available
    if camera_query.is_empty() {
        return; // No camera yet, try again next frame
    }

    // Camera is available, spawn initial enemies
    for _ in 0..enemy_counter.desired_count {
        spawn_enemy(
            &mut commands,
            &asset_server,
            &camera_query,
            &mut texture_atlas_layouts,
            &resolution,
            &windows,
            &mut meshes,
            &mut materials,
        );
        enemy_counter.current_count += 1;
    }

    // Mark initial spawn as complete
    enemy_counter.initial_spawn_done = true;
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
    // Update timers and remove expired hitboxes
    for (hitbox_entity, _parent, mut hitbox) in &mut hitbox_query {
        hitbox.timer.tick(time.delta());

        if hitbox.timer.finished() {
            hitbox.active = false;
            commands.entity(hitbox_entity).despawn();
        }
    }

    for (entity, animation_controller, _transform, player, current_animation) in &mut query {
        let current_state = animation_controller.get_current_state();

        let is_attacking = matches!(
            current_state,
            CharacterState::Attacking | CharacterState::ChargeAttacking
        );

        // Check if an active hitbox already exists
        let has_active_hitbox = hitbox_query
            .iter()
            .any(|(_, parent, hitbox)| parent.get() == entity && hitbox.active);

        // Remove old hitboxes if no longer attacking
        if !is_attacking {
            for (hitbox_entity, parent, _) in hitbox_query.iter() {
                if parent.get() == entity {
                    commands.entity(hitbox_entity).despawn();
                }
            }
            continue;
        }

        // Only create new hitbox if none active and it's the start of the attack
        if is_attacking && !has_active_hitbox {
            let should_create_hitbox = match current_animation.current_frame {
                4 => true,      // First attack
                13..16 => true, // Second attack (charged)
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

                // Create child entity for hitbox
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        AttackHitbox {
                            damage,
                            active: true,
                            size: hitbox_size,
                            timer: Timer::from_seconds(
                                ENEMY_ATTACK_HITBOX_DURATION,
                                TimerMode::Once,
                            ),
                        },
                        Transform::from_translation(Vec3::new(-offset_x, 0., 0.)),
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
                // If enemy is still alive, return to Idle
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
        // Only update, don't modify coordinates
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

        // If player is within detection range
        if distance < enemy.detection_range {
            // Determine direction enemy should face
            let old_facing = enemy.facing_right;
            enemy.facing_right = player_position.position.x > transform.translation.x;

            // Only update scale if direction changed
            if old_facing != enemy.facing_right {
                let scale_magnitude = transform.scale.x.abs();
                transform.scale.x = if enemy.facing_right {
                    -scale_magnitude
                } else {
                    scale_magnitude
                };
            }

            // If within attack range
            if distance < enemy.attack_range {
                // Stop movement and attack
                physics.velocity.x = 0.0;
                if can_enemy_move(&current_state) {
                    animation_controller.change_state(CharacterState::Attacking);
                }
            } else if can_enemy_move(&current_state) {
                // Move toward player only if able to move
                let direction = utils::direction_vector(enemy_pos, player_pos);
                physics.velocity.x = direction.x * enemy.speed;
                animation_controller.change_state(CharacterState::Running);
            } else {
                // If unable to move, stop horizontal movement
                physics.velocity.x = 0.0;
            }
        } else {
            // If player is outside detection range, stay still
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

        // Don't change animations if attacking or hurt
        if current_state == CharacterState::Attacking || current_state == CharacterState::Hurt {
            continue;
        }

        // If in the air, use jump animation
        if !physics.on_ground {
            animation_controller.change_state(CharacterState::Jumping);
        }
        // If on ground with no horizontal velocity, use idle
        else if physics.velocity.x.abs() < 0.1 {
            if current_state != CharacterState::Idle {
                animation_controller.change_state(CharacterState::Idle);
            }
        }
        // If on ground and moving, use run animation
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

        // Find enemy hitbox
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

        // Get player entity
        if let Ok(player_entity) = player_query.get_single() {
            for (attack_hitbox, attack_transform, parent) in &attack_hitboxes {
                if !attack_hitbox.active || parent.get() != player_entity {
                    continue;
                }

                let attack_pos = attack_transform.translation().truncate();

                // Use utility function to check collision
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

                        // Apply constant physical impulse based on attack direction
                        let direction = if attack_pos.x > enemy_pos.x {
                            -1.0
                        } else {
                            1.0
                        };
                        physics.velocity = Vec2::new(direction * 2150.0, direction * 120.0);
                        physics.on_ground = false;
                    }
                    break; // only one hit per frame
                }
            }
        }
    }
}

fn check_death(
    mut query: Query<(&mut Enemy, &mut AnimationController, &mut Transform)>,
    windows: Query<&Window>,
) {
    let window = if let Ok(window) = windows.get_single() {
        window
    } else {
        return; // Skip this frame if window is not available
    };
    let window_height = window.height();
    let death_threshold = -window_height * 0.5; // Muerte si cae por debajo de la mitad de la pantalla

    for (mut enemy, mut animation_controller, mut transform) in &mut query {
        // Verificar si el enemigo está muerto por salud
        if enemy.health <= 0.0 && !enemy.is_dead {
            enemy.is_dead = true;
            animation_controller.change_state(CharacterState::Dead);
            enemy.death_timer = Timer::from_seconds(ENEMY_DEATH_TIMER, TimerMode::Once);
        }

        // Verificar si el enemigo está fuera de los límites
        if transform.translation.x < -1000.0 || transform.translation.y < death_threshold {
            if !enemy.is_dead {
                enemy.is_dead = true;
                animation_controller.change_state(CharacterState::Dead);
                enemy.death_timer = Timer::from_seconds(ENEMY_DEATH_TIMER, TimerMode::Once);
            }
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
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    // Skip if camera isn't available
    if camera_query.is_empty() {
        return;
    }

    // If we have fewer enemies than desired, create new ones
    if enemy_counter.current_count < enemy_counter.desired_count {
        let to_spawn = enemy_counter.desired_count - enemy_counter.current_count;

        for _ in 0..to_spawn {
            spawn_enemy(
                &mut commands,
                &asset_server,
                &camera_query,
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
    camera_query: &Query<&Transform, With<Camera2d>>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    resolution: &resolution::Resolution,
    windows: &Query<&Window>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let window = windows.single();
    let window_height = window.height();
    let ground_height = -window_height * 0.3;

    // Get camera position safely
    let camera_transform = if let Ok(transform) = camera_query.get_single() {
        transform
    } else {
        // Fallback if camera not found
        return;
    };

    // Randomize spawn side (left or right of camera)
    let spawn_side = if rand::random::<bool>() { 1.0 } else { -1.0 };

    // Calculate spawn position relative to camera
    let spawn_x = camera_transform.translation.x + (ENEMY_SPAWN_OFFSET_X);
    let enemy_y = ground_height + ENEMY_SPAWN_OFFSET_Y * resolution.pixel_ratio;

    let idle_texture = asset_server.load("enemy/skeleton/skeletonIdle-Sheet64x64.png");
    let attack_texture = asset_server.load("enemy/skeleton/skeletonAttack-cropped.png");
    let move_texture = asset_server.load("enemy/skeleton/skeletonMove-Sheet64x64.png");
    let hurt_texture = asset_server.load("enemy/skeleton/skeletonHurt-Sheet64x64.png");
    let die_texture = asset_server.load("enemy/skeleton/skeletonDie-Sheet118x64_all.png");

    // Create atlas layouts
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

    // Create animation data
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

    // Initial animation (idle)
    let initial_animation = CurrentAnimation {
        current_frame: 0,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        total_frames: ENEMY_IDLE_FRAMES,
        looping: true,
        reverse_direction: false,
    };

    // Set facing direction based on spawn side
    let facing_right = spawn_side < 0.0;
    let scale_x = if facing_right {
        -ENEMY_SCALE_FACTOR
    } else {
        ENEMY_SCALE_FACTOR
    };

    // Create enemy entity with uniform scale
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
                facing_right,
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
                scale_x,
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
