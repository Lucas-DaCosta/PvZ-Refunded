use bevy::{audio::Volume, input::{common_conditions::input_just_pressed, mouse::AccumulatedMouseMotion}, prelude::*, window::{CursorOptions, PrimaryWindow, WindowFocused}};
// use bevy_rapier3d::{geometry::{Collider, Restitution}, plugin::{NoUserData, RapierPhysicsPlugin}, rapier::dynamics::RigidBody, render::RapierDebugRenderPlugin};
use bevy_rapier3d::prelude::*;
use rand::{SeedableRng, seq::IndexedRandom};

fn round_to(value: f32, decimal_places: i32) -> f32 {
    let factor: f32 = 10.0_f32.powi(decimal_places);
    (value * factor).round() / factor
}

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, RapierPhysicsPlugin::<NoUserData>::default(), RapierDebugRenderPlugin::default()));
    app.add_systems(Startup, (
        spawn_camera,
        load_sfx,
        spawn_map,
        spawn_menu,
        spawn_hud,
        setup_physics)
    .chain());
    app.insert_resource(Time::<Fixed>::from_hz(60.));
    app.add_systems(Update, 
        (player_look,
            player_move.after(player_look),
            focus_event,
            toggle_grab.run_if(input_just_pressed(KeyCode::Escape)),
            spawn_ball,
            shoot_ball.before(spawn_ball).before(focus_event),
            update_power_bar,
            update_player_coords,
            update_menu_visibility,
            update_hud_visibility,
            update_menu_sfx,
            update_in_game_sfx));
    app.add_observer(apply_grab);
    app.add_message::<BallSpawn>();
    app.init_resource::<BallData>();
    app.insert_resource(Power {
        charging: false,
        current: 0.
    });
    app.run();
}

#[derive(Component)]
struct Player {
    speed: f32,
    creative: bool,
    velocity: Vec3,
    sneaking: bool
}

impl Default for Player {
    fn default() -> Self {
        Player { speed: 50., creative: false, velocity: Vec3::Y * 20., sneaking: false }
    }
}

#[derive(Event, Deref)]
struct GrabEvent(bool);

#[derive(Message)]
struct BallSpawn {
    position: Vec3,
    velocity: Vec3,
    power: f32
}

#[derive(Resource)]
struct BallData {
    mesh: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
    rng: std::sync::Mutex<rand::rngs::StdRng>
}

impl BallData {
    fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }
    fn material(&self) -> Handle<StandardMaterial> {
        let mut rng = self.rng.lock().unwrap();
        self.materials.choose(&mut *rng).unwrap().clone()
    }
}

impl FromWorld for BallData {
    fn from_world(world: &mut World) -> Self {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(Sphere::new(1.));
        let mut materials = Vec::new();
        let mut mat_assets = world.resource_mut::<Assets<StandardMaterial>>();
        for i in 0..36 {
            let color = Color::hsl((i * 10) as f32, 1., 0.5);
            materials.push(mat_assets.add(StandardMaterial {
                base_color: color,
                ..Default::default()
            }));
        }
        let seed = *b"tunicIsBetterThanYouHEHEHEHAPTDR";
        BallData { mesh, materials, rng: std::sync::Mutex::new(rand::rngs::StdRng::from_seed(seed)) }
    }
}

#[derive(Resource)]
struct Power {
    charging: bool,
    current: f32
}

#[derive(Component)]
struct PowerBar {
    min: f32,
    max: f32
}

const NOT_CHARGING: Color = Color::linear_rgb(0.2, 0.2, 0.2);
const MIN_FILL: f32 = 12.5 / 10.;
const EMPTY_SPACE: f32 = 12.5 - MIN_FILL;

#[derive(Resource)]
struct SoundEffects {
    jump: Handle<AudioSource>,
    shotgun: Handle<AudioSource>,
    main_theme: Handle<AudioSource>,
    in_game_theme: Handle<AudioSource>
}

