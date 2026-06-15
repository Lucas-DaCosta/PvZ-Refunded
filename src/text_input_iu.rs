use crate::pause_menu::MenuUi;
use bevy::{input_focus::InputFocus, prelude::*};
use bevy_ui_text_input::*;

pub fn spawn_input_ui(mut commands: Commands) {
    commands
        .spawn((
            MenuUi,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(30.),
                height: Val::Vh(7.5),
                left: Val::Vw(40.),
                top: Val::Vh(0.),
                ..Default::default()
            },
        ))
        .with_child((
            Button,
            TextInputNode {
                focus_on_pointer_down: true,
                mode: TextInputMode::SingleLine,
                max_chars: Some(20),
                ..Default::default()
            },
            TextInputPrompt::new("Enter your text..."),
            Node {
                width: Val::Px(150.),
                height: Val::Px(50.),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgba(0., 0., 0., 1.)),
        ));
}

pub fn handle_input_ui(
    text_areas: Query<
        (&Interaction, Entity, &mut BackgroundColor),
        (Changed<Interaction>, (With<TextInputNode>, With<Button>)),
    >,
    mut focus: ResMut<InputFocus>,
) {
    for (interaction, text, mut bg) in text_areas {
        match interaction {
            Interaction::Pressed => focus.0 = Some(text),
            Interaction::Hovered => bg.0 = Color::linear_rgba(1., 0., 0., 0.5),
            Interaction::None => bg.0 = Color::linear_rgba(0., 0., 0., 1.),
        }
    }
}
