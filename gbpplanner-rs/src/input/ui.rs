use bevy::prelude::*;
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
}

impl UiAction {
    fn default_keyboard_input(action: UiAction) -> Option<UserInput> {
        match action {
            Self::ToggleLeftPanel => {
                Some(UserInput::Single(InputKind::Keyboard(KeyCode::H)))
            }
        }
    }
}

fn bind_ui_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    for action in UiAction::variants() {
        if let Some(input) = UiAction::default_keyboard_input(action) {
            input_map.insert(input, action);
        }
    }

    commands.spawn((InputManagerBundle::<UiAction> {
        input_map,
        ..Default::default()
    },));
}

fn ui_actions(mut query: Query<&ActionState<UiAction>>, mut left_panel: ResMut<UiState>) {
    if let Ok(action_state) = query.get_single() {
        if action_state.just_pressed(UiAction::ToggleLeftPanel) {
            left_panel.left_panel = !left_panel.left_panel;
        }
    }
}
