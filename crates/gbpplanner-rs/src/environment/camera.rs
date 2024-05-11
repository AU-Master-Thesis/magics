// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;

use crate::{
    config::Config,
    movement::{LinearMovementBundle, Local, Orbit, OrbitMovementBundle},
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

// const CAMERA_UP: Vec3 = Vec3::NEG_Y;
const CAMERA_UP: Vec3 = Vec3::Z;
const CAMERA_INITIAL_TARGET: Vec3 = Vec3::ZERO;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::ResetCamera>()
            .init_state::<CameraMovement>()
            .init_resource::<CameraSettings>()
            .add_systems(Startup, spawn_main_camera)
            .add_systems(
                Update,
                (
                    reset_main_camera.run_if(on_event::<events::ResetCamera>()),
                    (
                        // reset_main_camera,
                        activate_main_camera
                    )
                        .chain()
                        .run_if(
                            on_event::<ReloadSimulation>().or_else(on_event::<LoadSimulation>()),
                        ),
                ),
            );
    }
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

const DEFAULT_CAMERA_DISTANCE: f32 = 250.0;

impl CameraSettings {
    /// Returns the default camera settings
    pub fn reset_distance(&mut self, distance: Option<f32>) {
        if let Some(distance) = distance {
            self.start_pos = Vec3::new(0.0, distance, 0.0);
        } else {
            self.start_pos = Vec3::new(0.0, DEFAULT_CAMERA_DISTANCE, 0.0);
        }
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            speed: DEFAULT_CAMERA_DISTANCE / 10.0,
            angular_speed: 2.0,
            start_pos: Vec3::new(0.0, DEFAULT_CAMERA_DISTANCE, 0.0),
        }
    }
}

pub mod events {
    use bevy::ecs::event::Event;

    /// **Bevy** [`Event`] to reset the main camera's position and rotation
    #[derive(Event)]
    pub struct ResetCamera;
}

/// **Bevy** [`Component`] for the main camera
#[derive(Component, Debug)]
pub struct MainCamera;

impl MainCamera {
    // pub const INITINAL_DISTANCE: f32 = 250.0;

    pub fn initinal_transform() -> Transform {
        Transform {
            // translation: Vec3::Y * -Self::INITINAL_DISTANCE,
            translation: Vec3::Y * -DEFAULT_CAMERA_DISTANCE,
            ..Default::default()
        }
        .looking_at(Vec3::ZERO, Vec3::Z)
    }
}

/// **Bevy** [`State`] representing the main camera's movement mode
/// Enables the camera to `Pan` and `Orbit`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraMovement {
    #[default]
    Pan,
    Orbit,
}

impl CameraMovement {
    pub fn cycle(&mut self) {
        *self = match self {
            CameraMovement::Pan => CameraMovement::Orbit,
            CameraMovement::Orbit => CameraMovement::Pan,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            CameraMovement::Pan => CameraMovement::Orbit,
            CameraMovement::Orbit => CameraMovement::Pan,
        }
    }
}

/// **Bevy** [`Startup`] system to spawn the main camera
// fn spawn_main_camera(mut commands: Commands, config: Res<Config>) {
fn spawn_main_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: MainCamera::initinal_transform(),
            // transform,
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
fn reset_main_camera(
    mut main_camera: Query<(&mut Transform, &mut Orbit), With<MainCamera>>,
    mut next_camera_movement: ResMut<NextState<CameraMovement>>,
    mut cam_settings: ResMut<CameraSettings>,
    config: Res<Config>,
) {
    next_camera_movement.set(CameraMovement::default());
    cam_settings.reset_distance(Some(config.interaction.default_cam_distance));

    let (mut transform, mut orbit) = main_camera.single_mut();

    *transform = MainCamera::initinal_transform();
    orbit.origin = Vec3::ZERO;
}

fn activate_main_camera(mut main_camera: Query<&mut Camera, With<MainCamera>>) {
    let mut main_camera = main_camera.single_mut();
    main_camera.is_active = true;
    info!("Activated main camera");
}
