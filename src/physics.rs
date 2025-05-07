use bevy::prelude::*;

use crate::game::GameState;

// Physics Constants
const GRAVITY_STRENGTH: f32 = 980.0; // Approximately 9.8 m/s² in pixels
const MAX_FALL_SPEED: f32 = -1000.0;
const DEFAULT_GRAVITY_SCALE: f32 = 1.0;

// Componente para física básica
#[derive(Component)]
pub struct Physics {
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub on_ground: bool,
    pub gravity_scale: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            on_ground: false,
            gravity_scale: DEFAULT_GRAVITY_SCALE,
        }
    }
}

// Recurso global para configurar la gravedad
#[derive(Resource)]
pub struct GravitySettings {
    pub strength: f32,
}

impl Default for GravitySettings {
    fn default() -> Self {
        Self { strength: GRAVITY_STRENGTH }
    }
}

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GravitySettings>()
            .add_systems(Update, apply_gravity.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                apply_physics
                    .after(apply_gravity)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// Sistema que aplica la gravedad a los objetos con física
fn apply_gravity(_time: Res<Time>, gravity: Res<GravitySettings>, mut query: Query<&mut Physics>) {
    for mut physics in &mut query {
        if !physics.on_ground {
            // Aplicar aceleración de gravedad
            physics.acceleration.y -= gravity.strength * physics.gravity_scale;
        }
    }
}

// Sistema que actualiza la posición basada en la física
fn apply_physics(time: Res<Time>, mut query: Query<(&mut Transform, &mut Physics)>) {
    let delta = time.delta_secs();

    for (mut transform, mut physics) in &mut query {
        // Actualizar velocidad basada en aceleración
        let acceleration = physics.acceleration.clone();
        physics.velocity += acceleration * delta;

        // Limitar la velocidad de caída para evitar problemas con colisiones
        if physics.velocity.y < MAX_FALL_SPEED {
            physics.velocity.y = MAX_FALL_SPEED;
        }

        // Aplicar velocidad a la posición
        transform.translation.x += physics.velocity.x * delta;
        transform.translation.y += physics.velocity.y * delta;

        // Reiniciar aceleración después de aplicarla
        physics.acceleration = Vec2::ZERO;
    }
}
