use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::Rng;

use crate::animation::{FlipbookAnimation, PendingPop};
use crate::assets::GameAssets;
use crate::audio::{PlaySoundEvent, SoundEffect};
use crate::board::{BoardLayout, UpdateTileVisualEvent};
use crate::game::{GameState, SquareState};
use crate::storage::HighScores;
use crate::ui::AboutModalState;
use crate::{GameDifficulty, GameWrapper};

#[derive(Resource, Default)]
pub struct VirtualCursorPos(pub Option<Vec2>);

pub fn cursor_system(
    mut virtual_cursor: ResMut<VirtualCursorPos>,
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = window_q.get_single() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(cam_transform, cursor_pos) {
            virtual_cursor.0 = Some(world_pos);
            return;
        }
    }
    virtual_cursor.0 = None;
}

pub fn mouse_click_system(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    virtual_cursor: Res<VirtualCursorPos>,
    mut game_wrapper: ResMut<GameWrapper>,
    layout: Option<Res<BoardLayout>>,
    assets: Res<GameAssets>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut update_tile_ev: EventWriter<UpdateTileVisualEvent>,
    modal_state: Res<AboutModalState>,
    mut high_scores: ResMut<HighScores>,
    difficulty: Res<GameDifficulty>,
) {
    let Some(cursor) = virtual_cursor.0 else { return };
    let Some(layout) = layout else { return };

    // Don't interact with board if modal is open
    if modal_state.open {
        return;
    }

    let left_click = mouse_button_input.just_pressed(MouseButton::Left);
    let right_click = mouse_button_input.just_pressed(MouseButton::Right);
    let shift_held = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);

    if !left_click && !right_click {
        return;
    }

    // Check if clicked inside a tile
    let Some((col, row)) = layout.tile_from_virtual_pos(cursor) else {
        return;
    };

    let game = &mut game_wrapper.0;
    if game.game_ended() {
        return;
    }

    let is_alt = right_click || (left_click && shift_held);
    let current_state = game.get_square_state(col, row);

    let mut rng = rand::thread_rng();

    if is_alt {
        // Alt-click: Flag / Unflag or Chord
        if current_state == SquareState::Hidden {
            game.set_flag(col, row, true);
            sound_events.send(PlaySoundEvent(SoundEffect::Flag));
            update_tile_ev.send(UpdateTileVisualEvent { col, row });
        } else if current_state == SquareState::Flagged {
            game.set_flag(col, row, false);
            sound_events.send(PlaySoundEvent(SoundEffect::Unflag));
            update_tile_ev.send(UpdateTileVisualEvent { col, row });
        } else if current_state == SquareState::Revealed {
            // Chord Reveal!
            if game.can_reveal(col, row) {
                // Collect hidden neighbors for dart throw
                let field = game.field().unwrap();
                let adj_hidden: Vec<(usize, usize)> = field
                    .get_adjacent_indices(col, row)
                    .iter()
                    .map(|&i| field.coordinate(i))
                    .filter(|&(c, r)| game.get_square_state(c, r) == SquareState::Hidden)
                    .collect();

                if !adj_hidden.is_empty() {
                    sound_events.send(PlaySoundEvent(SoundEffect::ThrowDart));
                    for &(hc, hr) in &adj_hidden {
                        spawn_dart(&mut commands, &assets, layout.tile_center(hc, hr), layout.board_scale);
                    }
                }

                let reveals = game.reveal(col, row);
                handle_reveals(
                    &mut commands,
                    game,
                    &layout,
                    col,
                    row,
                    reveals,
                    &mut sound_events,
                    &mut update_tile_ev,
                    &mut high_scores,
                    &difficulty,
                    &mut rng,
                );
            }
        }
    } else {
        // Normal Left Click: Reveal
        if current_state == SquareState::Hidden {
            sound_events.send(PlaySoundEvent(SoundEffect::ThrowDart));
            spawn_dart(&mut commands, &assets, layout.tile_center(col, row), layout.board_scale);

            let reveals = game.reveal(col, row);
            handle_reveals(
                &mut commands,
                game,
                &layout,
                col,
                row,
                reveals,
                &mut sound_events,
                &mut update_tile_ev,
                &mut high_scores,
                &difficulty,
                &mut rng,
            );
        }
    }
}

