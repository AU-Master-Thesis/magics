use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    environment::camera::{CameraMovement, MainCamera, events::ResetCamera, CameraSettings},
    movement::{AngularVelocity, Orbit, Velocity, MovementPlugin},
    ui::ActionBlock,
};

use super::ChangingBinding;

/// Plugin for camera input handling
#[derive(Default)]
pub struct CameraInputPlugin;

impl Plugin for CameraInputPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MovementPlugin>() {
            info!(
                "Automatically adding `MovementPlugin` to the app, as it is needed to handle \
                 camera movement"
            );
            app.add_plugins(MovementPlugin);
        }

        app.init_resource::<CameraSensitivity>()
            .add_plugins(InputManagerPlugin::<CameraAction>::default())
            .add_systems(PostStartup, bind_camera_input)
            .add_systems(Update, (camera_actions, switch_camera));
    }
}

/// Resource for camera sensitivity settings
#[derive(Resource)]
pub struct CameraSensitivity {
    pub move_sensitivity: f32,
}

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self {
            move_sensitivity: 1.0,
        }
    }
}

/// Camera action types for input handling
#[derive(
    Actionlike,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Hash,
    Debug,
    Reflect,
    EnumIter,
    Default,
    strum_macros::IntoStaticStr,
)]
pub enum CameraAction {
    #[default]
    Move,
    MouseMove,
    ToggleMovementMode,
    ZoomIn,
    ZoomOut,
    Switch,
    Reset,
}

impl std::fmt::Display for CameraAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::Move => "Move",
            Self::MouseMove => "Mouse Move",
            Self::ToggleMovementMode => "Toggle Movement Mode",
            Self::ZoomIn => "Zoom In",
            Self::ZoomOut => "Zoom Out",
            Self::Switch => "Switch",
            Self::Reset => "Reset",
        })
    }
}

impl CameraAction {
    fn default_mouse_input(action: Self) -> Option<UserInput> {
        match action {
            Self::MouseMove => Some(UserInput::Chord(vec![
                InputKind::Mouse(MouseButton::Left),
                InputKind::DualAxis(DualAxis::mouse_motion()),
            ])),
            Self::ZoomIn => Some(UserInput::Single(InputKind::MouseWheel(
                MouseWheelDirection::Down,
            ))),
            Self::ZoomOut => Some(UserInput::Single(InputKind::MouseWheel(
                MouseWheelDirection::Up,
            ))),
            _ => None,
        }
    }

    const fn default_keyboard_input(action: Self) -> Option<UserInput> {
        match action {
            Self::Move => Some(UserInput::VirtualDPad(VirtualDPad::arrow_keys())),
            Self::ToggleMovementMode => {
                Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyC)))
            }
            Self::Switch => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::Tab))),
            Self::Reset => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyR))),
            _ => None,
        }
    }

    const fn default_gamepad_input(action: Self) -> Option<UserInput> {
        match action {
            Self::Move => Some(UserInput::Single(InputKind::DualAxis(
                DualAxis::right_stick(),
            ))),
            Self::ToggleMovementMode => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::North,
            ))),
            Self::ZoomIn => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::DPadDown,
            ))),
            Self::ZoomOut => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::DPadUp,
            ))),
            Self::Switch => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::East,
            ))),
            _ => None,
        }
    }
}

/// System to bind camera input actions
fn bind_camera_input(mut commands: Commands, main_camera: Query<Entity, With<MainCamera>>) {
    let mut input_map = InputMap::default();

    for action in CameraAction::iter() {
        if let Some(input) = CameraAction::default_mouse_input(action) {
            input_map.insert(action, input);
        }
        if let Some(input) = CameraAction::default_keyboard_input(action) {
            input_map.insert(action, input);
        }
        if let Some(input) = CameraAction::default_gamepad_input(action) {
            input_map.insert(action, input);
        }
    }

    if let Ok(entity) = main_camera.get_single() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::with_map(input_map));
    }
}

