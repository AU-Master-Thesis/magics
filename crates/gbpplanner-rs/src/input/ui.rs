use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::super::ui::UiState;
use crate::{input::ChangingBinding, ui::UiScaleType};

pub struct UiInputPlugin;

impl Plugin for UiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<UiAction>::default())
            .add_systems(PostStartup, bind_ui_input)
            .add_systems(Update, handle_ui_actions);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Default, Reflect, EnumIter, derive_more::Display)]
pub enum UiAction {
    #[default] // Necessary to implement `Default` for `EnumIter`
    #[display(fmt = "Toggle Left Panel")]
    ToggleLeftPanel,
    #[display(fmt = "Toggle Right Panel")]
    ToggleRightPanel,
    #[display(fmt = "Toggle Top Panel")]
    ToggleTopPanel,
    #[display(fmt = "Toggle Bottom Panel")]
    ToggleBottomPanel,
    #[display(fmt = "Toggle Metrics Window")]
    ToggleMetricsWindow,
    ChangeScaleKind,
}

impl UiAction {
    const fn default_keyboard_input(action: Self) -> UserInput {
        let input_kind = match action {
            Self::ToggleLeftPanel => InputKind::PhysicalKey(KeyCode::KeyH),
            Self::ToggleRightPanel => InputKind::PhysicalKey(KeyCode::KeyL),
            Self::ToggleTopPanel => InputKind::PhysicalKey(KeyCode::KeyK),
            Self::ToggleBottomPanel => InputKind::PhysicalKey(KeyCode::KeyJ),
            Self::ChangeScaleKind => InputKind::PhysicalKey(KeyCode::KeyU),
            Self::ToggleMetricsWindow => InputKind::PhysicalKey(KeyCode::KeyD), // d for diagnostics
        };

        UserInput::Single(input_kind)
    }
}

fn bind_ui_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    for action in UiAction::iter() {
        let input = UiAction::default_keyboard_input(action);
        input_map.insert(action, input);
    }

    commands.spawn(InputManagerBundle::with_map(input_map));
}

fn handle_ui_actions(
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
        ui_state.left_panel_visible = !ui_state.left_panel_visible;
    }

    if action_state.just_pressed(&UiAction::ToggleRightPanel) {
        ui_state.right_panel_visible = !ui_state.right_panel_visible;
    }

    if action_state.just_pressed(&UiAction::ToggleTopPanel) {
        ui_state.top_panel_visible = !ui_state.top_panel_visible;
    }
    if action_state.just_pressed(&UiAction::ToggleBottomPanel) {
        ui_state.bottom_panel_visible = !ui_state.bottom_panel_visible;
    }

    if action_state.just_pressed(&UiAction::ToggleMetricsWindow) {
        ui_state.metrics_window_visible = !ui_state.metrics_window_visible;
    }

    if action_state.just_pressed(&UiAction::ChangeScaleKind) {
        ui_state.scale_type = match ui_state.scale_type {
            UiScaleType::None => UiScaleType::Custom,
            UiScaleType::Custom => UiScaleType::Window,
            UiScaleType::Window => UiScaleType::None,
        };
    }
}
