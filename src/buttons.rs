use crate::settings::GameSettings;
use bevy::prelude::*;

#[derive(Component, PartialEq)]
pub enum ButtonAction {
    ToggleAudio,
    QuitGame,
}

pub fn handle_button(
    buttons: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut settings: ResMut<GameSettings>,
) {
    for (interaction, mut bg, action) in buttons {
        match interaction {
            Interaction::Pressed => match action {
                ButtonAction::ToggleAudio => settings.enable_audio = !settings.enable_audio,
                _ => {}
            },
            Interaction::Hovered => bg.0 = Color::linear_rgba(0.5, 0.5, 0.5, 1.),
            Interaction::None => bg.0 = Color::linear_rgba(0., 0., 0., 0.),
        }
    }
}

pub fn update_button(
    settings: Res<GameSettings>,
    button_actions: Query<(&ButtonAction, &Children)>,
    mut texts: Query<&mut Text>,
) {
    for (action, children) in button_actions {
        for &child in children {
            if let Ok(mut text) = texts.get_mut(child) {
                match action {
                    ButtonAction::ToggleAudio => {
                        let audio = if settings.enable_audio {
                            "Enabled"
                        } else {
                            "Disabled"
                        };
                        text.0 = format!("Audio : {audio}");
                    }
                    _ => {}
                }
            }
        }
    }
}
