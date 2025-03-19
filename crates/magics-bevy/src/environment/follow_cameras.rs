use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{
    movement::{Local, OrbitMovementBundle, Velocity},
    SimulationState,
};

/// Plugin for camera following functionality
pub struct FollowCamerasPlugin;

impl Plugin for FollowCamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (move_cameras, add_follow_cameras));
    }
}

/// PID controller component for smooth camera movement
#[allow(clippy::upper_case_acronyms)]
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

/// Component to tag an entity to be followed by a FollowCamera
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct FollowCameraMe {
    pub offset: Option<Vec3>,
    pub up_direction: Option<Vec3>,
    pub attached: bool,
}

impl From<Vec3> for FollowCameraMe {
    fn from(v: Vec3) -> Self {
        Self {
            offset: Some(v),
            up_direction: None,
            attached: false,
        }
    }
}

impl FollowCameraMe {
    /// Create a new FollowCameraMe component with the specified offset
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            offset: Some(Vec3::new(x, y, z)),
            up_direction: None,
            attached: false,
        }
    }

    /// Set the up direction for the camera
    #[must_use]
    pub const fn with_up_direction(mut self, up_direction: Vec3) -> Self {
        self.up_direction = Some(up_direction);
        self
    }

    /// Set whether the camera is attached to the entity
    #[must_use]
    pub const fn with_attached(mut self, attached: bool) -> Self {
        self.attached = attached;
        self
    }
}

/// Component to store the settings for a FollowCamera
#[derive(Component)]
pub struct FollowCameraSettings {
    pub target: Entity,
    pub offset: Vec3,
    pub pid: PID,
}

impl FollowCameraSettings {
    /// Create new follow camera settings targeting the specified entity
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

    /// Set the offset for the camera
    #[must_use]
    pub const fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }
}

/// Type of camera attachment
#[derive(Component, PartialEq, Eq)]
pub enum CameraType {
    /// Camera is attached as a child entity
    Attached,
    /// Camera follows the target but is not attached
    Free,
}

/// Bundle for a FollowCamera entity
#[derive(Bundle)]
pub struct FollowCameraBundle {
    pub settings: FollowCameraSettings,
    pub movement: OrbitMovementBundle,
    pub velocity: Velocity,
    pub camera: Camera3dBundle,
    pub camera_type: CameraType,
}

impl FollowCameraBundle {
    /// Create a new FollowCameraBundle
    fn new(
        entity: Entity,
        target: Option<&Transform>,
        params: FollowCameraMe,
    ) -> Self {
        let target_transform =
            target.map_or_else(|| Transform::from_translation(Vec3::ZERO), |t| *t);
        
        let offset = params
            .offset
            .map_or_else(|| Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0, |o| o);

        let up_direction = params.up_direction.unwrap_or(Vec3::Y);

        let (camera_type, transform) = if params.attached {
            (
                CameraType::Attached,
                Transform::from_translation(offset).looking_at(Vec3::ZERO, up_direction),
            )
        } else {
            (
                CameraType::Free,
                Transform::from_translation(target_transform.translation + offset)
                    .looking_at(target_transform.translation, up_direction),
            )
        };

        Self {
            settings: FollowCameraSettings::new(entity).with_offset(offset),
            movement: OrbitMovementBundle::default(),
            velocity: Velocity(Vec3::ZERO),
            camera: Camera3dBundle {
                transform,
                camera: Camera {
                    is_active: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            camera_type,
        }
    }
}

/// System to add a FollowCamera to any entity tagged with FollowCameraMe
fn add_follow_cameras(
    mut commands: Commands,
    entities_to_attach_a_follow_cam_to: Query<(Entity, &Transform, &FollowCameraMe)>,
    cameras: Query<Entity, With<Camera3d>>,
    children: Query<&Children>,
) {
    for (entity, transform, follow_camera_flag) in &entities_to_attach_a_follow_cam_to {
        // Check if a camera is already attached
        let camera_already_attached = children
            .iter_descendants(entity)
            .any(|e| cameras.get(e).is_ok());

        if camera_already_attached {
            // An entity can only have one follower camera attached to it
            continue;
        }

        // Create a new follow camera
        let follower_camera = commands
            .spawn((
                FollowCameraBundle::new(entity, Some(transform), *follow_camera_flag),
                Local,
            ))
            .id();

        // Make the camera a child of the entity
        commands.entity(entity).push_children(&[follower_camera]);
    }
}

/// System to move cameras tagged with FollowCamera component
#[allow(clippy::type_complexity)]
fn move_cameras(
    mut query_cameras: Query<(&mut Transform, &FollowCameraSettings, &CameraType), With<Camera>>,
    query_targets: Query<(Entity, &Transform), (With<FollowCameraMe>, Without<Camera>)>,
) {
    for (mut camera_transform, follow_settings, cam_type) in &mut query_cameras {
        // Skip attached cameras as they move with their parent
        if matches!(cam_type, CameraType::Attached) {
            continue;
        }
        
        // Find the target entity and update camera position
        for (target_entity, target_transform) in query_targets.iter() {
            if target_entity == follow_settings.target {
                // Calculate yaw angles
                let (target_yaw, ..) = target_transform.rotation.to_euler(EulerRot::YXZ);
                let (camera_yaw, ..) = camera_transform.rotation.to_euler(EulerRot::YXZ);
                let mut delta_yaw = (target_yaw + PI) - camera_yaw;

                // Normalize delta_yaw to -PI..PI
                if delta_yaw > PI {
                    delta_yaw -= PI * 2.0;
                } else if delta_yaw < -PI {
                    delta_yaw += PI * 2.0;
                }

                // Calculate target position
                let rotate_by_yaw = Quat::from_axis_angle(Vec3::Y, target_yaw);
                let offset = rotate_by_yaw * follow_settings.offset;
                let target_position = target_transform.translation + offset;

                // Calculate movement
                let delta = target_position - camera_transform.translation;
                let distance = delta.length();

                if distance < f32::EPSILON {
                    continue;
                }

                // Apply PID controller
                camera_transform.translation += delta * follow_settings.pid.p;
                
                // Rotate by yaw
                camera_transform.rotate(Quat::from_axis_angle(Vec3::Y, delta_yaw));
            }
        }
    }
}
