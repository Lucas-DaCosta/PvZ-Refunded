use crate::{InGameSfx, SoundEffects, balls::Power, player::Player};
use bevy::{audio::Volume, prelude::*, ui::widget::Text};

#[derive(Component)]
pub struct PowerBar {
    pub min: f32,
    pub max: f32,
}

pub fn round_to(value: f32, decimal_places: i32) -> f32 {
    let factor: f32 = 10.0_f32.powi(decimal_places);
    (value * factor).round() / factor
}

pub const NOT_CHARGING: Color = Color::linear_rgb(0.2, 0.2, 0.2);
pub const MIN_FILL: f32 = 12.5 / 10.;
pub const EMPTY_SPACE: f32 = 12.5 - MIN_FILL;

#[derive(Component)]
pub struct PlayerHud;

#[derive(Component)]
pub struct CoordsHud;

pub fn spawn_hud(
    mut commands: Commands,
    player: Single<&mut Transform, With<Player>>,
    sounds: Res<SoundEffects>,
) {
    let pos = player.translation;
    commands
        .spawn((
            PlayerHud,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(12.5),
                height: Val::Vh(2.5),
                bottom: Val::Vh(5.),
                left: Val::Vw(86.),
                border_radius: BorderRadius::all(Val::VMax(1.)),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgb(0.5, 0.5, 0.5)),
        ))
        .with_child((
            Node {
                position_type: PositionType::Absolute,
                min_width: Val::Vw(MIN_FILL),
                height: Val::Percent(100.),
                border_radius: BorderRadius::all(Val::VMax(1.)),
                ..Default::default()
            },
            BackgroundColor(NOT_CHARGING),
            PowerBar { min: 1., max: 2. },
        ));
    commands.spawn((
        PlayerHud,
        CoordsHud,
        Text::new(format!(
            "X: {}\nY: {}\nZ: {}",
            round_to(pos.x, 2),
            round_to(pos.y, 2),
            round_to(pos.z, 2)
        )),
        TextFont {
            font_size: 15.,
            ..Default::default()
        },
        TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Vh(90.),
            left: Val::Vw(1.),
            ..Default::default()
        },
    ));
    commands
        .spawn((
            PlayerHud,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::VMax(1.),
                    height: Val::VMax(0.15),
                    ..Default::default()
                },
                BackgroundColor(Color::linear_rgba(1., 1., 1., 1.)),
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::VMax(0.15),
                    height: Val::VMax(1.),
                    ..Default::default()
                },
                BackgroundColor(Color::linear_rgba(1., 1., 1., 1.)),
            ));
        });
    commands.spawn((
        InGameSfx,
        AudioPlayer::new(sounds.in_game_theme.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.2)),
    ));
}

pub fn enable_hud_visibility(visibility: Query<&mut Visibility, With<PlayerHud>>) {
    for mut vis in visibility {
        *vis = Visibility::Visible;
    }
}

pub fn disable_hud_visibility(visibility: Query<&mut Visibility, With<PlayerHud>>) {
    for mut vis in visibility {
        *vis = Visibility::Hidden;
    }
}

pub fn update_hud_player_coords(
    mut coords: Query<&mut Text, With<CoordsHud>>,
    player: Single<&mut Transform, With<Player>>,
) {
    let pos = player.translation;
    for mut text in &mut coords {
        text.0 = format!(
            "X: {}\nY: {}\nZ: {}",
            round_to(pos.x, 2),
            round_to(pos.y, 2),
            round_to(pos.z, 2)
        );
    }
}

pub fn update_power_bar(
    mut bars: Query<(&mut Node, &PowerBar, &mut BackgroundColor)>,
    power: Res<Power>,
) {
    for (mut bar, config, mut bg) in &mut bars {
        if !power.charging {
            bg.0 = NOT_CHARGING;
            bar.width = Val::Vw(MIN_FILL);
        } else {
            let percent = (power.current - config.min) / (config.max - config.min);
            bg.0 = Color::linear_rgb(1. - percent, percent, 0.);
            bar.width = Val::Vw(MIN_FILL + percent * EMPTY_SPACE);
        }
    }
}
