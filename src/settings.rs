use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSettings {
    pub enable_audio: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { enable_audio: true }
    }
}
