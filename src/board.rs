use bevy::prelude::*;

use crate::assets::GameAssets;
use crate::game::{GameState, SquareState};

pub const VIRTUAL_WIDTH: f32 = 2048.0;
pub const VIRTUAL_HEIGHT: f32 = 1536.0;
pub const SQUARE_SIZE: f32 = 80.0;
pub const EDGE_OFFSET: f32 = 32.0;
pub const HOLE_SIZE: f32 = 16.0 * SQUARE_SIZE + 2.0 * EDGE_OFFSET; // 1344.0
pub const BOARD_OFFSET_X: f32 = 352.0;
pub const BOARD_OFFSET_Y: f32 = 96.0;

#[derive(Event, Debug, Clone, Default)]
pub struct ResetGameEvent;

#[derive(Component)]
pub struct BoardRoot;

#[derive(Component)]
pub struct TileComponent {
    pub col: usize,
    pub row: usize,
}

#[derive(Event, Debug, Clone)]
pub struct UpdateTileVisualEvent {
    pub col: usize,
    pub row: usize,
}

#[derive(Resource)]
pub struct BoardLayout {
    pub cols: usize,
    pub rows: usize,
    pub board_scale: f32,
    pub board_size: f32,
}

impl BoardLayout {
    pub fn new(cols: usize, rows: usize) -> Self {
        let board_size = cols as f32 * SQUARE_SIZE + 2.0 * EDGE_OFFSET;
        let board_scale = HOLE_SIZE / board_size;
        Self {
            cols,
            rows,
            board_scale,
            board_size,
        }
    }

    pub fn tile_center(&self, col: usize, row: usize) -> Vec2 {
        let dart_x = BOARD_OFFSET_X
            + (EDGE_OFFSET + col as f32 * SQUARE_SIZE + SQUARE_SIZE * 0.5) * self.board_scale;
        let dart_y = BOARD_OFFSET_Y
            + (EDGE_OFFSET + row as f32 * SQUARE_SIZE + SQUARE_SIZE * 0.5) * self.board_scale;
        Vec2::new(dart_x - VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5 - dart_y)
    }

    pub fn tile_from_virtual_pos(&self, pos: Vec2) -> Option<(usize, usize)> {
        let dart_x = pos.x + VIRTUAL_WIDTH * 0.5;
        let dart_y = VIRTUAL_HEIGHT * 0.5 - pos.y;

        let local_x = (dart_x - BOARD_OFFSET_X) / self.board_scale - EDGE_OFFSET;
        let local_y = (dart_y - BOARD_OFFSET_Y) / self.board_scale - EDGE_OFFSET;

        if local_x < 0.0 || local_y < 0.0 {
            return None;
        }

        let col = (local_x / SQUARE_SIZE) as usize;
        let row = (local_y / SQUARE_SIZE) as usize;

        if col < self.cols && row < self.rows {
            Some((col, row))
        } else {
            None
        }
    }
}

pub fn spawn_board(
    mut commands: Commands,
    assets: Res<GameAssets>,
    game: Res<crate::GameWrapper>,
    old_board: Query<Entity, With<BoardRoot>>,
) {
    for entity in &old_board {
        commands.entity(entity).despawn_recursive();
    }

    let cols = game.0.width;
    let rows = game.0.height;
    let layout = BoardLayout::new(cols, rows);

    let board_scale = layout.board_scale;
    let board_size = layout.board_size;

    commands.insert_resource(layout);

    commands
        .spawn((
            BoardRoot,
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // 1. Outer Background Framing (Mirrored Quad)
            spawn_background_frame(parent, &assets);

            // 2. Inner Board Framing (Corners and Edges)
            spawn_inner_board_frame(parent, &assets, cols, board_size, board_scale);

            // 3. Squares (Balloons)
            for r in 0..rows {
                for c in 0..cols {
                    let dart_x = BOARD_OFFSET_X
                        + (EDGE_OFFSET + c as f32 * SQUARE_SIZE + SQUARE_SIZE * 0.5) * board_scale;
                    let dart_y = BOARD_OFFSET_Y
                        + (EDGE_OFFSET + r as f32 * SQUARE_SIZE + SQUARE_SIZE * 0.5) * board_scale;

                    let bevy_x = dart_x - VIRTUAL_WIDTH * 0.5;
                    let bevy_y = VIRTUAL_HEIGHT * 0.5 - dart_y;

                    parent.spawn((
                        TileComponent { col: c, row: r },
                        Sprite {
                            image: assets.balloon.clone(),
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(bevy_x, bevy_y, 10.0),
                            scale: Vec3::splat(board_scale),
                            ..default()
                        },
                    ));
                }
            }
        });
}