fn load_sfx(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.insert_resource(SoundEffects {
        jump: asset_server.load("audios/mario-jump.mp3"),
        shotgun: asset_server.load("audios/spas12.mp3"),
        main_theme: asset_server.load("audios/dexter-blood-theme.mp3"),
        in_game_theme: asset_server.load("audios/portal-radio.mp3")
    });
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 50., 0.)),
        Player::default(),
        RigidBody::Dynamic,
        Velocity::zero(),
        Collider::cuboid(2.5, 5., 2.5),
        GravityScale(10.),
        LockedAxes::ROTATION_LOCKED
    )).with_child((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 2.5, 0.)),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

fn spawn_map(
    mut commands: Commands,
    ball_data: Res<BallData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>
) {
    commands.spawn(DirectionalLight::default());
    for h in 0..ball_data.materials.len() {
        commands.spawn((
            Transform::from_translation(Vec3::new((-8. + h as f32) * 2., 5., -30.)),
            Mesh3d(ball_data.mesh()),
            MeshMaterial3d(ball_data.materials[h].clone()),
            Collider::ball(1.)
        ));
    }
    commands.spawn((
        Transform::from_translation(Vec3::new(0., 0., 0.)),
        Mesh3d(meshes.add(Cuboid::new(5000., 1., 5000.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(1., 0., 0.),
            ..Default::default()
        })),
        Collider::cuboid(2500., 0.1, 2500.)
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(30., 10., 0.)),
        Mesh3d(meshes.add(Cuboid::new(10., 100., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 0.),
            ..Default::default()
        })),
        Collider::cuboid(5., 50., 5.)
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(-30., 11., 0.)),
        Mesh3d(meshes.add(Cuboid::new(10., 10., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::cuboid(5., 5., 5.)
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(-30., 20., -20.)),
        Mesh3d(meshes.add(Cuboid::new(10., 10., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::cuboid(5., 5., 5.)
    ));
    commands.spawn((
        Transform::from_translation(Vec3::new(-30., 9.5, 20.)),
        Mesh3d(meshes.add(Cuboid::new(10., 10., 10.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 1., 1.),
            ..Default::default()
        })),
        Collider::cuboid(5., 5., 5.)
    ));
    commands.spawn((
        SceneRoot(asset_server.load("models/amogus/scene.gltf#Scene0")),
        Transform::from_translation(Vec3::new(50., 0., 50.)).with_scale(Vec3::splat(0.04)),
        Collider::cuboid(5., 5., 5.)
    ));
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

fn spawn_menu(
    mut commands: Commands,
    sounds: Res<SoundEffects>
) {
    commands.spawn((
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
    )).with_children(|parent| {
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
            TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.))
        ));
        parent.spawn((
            Text::new("- ZQSD/WASD to move\n\
                            - SPACE to jump\n\
                            - LEFT CTRL or Mouse4 to sneak\n\
                            - SHIFT to sprint\n\
                            - LEFT CLICK to throw ball\n\
                            - A to switch between creative/survival mode\n\
                            - ESHAP to show/unshow this menu"),
            Node {
                margin: UiRect::all(Val::Percent(5.)),
                ..Default::default()
            },
            TextFont {
                font_size: 25.,
                ..Default::default()
            },
            TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.))
        ));
    });
    commands.spawn((
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
    )).with_child((
        Text::new("PAUSE"),
        Node {
            margin: UiRect::all(Val::Auto),
            ..Default::default()
        },
        TextFont {
            font_size: 30.,
            ..Default::default()
        },
        TextColor(Color::linear_rgba(0.75, 0.75, 0.75, 1.))
        )
    );
    commands.spawn((
        MenuSfx,
        AudioPlayer::new(sounds.main_theme.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.2))
    ));
}

