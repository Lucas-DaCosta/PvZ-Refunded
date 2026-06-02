use bevy::prelude::*;

#[derive(Resource, serde::Deserialize)]
pub struct GameSettings {
    pub enable_audio: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { enable_audio: true }
    }
}

#[derive(Debug)]
pub enum SerdeErrors {
    FileNotExist,
    DeserFailed,
}

impl GameSettings {
    pub fn deser(file_name: String) -> Result<GameSettings, SerdeErrors> {
        let file = match std::fs::File::open(file_name) {
            Ok(file) => file,
            Err(_) => {
                return Err(SerdeErrors::FileNotExist);
            }
        };
        let deserialized: GameSettings = match serde_json::de::from_reader(&file) {
            Ok(deserialized) => deserialized,
            Err(err) => {
                eprintln!("failed to deserialise {}", err);
                return Err(SerdeErrors::DeserFailed);
            }
        };
        Ok(deserialized)
    }
}
