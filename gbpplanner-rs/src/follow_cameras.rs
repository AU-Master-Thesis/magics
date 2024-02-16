use ::bevy::prelude::*;

use crate::{
    movement::{Local, OrbitMovementBundle, Velocity},
    robot_spawner::RobotSpawnedEvent,
};

pub struct FollowCamerasPlugin;

impl Plugin for FollowCamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_cameras, add_follow_cameras));
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

#[derive(Component, Debug, Clone, Copy)]
pub struct FollowCameraMe {
    pub offset: Option<Vec3>,
}

impl Default for FollowCameraMe {
    fn default() -> Self {
        Self { offset: None }
    }
}

#[derive(Component)]
pub struct FollowCameraSettings {
    pub target: Entity,
    pub offset: Vec3,
    pub pid: PID,
}

impl FollowCameraSettings {
    pub fn new(target: Entity) -> Self {
        Self {
            target,
            offset: Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0,
            pid: PID {
                p: 1.0,
                ..Default::default()
            },
        }
    }

    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
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
    fn new(entity: Entity, target: Option<&Transform>, offset: Option<Vec3>) -> Self {
        let target = match target {
            Some(t) => *t, // Dereference to copy the Transform
            None => Transform::from_translation(Vec3::ZERO),
        };
        // let offset = Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0;
        let offset = match offset {
            Some(o) => o,
            None => Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0,
        };

        // TODO: Maybe add this back in
        // transform offset to local space of target entity
        // let offset = (target.compute_matrix() * offset.extend(1.0)).xyz();

        Self {
            settings: FollowCameraSettings::new(entity).with_offset(offset),
            movement: OrbitMovementBundle::default(),
            velocity: Velocity::new(Vec3::ZERO),
            camera: Camera3dBundle {
                transform: Transform::from_translation(target.translation + offset)
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
    // query: Query<(Entity, &Transform), With<FollowCameraMe>>,
    mut robot_spawned_events: EventReader<RobotSpawnedEvent>,
) {
    // info!("Outside event reader for loop");
    // for (entity, transform) in query.iter() {
    for event in robot_spawned_events.read() {
        // info!("Adding follow camera for entity: {:?}", event);
        commands.spawn((
            FollowCameraBundle::new(
                event.entity,
                Some(&event.transform),
                Some(
                    event
                        .follow_camera_flag
                        .offset
                        .expect("The event always has an offset"),
                ),
            ),
            Local,
        ));
    }
}

fn move_cameras(
    mut query_cameras: Query<(&mut Transform, &FollowCameraSettings), With<Camera>>,
    query_targets: Query<(Entity, &Transform), (With<FollowCameraMe>, Without<Camera>)>,
) {
    for (mut camera_transform, follow_settings) in query_cameras.iter_mut() {
        for (target_entity, target_transform) in query_targets.iter() {
            if target_entity == follow_settings.target {
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
