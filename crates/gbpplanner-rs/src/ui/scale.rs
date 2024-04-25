/// **Bevy** module that manages the scale/zoom of all egui UI elements.
use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseWheel, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{EguiPlugin, EguiSettings};

use super::UiState;
use crate::ui::UiScaleType;

/// **Bevy** plugin that manages the scale/zoom of all egui UI elements.
#[derive(Default)]
pub struct ScaleUiPlugin;

impl Plugin for ScaleUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.add_event::<ScaleUi>()
            .add_systems(Startup, scale_ui)
            .add_systems(Update, scale_ui);

        // Browser already use these input actions to scale the webpage
        if cfg!(not(target_arch = "wasm32")) {
            app.add_systems(
                Update,
                (Self::scale_ui_when_ctrl_scroll, Self::scale_ui_on_ctrl_plus_minus_equal),
            );
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiScale(f32);

impl Default for UiScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl std::fmt::Display for UiScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.percentage())
    }
}

#[allow(dead_code)]
impl UiScale {
    pub const MAX: f32 = 2.0;
    pub const MIN: f32 = 0.5;

    /// Get the UI scale as a percentage
    #[inline(always)]
    pub fn percentage(self) -> f32 {
        self.0 * 100.0
    }

    /// Set the UI scale
    #[inline(always)]
    pub const fn ratio(self) -> f32 {
        self.0
    }

    /// Set the UI scale
    /// The value is clamped between `UiScale::MIN` and `UiScale::MAX`
    pub fn set(&mut self, value: f32) {
        if (Self::MIN..=Self::MAX).contains(&value) {
            self.0 = value;
        }
    }
}

/// Simple **Bevy** trigger `Event`
/// Write to this event whenever you want the UI scale to update
#[derive(Event, Debug, Copy, Clone)]
pub enum ScaleUi {
    Reset,
    Set(f32),
    Increment(f32),
    Decrement(f32),
}

fn scale_ui(
    mut egui_settings: ResMut<EguiSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut ui_state: ResMut<UiState>,
    mut ui_scale_event: EventReader<ScaleUi>,
) {
    for event in ui_scale_event.read() {
        // info!("scale event: {:?}", event);
        let scale_factor = match ui_state.scale_type {
            UiScaleType::None => 1.0,
            UiScaleType::Window => {
                let primary_window = primary_window.single();
                1.0 / primary_window.scale_factor()
            }

            #[allow(clippy::cast_precision_loss)]
            UiScaleType::Custom => match event {
                ScaleUi::Reset => 1.0,
                ScaleUi::Set(scale) => *scale,
                ScaleUi::Increment(increment) => ui_state.scale_percent as f32 / 100.0 + increment,
                ScaleUi::Decrement(decrement) => ui_state.scale_percent as f32 / 100.0 - decrement,
            },
        };
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        ui_state.set_scale((scale_factor * 100.0) as usize);
        egui_settings.scale_factor = scale_factor;
    }
}

impl ScaleUiPlugin {
    pub const BUTTON_INCREMENT: f32 = 0.05;
    /// The increment/decrement amount used when the user scrolls the mouse
    /// wheel
    pub const SCROLL_INCREMENT: f32 = 0.01;

    /// **Bevy** system that scales the UI if the user is holding the `Control`
    /// key and scrolls up/down with the mouse wheel.
    fn scale_ui_when_ctrl_scroll(
        mut keyboard_events: EventReader<KeyboardInput>,
        mut mouse_wheel_events: EventReader<MouseWheel>,
        mut control_key_pressed: Local<bool>,
        mut ui_scale_event: EventWriter<ScaleUi>,
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

        if !*control_key_pressed {
            return;
        }

        for event in mouse_wheel_events.read() {
            if event.x != 0.0 {
                return;
            }

            if event.y > 0.0 {
                ui_scale_event.send(ScaleUi::Increment(Self::SCROLL_INCREMENT));
                return;
            } else if event.y < 0.0 {
                ui_scale_event.send(ScaleUi::Decrement(Self::SCROLL_INCREMENT));
                return;
            }
        }
    }

    /// **Bevy** system that scales the UI if the user presses ctrl +
    /// - <kbd>0</kbd>: reset the UI scale
    /// + <kbd>=</kbd>: increment the UI scale by `Self::BUTTON_INCREMENT`
    /// - <kbd>-</kbd>: decrement the UI scale by `Self::BUTTON_INCREMENT`
    fn scale_ui_on_ctrl_plus_minus_equal(
        mut keyboard_events: EventReader<KeyboardInput>,
        mut scale_ui_event: EventWriter<ScaleUi>,
        mut control_key_pressed: Local<bool>,
    ) {
        for event in keyboard_events.read() {
            match event.key_code {
                KeyCode::ControlLeft | KeyCode::ControlRight => match event.state {
                    ButtonState::Pressed => *control_key_pressed = true,
                    ButtonState::Released => *control_key_pressed = false,
                },
                _ => {}
            }

            if !*control_key_pressed {
                return;
            }

            let ButtonState::Pressed = event.state else {
                return;
            };

            let event = match event.key_code {
                KeyCode::Digit0 => ScaleUi::Reset,
                KeyCode::Equal => ScaleUi::Increment(Self::BUTTON_INCREMENT),
                KeyCode::Minus => ScaleUi::Decrement(Self::BUTTON_INCREMENT),
                _ => {
                    return;
                }
            };
            scale_ui_event.send(event);
        }
    }
}
