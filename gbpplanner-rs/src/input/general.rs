use super::super::theme::ThemeEvent;
use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

#[derive(Component)]
pub struct GeneralInputs;

pub struct GeneralInputPlugin;

impl Plugin for GeneralInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<GeneralAction>::default(),))
            .add_systems(PostStartup, (bind_general_input,))
            .add_systems(Update, (general_actions,));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum GeneralAction {
    ToggleTheme,
}

impl GeneralAction {
    fn default_keyboard_input(action: GeneralAction) -> Option<UserInput> {
        match action {
            Self::ToggleTheme => Some(UserInput::Single(InputKind::Keyboard(KeyCode::T))),
        }
    }
}

fn bind_general_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    for action in GeneralAction::variants() {
        if let Some(input) = GeneralAction::default_keyboard_input(action) {
            input_map.insert(input, action);
        }
    }

    commands.spawn((
        InputManagerBundle {
            input_map,
            ..Default::default()
        },
        GeneralInputs,
    ));
}

fn general_actions(
    mut theme_event: EventWriter<ThemeEvent>,
    query: Query<&ActionState<GeneralAction>, With<GeneralInputs>>,
) {
    if let Ok(action_state) = query.get_single() {
        if action_state.just_pressed(GeneralAction::ToggleTheme) {
            info!("Toggling theme");
            theme_event.send(ThemeEvent);
        }
    }
}