fn spawn_hud(
    mut commands: Commands,
    player: Single<&mut Transform, With<Player>>,
    sounds: Res<SoundEffects>
) {
    let pos = player.translation;
    commands.spawn((
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
    )).with_child((
        Node {
            position_type: PositionType::Absolute,
            min_width: Val::Vw(MIN_FILL),
            height: Val::Percent(100.),
            border_radius: BorderRadius::all(Val::VMax(1.)),
            ..Default::default()
        },
        BackgroundColor(NOT_CHARGING),
        PowerBar { min: 1., max: 2.}
    ));
    commands.spawn((
        PlayerHud,
        CoordsHud,
        Text::new(format!("X: {}\nY: {}\nZ: {}", round_to(pos.x, 2), round_to(pos.y, 2), round_to(pos.z, 2))),
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
        }
    ));
    commands.spawn((
        PlayerHud,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        }
    )).with_children(|parent| {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::VMax(1.),
                height: Val::VMax(0.15),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgba(1., 1., 1., 1.))
        ));
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::VMax(0.15),
                height: Val::VMax(1.),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgba(1., 1., 1., 1.))
        ));
    });
    commands.spawn((
        InGameSfx,
        AudioPlayer::new(sounds.in_game_theme.clone()),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.2))
    ));
}

fn update_menu_visibility(
    cursor: Single<&CursorOptions, (With<PrimaryWindow>, Changed<CursorOptions>)>,
    visibility: Query<&mut Visibility, With<MenuUi>>
) {
    for mut vis in visibility {
        if cursor.visible {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn update_menu_sfx(
    cursor: Single<&CursorOptions, (With<PrimaryWindow>, Changed<CursorOptions>)>,
    audios: Query<(Entity, &AudioSink), With<MenuSfx>>,
    mut commands: Commands,
) {
    for (entity, audio) in &audios {
        if !cursor.visible {
            audio.pause();
        } else {
            // Retire le AudioSink → Bevy le recrée et relance depuis le début
            commands.entity(entity).remove::<AudioSink>();
        }
    }
}

fn update_in_game_sfx(
    cursor: Single<&CursorOptions, (With<PrimaryWindow>, Changed<CursorOptions>)>,
    audios: Query<&AudioSink, With<InGameSfx>>
) {
    for audio in audios {
        if cursor.visible {
            audio.pause();
        } else {
            audio.play();
        }
    }
}

fn update_hud_visibility(
    cursor: Single<&CursorOptions, (With<PrimaryWindow>, Changed<CursorOptions>)>,
    visibility: Query<&mut Visibility, With<PlayerHud>>
) {
    for mut vis in visibility {
        if !cursor.visible {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn update_player_coords(
    mut coords: Query<&mut Text, With<CoordsHud>>,
    player: Single<&mut Transform, With<Player>>
) {
    let pos = player.translation;
    for mut text in &mut coords {
        text.0 = format!("X: {}\nY: {}\nZ: {}", round_to(pos.x, 2), round_to(pos.y, 2), round_to(pos.z, 2));
    }
}

fn update_power_bar(
    mut bars: Query<(&mut Node, &PowerBar, &mut BackgroundColor)>,
    power: Res<Power>
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
    window: Single<&Window, With<PrimaryWindow>>
) {
    if !window.focused { return; }
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

fn apply_grab(
    grab: On<GrabEvent>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>
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

fn focus_event(
    mut events: MessageReader<WindowFocused>,
    mut commands: Commands
) {
    if let Some(event) = events.read().last() {
        commands.trigger(GrabEvent(event.focused));
    }
}

fn toggle_grab(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands
) {
    window.focused = !window.focused;
    commands.trigger(GrabEvent(window.focused));
}

fn player_move(
    player: Single<(&mut Transform, &mut Player, &mut Velocity, &mut GravityScale), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut commands: Commands,
    sounds: Res<SoundEffects>
) {
    if cursor.visible {
        return;
    }
    println!("TGTPABOPUT1");
    let speed_multiplier = if input.pressed(KeyCode::ShiftLeft) { 3. } else { 1. };
    let mut delta = Vec3::ZERO;
    let (mut transform, mut player_data, mut velocity, mut gravity) = player.into_inner();
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
    } else if input.pressed(KeyCode::Space) {
        velocity.linvel.y = player_data.velocity.y;
        to_move.y += 1.;
        commands.spawn((
            InGameSfx,
            AudioPlayer::new(sounds.jump.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.1))
        ));
    }
    if player_data.creative && (input.pressed(KeyCode::ControlLeft) || mouse_input.pressed(MouseButton::Forward)) && !player_data.sneaking {
        to_move.y -= 1.;
    }

    if player_data.creative {
        gravity.0 = 0.;
    } else {
        gravity.0 = 10.
    }
    if input.just_pressed(KeyCode::KeyQ) {
        player_data.creative = !player_data.creative;
        *velocity = Velocity::zero();
    }
    to_move = to_move.normalize_or_zero();
    // if !player_data.creative {
    //     if to_move.x > 0. && hitbox.collisions.east { to_move.x = 0.};
    //     if to_move.x < 0. && hitbox.collisions.west { to_move.x = 0.};
    //     if to_move.z > 0. && hitbox.collisions.north { to_move.z = 0.};
    //     if to_move.z < 0. && hitbox.collisions.south { to_move.z = 0.};
    // }
    if player_data.creative {
        transform.translation += to_move * time.delta_secs() * player_data.speed * speed_multiplier;
    } else {
        let futur_move = to_move * player_data.speed * speed_multiplier;
        velocity.linvel.x = futur_move.x;
        velocity.linvel.z = futur_move.z;
    }
    if !player_data.creative && (input.just_pressed(KeyCode::ControlLeft) || mouse_input.just_pressed(MouseButton::Forward)) && !player_data.sneaking {
        transform.translation.y -= 1.;
        player_data.speed *= 0.25;
        player_data.sneaking = true;
    } else if !player_data.creative && player_data.sneaking && (input.just_released(KeyCode::ControlLeft) || mouse_input.just_released(MouseButton::Forward)) {
        transform.translation.y += 1.;
        player_data.speed *= 4.;
        player_data.sneaking = false;
    }
}

fn spawn_ball(
    mut events: MessageReader<BallSpawn>,
    mut commands: Commands,
    ball_data: Res<BallData>,
    sounds: Res<SoundEffects>
) {
    for spawn in events.read() {
        commands.spawn((
            Transform::from_translation(spawn.position),
            Mesh3d(ball_data.mesh()),
            MeshMaterial3d(ball_data.material()),
            Collider::ball(1.),
            RigidBody::Dynamic,
            Velocity {
                linvel: spawn.velocity * spawn.power * 20.,
                angvel: Vec3::ZERO
            },
            GravityScale(50.),
            Ccd::enabled()
            // Velocity(spawn.velocity * spawn.power * 5.),
            // Hitbox::new(Vec3::ZERO, 2., 2., 2.)
        ));
        commands.spawn((
            InGameSfx,
            AudioPlayer::new(sounds.shotgun.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.05))
        ));
    }

}

fn shoot_ball(
    mouse_inputs: Res<ButtonInput<MouseButton>>,
    player: Single<&mut Transform, (With<Player>, Without<Camera3d>)>,
    player_cam: Single<&mut Transform, (With<Camera3d>, Without<Player>)>,
    mut spawner: MessageWriter<BallSpawn>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut power: ResMut<Power>,
    time: Res<Time>
) {
    if cursor.visible {
        return;
    }
    if power.charging {
        if mouse_inputs.just_released(MouseButton::Left) {
            spawner.write(BallSpawn {
                position: player.transform_point(player_cam.translation),
                velocity: player.rotation * player_cam.forward().as_vec3() * 2.5,
                power: (power.current * 2.).exp()
            });
        }
        if mouse_inputs.pressed(MouseButton::Left) {
            power.current += time.delta_secs();
            power.current = power.current.clamp(1., 2.);
        } else {
            power.charging = false;
        }
    }
    if mouse_inputs.just_pressed(MouseButton::Left) {
        power.charging = true;
        power.current = 1.;
    }
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    /* Create the bouncing ball. */
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert(Transform::from_xyz(0.0, 50., 0.0))
        .insert(Mesh3d(meshes.add(Cuboid::new(2.5, 5., 2.5))))
        .insert(MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0., 0., 1.),
            ..Default::default() 
        })))
        .insert(LockedAxes::ROTATION_LOCKED);
}
