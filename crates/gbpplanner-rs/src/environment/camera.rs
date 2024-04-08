#![warn(missing_docs)]
// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;

use crate::movement::{LinearMovementBundle, Local, Orbit, OrbitMovementBundle};

const INITIAL_CAMERA_DISTANCE: f32 = 125.0;
// pub const SPEED: f32 = 20.0;
pub const SPEED: f32 = INITIAL_CAMERA_DISTANCE / 10.0;
pub const ANGULAR_SPEED: f32 = 2.0;

// const CAMERA_UP: Vec3 = Vec3::NEG_Z;
const CAMERA_UP: Vec3 = Vec3::NEG_Y;
const CAMERA_INITIAL_TARGET: Vec3 = Vec3::ZERO;
const CAMERA_INITIAL_POSITION: Vec3 = Vec3::new(0.0, INITIAL_CAMERA_DISTANCE, 0.0);

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CameraResetEvent>()
            .init_state::<CameraMovementMode>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, reset_camera.run_if(on_event::<CameraResetEvent>()));
    }
}

/// **Bevy** `Event` to reset the main camera's position and rotation
#[derive(Debug, Event)]
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
            transform: Transform::from_translation(CAMERA_INITIAL_POSITION)
                .looking_at(CAMERA_INITIAL_TARGET, CAMERA_UP),
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
    mut camera_query: Query<(&mut Transform, &mut Orbit), With<MainCamera>>,
    mut next_movement_mode: ResMut<NextState<CameraMovementMode>>,
) {
    next_movement_mode.set(CameraMovementMode::Pan);

    for (mut transform, mut orbit) in camera_query.iter_mut() {
        transform.translation = CAMERA_INITIAL_POSITION;
        transform.look_at(CAMERA_INITIAL_TARGET, CAMERA_UP);

        orbit.origin = Vec3::ZERO;
    }
}
