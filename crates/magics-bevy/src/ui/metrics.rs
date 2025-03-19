//! Metrics UI panel
//!
//! This module contains UI components for displaying performance metrics
//! and statistics about the simulation.

use bevy::prelude::*;
use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy_egui::{egui, EguiContexts};
use crate::SimulationState;

/// Resource for tracking simulation performance metrics
#[derive(Resource, Default)]
pub struct PerformanceMetrics {
    /// Frame rate history
    pub fps_history: Vec<f32>,
    /// Simulation step time history in milliseconds
    pub step_time_history: Vec<f32>,
    /// Maximum history length
    pub max_history: usize,
    /// Total number of simulation steps executed
    pub total_steps: usize,
}

impl PerformanceMetrics {
    /// Creates a new metrics resource with the specified history length
    pub fn new(max_history: usize) -> Self {
        Self {
            fps_history: Vec::with_capacity(max_history),
            step_time_history: Vec::with_capacity(max_history),
            max_history,
            total_steps: 0,
        }
    }

    /// Adds a new FPS value to the history
    pub fn add_fps(&mut self, fps: f32) {
        self.fps_history.push(fps);
        if self.fps_history.len() > self.max_history {
            self.fps_history.remove(0);
        }
    }

    /// Adds a new step time value to the history
    pub fn add_step_time(&mut self, step_time: f32) {
        self.step_time_history.push(step_time);
        if self.step_time_history.len() > self.max_history {
            self.step_time_history.remove(0);
        }
    }

    /// Increments the step counter
    pub fn increment_steps(&mut self) {
        self.total_steps += 1;
    }
}

/// Plugin for performance metrics
pub struct MetricsPlugin;

impl Plugin for MetricsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PerformanceMetrics::new(100))
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Update, update_performance_metrics)
            .add_systems(Update, render_metrics_panel);
    }
}

/// System for updating performance metrics
fn update_performance_metrics(
    time: Res<Time>,
    diagnostics: Res<Diagnostics>,
    mut metrics: ResMut<PerformanceMetrics>,
    simulation_state: Option<Res<SimulationState>>,
) {
    // Update FPS from diagnostics
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_value) = fps.smoothed() {
            metrics.add_fps(fps_value);
        }
    }

    // Track simulation step time (this would be measured in the actual implementation)
    // For now, we'll use a dummy value
    let step_time = 1.0; // ms
    metrics.add_step_time(step_time);

    // Increment step counter if the simulation is not paused
    if let Some(sim_state) = simulation_state {
        if !sim_state.paused {
            metrics.increment_steps();
        }
    }
}

/// System for rendering the metrics UI panel
fn render_metrics_panel(
    mut contexts: EguiContexts,
    metrics: Res<PerformanceMetrics>,
    simulation_state: Option<Res<SimulationState>>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Performance Metrics")
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Performance Metrics");
            ui.add_space(10.0);

            // Display FPS
            if !metrics.fps_history.is_empty() {
                let avg_fps = metrics.fps_history.iter().sum::<f32>() / metrics.fps_history.len() as f32;
                ui.label(format!("FPS: {:.1}", avg_fps));

                // FPS Plot
                egui::plot::Plot::new("fps_plot")
                    .height(100.0)
                    .show_axes([false, true])
                    .show(ui, |plot_ui| {
                        let fps_points: Vec<[f64; 2]> = metrics.fps_history
                            .iter()
                            .enumerate()
                            .map(|(i, &fps)| [i as f64, fps as f64])
                            .collect();

                        plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(fps_points))
                            .color(egui::Color32::from_rgb(100, 200, 100))
                            .name("FPS"));
                    });
            } else {
                ui.label("FPS: N/A");
            }

            ui.add_space(10.0);

            // Display simulation step time
            if !metrics.step_time_history.is_empty() {
                let avg_step_time = metrics.step_time_history.iter().sum::<f32>() / metrics.step_time_history.len() as f32;
                ui.label(format!("Simulation Step Time: {:.2} ms", avg_step_time));

                // Step Time Plot
                egui::plot::Plot::new("step_time_plot")
                    .height(100.0)
                    .show_axes([false, true])
                    .show(ui, |plot_ui| {
                        let step_time_points: Vec<[f64; 2]> = metrics.step_time_history
                            .iter()
                            .enumerate()
                            .map(|(i, &step_time)| [i as f64, step_time as f64])
                            .collect();

                        plot_ui.line(egui::plot::Line::new(egui::plot::PlotPoints::from(step_time_points))
                            .color(egui::Color32::from_rgb(200, 100, 100))
                            .name("Step Time (ms)"));
                    });
            } else {
                ui.label("Simulation Step Time: N/A");
            }

            ui.add_space(10.0);

            // Display simulation statistics
            ui.heading("Simulation Statistics");
            ui.label(format!("Total Steps: {}", metrics.total_steps));

            if let Some(sim_state) = simulation_state {
                ui.label(format!("Simulation Time: {:.2} s", sim_state.time));
                ui.label(format!("Number of Agents: {}", sim_state.agent_states.len()));
            }
        });
}
