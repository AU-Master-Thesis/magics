use ::bevy::prelude::*;

pub struct FollowCamerasPlugin;

impl Plugin for FollowCamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, add_follow_cameras);
    }
}

#[derive(Component)]
pub struct FollowCameraMe;

#[derive(Component)]
pub struct FollowCameraBundle {
    // pub target: Transform,
    pub smoothing: f32, // 0.0 = no smoothing, 1.0 = infinite smoothing
    pub target: Entity,
    pub camera: Camera3dBundle,
}

fn add_follow_cameras(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<FollowCameraMe>>,
) {
    for (entity, transform) in query.iter() {
        info!("Adding follow camera for entity: {:?}", entity);
        commands.entity(entity).insert(FollowCameraBundle {
            smoothing: 0.1,
            target: entity,
            camera: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    is_active: false,
                    ..Default::default()
                },
                ..Default::default()
            },
        });
    }
}
