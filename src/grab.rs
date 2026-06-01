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

pub fn focus_event(mut events: MessageReader<WindowFocused>, mut commands: Commands) {
    if let Some(event) = events.read().last() {
        commands.trigger(GrabEvent(event.focused));
    }
}

pub fn toggle_grab(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    window.focused = !window.focused;
    commands.trigger(GrabEvent(window.focused));
    match current_state.get() {
        GameState::Playing => next_state.set(GameState::Paused),
        GameState::Paused => next_state.set(GameState::Playing),
    }
}
