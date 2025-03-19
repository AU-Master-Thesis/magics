use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    input::ChangingBinding,
    movement::{AngularVelocity, Velocity},
    moveable_object::{self, MoveableObject, MoveableObjectMovementState},
    ui::ActionBlock,
};

/// Plugin for moveable object input handling
pub struct MoveableObjectInputPlugin;

impl Plugin for MoveableObjectInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MoveableObjectSensitivity>()
            .add_plugins(InputManagerPlugin::<MoveableObjectAction>::default())
            .add_systems(PostStartup, bind_moveable_object_input)
            .add_systems(Update, movement_actions);
    }
}

/// Resource for controlling the sensitivity of moveable object movement
#[derive(Resource)]
pub struct MoveableObjectSensitivity {
    /// Sensitivity for movement (translation)
    pub move_sensitivity: f32,
    /// Sensitivity for rotation
    pub rotate_sensitivity: f32,
}

impl Default for MoveableObjectSensitivity {
    fn default() -> Self {
        Self {
            move_sensitivity: 1.0,
            rotate_sensitivity: 1.0,
        }
    }
}

/// Actions available for moveable objects
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, EnumIter, Default)]
pub enum MoveableObjectAction {
    #[default]
    /// Move the object in the XZ plane
    Move,
    /// Rotate the object clockwise
    RotateClockwise,
    /// Rotate the object counter-clockwise
    RotateCounterClockwise,
    /// Increase movement speed
    Boost,
    /// Toggle object state
    Toggle,
}

impl std::fmt::Display for MoveableObjectAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Move => write!(f, "Move"),
            Self::RotateClockwise => write!(f, "Rotate Clockwise"),
            Self::RotateCounterClockwise => write!(f, "Rotate Counter Clockwise"),
            Self::Boost => write!(f, "Boost"),
            Self::Toggle => write!(f, "Toggle"),
        }
    }
}

impl MoveableObjectAction {
    /// Get the default keyboard input for an action
    const fn default_keyboard_input(action: Self) -> UserInput {
        match action {
            Self::Move => UserInput::VirtualDPad(VirtualDPad::wasd()),
            Self::RotateClockwise => UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyE)),
            Self::RotateCounterClockwise => {
                UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyQ))
            }
            Self::Boost => UserInput::Single(InputKind::PhysicalKey(KeyCode::ShiftLeft)),
            Self::Toggle => UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyF)),
        }
    }

    /// Get the default gamepad input for an action
    const fn default_gamepad_input(action: Self) -> UserInput {
        match action {
            Self::Move => UserInput::Single(InputKind::DualAxis(DualAxis::left_stick())),
            Self::RotateClockwise => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::RightTrigger))
            }
            Self::RotateCounterClockwise => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger))
            }
            Self::Boost => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger2))
            }
            Self::Toggle => UserInput::Single(InputKind::GamepadButton(GamepadButtonType::South)),
        }
    }
}

/// System to bind default inputs to moveable objects
fn bind_moveable_object_input(mut commands: Commands, query: Query<Entity, With<MoveableObject>>) {
    // Create an input map with default bindings
    let mut input_map = InputMap::default();

    // Add keyboard and gamepad inputs for each action
    for action in MoveableObjectAction::iter() {
        let keyboard_input = MoveableObjectAction::default_keyboard_input(action);
        input_map.insert(action, keyboard_input);

        let gamepad_input = MoveableObjectAction::default_gamepad_input(action);
        input_map.insert(action, gamepad_input);
    }

    // Attach input management to each moveable object
    if let Ok(entity) = query.get_single() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::with_map(input_map));
    }
}

/// System to handle movement actions for moveable objects
fn movement_actions(
    mut next_state: ResMut<NextState<MoveableObjectMovementState>>,
    state: Res<State<MoveableObjectMovementState>>,
    mut query: Query<
        (
            &ActionState<MoveableObjectAction>,
            &mut AngularVelocity,
            &mut Velocity,
        ),
        With<MoveableObject>,
    >,
    currently_changing: Res<ChangingBinding>,
    action_block: Option<Res<ActionBlock>>,
    sensitivity: Res<MoveableObjectSensitivity>,
) {
    // Get the action state and velocity components from the moveable object
    let Ok((action_state, mut angular_velocity, mut velocity)) = query.get_single_mut() else {
        return;
    };

    // Check if input is blocked by binding changes or UI focus
    let is_blocked = currently_changing.on_cooldown() 
        || currently_changing.is_changing()
        || action_block.map_or(false, |block| block.is_blocked());
    
    if is_blocked {
        // If input is blocked, stop all movement
        velocity.0 = Vec3::ZERO;
        angular_velocity.value = Vec3::ZERO;
        return;
    }

    // Handle movement
    if action_state.pressed(&MoveableObjectAction::Move) {
        let scale = match state.get() {
            MoveableObjectMovementState::Default => moveable_object::SPEED,
            MoveableObjectMovementState::Boost => moveable_object::BOOST_SPEED,
        };

        if let Some(action) = action_state
            .clamped_axis_pair(&MoveableObjectAction::Move)
            .map(|axis| axis.xy().normalize_or_zero())
        {
            velocity.0 = Vec3::new(-action.x, 0.0, action.y) * scale * sensitivity.move_sensitivity;
        }
    } else {
        velocity.0 = Vec3::ZERO;
    }

    // Handle boost toggle
    if action_state.just_pressed(&MoveableObjectAction::Boost) {
        match state.get() {
            MoveableObjectMovementState::Default => {
                next_state.set(MoveableObjectMovementState::Boost);
            }
            MoveableObjectMovementState::Boost => {
                next_state.set(MoveableObjectMovementState::Default);
            }
        }
    }

    // Handle rotation
    let rotation = match (
        action_state.pressed(&MoveableObjectAction::RotateClockwise),
        action_state.pressed(&MoveableObjectAction::RotateCounterClockwise),
    ) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        // No rotation if neither or both are pressed
        (false, false) => 0.0,
        (true, true) => 0.0, // Instead of panicking, just don't rotate if both are pressed
    };

    let rotation_scale = match state.get() {
        MoveableObjectMovementState::Default => moveable_object::ANGULAR_SPEED,
        MoveableObjectMovementState::Boost => moveable_object::BOOST_ANGULAR_SPEED,
    };

    angular_velocity.value = Vec3::new(
        0.0,
        rotation * rotation_scale * sensitivity.rotate_sensitivity,
        0.0,
    );
}
