# Magics Core

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A headless implementation of the GBP (Graph-Based Planning) simulation framework, allowing users to run multi-agent cooperative planning simulations without requiring a GUI or rendering backend.

## Overview

Magics Core provides the foundational simulation infrastructure for multi-agent path planning using factorized Gaussian Belief Propagation. It decouples the simulation logic from visualization components, enabling:

- Faster experimentation and testing
- Server-side/headless execution for batch processing
- Clean separation of simulation from rendering concerns
- CLI-based operation for automation and scripting

This crate serves as the computational engine for the full Magics library, providing all core functionality without UI dependencies.

## Project Structure

```
crates/magics-core/
├── src/
│   ├── bin/
│   │   └── magics-headless.rs   # Headless executable entry point
│   ├── planner/
│   │   ├── mod.rs               # Module organization
│   │   ├── planner.rs           # Core planning algorithms
│   │   ├── robot.rs             # Robot state and connectivity
│   │   ├── collisions.rs        # Collision detection and handling
│   │   ├── mission.rs           # Mission definition and execution
│   │   ├── spawner.rs           # Robot spawning functionality
│   │   └── tracking.rs          # Trajectory tracking and analysis
│   ├── types.rs                 # Core type definitions
│   ├── error.rs                 # Error handling
│   ├── output.rs                # Simulation output management
│   ├── environment.rs           # Environment representation
│   ├── simulation.rs            # Simulation engine
│   ├── factorgraph.rs           # Factor graph implementation
│   ├── state.rs                 # State management
│   └── lib.rs                   # Library entry point
└── Cargo.toml                   # Crate manifest
```

## Key Components

### Simulation Engine

- **GbpSimulation**: The main simulation implementation that orchestrates the planning process
- **SimulationRunner**: Handles execution flow, time stepping, and output generation
- **OutputManager**: Manages generation of simulation results in various formats (JSON, CSV, YAML)

### Planning System

- **Planner**: Core trait defining planning algorithm interface
- **GbpPlanner**: Implementation of the GBP planning algorithm
- **Path**: Representation of planned paths with waypoints

### Robot Management

- **Robot**: Representation of robot agents with physical and planning properties
- **RobotConnections**: Manages inter-robot communication and connectivity
- **RadioAntenna**: Simulates communication capabilities with configurable ranges

### Environment

- **GbpSimulationEnvironment**: Manages the simulation environment including obstacles and boundaries
- **Obstacle**: Represents environmental obstacles with collision properties

### Collision Detection

- **RobotCollisionDetector**: Detects and handles robot-robot collisions
- **ObstacleCollisionDetector**: Manages collisions between robots and environment obstacles

### Mission Planning

- **Mission**: Sequences of tasks for robots to execute
- **MissionTask**: Individual robot tasks with parameters
- **MissionTaskType**: Various types of mission tasks (MoveTo, Wait, Follow, etc.)

## Installation

Add magics-core to your project's dependencies:

```toml
[dependencies]
magics-core = { path = "path/to/magics-core" }
```

For headless execution, the crate can be run directly:

```bash
# Using just the binary
cargo run -p magics-core --bin magics-headless -- [OPTIONS]

# With direct headless mode
cargo run -p magics-core --bin magics-headless -- --headless enabled

# Using environment variable
MAGICS_HEADLESS=1 cargo run -p magics-core --bin magics-headless
```

## Usage

### Basic Simulation Setup

```rust
use magics_core::prelude::*;

// Initialize simulation environment and configuration
let environment = GbpSimulationEnvironment::new();
let config = SimulationConfig {
    time_step: 0.1,
    max_time: 100.0,
    random_seed: 42,
    output: OutputSettings {
        stdout: true,
        file_path: Some("results.json".to_string()),
        format: OutputFormat::Json,
        frequency: 10,
    },
};

// Create simulation
let simulation = GbpSimulation::new(environment, GbpConfig::default());
let mut runner = SimulationRunner::new(Box::new(simulation), config)?;

// Run simulation
runner.run()?;
```

### Adding Robots

