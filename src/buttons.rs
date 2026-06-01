use bevy::prelude::*;

use crate::{GameSettings, player::Player};

pub fn handle_button(
    buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut player: Single<&mut Player>,
) {
    for (interaction, mut bg) in buttons {
        match interaction {
            Interaction::Pressed => {
                player.audios = !player.audios;
            }
            Interaction::Hovered => bg.0 = Color::linear_rgba(0.5, 0.5, 0.5, 1.),
            Interaction::None => bg.0 = Color::linear_rgba(0., 0., 0., 0.),
        }
    }
}

pub fn update_button_audio(player: Single<&Player>, settings: Query<(&GameSettings, &mut Text)>) {
    for (setting, mut text) in settings {
        if *setting == GameSettings::Audio {
            let audio = if player.audios { "Enabled" } else { "Disabled" };
            text.0 = format!("Audio : {audio}");
        }
    }
}
