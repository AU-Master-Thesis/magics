#![deny(missing_docs)]

use bevy::{
    ecs::system::Res,
    input::{common_conditions::input_toggle_active, keyboard::KeyCode, ButtonInput},
};

/// Generates a [`Condition`]()-satisfying closure that returns
/// `true` when a keyboard input matching `keycode` has just been pressed.
///
/// # Example
///
/// ```
/// use bevy::{input::keycode::KeyCode, prelude::*};
/// fn main() {
///     App::new()
///         .add_systems(Update, my_system.run_if(pressed_key(KeyCode::KeyA)))
///         .run();
/// }
///
/// fn my_system() {
///     println!("just pressed a");
/// }
/// ```
pub fn pressed_key(keycode: KeyCode) -> impl Fn(Res<ButtonInput<KeyCode>>) -> bool {
    move |keyboard_input| keyboard_input.just_pressed(keycode)
}

/// Generates a [`Condition`]()-satisfying closure that returns
/// `true` when a keyboard input matching `keycode` has just been released.
///
/// # Example
///
/// ```
/// use bevy::{input::keycode::KeyCode, prelude::*};
/// fn main() {
///     App::new()
///         .add_systems(Update, my_system.run_if(released_key(KeyCode::KeyA)))
///         .run();
/// }
///
/// fn my_system() {
///     println!("just released a");
/// }
/// ```
pub fn released_key(keycode: KeyCode) -> impl Fn(Res<ButtonInput<KeyCode>>) -> bool {
    move |keyboard_input| keyboard_input.just_released(keycode)
}

// pub fn mouse_button_pressed()
