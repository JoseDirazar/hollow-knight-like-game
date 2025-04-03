use bevy::prelude::*;

use crate::animations::{
    AnimationController, AnimationData, CharacterAnimations, CharacterState, CurrentAnimation,
};
use crate::resolution;

// Plugin principal del jugador
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, process_player_input);
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

fn process_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut AnimationController, &mut Player, &mut Transform), With<Player>>,
) {
    for (mut animation_controller, mut player, mut transform) in &mut query {
        if keyboard.just_pressed(KeyCode::Space)
            && animation_controller.get_current_state() != CharacterState::Attacking
        {
            animation_controller.change_state(CharacterState::Attacking);
        }

        if keyboard.just_pressed(KeyCode::KeyV)
            && animation_controller.get_current_state() != CharacterState::ChargeAttacking
        {
            animation_controller.change_state(CharacterState::ChargeAttacking);
        }

        // Manejar dirección y movimiento
        let mut is_running = false;

        // Manejar movimiento a la derecha
        if keyboard.pressed(KeyCode::ArrowRight) {
            animation_controller.change_state(CharacterState::Running);
            player.facing_right = true;
            is_running = true;
            // Aplicar movimiento a la derecha
            transform.translation.x += player.speed * time.delta_secs();
            println!("{}", transform.translation.x);
        }

        // Manejar movimiento a la izquierda
        if keyboard.pressed(KeyCode::ArrowLeft) {
            animation_controller.change_state(CharacterState::Running);
            player.facing_right = false;
            is_running = true;
            // Aplicar movimiento a la izquierda
            transform.translation.x -= player.speed * time.delta_secs();
            println!("{}", transform.translation.x);
        }

        // Actualizar la escala para voltear el sprite según la dirección
        let scale_x = transform.scale.x.abs() * if player.facing_right { 1.0 } else { -1.0 };
        transform.scale.x = scale_x;

        // Volver a estado idle si no está corriendo
        if !is_running && animation_controller.get_current_state() == CharacterState::Running {
            animation_controller.change_state(CharacterState::Idle);
        }
    }
}
// Configuración inicial del jugador
fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    _resolution: Res<resolution::Resolution>,
) {
    // Cargar texturas
    let idle_texture = asset_server.load("hero/Idle.png");
    let attack_texture = asset_server.load("hero/Attack1.png");
    let charge_attack_texture = asset_server.load("hero/Attack2.png");
    let run_texture = asset_server.load("hero/Run.png");

    // Crear layouts de atlas
    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 11, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);
    let charge_attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);
    let run_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 8, 1, None, None);

    let idle_atlas_layout = texture_atlas_layouts.add(idle_layout);
    let attack_atlas_layout = texture_atlas_layouts.add(attack_layout);
    let charge_attack_attlas_layout = texture_atlas_layouts.add(charge_attack_layout);
    let run_atlas_layout = texture_atlas_layouts.add(run_layout);

    // Crear datos de animación
    let animations = CharacterAnimations {
        animations: vec![
            // Animación de idle
            AnimationData {
                state: CharacterState::Idle,
                texture: idle_texture.clone(),
                atlas_layout: idle_atlas_layout.clone(),
                frames: 11,    // De 1 a 6
                fps: 10.0,     // 10 frames por segundo
                looping: true, // La animación idle se repite
            },
            // Animación de ataque
            AnimationData {
                state: CharacterState::Attacking,
                texture: attack_texture.clone(),
                atlas_layout: attack_atlas_layout.clone(),
                frames: 7,
                fps: 20.0,      // Un poco más rápido que idle
                looping: false, // La animación de ataque no se repite
            },
            AnimationData {
                state: CharacterState::ChargeAttacking,
                texture: charge_attack_texture.clone(),
                atlas_layout: charge_attack_attlas_layout.clone(),
                frames: 7,
                fps: 12.0,      // Un poco más rápido que idle
                looping: false, // La animación de ataque no se repite
            },
            AnimationData {
                state: CharacterState::Running,
                texture: run_texture.clone(),
                atlas_layout: run_atlas_layout.clone(),
                frames: 8,
                fps: 15.0,
                looping: true,
            },
        ],
    };

    // Animación inicial (idle)
    let initial_animation = CurrentAnimation {
        current_frame: 0,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        total_frames: 6,
        looping: true,
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
        // Transformación
        Transform::from_scale(Vec3::splat(1.0)),
        // Componentes de animación
        AnimationController::default(),
        animations,
        initial_animation,
    ));
}
