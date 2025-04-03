use bevy::prelude::*;

use crate::resolution;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, animate_sprite)
            .add_systems(Update, (trigger_attack, animate_attack).chain());
    }
}

#[derive(Component)]
pub struct Player {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
    pub attack: f32,
    pub defense: f32,
    pub speed: f32,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, PartialEq)]
enum PlayerState {
    Idle,
    Attacking,
}

#[derive(Component)]
struct AttackAnimation {
    idle_texture: Handle<Image>,
    idle_atlas: Handle<TextureAtlasLayout>,
    attack_texture: Handle<Image>,
    attack_atlas: Handle<TextureAtlasLayout>,
    current_frame: usize,
    total_frames: usize,
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut Sprite,
        &PlayerState,
    )>,
) {
    for (indices, mut timer, mut sprite, state) in &mut query {
        // Solo animar el sprite idle cuando estamos en estado Idle
        if *state == PlayerState::Idle {
            timer.tick(time.delta());

            if timer.just_finished() {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = if atlas.index == indices.last {
                        indices.first
                    } else {
                        atlas.index + 1
                    };
                }
            }
        }
    }
}

fn animate_attack(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut PlayerState,
        &mut AnimationTimer,
        &mut Sprite,
        &mut AttackAnimation,
    )>,
) {
    for (entity, mut state, mut timer, mut sprite, mut attack_anim) in &mut query {
        if *state == PlayerState::Attacking {
            timer.tick(time.delta());

            if timer.just_finished() {
                attack_anim.current_frame += 1;

                if attack_anim.current_frame >= attack_anim.total_frames {
                    // Volver al estado idle cuando la animación de ataque termina
                    *state = PlayerState::Idle;
                    attack_anim.current_frame = 0;

                    // Restablecer a la textura y atlas de idle
                    sprite.image = attack_anim.idle_texture.clone();
                    sprite.texture_atlas = Some(TextureAtlas {
                        layout: attack_anim.idle_atlas.clone(),
                        index: 1, // Usar el primer frame de idle
                    });

                    // Reiniciar el timer para la animación idle
                    *timer = AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating));
                } else {
                    // Avanzar al siguiente frame de ataque
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.index = attack_anim.current_frame;
                    }
                }
            }
        }
    }
}

fn trigger_attack(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &mut PlayerState,
        &mut Sprite,
        &mut AttackAnimation,
        &mut AnimationTimer,
    )>,
) {
    for (mut state, mut sprite, mut attack_anim, mut timer) in &mut query {
        // Solo activar el ataque si estamos en estado Idle
        if keyboard_input.just_pressed(KeyCode::Space) && *state == PlayerState::Idle {
            *state = PlayerState::Attacking;
            attack_anim.current_frame = 0;

            // Cambiar a la textura y atlas de ataque
            sprite.image = attack_anim.attack_texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: attack_anim.attack_atlas.clone(),
                index: 0,
            });

            // Configurar el timer para la animación de ataque
            *timer = AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating));
        }
    }
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    resolution: Res<resolution::Resolution>,
) {
    let idle_texture = asset_server.load("hero/Idle.png");
    let attack_texture = asset_server.load("hero/Attack1.png");

    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 11, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);

    let idle_texture_atlas_layout = texture_atlas_layouts.add(idle_layout);
    let attack_texture_atlas_layout = texture_atlas_layouts.add(attack_layout);

    let animation_indices = AnimationIndices { first: 1, last: 6 };

    commands.spawn((
        Sprite::from_atlas_image(
            idle_texture.clone(),
            TextureAtlas {
                layout: idle_texture_atlas_layout.clone(),
                index: animation_indices.first,
            },
        ),
        Player {
            name: "Hero".to_string(),
            health: 100.0,
            max_health: 100.0,
            attack: 10.0,
            defense: 5.0,
            speed: 1.0,
        },
        Transform::from_scale(Vec3::splat(1.0)),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        PlayerState::Idle,
        AttackAnimation {
            idle_texture: idle_texture,
            idle_atlas: idle_texture_atlas_layout,
            attack_texture: attack_texture,
            attack_atlas: attack_texture_atlas_layout,
            current_frame: 0,
            total_frames: 7,
        },
    ));
}
