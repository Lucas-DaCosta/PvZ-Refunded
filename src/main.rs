use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    ui::widget::Text,
    window::{CursorOptions, PrimaryWindow, WindowFocused},
};
use bevy_rapier3d::prelude::*;

mod audios;
mod balls;
mod camera;
mod hud;
mod map;
mod pause_menu;
mod player;
use crate::audios::*;
use crate::balls::*;
use crate::camera::*;
use crate::hud::*;
use crate::map::*;
use crate::pause_menu::*;
use crate::player::*;

fn round_to(value: f32, decimal_places: i32) -> f32 {
    let factor: f32 = 10.0_f32.powi(decimal_places);
    (value * factor).round() / factor
}

#[derive(States, Debug, Clone, Hash, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
    ));
    app.init_state::<GameState>();
    app.add_systems(
        Startup,
        (spawn_camera, load_sfx, spawn_map, spawn_menu, spawn_hud).chain(),
    );
    app.add_systems(
        OnEnter(GameState::Playing),
        (enable_hud_visibility, enable_in_game_sfx),
    );
    app.add_systems(
        OnExit(GameState::Playing),
        (disable_hud_visibility, disable_in_game_sfx),
    );
    app.add_systems(
        OnEnter(GameState::Paused),
        (enable_menu_visibility, enable_menu_sfx, pause_game),
    );
    app.add_systems(
        OnExit(GameState::Paused),
        (disable_menu_visibility, disable_menu_sfx, unpause_game),
    );
    app.insert_resource(Time::<Fixed>::from_hz(60.));
    app.add_systems(
        Update,
        (
            player_look,
            switch_gamemode,
            player_move.run_if(in_state(GameState::Playing)),
            player_sneak
                .before(PhysicsSet::SyncBackend)
                .run_if(in_state(GameState::Playing)),
            player_jump
                .before(PhysicsSet::SyncBackend)
                .run_if(in_state(GameState::Playing)),
            focus_event,
            toggle_grab.run_if(input_just_pressed(KeyCode::Escape)),
            spawn_ball,
            shoot_ball.run_if(in_state(GameState::Playing)),
            update_power_bar,
            update_hud_player_coords,
            rotate_model,
            handle_button,
            disable_enable_sfx,
            update_button_audio,
        )
            .chain(),
    );
    app.add_observer(apply_grab);
    app.add_message::<BallSpawn>();
    app.init_resource::<BallData>();
    app.insert_resource(Power {
        charging: false,
        current: 0.,
    });
    app.run();
}

#[derive(Event, Deref)]
struct GrabEvent(bool);

#[derive(Component, PartialEq)]
enum GameSettings {
    Audio,
}

fn handle_button(
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

fn update_button_audio(player: Single<&Player>, settings: Query<(&GameSettings, &mut Text)>) {
    for (setting, mut text) in settings {
        if *setting == GameSettings::Audio {
            let audio = if player.audios { "Enabled" } else { "Disabled" };
            text.0 = format!("Audio : {audio}");
        }
    }
}

fn apply_grab(grab: On<GrabEvent>, mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    use bevy::window::CursorGrabMode;
    if **grab {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    } else {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }
}

fn focus_event(mut events: MessageReader<WindowFocused>, mut commands: Commands) {
    if let Some(event) = events.read().last() {
        commands.trigger(GrabEvent(event.focused));
    }
}

fn toggle_grab(
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

fn switch_gamemode(
    player: Single<(Entity, &mut Player, &mut Velocity), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let (entity, mut player_data, mut velocity) = player.into_inner();
    if input.just_pressed(KeyCode::KeyQ) {
        player_data.creative = !player_data.creative;
        *velocity = Velocity::zero();
        if player_data.creative {
            commands
                .entity(entity)
                .insert((RigidBodyDisabled, ColliderDisabled));
        } else {
            commands
                .entity(entity)
                .remove::<(RigidBodyDisabled, ColliderDisabled)>();
        }
    }
}

// fn toggle_physics(
//     mut config: Single<&mut RapierConfiguration>,
//     cursor: Single<&CursorOptions, (With<PrimaryWindow>, Changed<CursorOptions>)>,
// ) {
//     if cursor.visible {
//         config.physics_pipeline_active = false;
//     } else {
//         config.physics_pipeline_active = true;
//     }
// }

fn pause_game(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn unpause_game(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}
