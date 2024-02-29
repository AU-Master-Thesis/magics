use bevy::prelude::*;

mod camera;
mod general;
mod moveable_object;
mod ui;

use self::{
    camera::CameraInputPlugin, general::GeneralInputPlugin,
    moveable_object::MoveableObjectInputPlugin, ui::UiInputPlugin,
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraInputPlugin,
            MoveableObjectInputPlugin,
            GeneralInputPlugin,
            UiInputPlugin,
        ));
    }
}
