use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use strum_macros::EnumIter;

use crate::ui::ChangingBinding;

use super::super::{
    camera::{self, CameraMovementMode, MainCamera},
    movement::{AngularVelocity, Orbit, Velocity},
};

pub struct CameraInputPlugin;

impl Plugin for CameraInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<CameraAction>::default(),))
            .add_systems(PostStartup, (bind_camera_input /*bind_camera_switch*/,))
            .add_systems(Update, (camera_actions, switch_camera));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, EnumIter)]
pub enum CameraAction {
    Move,
    MouseMove,
    ToggleMovementMode,
    ZoomIn,
    ZoomOut,
    Switch,
}

impl ToString for CameraAction {
    fn to_string(&self) -> String {
        match self {
            Self::Move => "Move".to_string(),
            Self::MouseMove => "Mouse Move".to_string(),
            Self::ToggleMovementMode => "Toggle Movement Mode".to_string(),
            Self::ZoomIn => "Zoom In".to_string(),
            Self::ZoomOut => "Zoom Out".to_string(),
            Self::Switch => "Switch".to_string(),
        }
    }
}

impl Default for CameraAction {
    fn default() -> Self {
        Self::Move
    }
}

impl CameraAction {
    fn variants() -> &'static [Self] {
        #[allow(clippy::enum_glob_use)]
        use CameraAction::*;
        &[Move, MouseMove, ToggleMovementMode, ZoomIn, ZoomOut, Switch]
    }

    fn default_mouse_input(action: CameraAction) -> Option<UserInput> {
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

    fn default_keyboard_input(action: CameraAction) -> Option<UserInput> {
        match action {
            Self::Move => Some(UserInput::VirtualDPad(VirtualDPad::arrow_keys())),
            Self::ToggleMovementMode => {
                Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyC)))
            }
            Self::Switch => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::Tab))),
            _ => None,
        }
    }

    fn default_gamepad_input(action: CameraAction) -> Option<UserInput> {
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

fn bind_camera_input(mut commands: Commands, query: Query<Entity, With<MainCamera>>) {
    let mut input_map = InputMap::default();

    for &action in CameraAction::variants() {
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

    if let Ok(entity) = query.get_single() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::with_map(input_map));
        // .insert(InputManagerBundle::<CameraAction> {
        //     input_map,
        //     ..default()
        // }
        // );
    }
}

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
) {
    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }
    if let Ok((action_state, mut velocity, mut angular_velocity, orbit, transform, camera)) =
        query.get_single_mut()
    {
        if !camera.is_active {
            return;
        }
        let mut tmp_velocity = Vec3::ZERO;
        let mut tmp_angular_velocity = Vec3::ZERO;
        let camera_distance = transform.translation.distance(orbit.origin);

        if action_state.pressed(&CameraAction::MouseMove) {
            // info!("Mouse move camera");
            match state.get() {
                CameraMovementMode::Pan => {
                    if let Some(action) = action_state
                        .axis_pair(&CameraAction::MouseMove)
                        .map(|axis| axis.xy())
                    {
                        tmp_velocity.x = action.x * camera_distance / 10.0; // * camera::SPEED;
                        tmp_velocity.z = action.y * camera_distance / 10.0; // * camera::SPEED;
                    }
                }
                CameraMovementMode::Orbit => {
                    if let Some(action) = action_state
                        .axis_pair(&CameraAction::MouseMove)
                        .map(|axis| axis.xy())
                    {
                        tmp_angular_velocity.x = -action.x * 0.2; // * camera::ANGULAR_SPEED;
                        tmp_angular_velocity.y = action.y * 0.2; // * camera::ANGULAR_SPEED;
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
                        tmp_velocity.x = -action.x * camera::SPEED * camera_distance / 35.0;
                        tmp_velocity.z = action.y * camera::SPEED * camera_distance / 35.0;
                    }
                }
                CameraMovementMode::Orbit => {
                    // action represents the direction to move the camera around it's origin
                    if let Some(action) = action_state
                        .clamped_axis_pair(&CameraAction::Move)
                        .map(|axis| axis.xy().normalize())
                    {
                        tmp_angular_velocity.x = action.x * camera::ANGULAR_SPEED;
                        tmp_angular_velocity.y = action.y * camera::ANGULAR_SPEED;
                    }
                }
            }
        } else {
            tmp_velocity.x = 0.0;
            tmp_velocity.z = 0.0;
            tmp_angular_velocity.x = 0.0;
            tmp_angular_velocity.y = 0.0;
        }

        if action_state.pressed(&CameraAction::ZoomIn) {
            // info!("Zooming in");
            tmp_velocity.y = -camera::SPEED * camera_distance / 10.0;
        } else if action_state.pressed(&CameraAction::ZoomOut) {
            // info!("Zooming out");
            tmp_velocity.y = camera::SPEED * camera_distance / 10.0;
        } else {
            tmp_velocity.y = 0.0;
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

fn switch_camera(
    query: Query<&ActionState<CameraAction>>,
    mut query_cameras: Query<&mut Camera>,
    currently_changing: Res<ChangingBinding>,
) {
    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }
    let action_state = query.single();

    // collect all cameras in a vector
    // let mut cameras = vec![query_main_camera.single_mut()];
    let mut cameras = vec![];
    let mut last_active_camera = 0;

    for (i, camera) in query_cameras.iter_mut().enumerate() {
        if camera.is_active {
            last_active_camera = i;
        }
        cameras.push(camera);
    }

    if action_state.just_pressed(&CameraAction::Switch) {
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
