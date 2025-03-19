//! Pause and play functionality for the simulation
//!
//! This module contains systems for handling pause/play controls
//! for the simulation, enabling user interaction with the simulation timing.

use bevy::prelude::*;
use crate::SimulationState;

/// Plugin for pause/play functionality
pub struct PausePlayPlugin;

impl Default for PausePlayPlugin {
    fn default() -> Self {
        Self
    }
}

/// Resource for managing simulation speed
#[derive(Resource)]
pub struct SimulationSpeed {
    /// Current time scale factor 
    /// (1.0 = normal speed, 0.5 = half speed, 2.0 = double speed)
    pub scale: f32,
}

impl Default for SimulationSpeed {
    fn default() -> Self {
        Self {
            scale: 1.0,
        }
    }
}

impl Plugin for PausePlayPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SimulationSpeed>()
            .add_systems(Update, (
                handle_pause_play_input,
                handle_speed_control_input,
            ));
    }
}

/// Handle keyboard input for pausing/playing the simulation
fn handle_pause_play_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut simulation_state: Option<ResMut<SimulationState>>,
) {
    // Skip if there's no simulation
    let Some(mut simulation_state) = simulation_state else { return };

    // Toggle pause state when Space is pressed
    if keyboard_input.just_pressed(KeyCode::Space) {
        simulation_state.paused = !simulation_state.paused;

        if simulation_state.paused {
            info!("Simulation paused");
        } else {
            info!("Simulation resumed");
        }
    }
}

/// Handle keyboard input for controlling simulation speed
fn handle_speed_control_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut simulation_speed: ResMut<SimulationSpeed>,
) {
    // Increase speed with '+' key
    if keyboard_input.just_pressed(KeyCode::Plus) || keyboard_input.just_pressed(KeyCode::KeypadAdd) {
        simulation_speed.scale *= 1.5;
        info!("Simulation speed: {:.2}x", simulation_speed.scale);
    }

    // Decrease speed with '-' key
    if keyboard_input.just_pressed(KeyCode::Minus) || keyboard_input.just_pressed(KeyCode::KeypadSubtract) {
        simulation_speed.scale /= 1.5;
        info!("Simulation speed: {:.2}x", simulation_speed.scale);
    }

    // Reset speed with '0' key
    if keyboard_input.just_pressed(KeyCode::Key0) || keyboard_input.just_pressed(KeyCode::Numpad0) {
        simulation_speed.scale = 1.0;
        info!("Simulation speed reset to 1.0x");
    }
}
