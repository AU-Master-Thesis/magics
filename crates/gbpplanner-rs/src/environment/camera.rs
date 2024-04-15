#![warn(missing_docs)]
// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;

use crate::{
    config::Config,
    movement::{LinearMovementBundle, Local, Orbit, OrbitMovementBundle},
};

const CAMERA_UP: Vec3 = Vec3::NEG_Y;
const CAMERA_INITIAL_TARGET: Vec3 = Vec3::ZERO;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CameraResetEvent>()
            .init_state::<CameraMovementMode>()
            .add_systems(Startup, (init_cam_settings, spawn_camera).chain())
            .add_systems(Update, reset_camera.run_if(on_event::<CameraResetEvent>()));
    }
}

fn init_cam_settings(mut commands: Commands, config: Res<Config>) {
    commands.insert_resource(CameraSettings {
        speed: config.interaction.default_cam_distance / 10.0,
        angular_speed: 2.0,
        start_pos: Vec3::new(0.0, config.interaction.default_cam_distance, 0.0),
    });
}

/// **Bevy** [`Resource`] for the main camera's settings
/// Is initialised partially from the [`Config`] resource, otherwise with some
/// sensible defaults
#[derive(Debug, Resource)]
pub struct CameraSettings {
    /// The speed at which the camera moves in [`CameraMovementMode::Pan`]
    pub speed: f32,
    /// The speed at which the camera rotates in [`CameraMovementMode::Orbit`]
    pub angular_speed: f32,
    /// The initial position of the camera in 3D space
    pub start_pos: Vec3,
}

/// **Bevy** [`Event`] to reset the main camera's position and rotation
#[derive(Debug, Event)]
pub struct CameraResetEvent;

/// **Bevy** [`Component`] for the main camera
#[derive(Component, Debug)]
pub struct MainCamera;

/// **Bevy** [`State`] representing the main camera's movement mode
/// Enables the camera to `Pan` and `Orbit`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraMovementMode {
    #[default]
    Pan,
    Orbit,
}

/// **Bevy** [`Startup`] system to spawn the main camera
fn spawn_camera(mut commands: Commands, config: Res<Config>) {
    let transform = Transform::from_xyz(0.0, config.interaction.default_cam_distance, 0.0)
        .looking_at(CAMERA_INITIAL_TARGET, CAMERA_UP);

    commands.spawn((
        Camera3dBundle {
            transform,
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
    cam_settings: Res<CameraSettings>,
) {
    next_movement_mode.set(CameraMovementMode::Pan);

    for (mut transform, mut orbit) in &mut camera_query {
        transform.translation = cam_settings.start_pos;
        transform.look_at(CAMERA_INITIAL_TARGET, CAMERA_UP);

        orbit.origin = Vec3::ZERO;
    }
}
