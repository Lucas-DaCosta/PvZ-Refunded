use crate::player::Player;
use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::PrimaryWindow};
use bevy_rapier3d::prelude::*;

pub fn spawn_camera(mut commands: Commands) {
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

// player_look — split yaw (parent) and pitch (child camera)
pub fn player_look(
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

    // Yaw on parent (player)
    let (yaw, _, _) = player.rotation.to_euler(EulerRot::YXZ);
    let new_yaw = yaw - mouse_motion.delta.x * dt * sensitivity;
    player.rotation = Quat::from_rotation_y(new_yaw);

    // Pitch on child camera
    let (_, pitch, _) = camera.rotation.to_euler(EulerRot::YXZ);
    let new_pitch = (pitch - mouse_motion.delta.y * dt * sensitivity).clamp(-1.57, 1.57);
    camera.rotation = Quat::from_rotation_x(new_pitch);
}
