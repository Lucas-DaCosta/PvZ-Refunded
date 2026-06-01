use crate::{InGameSfx, SoundEffects};
use bevy::{audio::Volume, prelude::*};
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub creative: bool,
    pub velocity: Vec3,
    pub sneaking: bool,
    pub audios: bool,
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

pub fn player_move(
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

pub fn player_sneak(
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

pub fn player_jump(
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
