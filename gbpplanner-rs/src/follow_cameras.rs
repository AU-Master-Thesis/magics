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
    // pub p: f32,
    // pub p_heading: f32,
    // pub camera: Camera3dBundle,
}

impl FollowCameraBundle {
    fn new(entity: Entity) -> Self {
        Self {
            smoothing: 0.5,
            target: entity,
            offset: Vec3::new(0.0, 5.0, 10.0),
        }
    }
}

fn add_follow_cameras(
    mut commands: Commands,
    query: Query<Entity, With<FollowCameraMe>>,
) {
    for entity in query.iter() {
        info!("Adding follow camera for entity: {:?}", entity);
        commands.spawn((
            FollowCameraBundle::new(entity),
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 5.0, 10.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    is_active: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        ));
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
                let target_position = target_transform.translation
                    + target_transform.right() * follow_camera_bundle.offset.x
                    + target_transform.forward() * follow_camera_bundle.offset.z
                    + target_transform.up() * follow_camera_bundle.offset.y;

                // do proportional control to move the camera towards the target
                // let error = target_position - camera_transform.translation;
                // let new_position = camera_transform.translation
                //     + error * follow_camera_bundle.p * time.delta_seconds();
                // let new_position = target_position;

                camera_transform.translation = target_position
                    * follow_camera_bundle.smoothing
                    + camera_transform.translation
                        * (1.0 - follow_camera_bundle.smoothing);

                camera_transform.look_at(target_transform.translation, Vec3::Y);
            }
        }
    }
}
