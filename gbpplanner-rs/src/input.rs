use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
// use bevy_input_mapper::{
//     input::{events::*, gamepad::GamepadAxis},
//     InputMapper, InputMapperPlugin,
// };

pub struct InputPlugin;
// {
//     config: Config,
// }

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(InputManagerPlugin)
            // .add_state::<CameraMode>()
            // .add_plugins(InputMapperPlugin::<CameraMode>::new())
            // .add_system(Startup, bind_input)
            // .add_system(Update, handle_input)
            // .add_system(Update, logger);
            // This plugin maps inputs to an input-type agnostic action-state
            // We need to provide it with an enum which stores the possible actions a player could take
            .add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(Startup, spawn_player)
            .add_systems(Update, use_actions);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
    Run,
    Jump,
    UseItem,
}

// Exhaustively match `PlayerAction` and define the default binding to the input
impl PlayerAction {
    fn default_keyboard_mouse_input(action: PlayerAction) -> UserInput {
        // Match against the provided action to get the correct default keyboard-mouse input
        match action {
            Self::Run => UserInput::VirtualDPad(VirtualDPad::wasd()),
            Self::Jump => UserInput::Single(InputKind::Keyboard(KeyCode::Space)),
            Self::UseItem => UserInput::Single(InputKind::Mouse(MouseButton::Left)),
        }
    }

    fn default_gamepad_input(action: PlayerAction) -> UserInput {
        // Match against the provided action to get the correct default gamepad input
        match action {
            Self::Run => UserInput::Single(InputKind::DualAxis(DualAxis::left_stick())),
            Self::Jump => UserInput::Single(InputKind::GamepadButton(GamepadButtonType::South)),
            Self::UseItem => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::RightTrigger2))
            }
        }
    }
}

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    // Create an `InputMap` to add default inputs to
    let mut input_map = InputMap::default();

    // Loop through each action in `PlayerAction` and get the default `UserInput`,
    // then insert each default input into input_map
    for action in PlayerAction::variants() {
        input_map.insert(PlayerAction::default_keyboard_mouse_input(action), action);
        input_map.insert(PlayerAction::default_gamepad_input(action), action);
    }

    // Spawn the player with the populated input_map
    commands
        .spawn(InputManagerBundle::<PlayerAction> {
            input_map,
            ..default()
        })
        .insert(Player);
}

fn use_actions(query: Query<&ActionState<PlayerAction>, With<Player>>) {
    let action_state = query.single();

    // When the default input for `PlayerAction::Run` is pressed, print the clamped direction of the axis
    if action_state.pressed(PlayerAction::Run) {
        println!(
            "Moving in direction {}",
            action_state
                .clamped_axis_pair(PlayerAction::Run)
                .unwrap()
                .xy()
        );
    }

    // When the default input for `PlayerAction::Jump` is pressed, print "Jump!"
    if action_state.just_pressed(PlayerAction::Jump) {
        println!("Jumped!");
    }

    // When the default input for `PlayerAction::UseItem` is pressed, print "Used an Item!"
    if action_state.just_pressed(PlayerAction::UseItem) {
        println!("Used an Item!");
    }
}
