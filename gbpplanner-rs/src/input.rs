use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use crate::{
    moveable_object::{
        self, MoveableObject, MoveableObjectMovementState, MoveableObjectVisibilityState,
    },
    movement::{AngularVelocity, MovingObjectBundle, Velocity},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<InputAction>::default())
            .add_systems(PostStartup, bind_input)
            .add_systems(Update, movement_actions);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum InputAction {
    MoveObject,
    RotateObjectClockwise,
    RotateObjectCounterClockwise,
    MoveCamera,
    Boost,
    Toggle,
}

// Exhaustively match `InputAction` and define the default binding to the input
impl InputAction {
    fn default_keyboard_mouse_input(action: InputAction) -> UserInput {
        // Match against the provided action to get the correct default keyboard-mouse input
        match action {
            Self::MoveObject => UserInput::VirtualDPad(VirtualDPad::wasd()),
            Self::RotateObjectClockwise => UserInput::Single(InputKind::Keyboard(KeyCode::E)),
            Self::RotateObjectCounterClockwise => {
                UserInput::Single(InputKind::Keyboard(KeyCode::Q))
            }
            Self::MoveCamera => UserInput::VirtualDPad(VirtualDPad::arrow_keys()),
            Self::Boost => UserInput::Single(InputKind::Keyboard(KeyCode::ShiftLeft)),
            Self::Toggle => UserInput::Single(InputKind::Keyboard(KeyCode::F)),
        }
    }

    fn default_gamepad_input(action: InputAction) -> UserInput {
        // Match against the provided action to get the correct default gamepad input
        match action {
            Self::MoveObject => UserInput::Single(InputKind::DualAxis(DualAxis::left_stick())),
            Self::RotateObjectClockwise => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::RightTrigger))
            }
            Self::RotateObjectCounterClockwise => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger))
            }
            Self::MoveCamera => UserInput::Single(InputKind::DualAxis(DualAxis::right_stick())),
            Self::Boost => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger2))
            }
            Self::Toggle => UserInput::Single(InputKind::GamepadButton(GamepadButtonType::South)),
        }
    }
}

fn bind_input(mut commands: Commands, query: Query<(Entity, With<MoveableObject>)>) {
    // Create an `InputMap` to add default inputs to
    let mut input_map = InputMap::default();

    // Loop through each action in `InputAction` and get the default `UserInput`,
    // then insert each default input into input_map
    for action in InputAction::variants() {
        input_map.insert(InputAction::default_keyboard_mouse_input(action), action);
        input_map.insert(InputAction::default_gamepad_input(action), action);
    }

    if let Ok((entity, _)) = query.get_single() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::<InputAction> {
                input_map,
                ..default()
            });
    }
}

fn movement_actions(
    mut next_state: ResMut<NextState<MoveableObjectMovementState>>,
    state: Res<State<MoveableObjectMovementState>>,
    mut query: Query<
        (
            &ActionState<InputAction>,
            &mut AngularVelocity,
            &mut Velocity,
        ),
        With<MoveableObject>,
    >,
) {
    // let action_state = query.single();
    let Ok((action_state, mut angular_velocity, mut velocity)) = query.get_single_mut() else {
        return;
    };

    // When the default input for `InputAction::Move` is pressed, print the clamped direction of the axis
    if action_state.pressed(InputAction::MoveObject) {
        let scale = match state.get() {
            MoveableObjectMovementState::Default => moveable_object::SPEED,
            MoveableObjectMovementState::Boost => moveable_object::BOOST_SPEED,
        };

        let action = action_state
            .clamped_axis_pair(InputAction::MoveObject)
            .unwrap()
            .xy()
            .normalize();

        velocity.value = Vec3::new(-action.x, 0.0, action.y) * scale;

        info!(
            "Moving in direction {}",
            action_state
                .clamped_axis_pair(InputAction::MoveObject)
                .unwrap()
                .xy()
        );
    } else {
        velocity.value = Vec3::ZERO;
    }

    // When the default input for `InputAction::Boost` is pressed, print "Using Boost!"
    // Using `just_pressed`, to only trigger once, even if held down, as we want a toggling behaviour
    // -> use `pressed`, if a while-held behaviour is desired
    if action_state.just_pressed(InputAction::Boost) {
        info!("Using Boost!");
        match state.get() {
            MoveableObjectMovementState::Default => {
                next_state.set(MoveableObjectMovementState::Boost);
            }
            MoveableObjectMovementState::Boost => {
                next_state.set(MoveableObjectMovementState::Default);
            }
        }
    }

    // Rotation
    let rotation = match (
        action_state.pressed(InputAction::RotateObjectClockwise),
        action_state.pressed(InputAction::RotateObjectCounterClockwise),
    ) {
        (true, false) => {
            info!("Rotation -1");
            -1.0
        }
        (false, true) => {
            info!("Rotation 1");
            1.0
        }
        // Handles both false or both true cases, resulting in no rotation.
        _ => 0.0,
    };

    let rotation_scale = match state.get() {
        MoveableObjectMovementState::Default => moveable_object::ANGULAR_SPEED,
        MoveableObjectMovementState::Boost => moveable_object::BOOST_ANGULAR_SPEED,
    };

    angular_velocity.value = Vec3::new(
        0.0,
        rotation * rotation_scale * moveable_object::ANGULAR_SPEED,
        0.0,
    );
}
