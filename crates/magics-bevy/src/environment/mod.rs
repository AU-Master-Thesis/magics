//! Environment and camera systems for the Magics planner UI
//!
//! This module provides visualization and interaction with the simulation environment,
//! including camera controls, environment rendering, and cursor interaction.

pub mod camera;
pub mod cursor;
pub mod follow_cameras;
pub mod map;
pub mod map_generator;

use camera::CameraPlugin;
pub use camera::MainCamera;
use cursor::CursorToGroundPlugin;
pub use follow_cameras::FollowCameraMe;
use follow_cameras::FollowCamerasPlugin;
use map::MapPlugin;
pub use map_generator::ObstacleMarker;

use self::map_generator::GenMapPlugin;

/// Plugin that encapsulates all environment-related functionality
#[derive(Default)]
pub struct EnvironmentPlugin;

impl bevy::app::Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            CameraPlugin,
            FollowCamerasPlugin,
            MapPlugin,
            CursorToGroundPlugin,
            GenMapPlugin,
        ));
    }
}
