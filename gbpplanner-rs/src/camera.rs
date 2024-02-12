// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;
use bevy_infinite_grid::GridShadowCamera;

use crate::movement::{LinearMovementBundle, OrbitMovementBundle};

const CAMERA_DISTANCE: f32 = 40.0;
pub const SPEED: f32 = 10.0;
pub const ANGULAR_SPEED: f32 = 1.0;

#[derive(Component, Debug)]
pub struct MainCamera;

// Define camera movement state
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraMovementMode {
    #[default]
    Linear,
    Orbit,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<CameraMovementMode>()
            .add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, CAMERA_DISTANCE, 0.0)
                .looking_at(Vec3::ZERO, Vec3::Z),
            ..default()
        },
        LinearMovementBundle::default(),
        OrbitMovementBundle::default(),
        MainCamera,
        GridShadowCamera,
    ));
}
