use crate::balls::BallData;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct RotateModel(Vec3);

impl Default for RotateModel {
    fn default() -> Self {
        Self(Vec3::new(0., 1., 0.))
    }
}

pub fn spawn_map(
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
        Transform::from_translation(Vec3::new(30., 50., 0.)),
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

pub fn rotate_model(
    models: Query<(&mut Transform, &RotateModel), With<RotateModel>>,
    time: Res<Time>,
) {
    for (mut model, movement) in models {
        let speed = movement.0.length() * time.delta_secs();
        model.rotate(Quat::from_axis_angle(movement.0.normalize(), speed));
    }
}
