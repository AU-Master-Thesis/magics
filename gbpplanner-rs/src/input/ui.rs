use bevy::{input::keyboard::KeyboardInput, prelude::*, window::PrimaryWindow};
use bevy_egui::EguiSettings;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use strum_macros::EnumIter;

use crate::ui::ChangingBinding;

use super::super::ui::UiState;

pub struct UiInputPlugin;

impl Plugin for UiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<UiAction>::default(),))
            .add_systems(PostStartup, (bind_ui_input,))
            .add_systems(Update, (ui_actions,));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, EnumIter)]
pub enum UiAction {
    ToggleLeftPanel,
    ToggleRightPanel,
    ToggleScaleFactor,
}

impl UiAction {
    fn variants() -> &'static [Self] {
        &[
            UiAction::ToggleLeftPanel,
            UiAction::ToggleScaleFactor,
            UiAction::ToggleRightPanel,
        ]
    }

    fn default_keyboard_input(action: UiAction) -> Option<UserInput> {
        match action {
            Self::ToggleLeftPanel => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyH))),
            Self::ToggleRightPanel => {
                Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyL)))
            }
            Self::ToggleScaleFactor => {
                Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyU)))
            }
        }
    }
}

impl ToString for UiAction {
    fn to_string(&self) -> String {
        match self {
            Self::ToggleLeftPanel => "Toggle Left Panel".to_string(),
            Self::ToggleRightPanel => "Toggle Right Panel".to_string(),
            Self::ToggleScaleFactor => "Toggle Scale Factor".to_string(),
        }
    }
}

/// Necessary to implement `Default` for `EnumIter`
impl Default for UiAction {
    fn default() -> Self {
        Self::ToggleLeftPanel
    }
}

fn bind_ui_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    for &action in UiAction::variants() {
        if let Some(input) = UiAction::default_keyboard_input(action) {
            input_map.insert(action, input);
        }
    }

    commands.spawn(InputManagerBundle::with_map(input_map));
}

fn ui_actions(
    query: Query<&ActionState<UiAction>>,
    mut ui_state: ResMut<UiState>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut egui_settings: ResMut<EguiSettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
    currently_changing: Res<ChangingBinding>,
) {
    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }
    let Ok(action_state) = query.get_single() else {
        return;
    };

    if action_state.just_pressed(&UiAction::ToggleLeftPanel) {
        ui_state.left_panel = !ui_state.left_panel;
    }

    if action_state.just_pressed(&UiAction::ToggleRightPanel) {
        ui_state.right_panel = !ui_state.right_panel;
    }

    if action_state.just_pressed(&UiAction::ToggleScaleFactor) || toggle_scale_factor.is_none() {
        *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

        if let Ok(window) = windows.get_single() {
            let scale_factor = if toggle_scale_factor.unwrap() {
                1.0
            } else {
                1.0 / window.scale_factor()
            };
            egui_settings.scale_factor = scale_factor;
        }
    }
}
