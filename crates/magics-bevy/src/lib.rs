//! Bevy-specific UI and rendering for the Magics planner
//!
//! This crate provides the Bevy-based graphical user interface and rendering 
//! components for the Magics planner. It depends on magics-core for the 
//! underlying simulation logic, allowing the core simulation to run without UI.

#![feature(iter_repeat_n)]

use std::sync::{Arc, Mutex};
use bevy::prelude::*;
use magics_core::prelude::*;
use magics_core::simulation::Simulation;

/// Resource that holds the shared simulation instance
#[derive(Resource)]
pub struct SimulationState {
    /// Arc<Mutex<>> wrapped simulation for thread-safe access
    pub simulation: Arc<Mutex<Box<dyn Simulation>>>,
    /// Whether the simulation is currently paused
    pub paused: bool,
    /// Current simulation time
    pub time: f32,
    /// Current agent states
    pub agent_states: Vec<AgentState>,
}

/// Magics Bevy app plugin responsible for configuring and running the Bevy app
pub struct MagicsBevyPlugin {
    /// Reference to the core simulation
    pub simulation: Option<Arc<Mutex<Box<dyn Simulation>>>>,
    /// Whether to run in fullscreen mode
    pub fullscreen: bool,
    /// Path to scenarios directory
    pub simulations_dir: Option<std::path::PathBuf>,
    /// Initial scenario to load
    pub initial_scenario: Option<String>,
    /// Window width
    pub width: Option<u32>,
    /// Window height
    pub height: Option<u32>,
    /// Whether to record simulation to image sequence
    pub record: bool,
}

impl Default for MagicsBevyPlugin {
    fn default() -> Self {
        Self {
            simulation: None,
            fullscreen: false,
            simulations_dir: None,
            initial_scenario: None,
            width: None,
            height: None,
            record: false,
        }
    }
}

/// A set of distinct states the application can be in
#[derive(
    Debug,
    Default,
    States,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    derive_more::Display,
)]
pub enum AppState {
    /// Start of the application where assets are loaded into memory
    #[default]
    #[display(fmt = "Loading")]
    LoadingSimulationData,
    /// A simulation is running in the application
    #[display(fmt = "Running")]
    Running,
}

impl MagicsBevyPlugin {
    /// Configure window plugin based on plugin settings
    fn configure_window_plugin(&self) -> WindowPlugin {
        let window_mode = if self.fullscreen {
            WindowMode::BorderlessFullscreen
        } else {
            WindowMode::Windowed
        };

        let default_window_resolution = WindowResolution::default();
        let width = self.width
            .unwrap_or(default_window_resolution.physical_width());
        let height = self.height
            .unwrap_or(default_window_resolution.physical_height());

        WindowPlugin {
            primary_window: Some(Window {
                name: Some("Magics Planner".to_string()),
                focused: true,
                mode: window_mode,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                visible: true,
                resizable: !self.record,
                resolution: WindowResolution::new(width as f32, height as f32)
                    .with_scale_factor_override(1.0),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

// Add module imports
pub mod asset_loader;
pub mod environment;
pub mod input;
pub mod movement;
pub mod pause_play;
pub mod planner_visualization;
pub mod simulation_visualization;
pub mod theme;
pub mod ui;

// Re-export modules
pub use asset_loader::AssetLoaderPlugin;
pub use environment::{EnvironmentPlugin, MainCamera};
pub use input::InputPlugin;
pub use movement::MovementPlugin;
pub use pause_play::{PausePlayPlugin, SimulationSpeed};
pub use planner_visualization::PlannerVisualizationPlugin;
pub use simulation_visualization::SimulationVisualizationPlugin;
pub use ui::UiPlugin;

impl Plugin for MagicsBevyPlugin {
    fn build(&self, app: &mut App) {
        // Configure window
        let window_plugin = self.configure_window_plugin();
        
        // Add core plugins
        app
            .add_plugins(DefaultPlugins.set(window_plugin))
            .add_plugins((
                bevy_egui::EguiPlugin,
                bevy_mod_picking::DefaultPickingPlugins,
            ));
            
        // Add state management
        app.add_state::<AppState>();
        
        // Add simulation resource if provided
        if let Some(simulation) = &self.simulation {
            app.insert_resource(SimulationState {
                simulation: simulation.clone(),
                paused: false,
                time: 0.0,
                agent_states: Vec::new(),
            });
            
            // Add simulation visualization plugin with the simulation
            app.add_plugins(SimulationVisualizationPlugin {
                simulation: simulation.clone(),
            });
        }
        
        // Add UI and control plugins
        app.add_plugins((
            AssetLoaderPlugin,
            EnvironmentPlugin::default(),
            InputPlugin,
            MovementPlugin,
            PausePlayPlugin::default(),
            PlannerVisualizationPlugin {
                simulation_state: self.simulation.clone().map(|sim| {
                    Arc::new(Mutex::new(simulation_visualization::SimulationState::default()))
                }),
            },
            ui::UiPlugin,
            ui::settings::SettingsPlugin,
            ui::metrics::MetricsPlugin,
        ));
        
        // Add simulation update system
        app.add_systems(Update, update_simulation_state);
        
        // If recording is enabled, add systems for image export
        if self.record {
            // Record functionality will be added here
        }
    }
}

/// Extract current state from the simulation
fn update_simulation_state(
    mut simulation_state: ResMut<SimulationState>,
    time: Res<Time>,
) {
    // Skip if paused
    if simulation_state.paused {
        return;
    }
    
    // Lock simulation
    let mut simulation = match simulation_state.simulation.lock() {
        Ok(sim) => sim,
        Err(poisoned) => {
            // If the mutex is poisoned, we can still use the data
            poisoned.into_inner()
        }
    };
    
    // Step simulation
    if let Err(e) = simulation.step() {
        error!("Error stepping simulation: {}", e);
        return;
    }
    
    // Extract state
    let state = simulation.get_state();
    simulation_state.time = state.time.get();
    simulation_state.agent_states = state.agents.clone();
}
