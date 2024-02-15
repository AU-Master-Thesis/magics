use ::bevy::prelude::*;

use crate::movement::{AngularVelocity, Local, Orbit, OrbitMovementBundle, Velocity};

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
    // pub target_angle: f32,
    pub pid: PID,
    // pub p_heading: f32,
}

impl FollowCameraSettings {
    pub fn new(target: Entity) -> Self {
        Self {
            // smoothing: 0.5,
            target,
            offset: Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0,
            // target_angle: 0.0,
            pid: PID {
                p: 1.0,
                ..Default::default()
            },
        }
    }
}

#[derive(Bundle)]
pub struct FollowCameraBundle {
    pub settings: FollowCameraSettings,
    pub movement: OrbitMovementBundle,
    pub velocity: Velocity,
    pub camera: Camera3dBundle,
}

impl FollowCameraBundle {
    fn new(entity: Entity, target: Option<&Transform>) -> Self {
        // let target = target.unwrap_or_else(|| &Transform::from_translation(Vec3::ZERO));
        let target = match target {
            Some(t) => *t, // Dereference to copy the Transform
            None => Transform::from_translation(Vec3::ZERO),
        };
        let offset = Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0;

        // transform offset to local space of target entity
        let offset = (target.compute_matrix() * offset.extend(1.0)).xyz();

        Self {
            settings: FollowCameraSettings::new(entity),
            movement: OrbitMovementBundle::default(),
            velocity: Velocity::new(Vec3::ZERO),
            camera: Camera3dBundle {
                transform: Transform::from_translation(offset)
                    // .looking_at(target.translation + Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
                    .looking_at(target.translation, Vec3::Y),
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
    query: Query<(Entity, &Transform), With<FollowCameraMe>>,
) {
    for (entity, transform) in query.iter() {
        info!(
            "Adding follow camera for entity: {:?} with target translation: {:?}",
            entity, transform.translation
        );
        commands.spawn((FollowCameraBundle::new(entity, Some(transform)), Local));
    }
}

fn move_cameras(
    // time: Res<Time>,
    mut gizmos: Gizmos,
    mut query_cameras: Query<
        (
            &mut Transform,
            &FollowCameraSettings,
            &mut AngularVelocity,
            &mut Velocity,
            &mut Orbit,
            &mut Camera,
        ),
        With<Camera>,
    >,
    query_targets: Query<
        (Entity, &Transform, &Velocity, &AngularVelocity),
        (With<FollowCameraMe>, Without<Camera>),
    >,
) {
    for (
        mut camera_transform,
        follow_settings,
        mut camera_angular_velocity,
        mut camera_velocity,
        mut orbit,
        mut camera,
    ) in query_cameras.iter_mut()
    {
        for (target_entity, target_transform, target_velocity, target_angular_velocity) in
            query_targets.iter()
        {
            if target_entity == follow_settings.target {
                // let target_position = target_transform.translation
                //     + target_transform.right() * follow_settings.offset.x
                //     + target_transform.forward() * follow_settings.offset.z
                //     + target_transform.up() * follow_settings.offset.y;

                // // camera_transform.translation = target_position
                // //     * follow_camera_bundle.smoothing
                // //     + camera_transform.translation
                // //         * (1.0 - follow_camera_bundle.smoothing);

                // let delta = target_position - camera_transform.translation;
                // let distance = delta.length();

                // if distance < std::f32::EPSILON {
                //     continue;
                // }

                // let speed = distance * follow_settings.pid.p;
                // let direction = delta.normalize_or_zero();
                // velocity.value = direction * speed;
                // camera_velocity.value = target_velocity.value;
                // camera_angular_velocity.value = target_angular_velocity.value;

                // let (target_yaw, ..) = target_transform.rotation.to_euler(EulerRot::YXZ);
                // let (camera_yaw, ..) = camera_transform.rotation.to_euler(EulerRot::YXZ);
                // let delta_yaw = (target_yaw + std::f32::consts::PI) - camera_yaw;

                // let mut delta_yaw = delta_yaw;
                // if delta_yaw > std::f32::consts::PI {
                //     delta_yaw -= std::f32::consts::PI * 2.0;
                // } else if delta_yaw < -std::f32::consts::PI {
                //     delta_yaw += std::f32::consts::PI * 2.0;
                // }

                // let speed = delta_yaw * follow_settings.pid.p;
                // angular_velocity.value.x = speed;

                // orbit.origin = target_transform.translation;
                // camera_transform.look_at(target_transform.translation, Vec3::Y);

                let (target_yaw, ..) = target_transform.rotation.to_euler(EulerRot::YXZ);
                let (camera_yaw, ..) = camera_transform.rotation.to_euler(EulerRot::YXZ);
                let mut delta_yaw = (target_yaw + std::f32::consts::PI) - camera_yaw;

                if delta_yaw > std::f32::consts::PI {
                    delta_yaw -= std::f32::consts::PI * 2.0;
                } else if delta_yaw < -std::f32::consts::PI {
                    delta_yaw += std::f32::consts::PI * 2.0;
                }

                let rotate_by_yaw = Quat::from_axis_angle(Vec3::Y, target_yaw);
                let offset = rotate_by_yaw * follow_settings.offset;

                // gizmos.sphere(
                //     offset,
                //     Quat::from_array([0.0, 0.0, 0.0, 0.0]),
                //     0.2,
                //     Color::TURQUOISE,
                // );

                let target_position = target_transform.translation + offset;

                let delta = target_position - camera_transform.translation;
                let distance = delta.length();

                if distance < std::f32::EPSILON {
                    continue;
                }

                camera_transform.translation += delta * follow_settings.pid.p;
                // rotate by yaw
                camera_transform.rotate(Quat::from_axis_angle(Vec3::Y, delta_yaw));
            }
        }
    }
}
