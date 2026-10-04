use bevy::prelude::*;

use crate::audio::{PlaySoundEvent, SoundEffect};

#[derive(Component)]
pub struct FlipbookAnimation {
    pub timer: Timer,
    pub current_frame: usize,
    pub total_frames: usize,
    pub despawn_on_finish: bool,
}

impl FlipbookAnimation {
    pub fn new(fps: f32, total_frames: usize) -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / fps, TimerMode::Repeating),
            current_frame: 0,
            total_frames,
            despawn_on_finish: true,
        }
    }
}

#[derive(Component)]
pub struct PendingPop {
    pub timer: Timer,
    pub col: usize,
    pub row: usize,
    pub is_bomb: bool,
    pub center_pos: Vec3,
    pub scale: f32,
}

#[derive(Event, Debug, Clone)]
pub struct SpawnPopCascadeEvent {
    pub start_col: usize,
    pub start_row: usize,
    pub reveals: Vec<(usize, usize)>,
    pub is_loss: bool,
}

pub fn update_flipbooks(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FlipbookAnimation, &mut Sprite)>,
) {
    for (entity, mut anim, mut sprite) in &mut query {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            anim.current_frame += 1;
            if anim.current_frame >= anim.total_frames {
                if anim.despawn_on_finish {
                    commands.entity(entity).despawn_recursive();
                } else {
                    anim.current_frame = 0;
                    if let Some(atlas) = sprite.texture_atlas.as_mut() {
                        atlas.index = 0;
                    }
                }
            } else if let Some(atlas) = sprite.texture_atlas.as_mut() {
                atlas.index = anim.current_frame;
            }
        }
    }
}

pub fn tick_pending_pops(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PendingPop)>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    assets: Res<crate::assets::GameAssets>,
    mut update_tile_ev: EventWriter<crate::board::UpdateTileVisualEvent>,
) {
    for (entity, mut pop) in &mut query {
        pop.timer.tick(time.delta());
        if pop.timer.just_finished() {
            // 1. Play sound
            if pop.is_bomb {
                sound_events.send(PlaySoundEvent(SoundEffect::Bomb));
            } else {
                sound_events.send(PlaySoundEvent(SoundEffect::Pop));
            }

            // 2. Trigger tile visual state update
            update_tile_ev.send(crate::board::UpdateTileVisualEvent {
                col: pop.col,
                row: pop.row,
            });

            // 3. Spawn Flipbook animation
            let (image, layout, frames) = if pop.is_bomb {
                (
                    assets.explode_sheet.clone(),
                    assets.explode_layout.clone(),
                    24,
                )
            } else {
                (assets.pop_sheet.clone(), assets.pop_layout.clone(), 28)
            };

            commands.spawn((
                Sprite {
                    image,
                    texture_atlas: Some(TextureAtlas {
                        layout,
                        index: 0,
                    }),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(pop.center_pos.x, pop.center_pos.y, 20.0),
                    scale: Vec3::splat(pop.scale),
                    ..default()
                },
                FlipbookAnimation::new(60.0, frames),
            ));

            commands.entity(entity).despawn();
        }
    }
}
