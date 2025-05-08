use crate::animations::{
    AnimationController, AnimationData, CharacterAnimations, CharacterState, CurrentAnimation,
};
use crate::enemy::{AttackHitbox, CollisionHitbox, Enemy};
use crate::game::GameState;
use crate::physics::Physics;
use crate::resolution;
use crate::utils;

use bevy::prelude::*;
use bevy::sprite::Anchor;

// Constants
const PLAYER_INITIAL_HEALTH: f32 = 100.0;
const PLAYER_MAX_HEALTH: f32 = 100.0;
const PLAYER_ATTACK: f32 = 10.0;
const PLAYER_DEFENSE: f32 = 5.0;
const PLAYER_SPEED: f32 = 250.0;
const PLAYER_JUMP_FORCE: f32 = 500.0;
const PLAYER_HURT_IMMUNITY_TIME: f32 = 0.4;
const PLAYER_COLLISION_SIZE: Vec2 = Vec2::new(45.0, 45.0);
const PLAYER_ATTACK_HITBOX_SIZE: Vec2 = Vec2::new(40.0, 30.0);
const PLAYER_CHARGE_ATTACK_HITBOX_SIZE: Vec2 = Vec2::new(84.0, 30.0);
const PLAYER_ATTACK_HITBOX_DURATION: f32 = 0.1;
const PLAYER_ATTACK_HITBOX_OFFSET: f32 = 0.5;
const PLAYER_FEET_OFFSET: f32 = 10.0;

// Animation Constants
const PLAYER_IDLE_FRAMES: usize = 11;
const PLAYER_ATTACK_FRAMES: usize = 7;
const PLAYER_CHARGE_ATTACK_FRAMES: usize = 7;
const PLAYER_RUN_FRAMES: usize = 8;
const PLAYER_JUMP_FRAMES: usize = 3;
const PLAYER_HURT_FRAMES: usize = 4;
const PLAYER_FALL_FRAMES: usize = 3;

const PLAYER_IDLE_FPS: f32 = 10.0;
const PLAYER_ATTACK_FPS: f32 = 20.0;
const PLAYER_CHARGE_ATTACK_FPS: f32 = 12.0;
const PLAYER_RUN_FPS: f32 = 15.0;
const PLAYER_JUMP_FPS: f32 = 18.0;
const PLAYER_HURT_FPS: f32 = 10.0;
const PLAYER_FALL_FPS: f32 = 10.0;

// Plugin principal del jugador
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player).add_systems(
            Update,
            ((
                process_player_input,
                player_jump.after(process_player_input),
                update_animations,
                update_attack_hitbox,
                handle_damage,
            )
                .run_if(in_state(GameState::Playing)),),
        );
    }
}

// Componente de estadísticas del jugador
#[derive(Component)]
pub struct Player {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
    pub facing_right: bool,
    pub hurt_timer: Timer,
}

fn update_attack_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &AnimationController,
        &Transform,
        &Player,
        &CurrentAnimation,
    )>,
    mut hitbox_query: Query<(Entity, &Parent, &mut AttackHitbox)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    _resolution: Res<resolution::Resolution>,
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

        // Solo crear nuevo hitbox si no hay uno activo y estamos en el rango de tiempo deseado
        if is_attacking && !has_active_hitbox {
            let should_create_hitbox = match current_state {
                CharacterState::Attacking => current_animation.current_frame == 3,
                CharacterState::ChargeAttacking => current_animation.current_frame == 4,
                _ => false,
            };

            if should_create_hitbox {
                let damage = if current_state == CharacterState::Attacking {
                    player.attack
                } else {
                    player.attack * 2.0
                };

                let hitbox_size = if current_state == CharacterState::Attacking {
                    PLAYER_ATTACK_HITBOX_SIZE
                } else {
                    PLAYER_CHARGE_ATTACK_HITBOX_SIZE
                };
                let offset_x = hitbox_size.x * PLAYER_ATTACK_HITBOX_OFFSET;

                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        AttackHitbox {
                            damage,
                            active: true,
                            size: hitbox_size,
                            timer: Timer::from_seconds(
                                PLAYER_ATTACK_HITBOX_DURATION,
                                TimerMode::Once,
                            ),
                        },
                        Transform::from_translation(Vec3::new(offset_x, 0., 0.)),
                        Mesh2d(meshes.add(Rectangle::from_size(hitbox_size))),
                        MeshMaterial2d(materials.add(Color::Srgba(Srgba {
                            red: 0.,
                            green: 255.,
                            blue: 0.,
                            alpha: 0.7,
                        }))),
                    ));
                });
            }
        }
    }
}

