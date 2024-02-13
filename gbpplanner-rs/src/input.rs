use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use crate::{
    camera::{self, CameraMovementMode, MainCamera},
    moveable_object::{
        self, MoveableObject, MoveableObjectMovementState, MoveableObjectVisibilityState,
    },
    movement::{AngularVelocity, Orbit, Velocity},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<InputAction>::default())
            .add_systems(
                PostStartup,
                (
                    bind_moveable_object_input,
                    bind_camera_input,
                    // somthing more
                ),
            )
            .add_systems(
                Update,
                (
                    movement_actions,
                    camera_actions,
                    // somthing more
                ),
            );
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum InputAction {
    MoveObject,
    RotateObjectClockwise,
    RotateObjectCounterClockwise,
    Boost,
    Toggle,
    MoveCamera,
    ToggleCameraMovementMode,
    ZoomIn,
    ZoomOut,
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
            Self::Boost => UserInput::Single(InputKind::Keyboard(KeyCode::ShiftLeft)),
            Self::Toggle => UserInput::Single(InputKind::Keyboard(KeyCode::F)),
            Self::MoveCamera => UserInput::VirtualDPad(VirtualDPad::arrow_keys()),
            Self::ToggleCameraMovementMode => UserInput::Single(InputKind::Keyboard(KeyCode::C)),
            Self::ZoomIn => UserInput::Single(InputKind::MouseWheel(MouseWheelDirection::Down)),
            Self::ZoomOut => UserInput::Single(InputKind::MouseWheel(MouseWheelDirection::Up)),
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
            Self::Boost => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger2))
            }
            Self::Toggle => UserInput::Single(InputKind::GamepadButton(GamepadButtonType::South)),
            Self::MoveCamera => UserInput::Single(InputKind::DualAxis(DualAxis::right_stick())),
            Self::ToggleCameraMovementMode => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::North))
            }
            Self::ZoomIn => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadDown))
            }
            Self::ZoomOut => UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadUp)),
        }
    }
}

fn bind_camera_input(mut commands: Commands, query: Query<(Entity, With<MainCamera>)>) {
    let mut input_map = InputMap::default();

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

fn camera_actions(
    state: Res<State<CameraMovementMode>>,
    mut next_state: ResMut<NextState<CameraMovementMode>>,
    mut query: Query<
        (
            &ActionState<InputAction>,
            &mut Velocity,
            &mut AngularVelocity,
        ),
        With<MainCamera>,
    >,
) {
    if let Ok((action_state, mut velocity, mut angular_velocity)) = query.get_single_mut() {
        let mut tmp_velocity = Vec3::ZERO;
        let mut tmp_angular_velocity = Vec3::ZERO;

        if action_state.pressed(InputAction::MoveCamera) {
            match state.get() {
                CameraMovementMode::Linear => {
                    let action = action_state
                        .clamped_axis_pair(InputAction::MoveCamera)
                        .unwrap()
                        .xy()
                        .normalize_or_zero();

                    tmp_velocity.x = -action.x * camera::SPEED;
                    tmp_velocity.z = action.y * camera::SPEED;

                    info!(
                        "Moving camera in direction {}",
                        action_state
                            .clamped_axis_pair(InputAction::MoveCamera)
                            .unwrap()
                            .xy()
                    );
                }
                CameraMovementMode::Orbit => {
                    // action represents the direction to move the camera around it's origin
                    let action = action_state
                        .clamped_axis_pair(InputAction::MoveCamera)
                        .unwrap()
                        .xy()
                        .normalize();

                    tmp_angular_velocity.x = action.x * camera::ANGULAR_SPEED;
                    tmp_angular_velocity.y = action.y * camera::ANGULAR_SPEED;
                }
            }
        } else {
            tmp_velocity.x = 0.0;
            tmp_velocity.z = 0.0;
            tmp_angular_velocity.x = 0.0;
            tmp_angular_velocity.y = 0.0;
        }

        if action_state.pressed(InputAction::ZoomIn) {
            info!("Zooming in");
            tmp_velocity.y = -camera::SPEED;
        } else if action_state.pressed(InputAction::ZoomOut) {
            info!("Zooming out");
            tmp_velocity.y = camera::SPEED;
        } else {
            tmp_velocity.y = 0.0;
        }

        velocity.value = tmp_velocity;
        angular_velocity.value = tmp_angular_velocity;

        // Handling state changes
        if action_state.just_pressed(InputAction::ToggleCameraMovementMode) {
            next_state.set(match state.get() {
                CameraMovementMode::Linear => {
                    info!("Toggling camera mode: Linear -> Orbit");
                    CameraMovementMode::Orbit
                }
                CameraMovementMode::Orbit => {
                    info!("Toggling camera mode: Orbit -> Linear");
                    CameraMovementMode::Linear
                }
            });
        }
    }

    // toggle camera movement mode
}

fn bind_moveable_object_input(
    mut commands: Commands,
    query: Query<(Entity, With<MoveableObject>)>,
) {
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
            .normalize_or_zero();

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
