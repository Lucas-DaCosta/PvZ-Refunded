use crate::GameState;
use bevy::{
    prelude::*,
    window::{CursorOptions, PrimaryWindow, WindowFocused},
};

#[derive(Event, Deref)]
pub struct GrabEvent(bool);

pub fn apply_grab(
    grab: On<GrabEvent>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    use bevy::window::CursorGrabMode;
    if **grab {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    } else {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }
}

pub fn focus_event(
    mut events: MessageReader<WindowFocused>,
    mut commands: Commands,
    state: Res<State<GameState>>,
) {
    if let Some(event) = events.read().last() {
        if event.focused && *state.get() == GameState::Playing {
            commands.trigger(GrabEvent(true));
        } else if !event.focused {
            commands.trigger(GrabEvent(false));
        }
    }
}

pub fn toggle_grab(
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    match current_state.get() {
        GameState::Playing => {
            commands.trigger(GrabEvent(false));
            next_state.set(GameState::Paused)
        }
        GameState::Paused => {
            commands.trigger(GrabEvent(true));
            next_state.set(GameState::Playing)
        }
    }
}
