use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

use crate::{
    moveable_object::{
        self, MoveableObject, MoveableObjectMovementState, MoveableObjectVisibilityState,
    },
    movement::{MovingObjectBundle, Velocity},
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
            Self::MoveCamera => UserInput::VirtualDPad(VirtualDPad::arrow_keys()),
            Self::Boost => UserInput::Single(InputKind::Keyboard(KeyCode::ShiftLeft)),
            Self::Toggle => UserInput::Single(InputKind::Keyboard(KeyCode::F)),
        }
    }

    fn default_gamepad_input(action: InputAction) -> UserInput {
        // Match against the provided action to get the correct default gamepad input
        match action {
            Self::MoveObject => UserInput::Single(InputKind::DualAxis(DualAxis::left_stick())),
            Self::MoveCamera => UserInput::Single(InputKind::DualAxis(DualAxis::right_stick())),
            Self::Boost => {
                UserInput::Single(InputKind::GamepadButton(GamepadButtonType::LeftTrigger))
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

    // // Spawn the player with the populated input_map
    // commands.spawn(InputManagerBundle::<InputAction> {
    //     input_map,
    //     ..default()
    // });

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
        (&ActionState<InputAction>, &mut Transform, &mut Velocity),
        With<MoveableObject>,
    >,
) {
    // let action_state = query.single();
    let Ok((action_state, mut transform, mut velocity)) = query.get_single_mut() else {
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
            .xy();
        // .extend(0.0)
        // * scale;

        velocity.value = Vec3::new(-action.x, 0.0, action.y) * scale;

        println!(
            "Moving in direction {}",
            action_state
                .clamped_axis_pair(InputAction::MoveObject)
                .unwrap()
                .xy()
        );
    }

    // When the default input for `InputAction::Boost` is pressed, print "Using Boost!"
    if action_state.just_pressed(InputAction::Boost) {
        println!("Using Boost!");
        match state.get() {
            MoveableObjectMovementState::Default => {
                next_state.set(MoveableObjectMovementState::Boost);
            }
            MoveableObjectMovementState::Boost => {
                next_state.set(MoveableObjectMovementState::Default);
            }
        }
    }
}

// fn visibility_actions(
//     mut next_state: ResMut<NextState<MoveableObjectVisibilityState>>,
//     state: Res<State<MoveableObjectVisibilityState>>,
//     mut query: Query<(&ActionState<InputAction>, &Handle<ColorMaterial>)>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
// ) {
//     let (action_state, mut _handle) = query.single_mut();

//     // When the default input for `InputAction::Toggle` is pressed, print "Toggled moveable actor!"
//     if action_state.just_pressed(InputAction::Toggle) {
//         println!("Toggled moveable actor!");
//         // toogle the MoveableObject
//         if let Some(material) = materials.get_mut(_handle) {
//             match state.get() {
//                 MoveableObjectVisibilityState::Visible => {
//                     // hide the moveable object by setting the alpha to 0
//                     let mut color = material.color;
//                     color.set_a(0.0);
//                     material.color = color;
//                     next_state.set(MoveableObjectVisibilityState::Hidden);
//                 }
//                 MoveableObjectVisibilityState::Hidden => {
//                     // show the moveable object by setting the alpha to 1
//                     let mut color = material.color;
//                     color.set_a(1.0);
//                     material.color = color;
//                     next_state.set(MoveableObjectVisibilityState::Visible);
//                 }
//             }
//         }
//     }
// }