fn spawn_background_frame(parent: &mut ChildBuilder, assets: &GameAssets) {
    // StageXL:
    // ttl at (0, 0), 1024x96
    // ttr at (2048, 0) scaleX = -1
    // bbl at (0, 1534) scaleY = -1
    // bbr at (2048, 1534) scaleX = -1, scaleY = -1
    // stl at (0, 96), 352x671
    // str at (2048, 96) scaleX = -1
    // sbl at (0, 1438) scaleY = -1
    // sbr at (2048, 1438) scaleX = -1, scaleY = -1

    // Convert to Bevy center coordinates:
    // Top-left bar: 1024x96 -> center is (512, 48)
    let tl_center = Vec2::new(512.0 - 1024.0, 768.0 - 48.0);
    let tr_center = Vec2::new(1536.0 - 1024.0, 768.0 - 48.0);
    // Bottom bar: y = 1534 in StageXL, height 96, scaleY = -1 -> center is 1486.0
    let bl_center = Vec2::new(512.0 - 1024.0, 768.0 - 1486.0);
    let br_center = Vec2::new(1536.0 - 1024.0, 768.0 - 1486.0);

    // Side left: 352x671 -> top center is (176, 431.5)
    let sl_center = Vec2::new(176.0 - 1024.0, 768.0 - 431.5);
    let sr_center = Vec2::new((2048.0 - 176.0) - 1024.0, 768.0 - 431.5);
    // Side bottom: y = 1438 in StageXL, height 671, scaleY = -1 -> center is 1102.5
    let sbl_center = Vec2::new(176.0 - 1024.0, 768.0 - 1102.5);
    let sbr_center = Vec2::new((2048.0 - 176.0) - 1024.0, 768.0 - 1102.5);

    // Top bars
    parent.spawn((
        Sprite { image: assets.background_top_left.clone(), ..default() },
        Transform::from_translation(tl_center.extend(1.0)),
    ));
    parent.spawn((
        Sprite { image: assets.background_top_left.clone(), ..default() },
        Transform {
            translation: tr_center.extend(1.0),
            scale: Vec3::new(-1.0, 1.0, 1.0),
            ..default()
        },
    ));
    // Bottom bars
    parent.spawn((
        Sprite { image: assets.background_top_left.clone(), ..default() },
        Transform {
            translation: bl_center.extend(1.0),
            scale: Vec3::new(1.0, -1.0, 1.0),
            ..default()
        },
    ));
    parent.spawn((
        Sprite { image: assets.background_top_left.clone(), ..default() },
        Transform {
            translation: br_center.extend(1.0),
            scale: Vec3::new(-1.0, -1.0, 1.0),
            ..default()
        },
    ));

    // Side bars
    parent.spawn((
        Sprite { image: assets.background_side_left.clone(), ..default() },
        Transform::from_translation(sl_center.extend(1.0)),
    ));
    parent.spawn((
        Sprite { image: assets.background_side_left.clone(), ..default() },
        Transform {
            translation: sr_center.extend(1.0),
            scale: Vec3::new(-1.0, 1.0, 1.0),
            ..default()
        },
    ));
    parent.spawn((
        Sprite { image: assets.background_side_left.clone(), ..default() },
        Transform {
            translation: sbl_center.extend(1.0),
            scale: Vec3::new(1.0, -1.0, 1.0),
            ..default()
        },
    ));
    parent.spawn((
        Sprite { image: assets.background_side_left.clone(), ..default() },
        Transform {
            translation: sbr_center.extend(1.0),
            scale: Vec3::new(-1.0, -1.0, 1.0),
            ..default()
        },
    ));
}

