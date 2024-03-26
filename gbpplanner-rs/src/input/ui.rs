use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum_macros::EnumIter;

use super::super::ui::UiState;
use crate::ui::{ChangingBinding, UiScaleType};

pub struct UiInputPlugin;

impl Plugin for UiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<UiAction>::default(),))
            .add_systems(PostStartup, (bind_ui_input,))
            .add_systems(Update, (ui_actions,));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Default, Reflect, EnumIter)]
pub enum UiAction {
    #[default] // Necessary to implement `Default` for `EnumIter`
    ToggleLeftPanel,
    ToggleRightPanel,
    ChangeScaleKind,
}

impl UiAction {
    fn variants() -> &'static [Self] {
        &[
            UiAction::ToggleLeftPanel,
            UiAction::ChangeScaleKind,
            UiAction::ToggleRightPanel,
        ]
    }

    fn default_keyboard_input(action: UiAction) -> Option<UserInput> {
        Some(match action {
            Self::ToggleLeftPanel => UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyH)),
            Self::ToggleRightPanel => UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyL)),
            Self::ChangeScaleKind => UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyU)),
        })
    }
}

impl std::fmt::Display for UiAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToggleLeftPanel => write!(f, "Toggle Left Panel"),
            Self::ToggleRightPanel => write!(f, "Toggle Right Panel"),
            Self::ChangeScaleKind => write!(f, "Toggle Scale Factor"),
        }
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

    if action_state.just_pressed(&UiAction::ChangeScaleKind) {
        match ui_state.scale_type {
            UiScaleType::None => {
                ui_state.scale_type = UiScaleType::Custom;
            }
            UiScaleType::Custom => {
                ui_state.scale_type = UiScaleType::Window;
            }
            UiScaleType::Window => {
                ui_state.scale_type = UiScaleType::None;
            }
        }
    }
}
