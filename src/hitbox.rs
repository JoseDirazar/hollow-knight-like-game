use bevy::prelude::*;
use crate::{animations::{AnimationController, CharacterState, CurrentAnimation}, enemy::Enemy, player::Player};

// Component for any hitbox
#[derive(Component)]
pub struct Hitbox {
    pub damage: f32,
    pub active: bool,
    pub size: Vec2,
    pub offset: Vec2, // Offset from the entity's position
    pub frames_active: Vec<usize>, // Frames during which the hitbox is active
    pub cooldown: Timer, // Cooldown between hits
    pub hit_entities: Vec<Entity>, // Track which entities have been hit in this attack
    pub owner_state: CharacterState, // Track the state of the owner when hitbox was created
}

// Event emitted when a hitbox hits a target
#[derive(Event)]
pub struct HitEvent {
    pub source: Entity,
    pub target: Entity,
    pub damage: f32,
}

impl Default for Hitbox {
    fn default() -> Self {
        Self {
            damage: 0.0,
            active: false,
            size: Vec2::new(50.0, 30.0),
            offset: Vec2::ZERO,
            frames_active: Vec::new(),
            cooldown: Timer::from_seconds(0.5, TimerMode::Once),
            hit_entities: Vec::new(),
            owner_state: CharacterState::Idle,
        }
    }
}

// Plugin for the hitbox system
pub struct HitboxPlugin;

impl Plugin for HitboxPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<HitEvent>()
            .add_systems(Update, (
                update_hitbox_activity,
                handle_hitbox_collisions,
                cleanup_hitboxes,
            ).chain());
    }
}

fn handle_hitbox_collisions(
    mut commands: Commands,
    mut hitboxes: Query<(&mut Hitbox, &Transform, Entity)>,
    players: Query<(&Transform, Entity, &Player)>,
    enemies: Query<(&Transform, Entity, &Enemy)>,
    mut hit_events: EventWriter<HitEvent>,
) {
    for (mut hitbox, hitbox_transform, hitbox_entity) in &mut hitboxes {
        if !hitbox.active || !hitbox.cooldown.finished() {
            continue;
        }

        // Calcula la posición del hitbox con el offset
        let hitbox_pos = hitbox_transform.translation.truncate() + hitbox.offset;
        
        // Debug log para hitbox activo
        println!("[HITBOX] Active hitbox at {:?} with size {:?}", hitbox_pos, hitbox.size);
        
        // Verificar si la entidad es un jugador
        let is_player = players.iter().any(|(_, entity, _)| entity == hitbox_entity);
        let is_enemy = enemies.iter().any(|(_, entity, _)| entity == hitbox_entity);
        
        // Si el hitbox pertenece a un jugador, buscar colisiones con enemigos
        if is_player {
            for (target_transform, target_entity, _enemy) in &enemies {
                // Omitir si ya se golpeó a esta entidad en este ataque
                if hitbox.hit_entities.contains(&target_entity) {
                    continue;
                }

                // Calcular la distancia entre el hitbox y el objetivo
                let target_pos = target_transform.translation.truncate();
                let distance = (target_pos - hitbox_pos).length();
                
                // Debug log para verificación de colisión
                println!("[COLLISION] Player hitbox checking enemy. Distance: {}, Threshold: {}", 
                    distance, hitbox.size.x * 0.8);
                
                // Verificar colisión
                let collision_distance = hitbox.size.x * 0.8;
                if distance < collision_distance {
                    // Marcar la entidad como golpeada
                    hitbox.hit_entities.push(target_entity);
                    
                    // Debug log para golpe exitoso
                    println!("[HIT] Player hit enemy {:?} for {} damage", 
                        target_entity, hitbox.damage);
                    
                    // Emitir evento de golpe
                    hit_events.send(HitEvent {
                        source: hitbox_entity,
                        target: target_entity,
                        damage: hitbox.damage,
                    });
                    
                    // Reiniciar el cooldown
                    hitbox.cooldown.reset();
                }
            }
        }
        
        // Si el hitbox pertenece a un enemigo, buscar colisiones con el jugador
        if is_enemy {
            for (target_transform, target_entity, _player) in &players {
                // Omitir si ya se golpeó a esta entidad en este ataque
                if hitbox.hit_entities.contains(&target_entity) {
                    continue;
                }

                // Calcular la distancia entre el hitbox y el objetivo
                let target_pos = target_transform.translation.truncate();
                let distance = (target_pos - hitbox_pos).length();
                
                // Debug log para verificación de colisión
                println!("[COLLISION] Enemy hitbox checking player. Distance: {}, Threshold: {}", 
                    distance, hitbox.size.x * 0.8);
                
                // Verificar colisión
                let collision_distance = hitbox.size.x * 0.8;
                if distance < collision_distance {
                    // Marcar la entidad como golpeada
                    hitbox.hit_entities.push(target_entity);
                    
                    // Debug log para golpe exitoso
                    println!("[HIT] Enemy hit player {:?} for {} damage", 
                        target_entity, hitbox.damage);
                    
                    // Emitir evento de golpe
                    hit_events.send(HitEvent {
                        source: hitbox_entity,
                        target: target_entity,
                        damage: hitbox.damage,
                    });
                    
                    // Reiniciar el cooldown
                    hitbox.cooldown.reset();
                }
            }
        }
    }
}

