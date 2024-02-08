use bevy::prelude::*;
use bevy_input_mapper::{
    input::{events::*, gamepad::GamepadAxis, mouse::MouseAxis},
    InputMapper, InputMapperPlugin,
};

pub struct InputPlugin {
    config: Config,
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputMapperPlugin::default())
            .add_system(Startup, bind_input)
            .add_system(Update, handle_input)
            .add_system(Update, logger);
    }
}

/// Here, we define a State for Scenario.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum CameraMode {
    #[default]
    TwoDimensional,
    ThreeDimensional,
}

/// Binding input to actions in simulation
fn bind_input(mut im: ResMut<InputMapper<CameraMode>>) {
    // On default Scenario, pressing Space or Gamepad South triggers jump action.
    im.bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::Space, "change_mode")
        .bind_gamepad_button_press(
            CameraMode::TwoDimensional,
            GamepadButtonType::South,
            "change_mode",
        )
        // On swimming Scenario/State, pressing Space or Gamepad South triggers swim_up action.
        .bind_keyboard_key_press(CameraMode::ThreeDimensional, KeyCode::Space, "swim_up")
        .bind_gamepad_button_press(
            CameraMode::ThreeDimensional,
            GamepadButtonType::South,
            "swim_up",
        )
        // Here we bind gamepad's right stick and mouse movements to camera.
        .bind_gamepad_axis_move(
            CameraMode::TwoDimensional,
            GamepadAxis::NegativeLeftStickX,
            "move_left",
        )
        .bind_gamepad_axis_move(
            CameraMode::TwoDimensional,
            GamepadAxis::PositiveLeftStickX,
            "move_right",
        )
        .bind_gamepad_axis_move(
            CameraMode::TwoDimensional,
            GamepadAxis::NegativeLeftStickY,
            "move_down",
        )
        .bind_gamepad_axis_move(
            CameraMode::TwoDimensional,
            GamepadAxis::PositiveLeftStickY,
            "move_up",
        )
        .bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::W, "move_up")
        .bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::A, "move_left")
        .bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::S, "move_down")
        .bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::D, "move_right")
        .bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::ArrowUp, "move_up")
        .bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::ArrowLeft, "move_left")
        .bind_keyboard_key_press(CameraMode::TwoDimensional, KeyCode::ArrowDown, "move_down")
        .bind_keyboard_key_press(
            CameraMode::TwoDimensional,
            KeyCode::ArrowRight,
            "move_right",
        );
}

fn logger(
    mut action_active: EventReader<InputActionActive>,
    mut action_started: EventReader<InputActionStarted>,
    mut action_continuing: EventReader<InputActionContinuing>,
    mut action_finished: EventReader<InputActionFinished>,
) {
    for ev in action_active.iter() {
        info!("Action Active: {}, {}", ev.0, ev.1);
    }
    for ev in action_started.iter() {
        info!("Action Started: {}, {}", ev.0, ev.1);
    }
    for ev in action_continuing.iter() {
        info!("Action Continuing: {}, {}", ev.0, ev.1);
    }
    for ev in action_finished.iter() {
        info!("Action Finished: {}", ev.0);
    }
}
