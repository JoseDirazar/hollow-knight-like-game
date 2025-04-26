use bevy::prelude::*;

use crate::animations::{
    AnimationController, AnimationData, CharacterAnimations, CharacterState, CurrentAnimation,
};
use crate::enemy::AttackHitbox;
use crate::physics::Physics;
use crate::resolution; // Importar el sistema de física

// Plugin principal del jugador
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, process_player_input)
            .add_systems(Update, player_jump.after(process_player_input))
            .add_systems(Update, update_animations)
            .add_systems(Update, update_attack_hitbox);
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
}

fn can_move(state: &CharacterState) -> bool {
    match state {
        // Lista de estados en los que el personaje NO puede moverse
        CharacterState::Attacking => false,
        CharacterState::ChargeAttacking => false,
        CharacterState::Hurt => false,
        // Agrega cualquier otro estado que deba bloquear el movimiento

        // En cualquier otro estado, el personaje puede moverse
        _ => true,
    }
}

// Sistema separado para actualizar las animaciones según el estado físico
fn update_animations(mut query: Query<(&mut AnimationController, &Physics, &Player)>) {
    for (mut animation_controller, physics, _player) in &mut query {
        let current_state = animation_controller.get_current_state();

        // No cambiar las animaciones si está atacando, atacando con carga o herido
        if current_state == CharacterState::Attacking
            || current_state == CharacterState::ChargeAttacking
            || current_state == CharacterState::Hurt
        {
            continue;
        }

        // Si está en el aire, usar animación de salto
        if !physics.on_ground {
            // Solo cambiar a salto si no viene de un ataque
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
            && current_state != CharacterState::Jumping
        {
            animation_controller.change_state(CharacterState::Attacking);
        }

        // Ataque cargado con V
        if keyboard.just_pressed(KeyCode::KeyV)
            && current_state != CharacterState::ChargeAttacking
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
        let can_jump = can_move(&current_state); // Usar la misma lógica de can_move

        if keyboard.just_pressed(KeyCode::Space) && physics.on_ground && can_jump {
            physics.velocity.y = 500.0; // Fuerza de salto
            physics.on_ground = false;
        }
    }
}

// Configuración inicial del jugador
fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
    windows: Query<&Window>,
) {
    // Get window dimensions to position player properly
    let window = windows.single();
    let window_height = window.height();

    // Calcular la posición inicial del jugador
    // Nivel del suelo (30% desde abajo)
    let ground_height = -window_height * 0.3;
    let player_y = ground_height + 90.0 * resolution.pixel_ratio;

    // Cargar texturas
    let idle_texture = asset_server.load("hero/Idle.png");
    let attack_texture = asset_server.load("hero/Attack1.png");
    let charge_attack_texture = asset_server.load("hero/Attack2.png");
    let run_texture = asset_server.load("hero/Run.png");
    let jump_texture = asset_server.load("hero/Jump.png"); // Nueva textura para salto
    let take_hit_texture: Handle<Image> = asset_server.load("hero/TakeHit.png");
    // Crear layouts de atlas
    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 11, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);
    let charge_attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);
    let run_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 8, 1, None, None);
    let jump_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 3, 1, None, None); // Layout para salto
    let take_hit_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 4, 1, None, None);

    let idle_atlas_layout = texture_atlas_layouts.add(idle_layout);
    let attack_atlas_layout = texture_atlas_layouts.add(attack_layout);
    let charge_attack_attlas_layout = texture_atlas_layouts.add(charge_attack_layout);
    let run_atlas_layout = texture_atlas_layouts.add(run_layout);
    let jump_atlas_layout = texture_atlas_layouts.add(jump_layout); // Atlas para salto
    let take_hit_attlas_layout = texture_atlas_layouts.add(take_hit_layout);

    // Crear datos de animación
    let animations = CharacterAnimations {
        animations: vec![
            // Animación de idle
            AnimationData {
                state: CharacterState::Idle,
                texture: idle_texture.clone(),
                atlas_layout: idle_atlas_layout.clone(),
                frames: 11,
                fps: 10.0,
                looping: true,
                ping_pong: true,
            },
            // Animación de ataque
            AnimationData {
                state: CharacterState::Attacking,
                texture: attack_texture.clone(),
                atlas_layout: attack_atlas_layout.clone(),
                frames: 7,
                fps: 20.0,      // Un poco más rápido que idle
                looping: false, // La animación de ataque no se repite
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::ChargeAttacking,
                texture: charge_attack_texture.clone(),
                atlas_layout: charge_attack_attlas_layout.clone(),
                frames: 7,
                fps: 12.0,
                looping: false,
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Running,
                texture: run_texture.clone(),
                atlas_layout: run_atlas_layout.clone(),
                frames: 8,
                fps: 15.0,
                looping: true,
                ping_pong: false,
            },
            // Animación de salto
            AnimationData {
                state: CharacterState::Jumping,
                texture: jump_texture.clone(),
                atlas_layout: jump_atlas_layout.clone(),
                frames: 3,
                fps: 12.0,     // Un poco más lento que correr
                looping: true, // Loop para mantener la animación mientras está en el aire
                ping_pong: false,
            },
            AnimationData {
                state: CharacterState::Hurt,
                texture: take_hit_texture.clone(),
                atlas_layout: take_hit_attlas_layout.clone(),
                frames: 4,
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
        total_frames: 11,
        looping: true,
        reverse_direction: false,
    };

    // Crear entidad del jugador
    commands.spawn((
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
            health: 100.0,
            max_health: 100.0,
            attack: 10.0,
            defense: 5.0,
            speed: 250.0,
            facing_right: true, // Inicialmente mirando a la derecha
        },
        // Componente de física para gravedad
        Physics {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            on_ground: true, // Comienza en el suelo
            gravity_scale: 1.0,
        },
        // Transformación - Posicionar jugador sobre el nivel del suelo
        Transform::from_xyz(0.0, player_y, 0.0).with_scale(Vec3::splat(resolution.pixel_ratio)),
        // Componentes de animación
        AnimationController::default(),
        animations,
        initial_animation,
    ));
}

fn update_attack_hitbox(
    mut commands: Commands,
    mut query: Query<(Entity, &AnimationController, &Transform, &Player)>,
) {
    for (entity, animation_controller, transform, player) in &mut query {
        let current_state = animation_controller.get_current_state();

        // Si está atacando, crear o actualizar la hitbox
        if current_state == CharacterState::Attacking
            || current_state == CharacterState::ChargeAttacking
        {
            let damage = if current_state == CharacterState::Attacking {
                player.attack
            } else {
                player.attack * 2.0 // Ataque cargado hace más daño
            };

            // Crear o actualizar la hitbox de ataque
            commands.entity(entity).insert(AttackHitbox {
                damage,
                active: true,
                size: Vec2::new(50.0, 30.0), // Tamaño de la hitbox
            });
        } else {
            // Si no está atacando, remover la hitbox
            commands.entity(entity).remove::<AttackHitbox>();
        }
    }
}
