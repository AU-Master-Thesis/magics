use bevy::prelude::*;
use strum_macros::EnumIter;

mod camera;
mod general;
mod moveable_object;
mod ui;

pub use self::camera::CameraAction;
pub use self::general::GeneralAction;
pub use self::moveable_object::MoveableObjectAction;
pub use self::ui::UiAction;

use self::{
    camera::CameraInputPlugin, general::GeneralInputPlugin,
    moveable_object::MoveableObjectInputPlugin, ui::UiInputPlugin,
};

#[derive(Debug, EnumIter)]
pub enum InputAction {
    Camera(CameraAction),
    General(GeneralAction),
    MoveableObject(MoveableObjectAction),
    Ui(UiAction),
}

impl ToString for InputAction {
    fn to_string(&self) -> String {
        match self {
            Self::Camera(_) => "Camera".to_string(),
            Self::General(_) => "General".to_string(),
            Self::MoveableObject(_) => "Moveable Object".to_string(),
            Self::Ui(_) => "UI".to_string(),
        }
    }
}

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
