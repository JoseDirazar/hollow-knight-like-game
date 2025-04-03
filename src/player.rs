use bevy::prelude::*;

use crate::resolution;
use std::time::Duration;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_player)
            .add_systems(Update, animate_sprite)
            .add_systems(Update, trigger_attack)
            .add_systems(Update, animate_attack);
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
    texture: Handle<Image>,
    atlas: Handle<TextureAtlasLayout>,
    frames: usize,
    frame_timer: Timer,
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
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

fn animate_attack(
    time: Res<Time>,
    mut query: Query<(
        &mut PlayerState,
        &mut AnimationTimer,
        &mut Sprite,
        &AttackAnimation,
    )>,
) {
    for (mut state, mut timer, mut sprite, attack_animation) in &mut query {
        if *state == PlayerState::Attacking {
            timer.tick(time.delta());

            if timer.just_finished() {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    if atlas.index >= attack_animation.frames {
                        *state = PlayerState::Idle;
                        sprite.image = attack_animation.texture.clone();
                        sprite.texture_atlas.as_mut().unwrap().index = 1;
                        timer.reset();
                    } else {
                        atlas.index += 1;
                        timer.set_duration(Duration::from_secs_f32(
                            1.0 / attack_animation.frames as f32,
                        ));
                        timer.reset();
                    }
                }
            }
        }
    }
}

fn trigger_attack(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerState, &mut Sprite, &AttackAnimation)>,
) {
    for (mut state, mut sprite, attack_animation) in &mut query {
        if keyboard_input.just_pressed(KeyCode::Space) {
            *state = PlayerState::Attacking;
            sprite.image = attack_animation.texture.clone();
            sprite.texture_atlas = Some(TextureAtlas {
                layout: attack_animation.atlas.clone(),
                index: 1,
            });
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
    let attack_texture: Handle<Image> = asset_server.load("hero/Attack1.png");

    let idle_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 11, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::splat(180), 7, 1, None, None);

    let idle_texture_atlas_layout = texture_atlas_layouts.add(idle_layout);
    let attack_texture_atlas_layout = texture_atlas_layouts.add(attack_layout);

    let animation_indices = AnimationIndices { first: 1, last: 6 };

    commands.spawn((
        Sprite::from_atlas_image(
            idle_texture,
            TextureAtlas {
                layout: idle_texture_atlas_layout,
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
            texture: attack_texture,
            atlas: attack_texture_atlas_layout,
            frames: 6,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Once),
        },
    ));
}
