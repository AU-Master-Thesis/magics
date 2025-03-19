//! Settings UI panel
//!
//! This module contains UI components for configuring simulation settings
//! and application preferences.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::SimulationState;

/// Resource for storing settings
#[derive(Resource)]
pub struct Settings {
    /// Visual settings
    pub visual: VisualSettings,
    /// Simulation settings
    pub simulation: SimulationSettings,
    /// Application settings
    pub application: ApplicationSettings,
}

/// Visual settings for the application
#[derive(Clone)]
pub struct VisualSettings {
    /// Whether to use dark mode
    pub dark_mode: bool,
    /// Whether to show agent IDs
    pub show_agent_ids: bool,
    /// Whether to show trajectories
    pub show_trajectories: bool,
    /// Whether to show velocity vectors
    pub show_velocity_vectors: bool,
    /// Agent visualization scale
    pub agent_scale: f32,
    /// Environment visualization scale
    pub environment_scale: f32,
}

/// Simulation settings
#[derive(Clone)]
pub struct SimulationSettings {
    /// Number of iterations per step
    pub iterations_per_step: u32,
    /// Number of message passing iterations
    pub message_passing_iterations: u32,
    /// Whether to use adaptive step size
    pub adaptive_step_size: bool,
    /// Whether to enable collision avoidance
    pub enable_collision_avoidance: bool,
    /// Whether to enable trajectory planning
    pub enable_trajectory_planning: bool,
}

/// Application settings
#[derive(Clone)]
pub struct ApplicationSettings {
    /// Whether to use autosave
    pub autosave: bool,
    /// Autosave interval in seconds
    pub autosave_interval: f32,
    /// Whether to show debug information
    pub show_debug_info: bool,
    /// Whether to use hardware acceleration
    pub hardware_acceleration: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            visual: VisualSettings {
                dark_mode: true,
                show_agent_ids: true,
                show_trajectories: true,
                show_velocity_vectors: true,
                agent_scale: 1.0,
                environment_scale: 1.0,
            },
            simulation: SimulationSettings {
                iterations_per_step: 10,
                message_passing_iterations: 5,
                adaptive_step_size: true,
                enable_collision_avoidance: true,
                enable_trajectory_planning: true,
            },
            application: ApplicationSettings {
                autosave: false,
                autosave_interval: 60.0,
                show_debug_info: false,
                hardware_acceleration: true,
            },
        }
    }
}

/// Plugin for settings functionality
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<Settings>()
            .add_systems(Update, render_settings_panel);
    }
}

/// Current active settings tab
#[derive(PartialEq, Resource, Default)]
enum SettingsTab {
    #[default]
    Visual,
    Simulation,
    Application,
}

/// System for rendering the settings UI panel
fn render_settings_panel(
    mut contexts: EguiContexts,
    mut settings: ResMut<Settings>,
    mut settings_tab: Local<SettingsTab>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("Settings")
        .resizable(true)
        .collapsible(true)
        .show(ctx, |ui| {
            ui.heading("Settings");
            ui.add_space(10.0);

            // Settings tabs
            egui::TopBottomPanel::top("settings_tabs").show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut *settings_tab, SettingsTab::Visual, "Visual");
                    ui.selectable_value(&mut *settings_tab, SettingsTab::Simulation, "Simulation");
                    ui.selectable_value(&mut *settings_tab, SettingsTab::Application, "Application");
                });
            });

            ui.add_space(10.0);

            // Settings content based on selected tab
            match *settings_tab {
                SettingsTab::Visual => {
                    render_visual_settings(ui, &mut settings.visual);
                },
                SettingsTab::Simulation => {
                    render_simulation_settings(ui, &mut settings.simulation);
                },
                SettingsTab::Application => {
                    render_application_settings(ui, &mut settings.application);
                },
            }
        });
}


/// Renders visual settings UI
fn render_visual_settings(ui: &mut egui::Ui, settings: &mut VisualSettings) {
    ui.heading("Visual Settings");
    ui.add_space(10.0);

    ui.checkbox(&mut settings.dark_mode, "Dark Mode");
    ui.checkbox(&mut settings.show_agent_ids, "Show Agent IDs");
    ui.checkbox(&mut settings.show_trajectories, "Show Trajectories");
    ui.checkbox(&mut settings.show_velocity_vectors, "Show Velocity Vectors");

    ui.add_space(10.0);
    ui.label("Agent Scale:");
    ui.add(egui::Slider::new(&mut settings.agent_scale, 0.1..=2.0)
        .text("Scale"));

    ui.label("Environment Scale:");
    ui.add(egui::Slider::new(&mut settings.environment_scale, 0.1..=2.0)
        .text("Scale"));

    ui.add_space(10.0);
    if ui.button("Reset to Defaults").clicked() {
        *settings = VisualSettings {
            dark_mode: true,
            show_agent_ids: true,
            show_trajectories: true,
            show_velocity_vectors: true,
            agent_scale: 1.0,
            environment_scale: 1.0,
        };
    }
}

/// Renders simulation settings UI
fn render_simulation_settings(ui: &mut egui::Ui, settings: &mut SimulationSettings) {
    ui.heading("Simulation Settings");
    ui.add_space(10.0);

    ui.label("Iterations Per Step:");
    ui.add(egui::Slider::new(&mut settings.iterations_per_step, 1..=50)
        .text("Iterations"));

    ui.label("Message Passing Iterations:");
    ui.add(egui::Slider::new(&mut settings.message_passing_iterations, 1..=20)
        .text("Iterations"));

    ui.checkbox(&mut settings.adaptive_step_size, "Adaptive Step Size");
    ui.checkbox(&mut settings.enable_collision_avoidance, "Enable Collision Avoidance");
    ui.checkbox(&mut settings.enable_trajectory_planning, "Enable Trajectory Planning");

    ui.add_space(10.0);
    if ui.button("Reset to Defaults").clicked() {
        *settings = SimulationSettings {
            iterations_per_step: 10,
            message_passing_iterations: 5,
            adaptive_step_size: true,
            enable_collision_avoidance: true,
            enable_trajectory_planning: true,
        };
    }
}

/// Renders application settings UI
fn render_application_settings(ui: &mut egui::Ui, settings: &mut ApplicationSettings) {
    ui.heading("Application Settings");
    ui.add_space(10.0);

    ui.checkbox(&mut settings.autosave, "Autosave");
    if settings.autosave {
        ui.label("Autosave Interval (seconds):");
        ui.add(egui::Slider::new(&mut settings.autosave_interval, 10.0..=300.0)
            .text("Seconds"));
    }

    ui.checkbox(&mut settings.show_debug_info, "Show Debug Information");
    ui.checkbox(&mut settings.hardware_acceleration, "Use Hardware Acceleration");

    ui.add_space(10.0);
    if ui.button("Reset to Defaults").clicked() {
        *settings = ApplicationSettings {
            autosave: false,
            autosave_interval: 60.0,
            show_debug_info: false,
            hardware_acceleration: true,
        };
    }
}
