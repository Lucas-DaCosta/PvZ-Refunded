use bevy::{
    audio::Volume,
    input::{common_conditions::input_just_pressed, mouse::AccumulatedMouseMotion},
    prelude::*,
    ui::widget::Text,
    window::{CursorOptions, PrimaryWindow, WindowFocused},
};

use bevy_rapier3d::prelude::*;

mod balls;

use crate::balls::*;

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
            update_player_coords,
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

#[derive(Component)]
struct Player {
    speed: f32,
    creative: bool,
    velocity: Vec3,
    sneaking: bool,
    audios: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            speed: 50.,
            creative: false,
            velocity: Vec3::Y * 40.,
            sneaking: false,
            audios: true,
        }
    }
}

#[derive(Event, Deref)]
struct GrabEvent(bool);

#[derive(Component)]
struct PowerBar {
    min: f32,
    max: f32,
}

const NOT_CHARGING: Color = Color::linear_rgb(0.2, 0.2, 0.2);
const MIN_FILL: f32 = 12.5 / 10.;
const EMPTY_SPACE: f32 = 12.5 - MIN_FILL;

#[derive(Resource)]
struct SoundEffects {
    jump: Handle<AudioSource>,
    shotgun: Handle<AudioSource>,
    main_theme: Handle<AudioSource>,
    in_game_theme: Handle<AudioSource>,
}

