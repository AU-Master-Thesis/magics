//! Controls UI panel
//!
//! This module contains the UI components for the controls panel, which
//! allows users to control the simulation parameters and execution.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::SimulationState;
use crate::pause_play::SimulationSpeed;

/// System for rendering the simulation controls UI
pub fn render_controls_panel(
    mut contexts: EguiContexts,
    mut simulation_state: Option<ResMut<SimulationState>>,
    mut simulation_speed: ResMut<SimulationSpeed>,
) {
    let ctx = contexts.ctx_mut();
    
    // Create a new window for the controls
    egui::Window::new("Simulation Controls")
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Simulation Controls");
            ui.add_space(10.0);
            
            // Simulation status display
            if let Some(ref mut simulation_state) = simulation_state {
                // Pause/Play button
                if ui.button(if simulation_state.paused { "► Play" } else { "❚❚ Pause" }).clicked() {
                    simulation_state.paused = !simulation_state.paused;
                }
                
                ui.add_space(10.0);
                
                // Simulation Speed
                ui.label("Simulation Speed:");
                ui.add(egui::Slider::new(&mut simulation_speed.scale, 0.1..=5.0)
                    .logarithmic(true)
                    .text("Speed Factor"));
                
                if ui.button("Reset Speed (1.0x)").clicked() {
                    simulation_speed.scale = 1.0;
                }
                
                ui.add_space(10.0);
                
                // Simulation Time
                ui.label(format!("Simulation Time: {:.2} s", simulation_state.time));
                
                ui.add_space(10.0);
                
                // Agent information
                ui.collapsing("Agents", |ui| {
                    ui.label(format!("Number of Agents: {}", simulation_state.agent_states.len()));
                    
                    for (idx, agent) in simulation_state.agent_states.iter().enumerate() {
                        ui.collapsing(format!("Agent {}", idx), |ui| {
                            ui.label(format!("Position: ({:.2}, {:.2})", 
                                            agent.position.x, 
                                            agent.position.y));
                            ui.label(format!("Velocity: ({:.2}, {:.2})", 
                                            agent.velocity.x, 
                                            agent.velocity.y));
                        });
                    }
                });
            } else {
                ui.label("No simulation is running");
            }
        });
}

/// System for adding simulation control buttons
pub fn add_simulation_controls_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    
    // Add floating control buttons at the bottom of the screen
    egui::Window::new("Quick Controls")
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -10.0])
        .title_bar(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Restart").clicked() {
                    // Signal to restart the simulation
                    // This will be implemented later
                }
                
                if ui.button("Save").clicked() {
                    // Signal to save the simulation state
                    // This will be implemented later
                }
                
                if ui.button("Load").clicked() {
                    // Signal to load a simulation state
                    // This will be implemented later
                }
            });
        });
}
