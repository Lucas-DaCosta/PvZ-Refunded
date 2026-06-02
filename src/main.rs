use bevy::input_focus::InputDispatchPlugin;
use bevy::input_focus::tab_navigation::TabNavigationPlugin;
use bevy::ui_widgets::UiWidgetsPlugins;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_rapier3d::prelude::*;

mod audios;
mod balls;
mod buttons;
mod camera;
mod grab;
mod hud;
mod map;
mod pause_menu;
mod player;
mod settings;
use crate::audios::*;
use crate::balls::*;
use crate::buttons::*;
use crate::camera::*;
use crate::grab::*;
use crate::hud::*;
use crate::map::*;
use crate::pause_menu::*;
use crate::player::*;
use crate::settings::*;

#[derive(States, Debug, Clone, Hash, PartialEq, Eq, Default)]
enum GameState {
    #[default]
    Playing,
    Paused,
}

fn main() {
    let mut app = App::new();
    let settings = match GameSettings::deser("./saves/settings.json".to_owned()) {
        Ok(deser) => deser,
        Err(err) => {
            println!("Failed to load settings file : {:#?}", err);
            GameSettings::default()
        }
    };
    app.add_plugins((
        DefaultPlugins,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
        UiWidgetsPlugins,
        InputDispatchPlugin,
        TabNavigationPlugin,
    ));
    app.init_state::<GameState>();
    app.add_systems(
        Startup,
        (
            spawn_camera,
            load_sfx,
            spawn_map,
            spawn_menu,
            spawn_hud,
            setup_ui,
        )
            .chain(),
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
            update_button,
        )
            .chain(),
    );
    app.add_systems(Update, update_slider_thumb);
    app.add_observer(apply_grab);
    app.add_observer(sync_slider_value);
    app.add_message::<BallSpawn>();
    app.init_resource::<BallData>();
    app.insert_resource(settings);
    app.insert_resource(Power {
        charging: false,
        current: 0.,
    });
    app.run();
}

fn pause_game(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn unpause_game(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}
