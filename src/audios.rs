use crate::settings::GameSettings;
use bevy::prelude::*;

#[derive(Resource)]
pub struct SoundEffects {
    pub jump: Handle<AudioSource>,
    pub shotgun: Handle<AudioSource>,
    pub main_theme: Handle<AudioSource>,
    pub in_game_theme: Handle<AudioSource>,
}

pub fn load_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundEffects {
        jump: asset_server.load("audios/mario-jump.mp3"),
        shotgun: asset_server.load("audios/spas12.mp3"),
        main_theme: asset_server.load("audios/dexter-blood-theme.mp3"),
        in_game_theme: asset_server.load("audios/portal-radio.mp3"),
    });
}

#[derive(Component)]
pub struct MenuSfx;

#[derive(Component)]
pub struct InGameSfx;

pub fn disable_enable_sfx(audios: Query<&mut AudioSink>, settings: Res<GameSettings>) {
    for mut audio in audios {
        if settings.enable_audio {
            audio.unmute();
        } else {
            audio.mute();
        }
    }
}

pub fn disable_menu_sfx(audios: Query<&AudioSink, With<MenuSfx>>) {
    for audio in &audios {
        audio.pause();
    }
}

pub fn enable_menu_sfx(audios: Query<&AudioSink, With<MenuSfx>>) {
    for audio in &audios {
        audio.play();
    }
}

pub fn disable_in_game_sfx(audios: Query<&AudioSink, With<InGameSfx>>) {
    for audio in audios {
        audio.pause();
    }
}

pub fn enable_in_game_sfx(audios: Query<&AudioSink, With<InGameSfx>>) {
    for audio in audios {
        audio.play();
    }
}
