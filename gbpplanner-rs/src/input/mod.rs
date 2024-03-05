use bevy::prelude::*;
use strum_macros::EnumIter;

mod camera;
mod general;
mod moveable_object;
mod ui;

use crate::ui::ToDisplayString;

pub use self::camera::CameraAction;
pub use self::general::GeneralAction;
pub use self::moveable_object::MoveableObjectAction;
pub use self::ui::UiAction;

use self::{
    camera::CameraInputPlugin, general::GeneralInputPlugin,
    moveable_object::MoveableObjectInputPlugin, ui::UiInputPlugin,
};

/// Enumeration to collect the different kinds of input bindings
#[derive(Debug, EnumIter)]
pub enum InputAction {
    General(GeneralAction),
    Camera(CameraAction),
    MoveableObject(MoveableObjectAction),
    Ui(UiAction),
    Undefined,
}

impl Default for InputAction {
    fn default() -> Self {
        Self::Undefined
    }
}

impl ToString for InputAction {
    fn to_string(&self) -> String {
        match self {
            Self::Camera(_) => "Camera".to_string(),
            Self::General(_) => "General".to_string(),
            Self::MoveableObject(_) => "Moveable Object".to_string(),
            Self::Ui(_) => "UI".to_string(),
            Self::Undefined => "Undefined".to_string(),
        }
    }
}

impl ToDisplayString for InputAction {
    fn to_display_string(&self) -> String {
        match self {
            Self::Camera(action) => action.to_display_string(),
            Self::General(action) => action.to_display_string(),
            Self::MoveableObject(action) => action.to_display_string(),
            Self::Ui(action) => action.to_display_string(),
            Self::Undefined => "Undefined".to_string(),
        }
    }
}

impl ToDisplayString for CameraAction {
    fn to_display_string(&self) -> String {
        match self {
            CameraAction::Move => "Move Camera".to_string(),
            CameraAction::MouseMove => "Move Mouse".to_string(),
            CameraAction::ToggleMovementMode => "Toggle Movement Mode".to_string(),
            CameraAction::ZoomIn => "Zoom In".to_string(),
            CameraAction::ZoomOut => "Zoom Out".to_string(),
            CameraAction::Switch => "Switch Camera".to_string(),
        }
    }
}

impl ToDisplayString for GeneralAction {
    fn to_display_string(&self) -> String {
        match self {
            GeneralAction::ToggleTheme => "Toggle Theme".to_string(),
            GeneralAction::ExportGraph => "Export Graph".to_string(),
        }
    }
}

impl ToDisplayString for MoveableObjectAction {
    fn to_display_string(&self) -> String {
        match self {
            MoveableObjectAction::Move => "Move Object".to_string(),
            MoveableObjectAction::RotateClockwise => "Rotate Clockwise".to_string(),
            MoveableObjectAction::RotateCounterClockwise => "Rotate Counter Clockwise".to_string(),
            MoveableObjectAction::Boost => "Boost".to_string(),
            MoveableObjectAction::Toggle => "Toggle Object".to_string(),
        }
    }
}

impl ToDisplayString for UiAction {
    fn to_display_string(&self) -> String {
        match self {
            UiAction::ToggleLeftPanel => "Toggle Left Panel".to_string(),
            UiAction::ToggleScaleFactor => "Toggle Scale Factor".to_string(),
            // UiAction::ToggleRightPanel => "Toggle Right Panel".to_string(),
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
