use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::EguiSettings;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use super::super::ui::UiState;

pub struct UiInputPlugin;

impl Plugin for UiInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<UiAction>::default(),))
            .add_systems(PostStartup, (bind_ui_input,))
            .add_systems(Update, (ui_actions,));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum UiAction {
    ToggleLeftPanel,
    ToggleScaleFactor,
}

impl UiAction {
    fn variants() -> &'static [Self] {
        &[UiAction::ToggleLeftPanel, UiAction::ToggleScaleFactor]
    }

    fn default_keyboard_input(action: UiAction) -> Option<UserInput> {
        match action {
            Self::ToggleLeftPanel => {
                Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyH)))
            }
            Self::ToggleScaleFactor => {
                Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyU)))
            }
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

    // commands.spawn((InputManagerBundle::<UiAction> {
    //     input_map,
    //     ..Default::default()
    // },));
}

fn ui_actions(
    query: Query<&ActionState<UiAction>>,
    mut left_panel: ResMut<UiState>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut egui_settings: ResMut<EguiSettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(action_state) = query.get_single() else {
        return;
    };

    if action_state.just_pressed(&UiAction::ToggleLeftPanel) {
        left_panel.left_panel = !left_panel.left_panel;
    }

    if action_state.just_pressed(&UiAction::ToggleScaleFactor)
        || toggle_scale_factor.is_none()
    {
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
