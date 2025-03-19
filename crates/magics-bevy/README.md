# magics-bevy

[![Crates.io](https://img.shields.io/crates/v/magics-bevy.svg)](https://crates.io/crates/magics-bevy)
[![Documentation](https://docs.rs/magics-bevy/badge.svg)](https://docs.rs/magics-bevy)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Overview

The `magics-bevy` crate provides a visualization and user interface layer for the Magics Planner, a multi-agent planning system for robots and autonomous agents. This crate leverages the [Bevy](https://bevyengine.org/) game engine to create interactive visualizations and UI components, allowing users to analyze, control, and interact with simulations created using the `magics-core` library.

## Features

- **Interactive Visualization**: Render agents, environments, and trajectories with real-time updates
- **Comprehensive UI**: Control panels, data visualizations, settings, and metrics monitoring
- **Dependency Injection**: Clean architecture that connects to core simulation via a trait-based interface
- **Real-time Controls**: Pause, play, and adjust simulation speed
- **Data Analysis**: Plot agent positions, velocities, and other metrics in real-time
- **Theming**: Support for both light and dark themes with system preference detection
- **Recording**: Ability to record simulations to image sequences for later analysis

## Architecture

The `magics-bevy` crate is designed with a modular architecture that follows the dependency injection pattern:

```
┌─────────────────┐      ┌─────────────────┐     ┌─────────────────┐
│                 │      │                 │     │                 │
│   magics-cli    │─────▶│   magics-core   │◀────│   magics-bevy   │
│   (orchestrator)│      │   (simulation)  │     │   (visualization)│
│                 │      │                 │     │                 │
└─────────────────┘      └─────────────────┘     └─────────────────┘
        │                                               ▲
        │                                               │
        └───────────────────────────────────────────────┘
                          Dependency Injection
```

The main components are:

### UI Components

- **Simulation Visualization**: Renders the current state of agents and environment
- **Control Panels**: Provides user controls for simulation parameters
- **Data Visualization**: Plots and graphs for analyzing simulation metrics
- **Metrics Panel**: Performance monitoring and statistics
- **Settings Panel**: Configuration options for both visualization and simulation

### Core Systems

- **Theme System**: Manages application theming with dark/light mode support
- **Pause/Play System**: Controls simulation execution
- **State Extraction**: Pulls state from core simulation for visualization

## Main Components

### `MagicsBevyPlugin`

The main entry point for the visualization system. This plugin:

- Configures the Bevy application
- Sets up window and render settings
- Connects to the core simulation via dependency injection
- Initializes all UI components

```rust
// Example usage in magics-cli
App::new()
    .add_plugins(MagicsBevyPlugin {
        simulation: Some(simulation),
        fullscreen: args.fullscreen,
        simulations_dir: args.simulations_dir.clone(),
        initial_scenario: args.initial_scenario.clone(),
        width: args.width,
        height: args.height,
        record: args.record,
    })
    .run();
```

### `SimulationVisualizationPlugin`

Handles the visualization of simulation state, including:

- Agent rendering
- Environment obstacles
- Trajectory visualization
- Camera controls

### `UiPlugin`

Manages the user interface components:

- Control panels
- Data visualization
- Settings panels
- Metrics display

### `ThemePlugin`

Provides theming capabilities:

- Dark and light mode support
- System theme detection
- Keyboard shortcuts for theme switching
- Consistent color palette using Catppuccin

### `PausePlayPlugin`

Controls the simulation execution:

- Pause/play functionality
- Simulation speed adjustment
- Step-by-step execution
- Keyboard shortcuts

## Usage

### Basic Setup

To use the `magics-bevy` crate in your application:

```rust
use magics_bevy::{MagicsBevyPlugin, SimulationState};
use magics_core::simulation::{Simulation, GbpSimulation};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    // Create your simulation
    let mut simulation = GbpSimulation::new(environment, config);
    
    // Wrap in Arc<Mutex<>> for thread-safe sharing
    let simulation = Arc::new(Mutex::new(Box::new(simulation) as Box<dyn Simulation>));
    
    // Create and run the Bevy app with the simulation
    App::new()
        .add_plugins(MagicsBevyPlugin {
            simulation: Some(simulation),
            fullscreen: false,
            // Other configuration options...
            ..Default::default()
        })
        .run();
}
```

### Using Components Separately

You can also use individual components:

```rust
use magics_bevy::{
    SimulationVisualizationPlugin, 
    UiPlugin, 
    ThemePlugin, 
    PausePlayPlugin
};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            SimulationVisualizationPlugin { /* ... */ },
            UiPlugin,
            ThemePlugin,
            PausePlayPlugin::default(),
        ))
        .run();
}
```

## Configuration Options

`MagicsBevyPlugin` accepts the following configuration options:

- `simulation`: The simulation instance to visualize (wrapped in `Arc<Mutex<>>`)
- `fullscreen`: Whether to run in fullscreen mode
- `simulations_dir`: Path to the scenarios directory
- `initial_scenario`: The initial scenario to load
- `width`: Window width (defaults to system default if not specified)
- `height`: Window height (defaults to system default if not specified)
- `record`: Whether to record the simulation to an image sequence

## UI Controls

| Action                  | Keyboard Shortcut   |
|-------------------------|--------------------|
| Pause/Play Simulation   | Space              |
| Increase Speed          | +                  |
| Decrease Speed          | -                  |
| Reset Speed             | 0                  |
| Toggle Dark/Light Mode  | Ctrl+T             |
| Toggle Controls Panel   | Ctrl+1             |
| Toggle Settings Panel   | Ctrl+2             |
| Toggle Metrics Panel    | Ctrl+3             |
| Toggle All UI           | Tab                |

## Dependencies

Major dependencies include:

- `bevy`: The game engine that powers all visualization
- `bevy_egui`: Used for UI components
- `bevy_mod_picking`: Used for interacting with 3D elements
- `catppuccin`: Provides color schemes for theming
- `dark-light`: System theme detection

## Examples

### Visualizing a Simple Simulation

```rust
use magics_bevy::MagicsBevyPlugin;
use magics_core::simulation::{GbpSimulation, SimulationConfig};
use magics_core::environment::GbpSimulationEnvironment;
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

fn main() {
    // Create a simple environment with some agents
    let env = GbpSimulationEnvironment::new(/* environment config */);
    let config = SimulationConfig::default();
    
    // Create the simulation
    let simulation: Box<dyn Simulation> = Box::new(
        GbpSimulation::new(env, config)
    );
    
    // Add a planner
    let mut simulation_mut = simulation.as_any_mut()
        .downcast_mut::<GbpSimulation>()
        .unwrap();
    let planner = magics_core::planner::GbpPlanner::default();
    simulation_mut.set_planner(planner);
    
    // Wrap simulation for thread-safe sharing
    let simulation = Arc::new(Mutex::new(simulation));
    
    // Create and run the Bevy app
    App::new()
        .add_plugins(MagicsBevyPlugin {
            simulation: Some(simulation),
            ..Default::default()
        })
        .run();
}
```

### Custom UI Configuration

```rust
use magics_bevy::{MagicsBevyPlugin, ThemeSettings, UiState};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(MagicsBevyPlugin {
            /* ... */
        })
        // Customize theme
        .insert_resource(ThemeSettings {
            dark_mode: true,
            ..Default::default()
        })
        // Customize UI state
        .insert_resource(UiState {
            show_controls: true,
            show_settings: false,
            show_metrics: true,
            ..Default::default()
        })
        .run();
}
```

## Building and Running

To build and run the crate:

```bash
# Build the crate
cargo build --package magics-bevy

# Run with UI
cargo run --bin magics

# Run with specific scenario
cargo run --bin magics -- --scenario "Circle Experiment"

# Run in headless mode (no UI)
cargo run --bin magics -- --headless
```

## Contributing

Contributions are welcome! See the [CONTRIBUTING.md](../../CONTRIBUTING.md) file for more information.

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](../../LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](../../LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
