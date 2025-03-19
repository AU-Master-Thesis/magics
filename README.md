# Magics - Multi-agent Path Planning with Gaussian Belief Propagation

> Master Thesis Project in Computer Engineering at Aarhus University 2024 on "Simulating Multi-agent Path Planning in Complex environments using Gaussian Belief Propagation and Global Path Finding". [Thesis available here](https://drive.google.com/file/d/12g-7bqcy_yfkZdpKzxQAErayFJQhu4sE/view?usp=sharing)

## Table of Contents

- [Overview](#overview)
- [Demo](#demo)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Running the Simulator](#running-the-simulator)
  - [WSL Configuration](#wsl-configuration)
- [Keyboard Controls](#keyboard-controls)
  - [UI Controls](#ui-controls)
  - [Camera Controls](#camera-controls)
  - [Simulation Controls](#simulation-controls)
  - [General Controls](#general-controls)
- [Available Scenarios](#available-scenarios)
- [External Dependencies](#external-dependencies)
- [Troubleshooting](#troubleshooting)
- [Credits](#credits)

## Overview

Magics is a Rust implementation and improvement of the Gaussian Belief Propagation (GBP) planner algorithm for multi-agent path planning in complex environments. The project enables simulation of multiple robots navigating through environments while avoiding collisions with obstacles and other robots.

### Motivating Example

|:---------------------------------------------------------:|:---------------------------------------------------------:|
| **Waypoint Tracking** | **Path Tracking** |
| https://github.com/user-attachments/assets/832fe84b-4b8b-4473-bfe1-9d87153988af | https://github.com/user-attachments/assets/6b8df209-d1db-4f35-9271-1c61ef660ab6 |

## Demo

> The video below demonstrates some of the features of the simulation tool, and shows how the GBP algorithm can handle complex scenarios such as a multi-lane twoway junction.

[magics-functionality-demo-trimmed-for-github.webm](https://github.com/user-attachments/assets/8f5d0db6-dd2c-41a3-9a12-4ccddf80d4f3)

## Getting Started

### Prerequisites

- Rust toolchain (version 1.78+)
- Cargo build system
- External dependencies for graphics (see [External Dependencies](#external-dependencies))

### Installation

1. Clone the repository
2. Install the required dependencies for your platform

#### Nix/NixOS

The `./flake.nix` file provides a development shell with all the necessary dependencies to run the project. If you have `direnv` installed you can simply use the provided `.envrc` and type `direnv allow` to automatically enter it. Otherwise you can run:

```sh
# To enter the development environment
nix develop
```

### Building

The entire project can be built with the following command:

```sh
RUSTFLAGS=-Awarnings cargo build --release 
```

### Running the Simulator

```sh
# Open the simulator with default scenario
cargo run --release --bin magics

# List all available scenarios
cargo run --release --bin magics -- --list-scenarios

# Run a specific scenario
cargo run --release --bin magics -- -i <SCENARIO_NAME> # fx. "Circle Experiment"
cargo run --release --bin magics -- --initial-scenario <SCENARIO_NAME> # fx. "Circle Experiment"
```

> **Important**: When specifying a scenario, use the exact name as shown in the `--list-scenarios` output. Do not use file paths.

### WSL Configuration (windows 10.)

When running in Windows Subsystem for Linux (WSL), you need to configure an X server:

1. Install an X server on Windows (VcXsrv, Xming, or X410)
2. Launch the X server with "Disable access control" checked
3. Set the following environment variables in WSL:

```sh
export DISPLAY=:0
export WINIT_UNIX_BACKEND=x11
```

4. Run the application as normal

## Keyboard Controls

### UI Controls

| Key | Function |
|-----|----------|
| H | Toggle Left Panel |
| L | Toggle Right Panel |
| K | Toggle Top Panel |
| J | Toggle Bottom Panel |
| D | Toggle Metrics Window |
| U | Change Scale Kind |

### Camera Controls

| Key/Mouse | Function |
|-----------|----------|
| Arrow Keys | Move Camera |
| C | Toggle Camera Movement Mode (Pan/Orbit) |
| Tab | Switch Camera |
| R | Reset Camera |
| Mouse Wheel | Zoom In/Out |
| Left Mouse Button + Mouse Movement | Move Camera (Pan or Orbit depending on mode) |
| Middle Mouse Button | Pan Camera |
| Right-click Drag | Rotate Camera |

### Simulation Controls

| Key | Function |
|-----|----------|
| F5 | Reload Current Simulation |
| F6 | Load Next Simulation |
| F4 | Load Previous Simulation |
| Space | Pause/Play Simulation |

### General Controls

| Key | Function |
|-----|----------|
| T | Cycle Theme |
| G | Export Graph |
| Ctrl+S | Save Settings |
| Ctrl+P | Take Screenshot |
| Ctrl+Q | Quit Application |

## Available Scenarios

The simulator comes with several pre-configured scenarios to demonstrate different aspects of multi-agent path planning:

- Circle Experiment: Robots arranged in a circle swap positions
- Junction Experiment: Robots navigate through a four-way junction
- Structured Junction: A more complex junction with structured paths
- Collaborative Complex: Multiple robots collaborating in a complex environment
- And many more...

Use the `--list-scenarios` command to see all available scenarios.

## External Dependencies

Most dependencies are available through the `crates.io` registry and should work on all major platforms supported by the `cargo` build tool. However, some external dependencies are needed for the graphical session:

| Dependencies | Platform Specific |
|--------------|----------|
| `udev` | Linux |
| `alsa-lib` | Linux |
| `vulkan-loader` |  |
| `xorg.libX11` | Linux + X11 |
| `xorg.libXcursor` | Linux + X11 |
| `xorg.libXi` | Linux + X11 |
| `xorg.libXrandr` | Linux + X11 |
| `libxkbcommon` | Linux + X11 |
| `wayland` | Linux + Wayland |
| `egl-wayland` | Linux + Wayland |
| `freetype` | |
| `fontconfig` |  |

The exact name of the dependency might vary between platforms, and even between Linux distributions. Consult the respective package management tool used on your system for their exact names.

## Troubleshooting

### Common Issues

#### Display Issues in WSL
- **Problem**: "Failed to build event loop: Os(OsError { ... error: WaylandError(Connection(NoCompositor)) })"
- **Solution**: Set `export DISPLAY=:0` and `export WINIT_UNIX_BACKEND=x11` and in `Cargo.toml` under `bevy` remove `wayland`:

bevy = { version = "0.13", default-features = true, features = [
  # "wayland",
  # "dynamic_linking",
] }
derive_more = "0.99.17"

#### Scenario Loading Issues
- **Problem**: "simulation with name exists"
- **Solution**: Use exact scenario name as shown in `--list-scenarios` output in "" fx. "Circle Experiment"

#### Performance Issues
- **Problem**: Slow simulation or rendering
- **Solution**: Use release build, reduce number of robots, simplify environment

## Credits

The primary algorithm for GBP path planning is based on [gbpplanner](https://github.com/aalpatya/gbpplanner) by [Aalok Patwardhan](https://aalok.uk/) from Imperial College London and Dyson Robotics Lab. As part of this thesis, we have reimplemented and extended upon it in Rust!

## Thesis

The accompanying thesis is available online [here](https://drive.google.com/file/d/12g-7bqcy_yfkZdpKzxQAErayFJQhu4sE/view?usp=sharing).
