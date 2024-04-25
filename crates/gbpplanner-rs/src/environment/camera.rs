// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/camera.rs
use bevy::prelude::*;

use crate::{
    config::Config,
    movement::{LinearMovementBundle, Local, Orbit, OrbitMovementBundle},
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

const CAMERA_UP: Vec3 = Vec3::NEG_Y;
const CAMERA_INITIAL_TARGET: Vec3 = Vec3::ZERO;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResetCamera>()
            .init_state::<CameraMovement>()
            .init_resource::<CameraSettings>()
            .add_systems(Startup, spawn_main_camera)
            // .add_systems(Update, spawn_main_cmaer)
            // .add_systems(Update)
            .add_systems(
                Update,
                (
                    reset_main_camera.run_if(on_event::<ResetCamera>()),
                    // reset_main_camera.run_if(on_event::<ReloadSimulation>()),
                    // activate_main_camera.run_if(on_event::<ReloadSimulation>()),
                    // activate_main_camera
                    //     .after(reset_main_camera)
                    //     .run_if(on_event::<ReloadSimulation>()),
                    (reset_main_camera, activate_main_camera).chain().run_if(
                        on_event::<ReloadSimulation>().or_else(on_event::<LoadSimulation>()),
                    ),
                    // (reset_main_camera, activate_main_camera)
                    //     .chain()
                    //     .run_if(on_event::<LoadSimulation>()),
                ),
            );
    }
}

// fn init_cam_settings(mut commands: Commands, config: Res<Config>) {
//     commands.insert_resource(CameraSettings {
//         speed: config.interaction.default_cam_distance / 10.0,
//         angular_speed: 2.0,
//         start_pos: Vec3::new(0.0, config.interaction.default_cam_distance,
// 0.0),     });
// }

// impl FromWorld for CameraSettings {
//     fn from_world(world: &mut World) -> Self {
//         let config = world
//             .get_resource::<Config>()
//             .expect("Config resource is available in the ecs world");
//         CameraSettings {
//             speed: config.interaction.default_cam_distance / 10.0,
//             angular_speed: 2.0,
//             start_pos: Vec3::new(0.0,
// config.interaction.default_cam_distance, 0.0),         }
//     }
// }

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

/// **Bevy** [`Event`] to reset the main camera's position and rotation
#[derive(Debug, Event)]
pub struct ResetCamera;

/// **Bevy** [`Component`] for the main camera
#[derive(Component, Debug)]
pub struct MainCamera;

/// **Bevy** [`State`] representing the main camera's movement mode
/// Enables the camera to `Pan` and `Orbit`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraMovement {
    #[default]
    Pan,
    Orbit,
}

/// **Bevy** [`Startup`] system to spawn the main camera
// fn spawn_main_camera(mut commands: Commands, config: Res<Config>) {
fn spawn_main_camera(mut commands: Commands) {
    let default_cam_distance = 250.0;
    let transform = Transform::from_xyz(0.0, default_cam_distance, 0.0).looking_at(CAMERA_INITIAL_TARGET, CAMERA_UP);

    commands.spawn((
        Camera3dBundle { transform, ..default() },
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
    next_camera_movement.set(CameraMovement::Pan);
    cam_settings.reset_distance(Some(config.interaction.default_cam_distance));

    let (mut transform, mut orbit) = main_camera.single_mut();

    transform.translation = cam_settings.start_pos;
    transform.look_at(CAMERA_INITIAL_TARGET, CAMERA_UP);
    orbit.origin = Vec3::ZERO;
}

fn activate_main_camera(mut main_camera: Query<&mut Camera, With<MainCamera>>) {
    let mut main_camera = main_camera.single_mut();
    main_camera.is_active = true;
    info!("Activated main camera");
}
