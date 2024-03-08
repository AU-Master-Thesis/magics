pub mod camera;
pub mod environment;
pub mod follow_cameras;

use camera::CameraPlugin;
use environment::MapPlugin;
use follow_cameras::FollowCamerasPlugin;

pub struct EnvironmentPlugin;

impl bevy::app::Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((CameraPlugin, FollowCamerasPlugin, MapPlugin));
    }
}
