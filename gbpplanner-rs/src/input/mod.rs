use bevy::prelude::*;

mod camera_input;
mod moveable_object_input;

use self::{
    camera_input::CameraInputPlugin, moveable_object_input::MoveableObjectInputPlugin,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CameraInputPlugin, MoveableObjectInputPlugin));
    }
}
