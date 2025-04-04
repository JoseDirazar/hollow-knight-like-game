use bevy::prelude::*;
use std::time::Duration;

// Estado del personaje
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterState {
    Idle,
    Attacking,
    ChargeAttacking,
    Running,
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
    pub animations: Vec<AnimationData>,
}

// Datos de una animación específica
#[derive(Clone)]
pub struct AnimationData {
    pub state: CharacterState,
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub frames: usize,
    pub fps: f32,
    pub looping: bool,
}

// Componente para la animación actual
#[derive(Component)]
pub struct CurrentAnimation {
    pub current_frame: usize,
    pub timer: Timer,
    pub total_frames: usize,
    pub looping: bool,
}

// Plugin principal de animación
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_animation_state, animate_current_state).chain(),
        );
    }
}

// Sistema que actualiza el estado de animación
pub fn update_animation_state(
    mut _commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationController,
        &CharacterAnimations,
        &mut CurrentAnimation,
        &mut Sprite,
    )>,
) {
    for (_entity, mut controller, animations, mut current_animation, mut sprite) in &mut query {
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
pub fn animate_current_state(
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
                        if controller.get_current_state() == CharacterState::ChargeAttacking {
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