fn spawn_dart(commands: &mut Commands, assets: &GameAssets, center: Vec2, scale: f32) {
    // Dart Center offset
    let dart_pos = Vec3::new(center.x, center.y - 4.0 * scale, 30.0);

    // Dart
    commands.spawn((
        Sprite {
            image: assets.dart_sheet.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.dart_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Transform {
            translation: dart_pos,
            scale: Vec3::splat(scale),
            ..default()
        },
        FlipbookAnimation::new(60.0, 55),
    ));

    // Shadow
    commands.spawn((
        Sprite {
            image: assets.shadow_sheet.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: assets.shadow_layout.clone(),
                index: 0,
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(dart_pos.x, dart_pos.y, 25.0),
            scale: Vec3::splat(scale),
            ..default()
        },
        FlipbookAnimation::new(60.0, 55),
    ));
}

fn handle_reveals(
    commands: &mut Commands,
    game: &mut crate::game::Game,
    layout: &BoardLayout,
    start_col: usize,
    start_row: usize,
    reveals: Option<Vec<(usize, usize)>>,
    sound_events: &mut EventWriter<PlaySoundEvent>,
    update_tile_ev: &mut EventWriter<UpdateTileVisualEvent>,
    high_scores: &mut HighScores,
    _difficulty: &GameDifficulty,
    rng: &mut rand::rngs::ThreadRng,
) {
    let start_vec = Vec2::new(start_col as f32, start_row as f32);

    if let Some(reveals) = reveals {
        // Successful reveal (1 or more tiles)
        for &(c, r) in &reveals {
            let dist = (Vec2::new(c as f32, r as f32) - start_vec).length();
            let delay_frames = 12.0 + dist * 4.0 + rng.gen_range(0..10) as f32;
            let delay_secs = delay_frames / 60.0;

            commands.spawn(PendingPop {
                timer: Timer::from_seconds(delay_secs, TimerMode::Once),
                col: c,
                row: r,
                is_bomb: false,
                center_pos: layout.tile_center(c, r).extend(20.0),
                scale: layout.board_scale,
            });
        }

        // Check if won!
        if game.state() == GameState::Won {
            sound_events.send(PlaySoundEvent(SoundEffect::Win));
            if let Some(dur) = game.duration() {
                high_scores.update_record(game.width, game.height, game.bomb_count, dur.as_millis() as u64);
            }
            // Update all bomb tiles to safe
            for i in 0..game.field().unwrap().len() {
                let (c, r) = game.field().unwrap().coordinate(i);
                if game.get_square_state(c, r) == SquareState::Safe {
                    update_tile_ev.send(UpdateTileVisualEvent { col: c, row: r });
                }
            }
        }
    } else {
        // Lost! Hit a bomb
        assert_eq!(game.state(), GameState::Lost);

        // Explode the clicked bomb immediately
        commands.spawn(PendingPop {
            timer: Timer::from_seconds(0.05, TimerMode::Once),
            col: start_col,
            row: start_row,
            is_bomb: true,
            center_pos: layout.tile_center(start_col, start_row).extend(20.0),
            scale: layout.board_scale,
        });

        // Cascade explode remaining bombs and hidden squares
        let field = game.field().unwrap();
        for i in 0..field.len() {
            let (c, r) = field.coordinate(i);
            if c == start_col && r == start_row {
                continue;
            }
            let s = game.get_square_state(c, r);
            if s == SquareState::Bomb || s == SquareState::Hidden {
                let dist = (Vec2::new(c as f32, r as f32) - start_vec).length();
                let delay_frames = 12.0 + dist * 4.0 + rng.gen_range(0..10) as f32;
                let delay_secs = delay_frames / 60.0;

                commands.spawn(PendingPop {
                    timer: Timer::from_seconds(delay_secs, TimerMode::Once),
                    col: c,
                    row: r,
                    is_bomb: s == SquareState::Bomb,
                    center_pos: layout.tile_center(c, r).extend(20.0),
                    scale: layout.board_scale,
                });
            }
        }
    }
}
