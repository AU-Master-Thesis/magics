pub mod camera;
pub mod cursor;
pub mod follow_cameras;
pub mod map;

use camera::CameraPlugin;
use cursor::CursorToGroundPlugin;
use follow_cameras::FollowCamerasPlugin;
use map::MapPlugin;

pub struct EnvironmentPlugin;

impl bevy::app::Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins((
            CameraPlugin,
            FollowCamerasPlugin,
            MapPlugin,
            CursorToGroundPlugin,
        ));
    }
}