/// System to handle camera actions
#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity
)]
fn camera_actions(
    state: Res<State<CameraMovement>>,
    mut next_state: ResMut<NextState<CameraMovement>>,
    mut query: Query<
        (
            &ActionState<CameraAction>,
            &mut Velocity,
            &mut AngularVelocity,
            &Orbit,
            &Transform,
            &Camera,
        ),
        With<MainCamera>,
    >,
    currently_changing: Res<ChangingBinding>,
    action_block: Option<Res<ActionBlock>>,
    mut camera_reset_event: EventWriter<ResetCamera>,
    sensitivity: Res<CameraSensitivity>,
    windows: Query<&Window>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut control_key_pressed: Local<bool>,
    camera_settings: Res<CameraSettings>,
) {
    for event in keyboard_events.read() {
        match event.key_code {
            KeyCode::ControlLeft | KeyCode::ControlRight => match event.state {
                ButtonState::Pressed => *control_key_pressed = true,
                ButtonState::Released => *control_key_pressed = false,
            },
            _ => {}
        }
    }

    if let Ok((action_state, mut velocity, mut angular_velocity, orbit, transform, camera)) =
        query.get_single_mut()
    {
        let is_action_blocked =
            action_block.is_some() && action_block.as_ref().unwrap().is_blocked();

        if currently_changing.on_cooldown() || currently_changing.is_changing() || is_action_blocked
        {
            // Reset velocity when actions are blocked
            velocity.0 = Vec3::ZERO;
            angular_velocity.value = Vec3::ZERO;
            return;
        }

        if !camera.is_active {
            return;
        }

        if action_state.just_pressed(&CameraAction::Reset) {
            camera_reset_event.send(ResetCamera);
        }

        let mut tmp_velocity = Vec3::ZERO;
        let mut tmp_angular_velocity = Vec3::ZERO;
        let camera_distance = transform.translation.distance(orbit.origin);

        let _window = windows.single();

        if action_state.pressed(&CameraAction::MouseMove) {
            match state.get() {
                CameraMovement::Pan => {
                    if let Some(action) = action_state
                        .axis_pair(&CameraAction::MouseMove)
                        .map(|axis| axis.xy())
                    {
                        tmp_velocity.x =
                            action.x * camera_distance * sensitivity.move_sensitivity / 10.0;
                        tmp_velocity.z =
                            action.y * camera_distance * sensitivity.move_sensitivity / 10.0;
                    }
                }
                CameraMovement::Orbit => {
                    if let Some(action) = action_state
                        .axis_pair(&CameraAction::MouseMove)
                        .map(|axis| axis.xy())
                    {
                        tmp_angular_velocity.x = -action.x * sensitivity.move_sensitivity / 10.0;
                        tmp_angular_velocity.y = action.y * sensitivity.move_sensitivity / 10.0;
                    }
                }
            }
        } else if action_state.pressed(&CameraAction::Move) {
            match state.get() {
                CameraMovement::Pan => {
                    if let Some(action) = action_state
                        .clamped_axis_pair(&CameraAction::Move)
                        .map(|axis| axis.xy().normalize_or_zero())
                    {
                        tmp_velocity.x = -action.x
                            * camera_settings.speed
                            * camera_distance
                            * sensitivity.move_sensitivity
                            / 35.0;
                        tmp_velocity.z = action.y
                            * camera_settings.speed
                            * camera_distance
                            * sensitivity.move_sensitivity
                            / 35.0;
                    }
                }
                CameraMovement::Orbit => {
                    // action represents the direction to move the camera around it's origin
                    if let Some(direction) = action_state
                        .clamped_axis_pair(&CameraAction::Move)
                        .map(|axis| axis.xy().normalize())
                    {
                        if direction.x.is_nan() {
                            tmp_angular_velocity.x = 0.0;
                        } else {
                            tmp_angular_velocity.x = direction.x
                                * camera_settings.angular_speed
                                * sensitivity.move_sensitivity;
                        }

                        if direction.y.is_nan() {
                            tmp_angular_velocity.y = 0.0;
                        } else {
                            tmp_angular_velocity.y = direction.y
                                * camera_settings.angular_speed
                                * sensitivity.move_sensitivity;
                        }
                    }
                }
            }
        } else {
            tmp_velocity.x = 0.0;
            tmp_velocity.z = 0.0;
            tmp_angular_velocity.x = 0.0;
            tmp_angular_velocity.y = 0.0;
        }

        if !*control_key_pressed {
            tmp_velocity.y = if action_state.pressed(&CameraAction::ZoomIn) {
                -camera_settings.speed * camera_distance / 10.0
            } else if action_state.pressed(&CameraAction::ZoomOut) {
                camera_settings.speed * camera_distance / 10.0
            } else {
                0.0
            }
        }

        velocity.0 = tmp_velocity;
        angular_velocity.value = tmp_angular_velocity;

        // Handle camera movement mode changes
        if action_state.just_pressed(&CameraAction::ToggleMovementMode) {
            let current = state.get();
            let next = current.next();
            next_state.set(next);
        }
    }
}

/// System to switch between cameras
fn switch_camera(
    query: Query<&ActionState<CameraAction>>,
    mut query_cameras: Query<(&mut Camera, Option<&MainCamera>)>,
    currently_changing: Res<ChangingBinding>,
) {
    let action_state = query.single();
    if !action_state.just_pressed(&CameraAction::Switch) {
        return;
    }

    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }

    // Collect cameras
    let mut cameras = vec![];
    let mut last_active_camera = 0;

    for (i, (camera, _main_camera)) in query_cameras.iter_mut().enumerate() {
        if camera.is_active {
            last_active_camera = i;
        }
        cameras.push(camera);
    }

    match &mut cameras[..] {
        [] => {
            error!("There are no cameras in the world");
        }
        [camera] => {
            if camera.is_active {
                warn!("There is only one camera in the world, and it is already active");
            } else {
                warn!("There is only one camera in the world, activating it");
                camera.is_active = true;
            }
        }
        _ => {
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
}
