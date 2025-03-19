// Based on https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;
use gbp_config::Config;

use crate::{
    movement::{LinearMovementBundle, Local, Orbit, OrbitMovementBundle},
    SimulationState, // For our simulation state resource
};

// Camera orientation constants
const CAMERA_UP: Vec3 = Vec3::Z;
const CAMERA_INITIAL_TARGET: Vec3 = Vec3::ZERO;

/// Plugin for camera functionality
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
                    activate_main_camera,
                ),
            );
    }
}

/// Resource for the main camera's settings
#[derive(Debug, Resource)]
pub struct CameraSettings {
    /// The speed at which the camera moves in Pan mode
    pub speed: f32,
    /// The speed at which the camera rotates in Orbit mode
    pub angular_speed: f32,
    /// The initial position of the camera in 3D space
    pub start_pos: Vec3,
}

const DEFAULT_CAMERA_DISTANCE: f32 = 250.0;

impl CameraSettings {
    /// Reset the camera distance
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

/// Events module for camera-related events
pub mod events {
    use bevy::ecs::event::Event;

    /// Event to reset the main camera's position and rotation
    #[derive(Event)]
    pub struct ResetCamera;
}

/// Component marker for the main camera
#[derive(Component, Debug)]
pub struct MainCamera;

impl MainCamera {
    /// Get the initial transform for the main camera
    pub fn initinal_transform() -> Transform {
        Transform {
            translation: Vec3::Y * -DEFAULT_CAMERA_DISTANCE,
            ..Default::default()
        }
        .looking_at(Vec3::ZERO, Vec3::Z)
    }
}

/// State representing the main camera's movement mode
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraMovement {
    #[default]
    Pan,
    Orbit,
}

impl CameraMovement {
    /// Cycle through camera movement modes
    pub fn cycle(&mut self) {
        *self = match self {
            CameraMovement::Pan => CameraMovement::Orbit,
            CameraMovement::Orbit => CameraMovement::Pan,
        }
    }

    /// Get the next camera movement mode
    pub fn next(&self) -> Self {
        match self {
            CameraMovement::Pan => CameraMovement::Orbit,
            CameraMovement::Orbit => CameraMovement::Pan,
        }
    }
}

/// System to spawn the main camera
fn spawn_main_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: MainCamera::initinal_transform(),
            ..default()
        },
        LinearMovementBundle::default(),
        OrbitMovementBundle::default(),
        Local,
        MainCamera,
    ));
}

/// System to reset the main camera's position and rotation
fn reset_main_camera(
    mut main_camera: Query<(&mut Transform, &mut Orbit), With<MainCamera>>,
    mut next_camera_movement: ResMut<NextState<CameraMovement>>,
    mut cam_settings: ResMut<CameraSettings>,
    config: Option<Res<Config>>,
) {
    next_camera_movement.set(CameraMovement::default());
    
    // Use config if available, otherwise use default
    if let Some(config) = config {
        cam_settings.reset_distance(Some(config.interaction.default_cam_distance));
    } else {
        cam_settings.reset_distance(None);
    }

    let (mut transform, mut orbit) = main_camera.single_mut();

    *transform = MainCamera::initinal_transform();
    orbit.origin = Vec3::ZERO;
}

/// System to activate the main camera and set its position
fn activate_main_camera(
    mut q: Query<(&mut Camera, &mut Transform), With<MainCamera>>,
    mut cam_settings: ResMut<CameraSettings>,
    config: Option<Res<Config>>,
    sim_state: Option<Res<SimulationState>>,
) {
    // Skip if no main camera or no config
    let Ok((mut main_camera, mut tf)) = q.get_single_mut() else {
        return;
    };
    
    main_camera.is_active = true;
    
    // Set camera distance based on config if available
    if let Some(config) = config {
        tf.translation.y = -config.interaction.default_cam_distance;
        cam_settings.start_pos.y = -config.interaction.default_cam_distance;
    }
}