// Modificar el sistema update_hitbox_activity para agregar más logs
fn update_hitbox_activity(
    time: Res<Time>,
    mut hitboxes: Query<(&mut Hitbox, &CurrentAnimation, Entity, &AnimationController)>,
    players: Query<Entity, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    for (mut hitbox, animation, entity, controller) in &mut hitboxes {
        // Actualizar timer de cooldown
        hitbox.cooldown.tick(time.delta());
        
        // Verificar si el frame actual está en la lista de frames activos
        let is_active = hitbox.frames_active.contains(&animation.current_frame);
        
        // Si el frame no está activo, limpiar la lista de entidades golpeadas
        if !is_active {
            hitbox.hit_entities.clear();
        }
        
        // Actualizar estado activo
        let was_active = hitbox.active;
        hitbox.active = is_active;

        // Determinar si la entidad es un jugador o un enemigo
        let is_player = players.iter().any(|player_entity| player_entity == entity);
        let is_enemy = enemies.iter().any(|enemy_entity| enemy_entity == entity);
        let entity_type = if is_player { "PLAYER" } else if is_enemy { "ENEMY" } else { "UNKNOWN" };
        
        // Solo loggear cambios de estado para reducir spam
        if was_active != is_active {
            if is_active {
                println!("[{}_HITBOX] Activated on frame {} (active frames: {:?})", 
                    entity_type, animation.current_frame, hitbox.frames_active);
            } else {
                println!("[{}_HITBOX] Deactivated, current frame {}", 
                    entity_type, animation.current_frame);
            }
        }
        
        // Log periódico para mostrar el progreso de la animación
        if animation.current_frame % 5 == 0 {
            println!("[{}_ANIMATION] Current frame: {}, State: {:?}", 
                entity_type, animation.current_frame, controller.get_current_state());
        }
    }
}


// System to cleanup hitboxes when they're no longer needed
fn cleanup_hitboxes(
    mut commands: Commands,
    hitboxes: Query<(Entity, &Hitbox, &AnimationController)>,
) {
    for (entity, hitbox, controller) in &hitboxes {
        // Only remove hitbox if:
        // 1. The hitbox is not active
        // 2. The cooldown has finished
        // 3. The owner is no longer in the attacking state
        if !hitbox.active && 
           hitbox.cooldown.finished() && 
           controller.get_current_state() != CharacterState::Attacking &&
           controller.get_current_state() != CharacterState::ChargeAttacking {
            commands.entity(entity).remove::<Hitbox>();
            println!("[CLEANUP] Removing hitbox from entity {:?}", entity);
        }
    }
} 