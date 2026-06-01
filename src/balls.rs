use crate::{InGameSfx, Player, SoundEffects};
use bevy::{audio::Volume, prelude::*};
use bevy_rapier3d::prelude::*;
use rand::{SeedableRng, seq::IndexedRandom};

#[derive(Message)]
pub struct BallSpawn {
    pub position: Vec3,
    pub velocity: Vec3,
    pub power: f32,
}

#[derive(Resource)]
pub struct BallData {
    pub mesh: Handle<Mesh>,
    pub materials: Vec<Handle<StandardMaterial>>,
    pub rng: std::sync::Mutex<rand::rngs::StdRng>,
}

impl BallData {
    pub fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }
    pub fn material(&self) -> Handle<StandardMaterial> {
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
        BallData {
            mesh,
            materials,
            rng: std::sync::Mutex::new(rand::rngs::StdRng::from_seed(seed)),
        }
    }
}

#[derive(Resource)]
pub struct Power {
    pub charging: bool,
    pub current: f32,
}

pub fn shoot_ball(
    mouse_inputs: Res<ButtonInput<MouseButton>>,
    player: Single<&mut Transform, (With<Player>, Without<Camera3d>)>,
    player_cam: Single<&mut Transform, (With<Camera3d>, Without<Player>)>,
    mut spawner: MessageWriter<BallSpawn>,
    mut power: ResMut<Power>,
    time: Res<Time>,
) {
    if power.charging {
        if mouse_inputs.just_released(MouseButton::Left) {
            spawner.write(BallSpawn {
                position: player.transform_point(player_cam.translation),
                velocity: player.rotation * player_cam.forward().as_vec3() * 2.5,
                power: (power.current * 2.).exp(),
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

pub fn spawn_ball(
    mut events: MessageReader<BallSpawn>,
    mut commands: Commands,
    ball_data: Res<BallData>,
    sounds: Res<SoundEffects>,
) {
    for spawn in events.read() {
        commands.spawn((
            Transform::from_translation(spawn.position),
            Mesh3d(ball_data.mesh()),
            MeshMaterial3d(ball_data.material()),
            Collider::ball(1.),
            RigidBody::Dynamic,
            Velocity {
                linvel: spawn.velocity * spawn.power * 5.,
                angvel: Vec3::ZERO,
            },
            Restitution {
                coefficient: 0.7,
                combine_rule: CoefficientCombineRule::Max,
            },
            Damping {
                linear_damping: 0.25,
                angular_damping: 0.5,
            },
            GravityScale(50.),
            Ccd::enabled(),
        ));
        commands.spawn((
            InGameSfx,
            AudioPlayer::new(sounds.shotgun.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.05)),
        ));
    }
}