fn load_sfx(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundEffects {
        jump: asset_server.load("audios/mario-jump.mp3"),
        shotgun: asset_server.load("audios/spas12.mp3"),
        main_theme: asset_server.load("audios/dexter-blood-theme.mp3"),
        in_game_theme: asset_server.load("audios/portal-radio.mp3"),
    });
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn((
            Transform::from_translation(Vec3::new(0., 50., 0.)),
            Player::default(),
            RigidBody::Dynamic,
            Velocity::zero(),
            Collider::cuboid(2.5, 5., 2.5),
            GravityScale(15.),
            LockedAxes::ROTATION_LOCKED,
            Friction {
                coefficient: 0.,
                combine_rule: CoefficientCombineRule::Min,
            },
            InheritedVisibility::default(),
        ))
        .with_child((
            Camera3d::default(),
            Transform::from_translation(Vec3::new(0., 2.5, 0.)),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
}

#[derive(Component)]
struct RotateModel(Vec3);

impl Default for RotateModel {
    fn default() -> Self {
        Self(Vec3::new(0., 1., 0.))
    }
}

fn spawn_map(
    mut commands: Commands,
    ball_data: Res<BallData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(DirectionalLight::default());
    for h in 0..ball_data.materials.len() {
        commands.spawn((
            Transform::from_translation(Vec3::new((-8. + h as f32) * 2., 5., -30.)),
            Mesh3d(ball_data.mesh()),
            MeshMaterial3d(ball_data.materials[h].clone()),
            Collider::ball(1.),
        ));
    }
    commands.spawn((
        Transform::from_translation(Vec3::new(0., -0.1, 0.)),
        Mesh3d(meshes.add(Cuboid::new(5000., 0.2, 5000.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1., 0., 0.),
            ..Default::default()
        })),
        Collider::cuboid(2500., 0.1, 2500.),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(30., 10., 0.)),
        Mesh3d(meshes.add(Cuboid::new(10., 100., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 0.),
            ..Default::default()
        })),
        Collider::cuboid(5., 50., 5.),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(-30., 11., 0.)),
        Mesh3d(meshes.add(Cuboid::new(10., 10., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::cuboid(5., 5., 5.),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(-30., 20., -20.)),
        Mesh3d(meshes.add(Cuboid::new(10., 10., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::cuboid(5., 5., 5.),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(-30., 9.5, 20.)),
        Mesh3d(meshes.add(Cuboid::new(10., 10., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::cuboid(5., 5., 5.),
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 25., 70.)),
        Mesh3d(meshes.add(Cuboid::new(100., 50., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::cuboid(50., 25., 5.),
    ));
    commands.spawn((
        RotateModel(Vec3::NEG_Z * 2.5),
        SceneRoot(asset_server.load("models/peashooter-gw/scene.gltf#Scene0")),
        Transform::from_translation(Vec3::new(65., 0., 50.)).with_scale(Vec3::splat(7.)),
        AsyncSceneCollider {
            shape: Some(ComputedColliderShape::ConvexDecomposition(
                VHACDParameters {
                    resolution: 32,
                    max_convex_hulls: 4,
                    ..Default::default()
                },
            )),
            named_shapes: Default::default(),
        },
    ));
    commands.spawn((
        RotateModel::default(),
        SceneRoot(asset_server.load("models/amogus/scene.gltf#Scene0")),
        Transform::from_translation(Vec3::new(80., 0., 50.)).with_scale(Vec3::splat(6.)),
        AsyncSceneCollider {
            shape: Some(ComputedColliderShape::ConvexHull),
            named_shapes: Default::default(),
        },
    ));
    commands.spawn((
        RotateModel(Vec3::NEG_Y * 10.),
        Transform::from_translation(Vec3::new(95., 5., 50.)),
        Mesh3d(meshes.add(Capsule3d::new(2.5, 5.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 0., 1.),
            ..Default::default()
        })),
        Collider::capsule_y(2.5, 2.5),
    ));
    commands.spawn((
        RotateModel(Vec3::new(-0.25, -0.25, 0.)),
        Transform::from_translation(Vec3::new(110., 5., 50.)),
        Mesh3d(meshes.add(Capsule3d::new(2.5, 5.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::capsule_y(2.5, 2.5),
    ));
    commands.spawn((
        RotateModel(Vec3::new(-0.5, 1., 2.)),
        Transform::from_translation(Vec3::new(135., 5., 50.)),
        Mesh3d(meshes.add(Capsule3d::new(2.5, 5.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1., 0., 1.),
            ..Default::default()
        })),
        Collider::capsule_y(2.5, 2.5),
    ));
}

fn rotate_model(models: Query<(&mut Transform, &RotateModel), With<RotateModel>>, time: Res<Time>) {
    for (mut model, movement) in models {
        let speed = movement.0.length() * time.delta_secs();
        model.rotate(Quat::from_axis_angle(movement.0.normalize(), speed));
    }
}

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct PlayerHud;

#[derive(Component)]
struct CoordsHud;

#[derive(Component)]
struct MenuSfx;

#[derive(Component)]
struct InGameSfx;

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

fn spawn_menu(mut commands: Commands, sounds: Res<SoundEffects>) {
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
                    Node {
                        width: Val::Percent(90.),
                        height: Val::Percent(10.),
                        margin: UiRect::all(Val::Percent(5.)),
                        border: UiRect::vertical(Val::Px(2.)),
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.)),
                    BorderColor::all(Color::linear_rgba(1., 1., 1., 1.)),
                ))
                .with_child((
                    GameSettings::Audio,
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

fn update_button_audio(player: Single<&Player>, settings: Query<(&GameSettings, &mut Text)>) {
    for (setting, mut text) in settings {
        if *setting == GameSettings::Audio {
            let audio = if player.audios { "Enabled" } else { "Disabled" };
            text.0 = format!("Audio : {audio}");
        }
    }
}

fn spawn_hud(
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

fn enable_menu_visibility(visibility: Query<&mut Visibility, With<MenuUi>>) {
    for mut vis in visibility {
        *vis = Visibility::Visible;
    }
}

fn disable_menu_visibility(visibility: Query<&mut Visibility, With<MenuUi>>) {
    for mut vis in visibility {
        *vis = Visibility::Hidden;
    }
}

fn disable_enable_sfx(audios: Query<&mut AudioSink>, player: Single<&Player>) {
    for mut audio in audios {
        if player.audios {
            audio.unmute();
        } else {
            audio.mute();
        }
    }
}

fn disable_menu_sfx(audios: Query<&AudioSink, With<MenuSfx>>) {
    for audio in &audios {
        audio.pause();
    }
}

fn enable_menu_sfx(audios: Query<&AudioSink, With<MenuSfx>>) {
    for audio in &audios {
        audio.play();
    }
}

fn disable_in_game_sfx(audios: Query<&AudioSink, With<InGameSfx>>) {
    for audio in audios {
        audio.pause();
    }
}

fn enable_in_game_sfx(audios: Query<&AudioSink, With<InGameSfx>>) {
    for audio in audios {
        audio.play();
    }
}

fn enable_hud_visibility(visibility: Query<&mut Visibility, With<PlayerHud>>) {
    for mut vis in visibility {
        *vis = Visibility::Visible;
    }
}

fn disable_hud_visibility(visibility: Query<&mut Visibility, With<PlayerHud>>) {
    for mut vis in visibility {
        *vis = Visibility::Hidden;
    }
}

fn update_player_coords(
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

fn update_power_bar(
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

// player_look — sépare yaw (parent) et pitch (enfant caméra)
fn player_look(
    mut player: Single<&mut Transform, (With<Player>, Without<Camera3d>)>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<Player>)>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if !window.focused {
        return;
    }
    let dt = time.delta_secs();
    let sensitivity = 100. / window.width().min(window.height());

    // Yaw sur le parent (joueur)
    let (yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);
    let new_yaw = yaw - mouse_motion.delta.x * dt * sensitivity;
    player.rotation = Quat::from_rotation_y(new_yaw);

    // Pitch sur la caméra enfant
    let (_, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let new_pitch = (pitch - mouse_motion.delta.y * dt * sensitivity).clamp(-1.57, 1.57);
    camera.rotation = Quat::from_rotation_x(new_pitch);
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

fn player_move(
    player: Single<(&mut Transform, &Player, &mut Velocity), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    let speed_multiplier = if input.pressed(KeyCode::ShiftLeft) {
        2.
    } else {
        1.
    };
    let mut delta = Vec3::ZERO;
    let (mut transform, player_data, mut velocity) = player.into_inner();
    if input.pressed(KeyCode::KeyA) {
        delta.x -= 1.;
    }
    if input.pressed(KeyCode::KeyD) {
        delta.x += 1.;
    }
    if input.pressed(KeyCode::KeyW) {
        delta.z += 1.;
    }
    if input.pressed(KeyCode::KeyS) {
        delta.z -= 1.;
    }
    let forward = transform.forward().as_vec3() * delta.z;
    let right = transform.right().as_vec3() * delta.x;
    let mut to_move = forward + right;
    to_move.y = 0.;
    // fly or jump depending on player gamemode
    if player_data.creative && input.pressed(KeyCode::Space) {
        to_move.y += 1.;
    }
    if player_data.creative
        && (input.pressed(KeyCode::ControlLeft) || mouse_input.pressed(MouseButton::Forward))
        && !player_data.sneaking
    {
        to_move.y -= 1.;
    }
    to_move = to_move.normalize_or_zero();
    if player_data.creative {
        transform.translation += to_move * time.delta_secs() * player_data.speed * speed_multiplier;
    } else {
        let futur_move = to_move * player_data.speed * speed_multiplier;
        velocity.linvel.x = futur_move.x;
        velocity.linvel.z = futur_move.z;
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

fn player_sneak(
    player: Single<(&mut Player, &mut Transform, Entity), With<Player>>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let (mut player_data, mut transform, entity) = player.into_inner();
    if !player_data.creative
        && (input.just_pressed(KeyCode::ControlLeft)
            || mouse_input.just_pressed(MouseButton::Forward))
        && !player_data.sneaking
    {
        player_data.speed *= 0.25;
        player_data.sneaking = true;
        commands
            .entity(entity)
            .insert(Collider::cuboid(2.5, 2.5, 2.5));
        transform.translation.y -= 1.25;
    } else if !player_data.creative
        && player_data.sneaking
        && (input.just_released(KeyCode::ControlLeft)
            || mouse_input.just_released(MouseButton::Forward))
    {
        player_data.speed *= 4.;
        player_data.sneaking = false;
        commands
            .entity(entity)
            .insert(Collider::cuboid(2.5, 5., 2.5));
        transform.translation.y = 0.;
    }
}

fn player_jump(
    player: Single<(&mut Player, &mut Velocity), With<Player>>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    sounds: Res<SoundEffects>,
) {
    let (player_data, mut velocity) = player.into_inner();
    if !player_data.creative && input.pressed(KeyCode::Space) && velocity.linvel.y == 0. {
        velocity.linvel.y = player_data.velocity.y;
        commands.spawn((
            InGameSfx,
            AudioPlayer::new(sounds.jump.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.1)),
        ));
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
