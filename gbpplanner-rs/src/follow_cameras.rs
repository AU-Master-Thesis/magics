use ::bevy::prelude::*;

pub struct FollowCamerasPlugin;

impl Plugin for FollowCamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, add_follow_cameras)
            .add_systems(Update, move_cameras);
    }
}

#[derive(Component)]
pub struct FollowCameraMe;

#[derive(Component)]
pub struct FollowCameraBundle {
    // pub target: Transform,
    pub smoothing: f32, // 0.0 = no smoothing, 1.0 = infinite smoothing
    pub target: Entity,
    pub offset: Vec3,
    // pub camera: Camera3dBundle,
}

fn add_follow_cameras(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<FollowCameraMe>>,
) {
    for (entity, transform) in query.iter() {
        info!("Adding follow camera for entity: {:?}", entity);
        commands.spawn(
            // commands.entity(entity).insert(
            (
                FollowCameraBundle {
                    smoothing: 0.1,
                    target: entity,
                    offset: Vec3::new(0.0, 10.0, 10.0),
                    // camera: Camera3dBundle {
                    //     transform: Transform::from_xyz(0.0, 0.0, 0.0)
                    //         .looking_at(Vec3::ZERO, Vec3::Y),
                    //     camera: Camera {
                    //         is_active: false,
                    //         ..Default::default()
                    //     },
                    //     ..Default::default()
                    // },
                },
                Camera3dBundle {
                    transform: Transform::from_xyz(0.0, 10.0, 10.0)
                        .looking_at(Vec3::ZERO, Vec3::Y),
                    camera: Camera {
                        is_active: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
        );
    }
}

fn move_cameras(
    time: Res<Time>,
    mut query_cameras: Query<(&mut Transform, &FollowCameraBundle), With<Camera>>,
    query_targets: Query<(Entity, &Transform), (With<FollowCameraMe>, Without<Camera>)>,
) {
    for (mut camera_transform, follow_camera_bundle) in query_cameras.iter_mut() {
        for (target_entity, target_transform) in query_targets.iter() {
            if target_entity == follow_camera_bundle.target {
                let target_position =
                    target_transform.translation + follow_camera_bundle.offset;
                let camera_position = camera_transform.translation;
                let new_position =
                    camera_position.lerp(target_position, follow_camera_bundle.smoothing);
                camera_transform.translation = new_position;
                camera_transform.look_at(target_position, Vec3::Y);
            }
        }
    }
}