```rust
use magics_core::prelude::*;
use magics_core::planner::spawner::RobotSpawner;

// Create robot spawner
let mut spawner = RobotSpawner::new();

// Create individual robots
let robot_config = spawner.create_robot_config(
    Vector2::new(0.0, 0.0), // position
    0.5,                    // radius
    PlanningStrategy::RrtStar, // planning strategy
);

// Or create multiple robots in a pattern
let robot_configs = spawner.create_robot_grid(
    Vector2::new(0.0, 0.0), // start position
    3,                      // rows
    3,                      // columns
    2.0,                    // spacing
    0.5,                    // robot radius
    PlanningStrategy::OnlyLocal, // planning strategy
);

// Add robots to simulation
// (implementation depends on your specific use case)
```

### Defining Missions

```rust
use magics_core::prelude::*;
use magics_core::planner::mission::{Mission, MissionTask, MissionTaskType, MissionTaskParameters};
use std::collections::HashMap;

// Create a mission for a robot
let mut mission = Mission::new();

// Create a movement task
let mut params = MissionTaskParameters { 
    values: HashMap::new() 
};
params.values.insert(
    "position".to_string(), 
    MissionTaskParameterValue::Position(10.0, 20.0)
);

let task = MissionTask {
    id: "task1".to_string(),
    task_type: MissionTaskType::MoveTo,
    parameters: params,
};

// Add task to mission
mission.add_task(task);

// Assign mission to robot
// (implementation depends on your specific use case)
```

## Command-Line Interface

The magics-headless binary provides a comprehensive CLI for running simulations without a GUI:

```
Usage: magics-headless [OPTIONS] --headless <HEADLESS>

Options:
      --dump-default <DUMP_DEFAULT>
          Default configuration information to dump to stdout
          [possible values: config, formation, environment]

      --dump-environment <ENVIRONMENT_TYPE>
          Dump a specific environment type to stdout
          [possible values: intersection, intermediate, complex, circle, maze, test]

  -s --simulations-dir <SIMULATIONS_DIR>
          Path to directory with simulations to load [default: ./config/scenarios]

  -l --list-scenarios
          List all detected simulations

  -i --initial-scenario <INITIAL_SCENARIO>
          Initial scenario to load

      --headless <HEADLESS>
          Run the app without a window for rendering the environment
          [possible values: enabled, disabled]

  -m --metadata
          Print metadata about the project to stderr

      --output-file <OUTPUT_FILE>
          Path to output file for simulation results

      --output-format <OUTPUT_FORMAT>
          Format for output results
          [default: json]
          [possible values: json, csv, yaml]

  -h --help
          Print help information

  -V --version
          Print version information
```

## Configuration

### Simulation Configuration

The `SimulationConfig` struct controls core simulation parameters:

```rust
pub struct SimulationConfig {
    pub time_step: f64,      // Simulation time step in seconds
    pub max_time: f64,       // Maximum simulation time
    pub random_seed: u64,    // Random seed for reproducibility
    pub output: OutputSettings, // Output configuration
}
```

### Output Settings

Control how simulation results are generated:

```rust
pub struct OutputSettings {
    pub stdout: bool,                // Output to console
    pub file_path: Option<String>,   // Output file path
    pub format: OutputFormat,        // Output format
    pub frequency: usize,            // Output frequency
}
```

### Environment Types

Several predefined environment types are available:

- `Circle`: Simple circular environment
- `Complex`: Complex environment with multiple obstacles
- `Intersection`: Road intersection scenario
- `Maze`: Maze-like environment with navigable paths
- `Intermediate`: Medium complexity environment
- `Test`: Simple test environment

## Development

### Adding New Robot Behaviors

1. Define the behavior in an appropriate module (usually in `planner/`)
2. Update the appropriate traits/structs to support the new behavior
3. Add tests to verify functionality

### Creating Custom Environments

1. Implement your environment by extending the base `Environment` struct
2. Define obstacles and boundaries
3. Register the environment with the appropriate environment type

### Extending Output Formats

1. Update the `OutputFormat` enum with your new format
2. Implement the formatting logic in the `OutputManager`
3. Update the CLI parser to support the new format

## License

Magics Core is licensed under the MIT license.