fn handle_damage(
    mut player_query: Query<(
        &mut Player,
        &mut AnimationController,
        &Children,
        &mut Transform,
    )>,
    player_hitboxes: Query<(&CollisionHitbox, &GlobalTransform)>,
    enemy_attack_hitboxes: Query<(&AttackHitbox, &GlobalTransform, &Parent)>,
    enemy_query: Query<Entity, With<Enemy>>,
    time: Res<Time>,
) {
    for (mut player, mut animation_controller, children, mut _transform) in &mut player_query {
        // Si el timer de hurt está activo, el jugador es inmune
        player.hurt_timer.tick(time.delta());
        if !player.hurt_timer.finished() {
            continue;
        }

        // Encuentra el hitbox del jugador
        let mut player_hitbox_data = None;
        for &child in children.iter() {
            if let Ok((hitbox, transform)) = player_hitboxes.get(child) {
                if hitbox.active {
                    player_hitbox_data = Some((hitbox.size, transform.translation().truncate()));
                    break;
                }
            }
        }

        let (player_size, player_pos) = match player_hitbox_data {
            Some(data) => data,
            None => continue,
        };

        // Verificar colisión con los hitboxes de ataque de los enemigos
        for (attack_hitbox, attack_transform, parent) in &enemy_attack_hitboxes {
            if !attack_hitbox.active {
                continue;
            }

            // Verificar que el hitbox pertenece a un enemigo
            if !enemy_query.contains(parent.get()) {
                continue;
            }

            let attack_pos = attack_transform.translation().truncate();

            // Usar la función de utilidad para verificar la colisión
            if utils::check_rect_collision(player_pos, player_size, attack_pos, attack_hitbox.size)
            {
                let damage = attack_hitbox.damage - player.defense;
                if damage > 0.0 {
                    player.health -= damage;
                    animation_controller.change_state(CharacterState::Hurt);
                    player.hurt_timer.reset(); // Reiniciar el timer de inmunidad
                }
                break; // evita múltiples daños por frame
            }
        }
    }
}

fn process_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    _time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationController,
            &mut Player,
            &mut Transform,
            &mut Physics,
        ),
        With<Player>,
    >,
) {
    for (mut animation_controller, mut player, mut transform, mut physics) in &mut query {
        let current_state = animation_controller.get_current_state();
        let can_move_now = can_move(&current_state);

        // Ataque con Z en lugar de Espacio
        if keyboard.just_pressed(KeyCode::KeyZ)
            && current_state != CharacterState::Attacking
            && current_state != CharacterState::ChargeAttacking
            && current_state != CharacterState::Jumping
        {
            animation_controller.change_state(CharacterState::Attacking);
        }

        // Ataque cargado con V
        if keyboard.just_pressed(KeyCode::KeyV)
            && current_state != CharacterState::ChargeAttacking
            && current_state != CharacterState::Attacking
            && current_state != CharacterState::Jumping
        {
            animation_controller.change_state(CharacterState::ChargeAttacking);
        }

        // Solo aplicar movimiento horizontal si puede moverse
        if can_move_now {
            // Manejar movimiento a la derecha
            if keyboard.pressed(KeyCode::ArrowRight) {
                player.facing_right = true;
                physics.velocity.x = player.speed;
            }
            // Manejar movimiento a la izquierda
            else if keyboard.pressed(KeyCode::ArrowLeft) {
                player.facing_right = false;
                physics.velocity.x = -player.speed;
            }
            // Si no se presiona ninguna tecla de movimiento, detener el movimiento horizontal
            else {
                physics.velocity.x = 0.0;
            }
        } else {
            // Si no puede moverse (durante ataques), detener el movimiento horizontal
            physics.velocity.x = 0.0;
        }

        // Actualizar la escala para voltear el sprite según la dirección
        let scale_x = transform.scale.x.abs() * if player.facing_right { 1.0 } else { -1.0 };
        transform.scale.x = scale_x;
    }
}

