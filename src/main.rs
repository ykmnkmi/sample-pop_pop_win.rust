use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PresentMode;

use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use pop_pop_win::animation::{tick_pending_pops, update_flipbooks};
use pop_pop_win::assets::load_game_assets;
use pop_pop_win::audio::{audio_event_system, PlaySoundEvent};
use pop_pop_win::board::{
    reset_game_listener, spawn_board, update_tile_visual_system, ResetGameEvent,
    UpdateTileVisualEvent, VIRTUAL_HEIGHT, VIRTUAL_WIDTH,
};
use pop_pop_win::game::Game;
use pop_pop_win::input::{cursor_system, mouse_click_system, VirtualCursorPos};
use pop_pop_win::storage::HighScores;
use pop_pop_win::ui::{
    apply_bevy_settings_system, manage_about_modal_system, setup_ui, ui_interaction_system,
    update_hud_text, AboutModalState, BevySettings, FpsTracker,
};
use pop_pop_win::{GameDifficulty, GameWrapper};

fn main() {
    let initial_diff = GameDifficulty::Hard;
    let (cols, rows, bombs) = initial_diff.dimensions();

    App::new()
        .add_plugins(EmbeddedAssetPlugin {
            mode: PluginMode::ReplaceDefault,
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pop, Pop, Win! - Bevy Edition".into(),
                        resolution: (1024.0_f32, 768.0_f32).into(),
                        resizable: true,
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(ClearColor(Color::srgb_u8(180, 173, 127))) // 0xb4ad7f
        .insert_resource(initial_diff)
        .insert_resource(GameWrapper(Game::new(cols, rows, bombs)))
        .insert_resource(HighScores::load())
        .insert_resource(VirtualCursorPos::default())
        .insert_resource(AboutModalState::default())
        .insert_resource(BevySettings::default())
        .insert_resource(FpsTracker::default())
        .add_event::<PlaySoundEvent>()
        .add_event::<UpdateTileVisualEvent>()
        .add_event::<ResetGameEvent>()
        .add_systems(
            Startup,
            (
                setup_camera,
                load_game_assets,
                (spawn_board, setup_ui).after(load_game_assets),
            ),
        )
        .add_systems(
            Update,
            (
                cursor_system,
                mouse_click_system,
                update_flipbooks,
                tick_pending_pops,
                update_tile_visual_system,
                reset_game_listener,
                audio_event_system,
                ui_interaction_system,
                update_hud_text,
                manage_about_modal_system,
                apply_bevy_settings_system,
                #[cfg(not(target_arch = "wasm32"))]
                set_window_icon,
            ),
        )
        .run();
}

#[cfg(not(target_arch = "wasm32"))]
fn set_window_icon(
    windows: NonSend<bevy::winit::WinitWindows>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Ok(primary_entity) = primary_window.get_single() else { return };
    let Some(primary) = windows.get_window(primary_entity) else { return };

    let image = image::load_from_memory(include_bytes!("../assets/icon.png"))
        .expect("Failed to decode balloon icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    if let Ok(icon) = winit::window::Icon::from_rgba(rgba, width, height) {
        primary.set_window_icon(Some(icon));
        *done = true;
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: VIRTUAL_WIDTH,
                min_height: VIRTUAL_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        },
    ));
}
