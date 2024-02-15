use ::bevy::prelude::*;

use crate::movement::{LinearMovementBundle, Velocity};

pub struct FollowCamerasPlugin;

impl Plugin for FollowCamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, add_follow_cameras)
            .add_systems(Update, move_cameras);
    }
}

#[derive(Component)]
pub struct PID {
    pub p: f32,
    pub i: f32,
    pub d: f32,
}

impl Default for PID {
    fn default() -> Self {
        Self {
            p: 1.0,
            i: 0.0,
            d: 0.0,
        }
    }
}

#[derive(Component)]
pub struct FollowCameraMe;

#[derive(Component)]
pub struct FollowCameraSettings {
    // pub smoothing: f32, // 0.0 = infinite smoothing, 1.0 = no smoothing
    pub target: Entity,
    pub offset: Vec3,
    pub pid: PID,
    // pub p_heading: f32,
}

impl FollowCameraSettings {
    pub fn new(target: Entity) -> Self {
        Self {
            // smoothing: 0.5,
            target,
            offset: Vec3::new(0.0, 5.0, 10.0),
            pid: PID {
                p: 6.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct FollowCameraBundle {
    pub settings: FollowCameraSettings,
    pub linear_movement: LinearMovementBundle,
    pub camera: Camera3dBundle,
}

impl FollowCameraBundle {
    fn new(entity: Entity) -> Self {
        Self {
            settings: FollowCameraSettings::new(entity),
            linear_movement: LinearMovementBundle::default(),
            camera: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 5.0, 10.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    is_active: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

fn add_follow_cameras(
    mut commands: Commands,
    query: Query<Entity, With<FollowCameraMe>>,
) {
    for entity in query.iter() {
        info!("Adding follow camera for entity: {:?}", entity);
        commands.spawn((FollowCameraBundle::new(entity),));
    }
}

fn move_cameras(
    time: Res<Time>,
    mut query_cameras: Query<
        (&mut Transform, &FollowCameraSettings, &mut Velocity),
        With<Camera>,
    >,
    query_targets: Query<(Entity, &Transform), (With<FollowCameraMe>, Without<Camera>)>,
) {
    for (mut camera_transform, follow_settings, mut velocity) in query_cameras.iter_mut()
    {
        for (target_entity, target_transform) in query_targets.iter() {
            if target_entity == follow_settings.target {
                let target_position = target_transform.translation
                    + target_transform.right() * follow_settings.offset.x
                    + target_transform.forward() * follow_settings.offset.z
                    + target_transform.up() * follow_settings.offset.y;

                // camera_transform.translation = target_position
                //     * follow_camera_bundle.smoothing
                //     + camera_transform.translation
                //         * (1.0 - follow_camera_bundle.smoothing);

                let delta = target_position - camera_transform.translation;
                let distance = delta.length();

                if distance < std::f32::EPSILON {
                    continue;
                }

                let speed = distance * follow_settings.pid.p;
                let direction = delta.normalize_or_zero();
                velocity.value = direction * speed;

                camera_transform.look_at(target_transform.translation, Vec3::Y);
            }
        }
    }
}
