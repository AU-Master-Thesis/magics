use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
// use bevy_inspector_egui::egui::scroll_area::State;
use leafwing_input_manager::{prelude::*, user_input::InputKind};

pub struct InputPlugin;
// {
//     config: Config,
// }

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<InputAction>::default())
            .add_state::<MoveableObjectMovementState>()
            .add_state::<MoveableObjectVisibilityState>()
            .add_systems(Startup, bind_input)
            .add_systems(Update, (visibility_actions, movement_actions));
    }
}

/// Here, we define a State for Scenario.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MoveableObjectMovementState {
    #[default]
    Default,
    Boost,
}

// define visibility state for the moveable object
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MoveableObjectVisibilityState {
    #[default]
    Visible,
    Hidden,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum InputAction {
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

#[derive(Component)]
struct MoveableObject;

fn bind_input(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create an `InputMap` to add default inputs to
    let mut input_map = InputMap::default();

    // Loop through each action in `InputAction` and get the default `UserInput`,
    // then insert each default input into input_map
    for action in InputAction::variants() {
        input_map.insert(InputAction::default_keyboard_mouse_input(action), action);
        input_map.insert(InputAction::default_gamepad_input(action), action);
    }

    // Spawn the player with the populated input_map
    commands
        .spawn(InputManagerBundle::<InputAction> {
            input_map,
            ..default()
        })
        .insert(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(50.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            // transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        })
        .insert(MoveableObject);
}

fn movement_actions(
    mut next_state: ResMut<NextState<MoveableObjectMovementState>>,
    state: Res<State<MoveableObjectMovementState>>,
    mut query: Query<(&ActionState<InputAction>, &mut Transform)>,
) {
    // let action_state = query.single();
    let (action_state, mut _moveable_object) = query.single_mut();

    // When the default input for `InputAction::Move` is pressed, print the clamped direction of the axis
    if action_state.pressed(InputAction::MoveObject) {
        let scale = match state.get() {
            MoveableObjectMovementState::Default => 3.0,
            MoveableObjectMovementState::Boost => 10.0,
        };
        _moveable_object.translation += action_state
            .clamped_axis_pair(InputAction::MoveObject)
            .unwrap()
            .xy()
            .extend(0.0)
            * scale;
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

    // // When the default input for `InputAction::Toggle` is pressed, print "Toggled moveable actor!"
    // if action_state.just_pressed(InputAction::Toggle) {
    //     println!("Toggled moveable actor!");
    //     // toogle the MoveableObject
    // }
}

fn visibility_actions(
    mut next_state: ResMut<NextState<MoveableObjectVisibilityState>>,
    state: Res<State<MoveableObjectVisibilityState>>,
    mut query: Query<(&ActionState<InputAction>, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (action_state, mut _handle) = query.single_mut();

    // When the default input for `InputAction::Toggle` is pressed, print "Toggled moveable actor!"
    if action_state.just_pressed(InputAction::Toggle) {
        println!("Toggled moveable actor!");
        // toogle the MoveableObject
        if let Some(material) = materials.get_mut(_handle) {
            match state.get() {
                MoveableObjectVisibilityState::Visible => {
                    // hide the moveable object by setting the alpha to 0
                    let mut color = material.color;
                    color.set_a(0.0);
                    material.color = color;
                    next_state.set(MoveableObjectVisibilityState::Hidden);
                }
                MoveableObjectVisibilityState::Hidden => {
                    // show the moveable object by setting the alpha to 1
                    let mut color = material.color;
                    color.set_a(1.0);
                    material.color = color;
                    next_state.set(MoveableObjectVisibilityState::Visible);
                }
            }
        }
    }
}
