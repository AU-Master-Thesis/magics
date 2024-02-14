use bevy::{core_pipeline::core_2d::graph::input, prelude::*};
use leafwing_input_manager::{
    axislike::{MouseMotionAxisType, VirtualAxis},
    prelude::*,
    user_input::InputKind,
};

use crate::{
    camera::{self, CameraMovementMode, MainCamera},
    follow_cameras::FollowCameraMe,
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
                    bind_camera_switch,
                    // somthing more
                ),
            )
            .add_systems(
                Update,
                (
                    movement_actions,
                    camera_actions,
                    switch_camera,
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
    MouseMoveCamera,
    ToggleCameraMovementMode,
    ZoomIn,
    ZoomOut,
    SwitchCamera,
}

fn something() -> UserInput {
    UserInput::Single(InputKind::Keyboard(KeyCode::P))
}

// Exhaustively match `InputAction` and define the default binding to the input
impl InputAction {
    fn default_mouse_input(action: InputAction) -> UserInput {
        use InputAction::*;
        match action {
            MoveObject => something(),
            RotateObjectClockwise => something(),
            RotateObjectCounterClockwise => something(),
            Boost => something(),
            Toggle => something(),
            MoveCamera => something(),
            MouseMoveCamera => {
                // UserInput::Single(InputKind::DualAxis(DualAxis::mouse_motion()))
                UserInput::Chord(vec![
                    InputKind::Mouse(MouseButton::Left),
                    InputKind::DualAxis(DualAxis::mouse_motion()),
                ])
            }
            ToggleCameraMovementMode => something(),
            ZoomIn => something(),
            ZoomOut => something(),
            SwitchCamera => something(),
        }
    }

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
            Self::MouseMoveCamera => something(),
            Self::ToggleCameraMovementMode => UserInput::Single(InputKind::Keyboard(KeyCode::C)),
            Self::ZoomIn => UserInput::Single(InputKind::MouseWheel(MouseWheelDirection::Down)),
            Self::ZoomOut => UserInput::Single(InputKind::MouseWheel(MouseWheelDirection::Up)),
            Self::SwitchCamera => UserInput::Single(InputKind::Keyboard(KeyCode::Tab)),
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
            Self::MouseMoveCamera => something(),
            Self::ToggleCameraMovementMode => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::North))
            }
            Self::ZoomIn => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadDown))
            }
            Self::ZoomOut => UserInput::Single(InputKind::GamepadButton(GamepadButtonType::DPadUp)),
            Self::SwitchCamera => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::East))
            }
        }
    }
}

fn bind_camera_input(mut commands: Commands, query: Query<(Entity, With<MainCamera>)>) {
    let mut input_map = InputMap::default();

    for action in InputAction::variants() {
        input_map.insert(InputAction::default_mouse_input(action), action);
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
            &Orbit,
            &Transform,
        ),
        With<MainCamera>,
    >,
) {
    if let Ok((action_state, mut velocity, mut angular_velocity, orbit, transform)) =
        query.get_single_mut()
    {
        let mut tmp_velocity = Vec3::ZERO;
        let mut tmp_angular_velocity = Vec3::ZERO;
        let camera_distance = transform.translation.distance(orbit.origin);

        if action_state.pressed(InputAction::MouseMoveCamera) {
            info!("Mouse move camera");
            match state.get() {
                CameraMovementMode::Pan => {
                    let action = action_state
                        .axis_pair(InputAction::MouseMoveCamera)
                        .unwrap()
                        .xy();

                    // velocity.value = Vec3::new(-action.x, 0.0, action.y) * camera::SPEED;
                    // tmp_velocity = Vec3::new(-action.x, 0.0, action.y) * camera::SPEED;
                    tmp_velocity.x = action.x * camera_distance / 100.0; // * camera::SPEED;
                    tmp_velocity.z = action.y * camera_distance / 100.0; // * camera::SPEED;
                }
                CameraMovementMode::Orbit => {
                    let action = action_state
                        .axis_pair(InputAction::MouseMoveCamera)
                        .unwrap()
                        .xy();

                    // angular_velocity.value = Vec3::new(-action.x, 0.0, action.y) * camera::SPEED;
                    tmp_angular_velocity.x = -action.x * 0.2; // * camera::ANGULAR_SPEED;
                    tmp_angular_velocity.y = action.y * 0.2; // * camera::ANGULAR_SPEED;
                }
            }
        } else if action_state.pressed(InputAction::MoveCamera) {
            match state.get() {
                CameraMovementMode::Pan => {
                    let action = action_state
                        .clamped_axis_pair(InputAction::MoveCamera)
                        .unwrap()
                        .xy()
                        .normalize_or_zero();

                    tmp_velocity.x = -action.x * camera::SPEED * camera_distance / 35.0;
                    tmp_velocity.z = action.y * camera::SPEED * camera_distance / 35.0;

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
            tmp_velocity.y = -camera::SPEED * camera_distance / 10.0;
        } else if action_state.pressed(InputAction::ZoomOut) {
            info!("Zooming out");
            tmp_velocity.y = camera::SPEED * camera_distance / 10.0;
        } else {
            tmp_velocity.y = 0.0;
        }

        velocity.value = tmp_velocity;
        angular_velocity.value = tmp_angular_velocity;

        // Handling state changes
        if action_state.just_pressed(InputAction::ToggleCameraMovementMode) {
            next_state.set(match state.get() {
                CameraMovementMode::Pan => {
                    info!("Toggling camera mode: Linear -> Orbit");
                    CameraMovementMode::Orbit
                }
                CameraMovementMode::Orbit => {
                    info!("Toggling camera mode: Orbit -> Linear");
                    CameraMovementMode::Pan
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

#[derive(Component)]
pub struct GlobalInputs;

fn bind_camera_switch(mut commands: Commands) {
    let mut input_map = InputMap::default();
    input_map.insert(
        UserInput::Single(InputKind::Keyboard(KeyCode::Tab)),
        InputAction::SwitchCamera,
    );

    commands.spawn((
        InputManagerBundle::<InputAction> {
            input_map,
            ..default()
        },
        GlobalInputs,
    ));
}

fn switch_camera(
    query: Query<&ActionState<InputAction>, With<GlobalInputs>>,
    mut query_follow_cameras: Query<&mut Camera, With<FollowCameraMe>>,
    mut query_main_camera: Query<&mut Camera, (With<MainCamera>, Without<FollowCameraMe>)>,
) {
    let action_state = query.single();

    // collect all cameras in a vector
    let mut cameras = vec![query_main_camera.single_mut()];
    let mut last_active_camera = 0;
    for (i, camera) in query_follow_cameras.iter_mut().enumerate() {
        if camera.is_active {
            last_active_camera = i;
        }
        cameras.push(camera);
    }

    if action_state.just_pressed(InputAction::SwitchCamera) {
        let next_active_camera = (last_active_camera + 1) % cameras.len();
        info!(
            "Switching camera from {} to {}, with a total of {} cameras",
            last_active_camera,
            next_active_camera,
            cameras.len()
        );
        cameras[last_active_camera].is_active = false;
        cameras[next_active_camera].is_active = true;
    }
}
