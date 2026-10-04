use bevy::audio::PlaybackSettings;
use bevy::prelude::*;
use rand::Rng;

use crate::assets::GameAssets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEffect {
    Click,
    ThrowDart,
    Pop,
    Bomb,
    Flag,
    Unflag,
    Win,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct PlaySoundEvent(pub SoundEffect);

pub fn audio_event_system(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    mut events: EventReader<PlaySoundEvent>,
    settings: Option<Res<crate::ui::BevySettings>>,
) {
    let Some(assets) = assets else { return };

    if let Some(ref settings) = settings {
        if !settings.sound_enabled {
            events.clear();
            return;
        }
    }

    let mut rng = rand::thread_rng();

    for event in events.read() {
        let handle = match event.0 {
            SoundEffect::Click => assets.sfx_click.clone(),
            SoundEffect::ThrowDart => assets.sfx_dart.clone(),
            SoundEffect::Flag => assets.sfx_flag.clone(),
            SoundEffect::Unflag => assets.sfx_unflag.clone(),
            SoundEffect::Win => assets.sfx_win.clone(),
            SoundEffect::Pop => {
                let idx = rng.gen_range(0..assets.sfx_pops.len());
                assets.sfx_pops[idx].clone()
            }
            SoundEffect::Bomb => {
                let idx = rng.gen_range(0..assets.sfx_bombs.len());
                assets.sfx_bombs[idx].clone()
            }
        };

        commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN));
    }
}
