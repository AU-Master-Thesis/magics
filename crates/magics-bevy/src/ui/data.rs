//! Data visualization UI components
//!
//! This module contains UI components for visualizing simulation data,
//! including plots, charts, and other data visualization tools.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::SimulationState;

/// System for rendering data visualization UI
pub fn render_data_visualizations(
    mut contexts: EguiContexts,
    simulation_state: Option<Res<SimulationState>>,
) {
    let ctx = contexts.ctx_mut();
    
    // Create a data visualization window
    egui::Window::new("Data Visualization")
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Simulation Data");
            ui.add_space(10.0);
            
            if let Some(simulation_state) = simulation_state {
                // Position plot
                ui.collapsing("Position Plot", |ui| {
                    egui::plot::Plot::new("agent_positions")
                        .view_aspect(1.0)
                        .show(ui, |plot_ui| {
                            // Create scatter points for each agent's position
                            let points: Vec<egui::plot::PlotPoint> = simulation_state
                                .agent_states
                                .iter()
                                .map(|agent| {
                                    egui::plot::PlotPoint::new(agent.position.x, agent.position.y)
                                })
                                .collect();
                            
                            plot_ui.points(
                                egui::plot::Points::new(points)
                                    .name("Agent Positions")
                                    .radius(5.0)
                                    .color(egui::Color32::from_rgb(100, 200, 100)),
                            );
                        });
                });
                
                // Velocity plot
                ui.collapsing("Velocity Plot", |ui| {
                    egui::plot::Plot::new("agent_velocities")
                        .view_aspect(1.0)
                        .show(ui, |plot_ui| {
                            // Create scatter points for each agent's velocity
                            let points: Vec<egui::plot::PlotPoint> = simulation_state
                                .agent_states
                                .iter()
                                .map(|agent| {
                                    egui::plot::PlotPoint::new(agent.velocity.x, agent.velocity.y)
                                })
                                .collect();
                            
                            plot_ui.points(
                                egui::plot::Points::new(points)
                                    .name("Agent Velocities")
                                    .radius(5.0)
                                    .color(egui::Color32::from_rgb(200, 100, 100)),
                            );
                        });
                });
                
                // Time series plot
                ui.collapsing("Time Series", |ui| {
                    // In a real implementation, we would store historical data
                    // and plot it over time. For now, this is a placeholder.
                    ui.label("Time series plots will be implemented here");
                    
                    // Placeholder for time series plot
                    egui::plot::Plot::new("time_series")
                        .view_aspect(2.0)
                        .show(ui, |_plot_ui| {
                            // Time series data would be plotted here
                        });
                });
            } else {
                ui.label("No simulation data available");
            }
        });
}

/// System for rendering state table
pub fn render_state_table(
    mut contexts: EguiContexts,
    simulation_state: Option<Res<SimulationState>>,
) {
    let ctx = contexts.ctx_mut();
    
    // Create a window for the state table
    egui::Window::new("State Table")
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Agent States");
            
            if let Some(simulation_state) = simulation_state {
                // Simple table showing agent states
                egui::Grid::new("agent_states_grid")
                    .striped(true)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        // Table header
                        ui.label("ID");
                        ui.label("Position X");
                        ui.label("Position Y");
                        ui.label("Velocity X");
                        ui.label("Velocity Y");
                        ui.end_row();
                        
                        // Table rows
                        for (i, agent) in simulation_state.agent_states.iter().enumerate() {
                            ui.label(format!("{}", i));
                            ui.label(format!("{:.2}", agent.position.x));
                            ui.label(format!("{:.2}", agent.position.y));
                            ui.label(format!("{:.2}", agent.velocity.x));
                            ui.label(format!("{:.2}", agent.velocity.y));
                            ui.end_row();
                        }
                    });
            } else {
                ui.label("No simulation data available");
            }
        });
}
