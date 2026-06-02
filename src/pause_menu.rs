use crate::{MenuSfx, SoundEffects, buttons::ButtonAction};
use bevy::{audio::Volume, prelude::*, ui::widget::Text};

#[derive(Component)]
pub struct MenuUi;

pub fn spawn_menu(mut commands: Commands, sounds: Res<SoundEffects>) {
    commands
        .spawn((
            MenuUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(30.),
                height: Val::Vh(90.),
                bottom: Val::Vh(5.),
                left: Val::Vw(1.5),
                top: Val::Vh(5.),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::VMax(1.)),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgba(0., 0., 0., 0.67)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Controls :"),
                Node {
                    margin: UiRect::all(Val::Percent(5.)),
                    ..Default::default()
                },
                TextFont {
                    font_size: 30.,
                    ..Default::default()
                },
                TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.)),
            ));
            parent.spawn((
                Text::new(
                    "- ZQSD/WASD to move\n\
                            - SPACE to jump\n\
                            - LEFT CTRL or Mouse4 to sneak\n\
                            - SHIFT to sprint\n\
                            - LEFT CLICK to throw ball\n\
                            - A to switch between creative/survival mode\n\
                            - ESHAP to show/unshow this menu",
                ),
                Node {
                    margin: UiRect::all(Val::Percent(5.)),
                    ..Default::default()
                },
                TextFont {
                    font_size: 25.,
                    ..Default::default()
                },
                TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.)),
            ));
        });
    commands
        .spawn((
            MenuUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(15.),
                height: Val::Vh(5.),
                bottom: Val::Vh(47.5),
                left: Val::Vw(42.5),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::VMax(1.)),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgba(0., 0., 0., 0.67)),
            Visibility::Hidden,
        ))
        .with_child((
            Text::new("PAUSE"),
            Node {
                margin: UiRect::all(Val::Auto),
                ..Default::default()
            },
            TextFont {
                font_size: 30.,
                ..Default::default()
            },
            TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.)),
        ));
    commands
        .spawn((
            MenuUi,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(30.),
                height: Val::Vh(90.),
                bottom: Val::Vh(5.),
                left: Val::Vw(68.5),
                top: Val::Vh(5.),
                flex_direction: FlexDirection::Column,
                border_radius: BorderRadius::all(Val::VMax(1.)),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgba(0., 0., 0., 0.67)),
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Settings :"),
                Node {
                    margin: UiRect::all(Val::Percent(5.)),
                    ..Default::default()
                },
                TextFont {
                    font_size: 30.,
                    ..Default::default()
                },
                TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.)),
            ));
            parent
                .spawn((
                    Button,
                    ButtonAction::ToggleAudio,
                    Node {
                        width: Val::Percent(90.),
                        height: Val::Percent(10.),
                        margin: UiRect::all(Val::Percent(5.)),
                        border: UiRect::vertical(Val::Px(2.)),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    BorderColor::all(Color::linear_rgba(1., 1., 1., 1.)),
                ))
                .with_child((
                    Text::new("Audio : Enabled"),
                    TextFont {
                        font_size: 20.,
                        ..Default::default()
                    },
                    Node {
                        margin: UiRect::all(Val::Percent(2.5)),
                        ..Default::default()
                    },
                ));
        });
    commands.spawn((
        MenuSfx,
        AudioPlayer::new(sounds.main_theme.clone()),
        PlaybackSettings::LOOP
            .with_volume(Volume::Linear(0.2))
            .paused(),
    ));
}

pub fn enable_menu_visibility(visibility: Query<&mut Visibility, With<MenuUi>>) {
    for mut vis in visibility {
        *vis = Visibility::Visible;
    }
}

pub fn disable_menu_visibility(visibility: Query<&mut Visibility, With<MenuUi>>) {
    for mut vis in visibility {
        *vis = Visibility::Hidden;
    }
}
