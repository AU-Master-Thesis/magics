use std::fmt::Display;

use bevy::prelude::*;
use strum_macros::EnumIter;

mod camera;
mod general;
mod moveable_object;
pub mod screenshot;
mod ui;

pub use camera::{CameraAction, CameraSensitivity};
pub use general::GeneralAction;
pub use moveable_object::{MoveableObjectAction, MoveableObjectSensitivity};
use screenshot::ScreenshotPlugin;
pub use ui::UiAction;

use self::{camera::CameraInputPlugin, general::GeneralInputPlugin, ui::UiInputPlugin};
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
            Self::Move => "Move Camera".to_string(),
            Self::MouseMove => "Move Mouse".to_string(),
            Self::ToggleMovementMode => "Toggle Movement Mode".to_string(),
            Self::ZoomIn => "Zoom In".to_string(),
            Self::ZoomOut => "Zoom Out".to_string(),
            Self::Switch => "Switch Camera".to_string(),
            Self::Reset => "Reset Camera".to_string(),
        }
    }
}

impl ToDisplayString for GeneralAction {
    fn to_display_string(&self) -> String {
        match self {
            Self::ToggleTheme => "Toggle Theme".to_string(),
            Self::ExportGraph => "Export Graph".to_string(),
            Self::ScreenShot => "Take Screenshot".to_string(),
            Self::QuitApplication => "Quit Application".to_string(),
            Self::PausePlaySimulation => "Pause/Play Simulation".to_string(),
        }
    }
}

impl ToDisplayString for MoveableObjectAction {
    fn to_display_string(&self) -> String {
        match self {
            Self::Move => "Move Object".to_string(),
            Self::RotateClockwise => "Rotate Clockwise".to_string(),
            Self::RotateCounterClockwise => "Rotate Counter Clockwise".to_string(),
            Self::Boost => "Boost".to_string(),
            Self::Toggle => "Toggle Object".to_string(),
        }
    }
}

impl ToDisplayString for UiAction {
    fn to_display_string(&self) -> String {
        self.to_string()
        // match self {
        //     UiAction::ToggleLeftPanel => "Toggle Left Panel".to_string(),
        //     UiAction::ToggleRightPanel => "Toggle Right Panel".to_string(),
        //     UiAction::ChangeScaleKind => "Toggle Scale Factor".to_string(),
        //     UiAction::ToggleTopPanel => "Toggle Top Panel".to_string(),
        //     UiAction::ToggleBottomPanel => "Toggle Bottom Panel".to_string(),
        //     UiAction::ToggleMetricsWindow => "",
        // }
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CameraInputPlugin,
            // MoveableObjectInputPlugin,
            GeneralInputPlugin,
            UiInputPlugin,
            ScreenshotPlugin::default(),
        ));
    }
}