// Modificar el sistema de salto para usar la tecla de espacio
fn player_jump(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Physics, &AnimationController), With<Player>>,
) {
    for (mut physics, animation_controller) in &mut query {
        let current_state = animation_controller.get_current_state();
        let can_jump = can_move(&current_state);

        if keyboard.just_pressed(KeyCode::Space) && physics.on_ground && can_jump {
            physics.velocity.y = PLAYER_JUMP_FORCE;
            physics.on_ground = false;
        }
    }
}

fn can_move(state: &CharacterState) -> bool {
    match state {
        CharacterState::Attacking => false,
        CharacterState::ChargeAttacking => false,
        CharacterState::Hurt => false,
        _ => true,
    }
}

fn update_animations(mut query: Query<(&mut AnimationController, &Physics, &Player)>) {
    for (mut animation_controller, physics, player) in &mut query {
        let current_state = animation_controller.get_current_state();

        // Si está en estado Hurt y el timer ha terminado, volver a Idle
        if current_state == CharacterState::Hurt && player.hurt_timer.finished() {
            animation_controller.change_state(CharacterState::Idle);
            continue;
        }

        // No cambiar las animaciones si está atacando o herido
        if current_state == CharacterState::Attacking
            || current_state == CharacterState::ChargeAttacking
            || current_state == CharacterState::Hurt
        {
            continue;
        }

        // Si está en el aire y la velocidad vertical es negativa, usar animación de caída
        if !physics.on_ground && physics.velocity.y < 0.0 {
            animation_controller.change_state(CharacterState::Falling);
        }
        // Si está en el aire y la velocidad vertical es positiva o cero, usar animación de salto
        else if !physics.on_ground {
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

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
    windows: Query<&Window>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Get window dimensions to position player properly
    let window = windows.single();
    let window_height = window.height();

    // Calcular la posición inicial del jugador
    // Nivel del suelo (30% desde abajo)
    let ground_height = -window_height * 0.3;
    let _player_y = ground_height + 90.0 * resolution.pixel_ratio;

    // Cargar texturas
    let idle_texture = asset_server.load("hero/Idle.png");
    let attack_texture = asset_server.load("hero/Attack1.png");
    let charge_attack_texture = asset_server.load("hero/Attack2.png");
    let run_texture = asset_server.load("hero/Run.png");
    let jump_texture = asset_server.load("hero/Jump.png");
    let hurt_texture = asset_server.load("hero/Hurt.png"); // Agregar textura de hurt
    let fall_texture = asset_server.load("hero/Fall.png");

    // Crear layouts de atlas
    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 11, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);
    let charge_attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);
    let run_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 8, 1, None, None);
    let jump_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 3, 1, None, None);
    let hurt_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 4, 1, None, None); // Layout para hurt
    let fall_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 3, 1, None, None);

    let idle_atlas_layout = texture_atlas_layouts.add(idle_layout);
    let attack_atlas_layout = texture_atlas_layouts.add(attack_layout);
    let charge_attack_attlas_layout = texture_atlas_layouts.add(charge_attack_layout);
    let run_atlas_layout = texture_atlas_layouts.add(run_layout);
    let jump_atlas_layout = texture_atlas_layouts.add(jump_layout);
    let hurt_atlas_layout = texture_atlas_layouts.add(hurt_layout); // Atlas para hurt
    let fall_atlas_layout = texture_atlas_layouts.add(fall_layout);

    // Crear datos de animación
    let animations = CharacterAnimations {
        animations: vec![
            // Animación de idle
            AnimationData {
                state: CharacterState::Idle,
                texture: idle_texture.clone(),
                atlas_layout: idle_atlas_layout.clone(),
                frames: PLAYER_IDLE_FRAMES,
                fps: PLAYER_IDLE_FPS,
                looping: true,
                ping_pong: true,
            },
            // Animación de ataque
            AnimationData {
                state: CharacterState::Attacking,
                texture: attack_texture.clone(),
                atlas_layout: attack_atlas_layout.clone(),
                frames: PLAYER_ATTACK_FRAMES,
                fps: PLAYER_ATTACK_FPS,
                looping: false,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::ChargeAttacking,
                texture: charge_attack_texture.clone(),
                atlas_layout: charge_attack_attlas_layout.clone(),
                frames: PLAYER_CHARGE_ATTACK_FRAMES,
                fps: PLAYER_CHARGE_ATTACK_FPS,
                looping: false,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Running,
                texture: run_texture.clone(),
                atlas_layout: run_atlas_layout.clone(),
                frames: PLAYER_RUN_FRAMES,
                fps: PLAYER_RUN_FPS,
                looping: true,
                ping_pong: false,
            },
            // Animación de salto
            AnimationData {
                state: CharacterState::Jumping,
                texture: jump_texture.clone(),
                atlas_layout: jump_atlas_layout.clone(),
                frames: PLAYER_JUMP_FRAMES,
                fps: PLAYER_JUMP_FPS,
                looping: true,
                ping_pong: false,
            },
            // Animación de hurt
            AnimationData {
                state: CharacterState::Hurt,
                texture: hurt_texture.clone(),
                atlas_layout: hurt_atlas_layout.clone(),
                frames: PLAYER_HURT_FRAMES,
                fps: PLAYER_HURT_FPS,
                looping: false,
                ping_pong: false,
            },
            // Animación de caída
            AnimationData {
                state: CharacterState::Falling,
                texture: fall_texture.clone(),
                atlas_layout: fall_atlas_layout.clone(),
                frames: PLAYER_FALL_FRAMES,
                fps: PLAYER_FALL_FPS,
                looping: true,
                ping_pong: false,
            },
        ],
    };

    // Animación inicial (idle)
    let initial_animation = CurrentAnimation {
        current_frame: 0,
        timer: Timer::from_seconds(0.01, TimerMode::Repeating),
        total_frames: PLAYER_IDLE_FRAMES,
        looping: true,
        reverse_direction: false,
    };

    // Crear entidad del jugador
    commands
        .spawn((
            // Sprite inicial
            Sprite::from_atlas_image(
                idle_texture,
                TextureAtlas {
                    layout: idle_atlas_layout,
                    index: 0,
                },
            ),
            // Estadísticas del jugador
            Player {
                name: "Hero".to_string(),
                health: PLAYER_INITIAL_HEALTH,
                max_health: PLAYER_MAX_HEALTH,
                attack: PLAYER_ATTACK,
                defense: PLAYER_DEFENSE,
                speed: PLAYER_SPEED,
                facing_right: true, // Inicialmente mirando a la derecha
                hurt_timer: Timer::from_seconds(PLAYER_HURT_IMMUNITY_TIME, TimerMode::Once), // Timer para inmunidad
            },
            Physics {
                velocity: Vec2::ZERO,
                acceleration: Vec2::ZERO,
                on_ground: true, // Comienza en el suelo
                gravity_scale: 1.0,
            },
            Transform::from_xyz(0.0, 400., 0.0).with_scale(Vec3::splat(resolution.pixel_ratio)),
            Anchor::Center,
            AnimationController::default(),
            animations,
            initial_animation,
        ))
        .with_children(|parent| {
            parent.spawn((
                CollisionHitbox {
                    active: true,
                    size: PLAYER_COLLISION_SIZE * resolution.pixel_ratio,
                },
                Mesh2d(meshes.add(Rectangle::from_size(PLAYER_COLLISION_SIZE))),
                MeshMaterial2d(materials.add(Color::Srgba(Srgba {
                    red: 255.,
                    green: 0.,
                    blue: 0.,
                    alpha: 0.1,
                }))),
                Transform::from_scale(Vec3::splat(resolution.pixel_ratio))
                    .with_translation(Vec3::new(0.0, -PLAYER_FEET_OFFSET * 0.5, 0.0)),
                Anchor::Center,
            ));
        });
}
