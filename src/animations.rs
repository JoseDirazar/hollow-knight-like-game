use bevy::prelude::*;

// Estado del personaje
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterState {
    Idle,
    Attacking,
    ChargeAttacking,
    Running,
    Jumping,
    Hurt,
    Dead,
    Falling,
}
#[derive(Component)]
pub struct CharacterDimensions {
    pub height: f32,
    pub feet_offset: f32,
}

#[derive(Component)]
pub struct AnimationController {
    current_state: CharacterState,
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

#[derive(Component)]
pub struct CharacterAnimations {
    pub animations: Vec<AnimationData>,
}

#[derive(Clone)]
pub struct AnimationData {
    pub state: CharacterState,
    pub texture: Handle<Image>,
    pub atlas_layout: Handle<TextureAtlasLayout>,
    pub frames: usize,
    pub fps: f32,
    pub looping: bool,
    pub ping_pong: bool,
}

#[derive(Component)]
pub struct CurrentAnimation {
    pub current_frame: usize,
    pub timer: Timer,
    pub total_frames: usize,
    pub looping: bool,
    pub reverse_direction: bool,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_animation_state, animate_current_state).chain(),
        );
    }
}

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
                    reverse_direction: false,
                };
            }
        }
    }
}

pub fn animate_current_state(
    time: Res<Time>,
    mut query: Query<(
        &mut CurrentAnimation,
        &mut AnimationController,
        &mut Sprite,
        &CharacterAnimations,
    )>,
) {
    for (mut animation, mut controller, mut sprite, character_animations) in &mut query {
        // Update the animation timer
        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Buscar la configuración de animación actual
                let current_state = controller.get_current_state();
                let current_animation_data = character_animations
                    .animations
                    .iter()
                    .find(|anim| anim.state == current_state);

                let ping_pong = current_animation_data
                    .map(|data| data.ping_pong)
                    .unwrap_or(false);

                // Determine direction of animation
                if animation.reverse_direction && ping_pong {
                    animation.current_frame -= 1;
                    // If we've reached the first frame, change direction
                    if animation.current_frame <= 0 {
                        animation.current_frame = 0;
                        animation.reverse_direction = false;
                    }
                } else {
                    animation.current_frame += 1;
                    // If we've reached the last frame
                    if animation.current_frame >= animation.total_frames {
                        if animation.looping {
                            if ping_pong {
                                // Para animaciones ping-pong (como idle)
                                animation.current_frame = animation.total_frames - 1;
                                animation.reverse_direction = true;
                            } else {
                                // Para animaciones de loop regular (como running)
                                animation.current_frame = 0;
                            }
                        } else {
                            // Para animaciones sin loop (como ataques)
                            animation.current_frame = animation.total_frames - 1;
                            if controller.get_current_state() == CharacterState::Attacking {
                                controller.change_state(CharacterState::Idle);
                            }
                            if controller.get_current_state() == CharacterState::ChargeAttacking {
                                controller.change_state(CharacterState::Idle);
                            }
                        }
                    }
                }

                // Update atlas index
                atlas.index = animation.current_frame;
            }
        }
    }
}
