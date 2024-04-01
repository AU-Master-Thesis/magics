pub mod camera;
pub mod cursor;
pub mod follow_cameras;
pub mod gen_map;
pub mod map;

use camera::CameraPlugin;
use cursor::CursorToGroundPlugin;
pub use follow_cameras::FollowCameraMe;
use follow_cameras::FollowCamerasPlugin;
use map::MapPlugin;

use self::gen_map::GenMapPlugin;
pub use self::gen_map::MapCell;

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
