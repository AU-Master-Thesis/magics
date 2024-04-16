use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use leafwing_input_manager::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use super::{
    super::{
        environment::camera::{CameraMovementMode, MainCamera},
        movement::{AngularVelocity, Orbit, Velocity},
    },
    ChangingBinding,
};
use crate::{
    environment::{
        self,
        camera::{CameraResetEvent, CameraSettings},
    },
    movement::MovementPlugin,
    ui::ActionBlock,
};

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

/// **Bevy** [`Resource`] for the sensitivity of the movement of the
/// [`MoveableObject`] Works as a scaling factor for the movement and rotation
/// of the [`MoveableObject`] Defaults to 1.0 for both `move_sensitivity` and
/// `rotate_sensitivity`
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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, EnumIter, Default)]
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

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity
)]
fn camera_actions(
    state: Res<State<CameraMovementMode>>,
    mut next_state: ResMut<NextState<CameraMovementMode>>,
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
    // mut query_cameras: Query<&mut Camera>,
    currently_changing: Res<ChangingBinding>,
    action_block: Option<Res<ActionBlock>>,
    mut camera_reset_event: EventWriter<CameraResetEvent>,
    sensitivity: Res<CameraSensitivity>,
    // mut window: Query<&PrimaryWindow>,
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

    // if !*control_key_pressed {
    //     return;
    // }

    if let Ok((action_state, mut velocity, mut angular_velocity, orbit, transform, camera)) =
        query.get_single_mut()
    {
        let is_action_blocked =
            action_block.is_some() && action_block.as_ref().unwrap().is_blocked();

        if currently_changing.on_cooldown() || currently_changing.is_changing() || is_action_blocked
        {
            // info!("Camera actions are blocked");
            velocity.value = Vec3::ZERO;
            angular_velocity.value = Vec3::ZERO;
            return;
        }

        if !camera.is_active {
            return;
        }

        if action_state.just_pressed(&CameraAction::Reset) {
            camera_reset_event.send(CameraResetEvent);
        }

        let mut tmp_velocity = Vec3::ZERO;
        let mut tmp_angular_velocity = Vec3::ZERO;
        let camera_distance = transform.translation.distance(orbit.origin);

        let _window = windows.single();

        if action_state.pressed(&CameraAction::MouseMove) {
            // info!("Mouse move camera");
            match state.get() {
                CameraMovementMode::Pan => {
                    if let Some(action) = action_state
                        .axis_pair(&CameraAction::MouseMove)
                        .map(|axis| axis.xy())
                    {
                        tmp_velocity.x =
                            action.x * camera_distance * sensitivity.move_sensitivity / 10.0;
                        // * camera_settings.speed;
                        // * windows_height_scaling
                        tmp_velocity.z =
                            action.y * camera_distance * sensitivity.move_sensitivity / 10.0;
                        // * windows_height_scaling
                        // * camera_settings.speed;
                    }
                }
                CameraMovementMode::Orbit => {
                    if let Some(action) = action_state
                        .axis_pair(&CameraAction::MouseMove)
                        .map(|axis| axis.xy())
                    {
                        tmp_angular_velocity.x = -action.x * sensitivity.move_sensitivity / 10.0;
                        // * camera_settings.angular_speed;
                        // * windows_height_scaling
                        tmp_angular_velocity.y = action.y * sensitivity.move_sensitivity / 10.0;
                        // * camera_settings.angular_speed;
                        // * windows_height_scaling
                    }
                }
            }
        } else if action_state.pressed(&CameraAction::Move) {
            match state.get() {
                CameraMovementMode::Pan => {
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
                CameraMovementMode::Orbit => {
                    // action represents the direction to move the camera around it's origin
                    if let Some(action) = action_state
                        .clamped_axis_pair(&CameraAction::Move)
                        .map(|axis| axis.xy().normalize())
                    {
                        tmp_angular_velocity.x =
                            action.x * camera_settings.angular_speed * sensitivity.move_sensitivity;
                        tmp_angular_velocity.y =
                            action.y * camera_settings.angular_speed * sensitivity.move_sensitivity;
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
            if action_state.pressed(&CameraAction::ZoomIn) {
                // info!("Zooming in");
                tmp_velocity.y = -camera_settings.speed * camera_distance / 10.0;
            } else if action_state.pressed(&CameraAction::ZoomOut) {
                // info!("Zooming out");
                tmp_velocity.y = camera_settings.speed * camera_distance / 10.0;
            } else {
                tmp_velocity.y = 0.0;
            }
        }

        velocity.value = tmp_velocity;
        angular_velocity.value = tmp_angular_velocity;

        // Handling state changes
        if action_state.just_pressed(&CameraAction::ToggleMovementMode) {
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
}

// #[derive(Debug, Event, Clone, Copy)]
// pub enum ChangeCameraFocus;

fn switch_camera(
    query: Query<&ActionState<CameraAction>>,
    mut query_cameras: Query<(&mut Camera, Option<&MainCamera>)>,
    currently_changing: Res<ChangingBinding>,
    // mut
) {
    let action_state = query.single();
    if !action_state.just_pressed(&CameraAction::Switch) {
        return;
    }

    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }

    // let last_active_camera: Option<_> = query_cameras.iter_mut().find(|camera|
    // camera.is_active);

    // if let Some(last_active_camera) = query_cameras.iter_mut()
    //     .find(|camera| camera.is_active) {}

    // }
    // FIXME: avoid heap allocation
    // collect all cameras in a vector
    // let mut cameras = vec![query_main_camera.single_mut()];
    let mut cameras = vec![];

    let mut last_active_camera = 0;

    for (i, (camera, _main_camera)) in query_cameras.iter_mut().enumerate() {
        if camera.is_active {
            last_active_camera = i;
        }
        cameras.push(camera);
    }

    info!("cameras.len() = {}", cameras.len());

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
