use std::fmt::Display;

use bevy::prelude::*;
use strum_macros::EnumIter;

mod camera;
mod general;
mod moveable_object;
mod ui;

pub use camera::{CameraAction, CameraSensitivity};
pub use general::{GeneralAction, ScreenShotEvent};
pub use moveable_object::{MoveableObjectAction, MoveableObjectSensitivity};
pub use ui::UiAction;

use self::{
    camera::CameraInputPlugin, general::GeneralInputPlugin,
    moveable_object::MoveableObjectInputPlugin, ui::UiInputPlugin,
};
use crate::ui::ToDisplayString;

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

impl Display for InputAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Camera(_) => write!(f, "Camera"),
            Self::General(_) => write!(f, "General"),
            Self::MoveableObject(_) => write!(f, "Moveable Object"),
            Self::Ui(_) => write!(f, "UI"),
            Self::Undefined => write!(f, "Undefined"),
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
            CameraAction::Reset => "Reset Camera".to_string(),
        }
    }
}

impl ToDisplayString for GeneralAction {
    fn to_display_string(&self) -> String {
        match self {
            GeneralAction::ToggleTheme => "Toggle Theme".to_string(),
            GeneralAction::ExportGraph => "Export Graph".to_string(),
            GeneralAction::ScreenShot => "Take Screenshot".to_string(),
            GeneralAction::QuitApplication => "Quit Application".to_string(),
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
            UiAction::ToggleRightPanel => "Toggle Right Panel".to_string(),
            UiAction::ChangeScaleKind => "Toggle Scale Factor".to_string(),
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
