use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
use bevy::winit::{UpdateMode, WinitSettings};

use crate::assets::GameAssets;
use crate::audio::{PlaySoundEvent, SoundEffect};
use crate::board::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::input::VirtualCursorPos;
use crate::storage::HighScores;
use crate::{GameDifficulty, GameWrapper};

#[derive(Resource, Debug, Clone)]
pub struct BevySettings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub sound_enabled: bool,
    pub show_fps: bool,
    pub reactive_mode: bool,
}

impl Default for BevySettings {
    fn default() -> Self {
        Self {
            vsync: false,
            fullscreen: false,
            sound_enabled: true,
            show_fps: false,
            reactive_mode: false,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct FpsTracker {
    pub smoothed_fps: f64,
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self { smoothed_fps: 60.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BevyOptionType {
    VSync,
    WindowMode,
    Sound,
    FpsCounter,
    ReactiveMode,
}

#[derive(Component)]
pub struct BevyOptionButton {
    pub option_type: BevyOptionType,
    pub half_w: f32,
    pub half_h: f32,
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct NewGameButton;

#[derive(Component)]
pub struct LogoButton;

#[derive(Resource, Default)]
pub struct AboutModalState {
    pub open: bool,
}

#[derive(Component)]
pub struct ModalRoot;

#[derive(Component)]
pub struct DifficultyButton(pub GameDifficulty);

pub fn setup_ui(
    mut commands: Commands,
    assets: Res<GameAssets>,
) {
    // 1. New Game Button: (450, 20), size 294x92
    // Bevy center: x = (450 + 147) - 1024 = -427, y = 768 - (20 + 46) = 702
    commands.spawn((
        NewGameButton,
        Sprite {
            image: assets.button_new_game.clone(),
            ..default()
        },
        Transform::from_xyz(-427.0, 702.0, 50.0),
    ));

    // 2. Logo Button: centered horizontally at y=20, size 318x94
    // Bevy center: x = 0, y = 768 - (20 + 47) = 701
    commands.spawn((
        LogoButton,
        Sprite {
            image: assets.logo_win.clone(),
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 701.0, 50.0),
            scale: Vec3::splat(1.2),
            ..default()
        },
    ));

    // 3. HUD Text: top-right at (1400, 20) -> Bevy: x = 1400 - 1024 = 376, y = 768 - 20 = 748
    commands.spawn((
        HudText,
        Text2d::new("Bombs Left: --\nTime: 0.0"),
        TextFont {
            font: assets.font_slackey.clone(),
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::BLACK),
        TextLayout::default(),
        Transform::from_xyz(376.0, 700.0, 50.0),
    ));
}

pub fn update_hud_text(
    time: Res<Time>,
    mut fps_tracker: ResMut<FpsTracker>,
    settings: Res<BevySettings>,
    game: Res<GameWrapper>,
    high_scores: Res<HighScores>,
    mut text_q: Query<&mut Text2d, With<HudText>>,
) {
    let Ok(mut text) = text_q.get_single_mut() else { return };

    let dt = time.delta_secs_f64();
    if dt > 0.0 {
        let current_fps = 1.0 / dt;
        fps_tracker.smoothed_fps = fps_tracker.smoothed_fps * 0.9 + current_fps * 0.1;
    }

    let bombs = game.0.bombs_left();
    let time_sec = if let Some(dur) = game.0.duration() {
        dur.as_secs_f64()
    } else {
        0.0
    };

    let record = high_scores.get_record(game.0.width, game.0.height, game.0.bomb_count);
    let mut str_val = format!("Bombs Left: {}\nTime: {:.1}", bombs, time_sec);
    if let Some(rec) = record {
        str_val.push_str(&format!("\nRecord: {:.1}", rec as f64 / 1000.0));
    }
    if settings.show_fps {
        str_val.push_str(&format!("\nFPS: {:.0}", fps_tracker.smoothed_fps.round()));
    }

    text.0 = str_val;
}

pub fn ui_interaction_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    cursor: Res<VirtualCursorPos>,
    mut modal_state: ResMut<AboutModalState>,
    mut game: ResMut<GameWrapper>,
    mut sound_events: EventWriter<PlaySoundEvent>,
    mut reset_events: EventWriter<crate::board::ResetGameEvent>,
    assets: Res<GameAssets>,
    mut new_game_btn_q: Query<(&mut Sprite, &Transform), With<NewGameButton>>,
    logo_btn_q: Query<&Transform, With<LogoButton>>,
    mut difficulty_res: ResMut<GameDifficulty>,
    difficulty_buttons_q: Query<(&DifficultyButton, &Transform)>,
    mut bevy_settings: ResMut<BevySettings>,
    option_buttons_q: Query<(&BevyOptionButton, &Transform)>,
) {
    let clicked = mouse_buttons.just_pressed(MouseButton::Left);

    // Toggle Help modal via Keyboard ('H' or 'Escape')
    if keyboard.just_pressed(KeyCode::KeyH) {
        modal_state.open = !modal_state.open;
        sound_events.send(PlaySoundEvent(SoundEffect::Click));
    } else if keyboard.just_pressed(KeyCode::Escape) && modal_state.open {
        modal_state.open = false;
        sound_events.send(PlaySoundEvent(SoundEffect::Click));
    }

    let Some(c) = cursor.0 else { return };

    // 1. New Game button hover & click: center (-427, 702), size 294x92
    if let Ok((mut sprite, trans)) = new_game_btn_q.get_single_mut() {
        let half_w = 294.0 * 0.5;
        let half_h = 92.0 * 0.5;
        let hovered = c.x >= trans.translation.x - half_w
            && c.x <= trans.translation.x + half_w
            && c.y >= trans.translation.y - half_h
            && c.y <= trans.translation.y + half_h;

        if hovered {
            sprite.image = assets.button_new_game_clicked.clone();
            if clicked && !modal_state.open {
                sound_events.send(PlaySoundEvent(SoundEffect::Click));
                game.0 = crate::game::Game::new(game.0.width, game.0.height, game.0.bomb_count);
                reset_events.send(crate::board::ResetGameEvent);
            }
        } else {
            sprite.image = assets.button_new_game.clone();
        }
    }

    // 2. Logo button click: center (0, 701), size 318x94 * 1.2
    if let Ok(trans) = logo_btn_q.get_single() {
        let half_w = 318.0 * 1.2 * 0.5;
        let half_h = 94.0 * 1.2 * 0.5;
        let hovered = c.x >= trans.translation.x - half_w
            && c.x <= trans.translation.x + half_w
            && c.y >= trans.translation.y - half_h
            && c.y <= trans.translation.y + half_h;

        if hovered && clicked {
            modal_state.open = !modal_state.open;
            sound_events.send(PlaySoundEvent(SoundEffect::Click));
        }
    }

    // 3. Modal Difficulty Clicks
    if modal_state.open && clicked {
        for (btn, trans) in &difficulty_buttons_q {
            let half_w = 120.0;
            let half_h = 30.0;
            if c.x >= trans.translation.x - half_w
                && c.x <= trans.translation.x + half_w
                && c.y >= trans.translation.y - half_h
                && c.y <= trans.translation.y + half_h
            {
                // Switch difficulty!
                sound_events.send(PlaySoundEvent(SoundEffect::Click));
                *difficulty_res = btn.0;
                let (w, h, m) = btn.0.dimensions();
                game.0 = crate::game::Game::new(w, h, m);
                modal_state.open = false;
                reset_events.send(crate::board::ResetGameEvent);
                return;
            }
        }

        // 4. Modal Bevy Option Button Clicks
        for (btn, trans) in &option_buttons_q {
            if c.x >= trans.translation.x - btn.half_w
                && c.x <= trans.translation.x + btn.half_w
                && c.y >= trans.translation.y - btn.half_h
                && c.y <= trans.translation.y + btn.half_h
            {
                sound_events.send(PlaySoundEvent(SoundEffect::Click));
                match btn.option_type {
                    BevyOptionType::VSync => {
                        bevy_settings.vsync = !bevy_settings.vsync;
                    }
                    BevyOptionType::WindowMode => {
                        bevy_settings.fullscreen = !bevy_settings.fullscreen;
                    }
                    BevyOptionType::Sound => {
                        bevy_settings.sound_enabled = !bevy_settings.sound_enabled;
                    }
                    BevyOptionType::FpsCounter => {
                        bevy_settings.show_fps = !bevy_settings.show_fps;
                    }
                    BevyOptionType::ReactiveMode => {
                        bevy_settings.reactive_mode = !bevy_settings.reactive_mode;
                    }
                }
                break;
            }
        }
    }
}

pub fn manage_about_modal_system(
    mut commands: Commands,
    modal_state: Res<AboutModalState>,
    bevy_settings: Res<BevySettings>,
    assets: Res<GameAssets>,
    modal_q: Query<Entity, With<ModalRoot>>,
) {
    if modal_state.is_changed() || (modal_state.open && bevy_settings.is_changed()) {
        for entity in &modal_q {
            commands.entity(entity).despawn_recursive();
        }

        if modal_state.open {
            commands
                .spawn((
                    ModalRoot,
                    Transform::default(),
                    Visibility::default(),
                ))
                .with_children(|parent| {
                    // Dark semi-transparent backdrop
                    parent.spawn((
                        Sprite {
                            color: Color::srgba(0.0, 0.0, 0.0, 0.75),
                            custom_size: Some(Vec2::new(VIRTUAL_WIDTH, VIRTUAL_HEIGHT)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 90.0),
                    ));

                    // Modal Dialog Panel (cream paper color #f4ebd0)
                    parent.spawn((
                        Sprite {
                            color: Color::srgb(0.95, 0.92, 0.82),
                            custom_size: Some(Vec2::new(1250.0, 880.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 95.0),
                    ));

                    // Modal Title
                    parent.spawn((
                        Text2d::new("POP, POP, WIN!"),
                        TextFont {
                            font: assets.font_slackey.clone(),
                            font_size: 44.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.2, 0.2)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Transform::from_xyz(0.0, 360.0, 100.0),
                    ));

                    // Difficulty Label
                    parent.spawn((
                        Text2d::new("SELECT DIFFICULTY:"),
                        TextFont {
                            font: assets.font_slackey.clone(),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.3, 0.2)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Transform::from_xyz(0.0, 290.0, 100.0),
                    ));

                    // 4 Difficulty buttons
                    let diffs = [
                        (GameDifficulty::Easy, "Easy (7x7)", -375.0),
                        (GameDifficulty::Medium, "Medium (11x11)", -125.0),
                        (GameDifficulty::Hard, "Hard (16x16)", 125.0),
                        (GameDifficulty::Extreme, "Extreme (24x24)", 375.0),
                    ];

                    for (diff, label, x_pos) in diffs {
                        parent
                            .spawn((
                                DifficultyButton(diff),
                                Sprite {
                                    color: Color::srgb(0.35, 0.55, 0.35),
                                    custom_size: Some(Vec2::new(210.0, 52.0)),
                                    ..default()
                                },
                                Transform::from_xyz(x_pos, 220.0, 100.0),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text2d::new(label),
                                    TextFont {
                                        font: assets.font_slackey.clone(),
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    TextLayout::new_with_justify(JustifyText::Center),
                                    Transform::from_xyz(0.0, 0.0, 101.0),
                                ));
                            });
                    }

                    // Instructions / How to play (Left-aligned)
                    let instructions = "\
HOW TO PLAY:
• Click on balloons to pop them and clear the field.
• Numbers tell how many bombs are adjacent (including diagonals).
• Freeze (flag) suspected bombs by Shift-clicking or Right-clicking.
• Chord Reveal: Shift- or Right-click an open number when all its neighbor bombs are flagged.
• Warning: Chord-popping with incorrect flags will pop a bomb!
• First click is always 100% safe!";

                    parent.spawn((
                        Text2d::new(instructions),
                        TextFont {
                            font: assets.font_slackey.clone(),
                            font_size: 19.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.2, 0.2)),
                        TextLayout::new_with_justify(JustifyText::Left),
                        bevy::sprite::Anchor::TopLeft,
                        Transform::from_xyz(-540.0, 140.0, 100.0),
                    ));

                    // Section: BEVY ENGINE & DISPLAY OPTIONS
                    parent.spawn((
                        Text2d::new("BEVY ENGINE & DISPLAY OPTIONS:"),
                        TextFont {
                            font: assets.font_slackey.clone(),
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.3, 0.2)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Transform::from_xyz(0.0, -115.0, 100.0),
                    ));

                    // Option Buttons Row 1: VSync, Window Mode, Sound
                    let vsync_label = if bevy_settings.vsync {
                        "VSync: ON"
                    } else {
                        "VSync: OFF (Smooth)"
                    };
                    let vsync_color = if bevy_settings.vsync {
                        Color::srgb(0.35, 0.55, 0.35)
                    } else {
                        Color::srgb(0.25, 0.45, 0.65)
                    };

                    let win_label = if bevy_settings.fullscreen {
                        "Window: Fullscreen"
                    } else {
                        "Window: Windowed"
                    };
                    let win_color = if bevy_settings.fullscreen {
                        Color::srgb(0.45, 0.35, 0.65)
                    } else {
                        Color::srgb(0.35, 0.55, 0.35)
                    };

                    let sound_label = if bevy_settings.sound_enabled {
                        "Sound: ON"
                    } else {
                        "Sound: MUTED"
                    };
                    let sound_color = if bevy_settings.sound_enabled {
                        Color::srgb(0.35, 0.55, 0.35)
                    } else {
                        Color::srgb(0.65, 0.35, 0.35)
                    };

                    let row1 = [
                        (BevyOptionType::VSync, vsync_label, vsync_color, -360.0),
                        (BevyOptionType::WindowMode, win_label, win_color, 0.0),
                        (BevyOptionType::Sound, sound_label, sound_color, 360.0),
                    ];

                    for (opt_type, label, col, x_pos) in row1 {
                        parent
                            .spawn((
                                BevyOptionButton {
                                    option_type: opt_type,
                                    half_w: 130.0,
                                    half_h: 24.0,
                                },
                                Sprite {
                                    color: col,
                                    custom_size: Some(Vec2::new(260.0, 48.0)),
                                    ..default()
                                },
                                Transform::from_xyz(x_pos, -175.0, 100.0),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text2d::new(label),
                                    TextFont {
                                        font: assets.font_slackey.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    TextLayout::new_with_justify(JustifyText::Center),
                                    Transform::from_xyz(0.0, 0.0, 101.0),
                                ));
                            });
                    }

                    // Option Buttons Row 2: FPS Counter, Reactive Mode
                    let fps_label = if bevy_settings.show_fps {
                        "FPS HUD: ON"
                    } else {
                        "FPS HUD: OFF"
                    };
                    let fps_color = if bevy_settings.show_fps {
                        Color::srgb(0.35, 0.55, 0.35)
                    } else {
                        Color::srgb(0.45, 0.45, 0.45)
                    };

                    let reactive_label = if bevy_settings.reactive_mode {
                        "Engine: Reactive (Eco)"
                    } else {
                        "Engine: Continuous"
                    };
                    let reactive_color = if bevy_settings.reactive_mode {
                        Color::srgb(0.25, 0.55, 0.55)
                    } else {
                        Color::srgb(0.35, 0.55, 0.35)
                    };

                    let row2 = [
                        (BevyOptionType::FpsCounter, fps_label, fps_color, -180.0),
                        (BevyOptionType::ReactiveMode, reactive_label, reactive_color, 180.0),
                    ];

                    for (opt_type, label, col, x_pos) in row2 {
                        parent
                            .spawn((
                                BevyOptionButton {
                                    option_type: opt_type,
                                    half_w: 160.0,
                                    half_h: 24.0,
                                },
                                Sprite {
                                    color: col,
                                    custom_size: Some(Vec2::new(320.0, 48.0)),
                                    ..default()
                                },
                                Transform::from_xyz(x_pos, -245.0, 100.0),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text2d::new(label),
                                    TextFont {
                                        font: assets.font_slackey.clone(),
                                        font_size: 16.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                    TextLayout::new_with_justify(JustifyText::Center),
                                    Transform::from_xyz(0.0, 0.0, 101.0),
                                ));
                            });
                    }

                    // Informational Tip
                    parent.spawn((
                        Text2d::new("💡 Tip: Turn VSync OFF to prevent window drag stutter. Reactive mode saves battery when idle."),
                        TextFont {
                            font: assets.font_slackey.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.45, 0.4, 0.35)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Transform::from_xyz(0.0, -315.0, 100.0),
                    ));

                    // Dismiss instruction
                    parent.spawn((
                        Text2d::new("Press [H] or [Esc] to close this window"),
                        TextFont {
                            font: assets.font_slackey.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.3, 0.3)),
                        TextLayout::new_with_justify(JustifyText::Center),
                        Transform::from_xyz(0.0, -365.0, 100.0),
                    ));
                });
        }
    }
}

pub fn apply_bevy_settings_system(
    settings: Res<BevySettings>,
    mut window_q: Query<&mut Window>,
    winit_settings: Option<ResMut<WinitSettings>>,
) {
    if !settings.is_changed() {
        return;
    }

    for mut window in &mut window_q {
        let target_present_mode = if settings.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        if window.present_mode != target_present_mode {
            window.present_mode = target_present_mode;
        }

        let target_mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
        if window.mode != target_mode {
            window.mode = target_mode;
        }
    }

    if let Some(mut winit) = winit_settings {
        if settings.reactive_mode {
            winit.focused_mode = UpdateMode::reactive(std::time::Duration::from_millis(50));
            winit.unfocused_mode = UpdateMode::reactive_low_power(std::time::Duration::from_millis(100));
        } else {
            winit.focused_mode = UpdateMode::Continuous;
            winit.unfocused_mode = UpdateMode::Continuous;
        }
    }
}

