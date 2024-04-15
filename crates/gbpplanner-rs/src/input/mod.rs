use std::fmt::Display;

use bevy::prelude::*;
use strum_macros::EnumIter;

pub mod camera;
mod general;
mod moveable_object;
pub mod screenshot;
mod ui;

pub use camera::{CameraAction, CameraSensitivity};
pub use general::{DrawSettingsEvent, EnvironmentEvent, ExportGraphEvent, GeneralAction};
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
            Self::CycleTheme => "Toggle Theme".to_string(),
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
        app.init_resource::<ChangingBinding>()
            .add_plugins((
                CameraInputPlugin,
                // MoveableObjectInputPlugin,
                GeneralInputPlugin,
                UiInputPlugin,
                ScreenshotPlugin::default(),
            ))
            .add_systems(Update, binding_cooldown_system);
    }
}

/// **Bevy** [`Resource`] to store the currently changing binding
/// If this is not `default`, then all input will be captured and the binding
/// will be updated Blocks ALL actions (including UI actions) while changing a
/// binding
#[derive(Debug, Default, Resource)]
pub struct ChangingBinding {
    pub action:  InputAction,
    pub binding: usize,
    cooldown:    f32,
}

impl ChangingBinding {
    pub fn new(action: InputAction, binding: usize) -> Self {
        Self {
            action,
            binding,
            cooldown: 0.0,
        }
    }

    #[inline]
    pub fn is_changing(&self) -> bool {
        !matches!(self.action, InputAction::Undefined)
    }

    #[inline]
    pub fn on_cooldown(&self) -> bool {
        self.cooldown > 0.0
    }

    #[inline]
    pub fn with_cooldown(mut self, cooldown: f32) -> Self {
        self.cooldown = cooldown;
        self
    }

    /// Decrease the cooldown by `delta`, ensuring that it does not go below 0
    pub fn decrease_cooldown(&mut self, delta: f32) {
        self.cooldown -= delta;
        if self.cooldown < 0.0 {
            self.cooldown = 0.0;
        }
    }

    /// Refresh the cooldown
    #[inline]
    pub fn refresh_cooldown(&mut self) {
        self.cooldown = 0.1;
    }
}

fn binding_cooldown_system(time: Res<Time<Real>>, mut currently_changing: ResMut<ChangingBinding>) {
    if currently_changing.on_cooldown() {
        // info!("Cooldown: {}", currently_changing.cooldown);
        currently_changing.decrease_cooldown(time.delta_seconds());
    }
}
