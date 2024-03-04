use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};
use strum_macros::EnumIter;

use super::super::{
    moveable_object::{self, MoveableObject, MoveableObjectMovementState},
    movement::{AngularVelocity, Velocity},
};

pub struct MoveableObjectInputPlugin;

impl Plugin for MoveableObjectInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InputManagerPlugin::<MoveableObjectAction>::default(),))
            .add_systems(PostStartup, (bind_moveable_object_input,))
            .add_systems(Update, (movement_actions,));
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, EnumIter)]
pub enum MoveableObjectAction {
    Move,
    RotateClockwise,
    RotateCounterClockwise,
    Boost,
    Toggle,
}

impl ToString for MoveableObjectAction {
    fn to_string(&self) -> String {
        match self {
            Self::Move => "Move".to_string(),
            Self::RotateClockwise => "Rotate Clockwise".to_string(),
            Self::RotateCounterClockwise => "Rotate Counter Clockwise".to_string(),
            Self::Boost => "Boost".to_string(),
            Self::Toggle => "Toggle".to_string(),
        }
    }
}

impl Default for MoveableObjectAction {
    fn default() -> Self {
        Self::Move
    }
}

impl MoveableObjectAction {
    fn variants() -> &'static [MoveableObjectAction] {
        &[
            MoveableObjectAction::Move,
            MoveableObjectAction::RotateClockwise,
            MoveableObjectAction::RotateCounterClockwise,
            MoveableObjectAction::Boost,
            MoveableObjectAction::Toggle,
        ]
    }

    fn default_keyboard_input(action: MoveableObjectAction) -> Option<UserInput> {
        match action {
            Self::Move => Some(UserInput::VirtualDPad(VirtualDPad::wasd())),
            Self::RotateClockwise => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyE))),
            Self::RotateCounterClockwise => {
                Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyQ)))
            }
            Self::Boost => Some(UserInput::Single(InputKind::PhysicalKey(
                KeyCode::ShiftLeft,
            ))),
            Self::Toggle => Some(UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyF))),
        }
    }

    fn default_gamepad_input(action: MoveableObjectAction) -> Option<UserInput> {
        match action {
            Self::Move => Some(UserInput::Single(InputKind::DualAxis(
                DualAxis::left_stick(),
            ))),
            Self::RotateClockwise => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::RightTrigger,
            ))),
            Self::RotateCounterClockwise => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::LeftTrigger,
            ))),
            Self::Boost => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::LeftTrigger2,
            ))),
            Self::Toggle => Some(UserInput::Single(InputKind::GamepadButton(
                GamepadButtonType::South,
            ))),
        }
    }
}
fn bind_moveable_object_input(mut commands: Commands, query: Query<Entity, With<MoveableObject>>) {
    // Create an `InputMap` to add default inputs to
    let mut input_map = InputMap::default();

    // Loop through each action in `MoveableObjectAction` and get the default `UserInput`,
    // then insert each default input into input_map

    // // input_map.insert()
    // for &action in &[
    //     MoveableObjectAction::Move,
    //     MoveableObjectAction::RotateClockwise,
    //     MoveableObjectAction::RotateCounterClockwise,
    //     MoveableObjectAction::Boost,
    //     MoveableObjectAction::Toggle,
    // ] {
    for &action in MoveableObjectAction::variants() {
        if let Some(input) = MoveableObjectAction::default_keyboard_input(action) {
            input_map.insert(action, input);
        }
        if let Some(input) = MoveableObjectAction::default_gamepad_input(action) {
            input_map.insert(action, input);
        }
    }

    if let Ok(entity) = query.get_single() {
        commands
            .entity(entity)
            .insert(InputManagerBundle::with_map(input_map));
        // .insert(InputManagerBundle::<MoveableObjectAction> {
        //     input_map,
        //     ..default()
        // });
    }
}

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
) {
    // let action_state = query.single();
    let Ok((action_state, mut angular_velocity, mut velocity)) = query.get_single_mut() else {
        return;
    };

    // When the default input for `MoveableObjectAction::Move` is pressed, print the clamped direction of the axis
    if action_state.pressed(&MoveableObjectAction::Move) {
        let scale = match state.get() {
            MoveableObjectMovementState::Default => moveable_object::SPEED,
            MoveableObjectMovementState::Boost => moveable_object::BOOST_SPEED,
        };

        if let Some(action) = action_state
            .clamped_axis_pair(&MoveableObjectAction::Move)
            .map(|axis| axis.xy().normalize_or_zero())
        {
            // let action = action_state
            //     .clamped_axis_pair(&MoveableObjectAction::Move)
            //     .unwrap()
            //     .xy()
            //     .normalize_or_zero();

            velocity.value = Vec3::new(-action.x, 0.0, action.y) * scale;
        }
    } else {
        velocity.value = Vec3::ZERO;
    }

    // When the default input for `MoveableObjectAction::Boost` is pressed, print "Using Boost!"
    // Using `just_pressed`, to only trigger once, even if held down, as we want a toggling behaviour
    // -> use `pressed`, if a while-held behaviour is desired
    if action_state.just_pressed(&MoveableObjectAction::Boost) {
        // info!("Using Boost!");
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
        action_state.pressed(&MoveableObjectAction::RotateClockwise),
        action_state.pressed(&MoveableObjectAction::RotateCounterClockwise),
    ) {
        (true, false) => -1.0,
        (false, true) => 1.0,
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
