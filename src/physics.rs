use bevy::prelude::*;

// Componente para física básica
#[derive(Component, Debug)]
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
            gravity_scale: 1.0,
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
        Self { strength: 980.0 } // Aproximadamente 9.8 m/s² en pixeles
    }
}

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GravitySettings>()
            .add_systems(Update, apply_gravity)
            .add_systems(Update, apply_physics.after(apply_gravity));
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
        if physics.velocity.y < -1000.0 {
            physics.velocity.y = -1000.0;
        }

        // Aplicar velocidad a la posición
        transform.translation.x += physics.velocity.x * delta;
        transform.translation.y += physics.velocity.y * delta;

        // Reiniciar aceleración después de aplicarla
        physics.acceleration = Vec2::ZERO;

        // Ya no necesitamos esta colisión simple con el suelo, ahora se maneja en ground.rs
        // if transform.translation.y <= 0.0 {
        //    transform.translation.y = 0.0;
        //    physics.velocity.y = 0.0;
        //    physics.on_ground = true;
        // } else {
        //    physics.on_ground = false;
        // }
    }
}

// Este sistema ahora está en player.rs para mantener la lógica de juego separada
// Pero lo dejamos comentado aquí como referencia
/*
fn player_jump(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Physics, With<crate::player::Player>>,
) {
    for mut physics in &mut query {
        if keyboard.just_pressed(KeyCode::Space) && physics.on_ground {
            physics.velocity.y = 500.0; // Fuerza de salto
            physics.on_ground = false;
        }
    }
}
*/
