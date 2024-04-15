#![forbid(missing_docs)]
//! Simple **Bevy** plugin that toggles fullscreen of the primary window.
//!
//! Users can toggle fullscreen by emitting the `ToggleFullscreenEvent` event.
//!
//! By default `F11` will be bound to toggle fullscreen. Users can change this
//! by setting `ToggleFullscreenPlugin`'s `bind_f11` field to `false`.
//!
//! # Examples
//! ```rust
//! use bevy::prelude::*;
//! use gbpplanner_rs::toggle_fullscreen::ToggleFullscreenPlugin;
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(ToggleFullscreenPlugin::default())
//!     .run();
//! ```

use bevy::{input::common_conditions::input_just_released, prelude::*, window::WindowMode};

/// prelude module bringing entire public API into score
pub mod prelude {
    pub use super::{ToggleFullscreen, ToggleFullscreenPlugin};
}

/// Event that toggles fullscreen of the primary window.
#[derive(Debug, Event)]
pub struct ToggleFullscreen;

/// Plugin that toggles fullscreen of the primary window.
#[derive(Debug)]
pub struct ToggleFullscreenPlugin {
    /// Keybind used to toggle fullscreen. If `None` no keybind is registered
    pub keybind: Option<KeyCode>,
}

impl Default for ToggleFullscreenPlugin {
    /// By default keybind is set to `Some(KeyCode::F11)`
    fn default() -> Self {
        Self {
            keybind: Some(KeyCode::F11),
        }
    }
}

impl Plugin for ToggleFullscreenPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(target_arch = "wasm32") {
            warn!(
                "ToggleFullscreenPlugin: on target 'wasm32' the window cannot be fullscreened. no \
                 systems registered."
            );
            return;
        }

        app.add_event::<ToggleFullscreen>()
            .add_systems(PostUpdate, toggle_fullscreen);

        if let Some(keycode) = self.keybind {
            app.add_systems(
                Update,
                emit_toggle_fullscreen.run_if(input_just_released(keycode)),
            );
        }
    }
}

// /// Emits `ToggleFullscreenEvent` when `F11` is pressed.
// fn emit_toggle_fullscreen_when_f11_is_pressed(
//     mut keyboard_input: EventReader<KeyboardInput>,
//     mut event_writer: EventWriter<ToggleFullscreenEvent>,
// ) {
//     for event in keyboard_input.read() {
//         if let (KeyCode::F11, ButtonState::Pressed) = (event.key_code,
// event.state) {             event_writer.send(ToggleFullscreenEvent);
//             return;
//         }
//     }
// }

/// Emit `ToggleFullscreenEvent` event
fn emit_toggle_fullscreen(mut event_writer: EventWriter<ToggleFullscreen>) {
    event_writer.send(ToggleFullscreen);
}

/// Toggles fullscreen of the primary window, when `ToggleFullscreenEvent` is
/// emitted.
fn toggle_fullscreen(
    mut query: Query<&mut Window>,
    mut event_reader: EventReader<ToggleFullscreen>,
) {
    for _ in event_reader.read() {
        for mut window in query.iter_mut() {
            use WindowMode::{BorderlessFullscreen, Fullscreen, SizedFullscreen, Windowed};
            let new_window_mode = match window.mode {
                Windowed => BorderlessFullscreen,
                SizedFullscreen | Fullscreen | BorderlessFullscreen => Windowed,
            };

            info!(
                "changed window mode from: {:?} to: {:?}",
                window.mode, new_window_mode
            );
            window.mode = new_window_mode;
        }
    }
}
