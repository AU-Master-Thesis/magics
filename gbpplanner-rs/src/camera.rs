// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;
use bevy_infinite_grid::GridShadowCamera;

use crate::movement::{LinearMovementBundle, Local, OrbitMovementBundle};

const INITIAL_CAMERA_DISTANCE: f32 = 150.0;
pub const SPEED: f32 = 20.0;
pub const ANGULAR_SPEED: f32 = 2.0;

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

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CameraMovementMode>()
        // app.add_state::<CameraMovementMode>()
            .add_systems(Startup, spawn_camera);
    }
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
        // GridShadowCamera,
    ));
}
