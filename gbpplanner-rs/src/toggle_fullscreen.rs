use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    window::WindowMode,
};

#[derive(Event)]
struct ToggleFullscreen;

pub struct ToggleFullscreenPlugin;

impl Plugin for ToggleFullscreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleFullscreen>().add_systems(
            Update,
            (
                emit_toggle_fullscreen_when_f11_is_pressed,
                toggle_fullscreen,
            ),
        );
    }
}

fn emit_toggle_fullscreen_when_f11_is_pressed(
    mut keyboard_input: EventReader<KeyboardInput>,
    mut event_writer: EventWriter<ToggleFullscreen>,
) {
    for event in keyboard_input.read() {
        if let (KeyCode::F11, ButtonState::Pressed) = (event.key_code, event.state) {
            event_writer.send(ToggleFullscreen);
            return;
        }
    }
}

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
