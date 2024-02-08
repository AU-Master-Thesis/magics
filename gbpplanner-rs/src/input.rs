use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

pub struct InputPlugin;
// {
//     config: Config,
// }

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<InputAction>::default())
            .add_systems(Startup, bind_input)
            .add_systems(Update, use_actions);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum InputAction {
    Move,
    Boost,
    Toggle,
}

// Exhaustively match `InputAction` and define the default binding to the input
impl InputAction {
    fn default_keyboard_mouse_input(action: InputAction) -> UserInput {
        // Match against the provided action to get the correct default keyboard-mouse input
        match action {
            Self::Move => UserInput::VirtualDPad(VirtualDPad::wasd()),
            Self::Boost => UserInput::Single(InputKind::Keyboard(KeyCode::ShiftLeft)),
            Self::Toggle => UserInput::Single(InputKind::Keyboard(KeyCode::F)),
        }
    }

    fn default_gamepad_input(action: InputAction) -> UserInput {
        // Match against the provided action to get the correct default gamepad input
        match action {
            Self::Move => UserInput::Single(InputKind::DualAxis(DualAxis::left_stick())),
            Self::Boost => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger))
            }
            Self::Toggle => UserInput::Single(InputKind::GamepadButton(GamepadButtonType::South)),
        }
    }
}

#[derive(Component)]
struct Player;

fn bind_input(mut commands: Commands) {
    // Create an `InputMap` to add default inputs to
    let mut input_map = InputMap::default();

    // Loop through each action in `InputAction` and get the default `UserInput`,
    // then insert each default input into input_map
    for action in InputAction::variants() {
        input_map.insert(InputAction::default_keyboard_mouse_input(action), action);
        input_map.insert(InputAction::default_gamepad_input(action), action);
    }

    // Spawn the player with the populated input_map
    commands
        .spawn(InputManagerBundle::<InputAction> {
            input_map,
            ..default()
        })
        .insert(Player);
}

fn use_actions(query: Query<&ActionState<InputAction>, With<Player>>) {
    let action_state = query.single();

    // When the default input for `InputAction::Move` is pressed, print the clamped direction of the axis
    if action_state.pressed(InputAction::Move) {
        println!(
            "Moving in direction {}",
            action_state
                .clamped_axis_pair(InputAction::Move)
                .unwrap()
                .xy()
        );
    }

    // When the default input for `InputAction::Boost` is pressed, print "Using Boost!"
    if action_state.just_pressed(InputAction::Boost) {
        println!("Using Boost!");
    }

    // When the default input for `InputAction::Toggle` is pressed, print "Toggled moveable actor!"
    if action_state.just_pressed(InputAction::Toggle) {
        println!("Toggled moveable actor!");
    }
}
