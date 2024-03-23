#![warn(missing_docs)]
// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;

use crate::movement::{LinearMovementBundle, Local, Orbit, OrbitMovementBundle};

const INITIAL_CAMERA_DISTANCE: f32 = 125.0;
// pub const SPEED: f32 = 20.0;
pub const SPEED: f32 = INITIAL_CAMERA_DISTANCE / 10.0;
pub const ANGULAR_SPEED: f32 = 2.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CameraResetEvent>()
            .init_state::<CameraMovementMode>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, reset_camera);
    }
}

/// **Bevy** `Event` to reset the main camera's position and rotation
#[derive(Debug, Default, Event)]
pub struct CameraResetEvent;

/// **Bevy** `Component` for the main camera
#[derive(Component, Debug)]
pub struct MainCamera;

/// **Bevy** `State` representing the main camera's movement mode
/// Enables the camera to `Pan` and `Orbit`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraMovementMode {
    #[default]
    Pan,
    Orbit,
}

/// `Startup` system to spawn the main camera
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, INITIAL_CAMERA_DISTANCE, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        LinearMovementBundle::default(),
        OrbitMovementBundle::default(),
        Local,
        MainCamera,
    ));
}

/// **Bevy** [`Update`] system listening to [`CameraResetEvent`]
/// To reset the main camera's position and rotation
fn reset_camera(
    mut camera_query: Query<(&MainCamera, &mut Transform, &mut Orbit)>,
    mut next_movement_mode: ResMut<NextState<CameraMovementMode>>,
    mut camera_reset_event: EventReader<CameraResetEvent>,
) {
    for _ in camera_reset_event.read() {
        next_movement_mode.set(CameraMovementMode::Pan);
        for (_, mut transform, mut orbit) in camera_query.iter_mut() {
            transform.translation = Vec3::new(0.0, INITIAL_CAMERA_DISTANCE, 0.0);
            transform.look_at(Vec3::ZERO, Vec3::Z);

            orbit.origin = Vec3::ZERO;
        }
    }
}
