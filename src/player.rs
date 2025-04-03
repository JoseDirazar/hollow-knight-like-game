use bevy::prelude::*;
use std::time::Duration;

use crate::resolution;

// Plugin principal del jugador
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player).add_systems(
            Update,
            (
                process_player_input,
                update_animation_state,
                animate_current_state,
            )
                .chain(),
        );
    }
}

// ------ COMPONENTES ------

// Componente de estadísticas del jugador
#[derive(Component)]
pub struct Player {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
}

// Estado del personaje
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterState {
    Idle,
    Attacking,
    // Puedes agregar fácilmente más estados:
    // Walking,
    // Jumping,
    // TakingDamage,
    // etc.
}

// Componente para administrar las animaciones
#[derive(Component)]
pub struct AnimationController {
    // Estado actual del personaje
    current_state: CharacterState,
    // Estado a cambiar en el próximo frame (útil para transiciones)
    next_state: Option<CharacterState>,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current_state: CharacterState::Idle,
            next_state: None,
        }
    }
}

impl AnimationController {
    pub fn change_state(&mut self, new_state: CharacterState) {
        if self.current_state != new_state {
            self.next_state = Some(new_state);
        }
    }

    pub fn apply_next_state(&mut self) -> bool {
        if let Some(next) = self.next_state.take() {
            self.current_state = next;
            true
        } else {
            false
        }
    }

    pub fn get_current_state(&self) -> CharacterState {
        self.current_state
    }
}

// Componente que contiene todas las animaciones disponibles
#[derive(Component)]
pub struct CharacterAnimations {
    animations: Vec<AnimationData>,
}

// Datos de una animación específica
#[derive(Clone)]
pub struct AnimationData {
    state: CharacterState,
    texture: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    frames: usize,
    fps: f32,
    looping: bool,
}

// Componente para la animación actual
#[derive(Component)]
pub struct CurrentAnimation {
    current_frame: usize,
    timer: Timer,
    total_frames: usize,
    looping: bool,
}

// ------ SISTEMAS ------

// Sistema que procesa la entrada del jugador
fn process_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut AnimationController, With<Player>>,
) {
    for mut controller in &mut query {
        // Solo cambiar a atacar si estamos en idle
        if keyboard.just_pressed(KeyCode::Space)
            && controller.get_current_state() == CharacterState::Idle
        {
            controller.change_state(CharacterState::Attacking);
        }
    }
}

// Sistema que actualiza el estado de animación
fn update_animation_state(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationController,
        &CharacterAnimations,
        &mut CurrentAnimation,
        &mut Sprite,
    )>,
) {
    for (entity, mut controller, animations, mut current_animation, mut sprite) in &mut query {
        // Si hay un cambio de estado pendiente
        if controller.apply_next_state() {
            let current_state = controller.get_current_state();

            // Buscar la animación correspondiente al nuevo estado
            if let Some(animation_data) = animations
                .animations
                .iter()
                .find(|anim| anim.state == current_state)
            {
                // Actualizar sprite y animación
                sprite.image = animation_data.texture.clone();
                sprite.texture_atlas = Some(TextureAtlas {
                    layout: animation_data.atlas_layout.clone(),
                    index: 0,
                });

                // Configurar la nueva animación
                *current_animation = CurrentAnimation {
                    current_frame: 0,
                    timer: Timer::from_seconds(1.0 / animation_data.fps, TimerMode::Repeating),
                    total_frames: animation_data.frames,
                    looping: animation_data.looping,
                };
            }
        }
    }
}

// Sistema que anima el sprite según el estado actual
fn animate_current_state(
    time: Res<Time>,
    mut query: Query<(&mut CurrentAnimation, &mut AnimationController, &mut Sprite)>,
) {
    for (mut animation, mut controller, mut sprite) in &mut query {
        // Actualizar el timer de la animación
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Avanzar al siguiente frame
                animation.current_frame += 1;

                // Verificar si la animación ha terminado
                if animation.current_frame >= animation.total_frames {
                    if animation.looping {
                        // Reiniciar la animación si es cíclica (como idle)
                        animation.current_frame = 0;
                    } else {
                        // Si no es cíclica (como ataque), volver a idle
                        animation.current_frame = animation.total_frames - 1;
                        if controller.get_current_state() == CharacterState::Attacking {
                            controller.change_state(CharacterState::Idle);
                        }
                    }
                }

                // Actualizar el índice del atlas
                atlas.index = animation.current_frame;
            }
        }
    }
}

// Configuración inicial del jugador
fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
) {
    // Cargar texturas
    let idle_texture = asset_server.load("hero/Idle.png");
    let attack_texture = asset_server.load("hero/Attack1.png");

    // Crear layouts de atlas
    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 11, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);

    let idle_atlas_layout = texture_atlas_layouts.add(idle_layout);
    let attack_atlas_layout = texture_atlas_layouts.add(attack_layout);

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
            speed: 1.0,
        },
        // Transformación
        Transform::from_scale(Vec3::splat(1.0)),
        // Componentes de animación
        AnimationController::default(),
        animations,
        initial_animation,
    ));
}