fn spawn_inner_board_frame(
    parent: &mut ChildBuilder,
    assets: &GameAssets,
    cols: usize,
    board_size: f32,
    board_scale: f32,
) {
    let board_to_screen = |lx: f32, ly: f32, w: f32, h: f32| -> Vec3 {
        let sx = BOARD_OFFSET_X + (lx + w * 0.5) * board_scale;
        let sy = BOARD_OFFSET_Y + (ly + h * 0.5) * board_scale;
        Vec3::new(sx - VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5 - sy, 5.0)
    };

    // 4 Corners: 112 x 112
    parent.spawn((
        Sprite { image: assets.board_corner_tl.clone(), ..default() },
        Transform {
            translation: board_to_screen(0.0, 0.0, 112.0, 112.0),
            scale: Vec3::splat(board_scale),
            ..default()
        },
    ));
    parent.spawn((
        Sprite { image: assets.board_corner_tr.clone(), ..default() },
        Transform {
            translation: board_to_screen(board_size - 112.0, 0.0, 112.0, 112.0),
            scale: Vec3::splat(board_scale),
            ..default()
        },
    ));
    parent.spawn((
        Sprite { image: assets.board_corner_bl.clone(), ..default() },
        Transform {
            translation: board_to_screen(0.0, board_size - 112.0, 112.0, 112.0),
            scale: Vec3::splat(board_scale),
            ..default()
        },
    ));
    parent.spawn((
        Sprite { image: assets.board_corner_br.clone(), ..default() },
        Transform {
            translation: board_to_screen(board_size - 112.0, board_size - 112.0, 112.0, 112.0),
            scale: Vec3::splat(board_scale),
            ..default()
        },
    ));

    // Repeated sides for i in 0..(cols - 2)
    for i in 0..(cols.saturating_sub(2)) {
        let offset = 112.0 + i as f32 * 80.0;
        // Top side: 80 x 112
        parent.spawn((
            Sprite { image: assets.board_side_top.clone(), ..default() },
            Transform {
                translation: board_to_screen(offset, 0.0, 80.0, 112.0),
                scale: Vec3::splat(board_scale),
                ..default()
            },
        ));
        // Bottom side: 80 x 112
        parent.spawn((
            Sprite { image: assets.board_side_bottom.clone(), ..default() },
            Transform {
                translation: board_to_screen(offset, board_size - 112.0, 80.0, 112.0),
                scale: Vec3::splat(board_scale),
                ..default()
            },
        ));
        // Left side: 112 x 80
        parent.spawn((
            Sprite { image: assets.board_side_left.clone(), ..default() },
            Transform {
                translation: board_to_screen(0.0, offset, 112.0, 80.0),
                scale: Vec3::splat(board_scale),
                ..default()
            },
        ));
        // Right side: 112 x 80
        parent.spawn((
            Sprite { image: assets.board_side_right.clone(), ..default() },
            Transform {
                translation: board_to_screen(board_size - 112.0, offset, 112.0, 80.0),
                scale: Vec3::splat(board_scale),
                ..default()
            },
        ));
    }
}

pub fn update_tile_visual_system(
    mut events: EventReader<UpdateTileVisualEvent>,
    mut game: ResMut<crate::GameWrapper>,
    assets: Res<GameAssets>,
    mut query: Query<(&TileComponent, &mut Sprite)>,
) {
    let mut updated_tiles = std::collections::HashSet::new();
    for ev in events.read() {
        updated_tiles.insert((ev.col, ev.row));
    }

    if updated_tiles.is_empty() {
        return;
    }

    for (tile, mut sprite) in &mut query {
        if updated_tiles.contains(&(tile.col, tile.row)) {
            let state = game.0.get_square_state(tile.col, tile.row);
            sprite.image = match state {
                SquareState::Hidden => {
                    if game.0.state() == GameState::Lost {
                        let idx = (tile.col + tile.row) % 4;
                        assets.balloon_bits[idx].clone()
                    } else {
                        assets.balloon.clone()
                    }
                }
                SquareState::Flagged => assets.balloon_flagged.clone(),
                SquareState::Safe => assets.balloon_safe.clone(),
                SquareState::Bomb => assets.crater_bomb.clone(),
                SquareState::Revealed => {
                    let count = game.0.field_mut().get_adjacent_count(tile.col, tile.row) as usize;
                    assets.numbers[count].clone()
                }
            };
        }
    }
}

pub fn update_all_tiles_system(
    mut game: ResMut<crate::GameWrapper>,
    assets: Res<GameAssets>,
    mut query: Query<(&TileComponent, &mut Sprite)>,
) {
    for (tile, mut sprite) in &mut query {
        let state = game.0.get_square_state(tile.col, tile.row);
        sprite.image = match state {
            SquareState::Hidden => {
                if game.0.state() == GameState::Lost {
                    let idx = (tile.col + tile.row) % 4;
                    assets.balloon_bits[idx].clone()
                } else {
                    assets.balloon.clone()
                }
            }
            SquareState::Flagged => assets.balloon_flagged.clone(),
            SquareState::Safe => assets.balloon_safe.clone(),
            SquareState::Bomb => assets.crater_bomb.clone(),
            SquareState::Revealed => {
                let count = game.0.field_mut().get_adjacent_count(tile.col, tile.row) as usize;
                assets.numbers[count].clone()
            }
        };
    }
}

pub fn reset_game_listener(
    commands: Commands,
    assets: Res<GameAssets>,
    game: Res<crate::GameWrapper>,
    old_board: Query<Entity, With<BoardRoot>>,
    mut reset_ev: EventReader<ResetGameEvent>,
) {
    if reset_ev.read().next().is_some() {
        spawn_board(commands, assets, game, old_board);
    }
}
