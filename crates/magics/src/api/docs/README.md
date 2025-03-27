# Magics API Documentation

Welcome to the Magics API documentation. This documentation provides detailed information about the Magics API, which allows external control and monitoring of the Magics simulation.

## Overview

The Magics API is a ZeroMQ-based API that enables external applications to interact with the Magics simulation. It provides access to agent states, environment information, and allows control over simulation parameters such as factor weights and simulation speed.

The API is designed to support reinforcement learning research through integration with OpenAI Gym, making it possible to use the Magics simulation as an environment for training reinforcement learning agents.

## Documentation Structure

This documentation is organized into several sections:

- [Getting Started](./getting_started.md) - Quick start guide for using the API
- [Core Concepts](./core_concepts.md) - Explanation of key concepts in the API
- [Commands Reference](./commands.md) - Detailed information about available API commands

### Data Structures

- [Agent State](./data_structures/agent_state.md) - Information about agent state representation
- [Environment State](./data_structures/environment_state.md) - Information about environment state representation
- [Factor Graph](./data_structures/factor_graph.md) - Information about factor graph representation
- [Collision Information](./data_structures/collision_info.md) - Information about collision detection and tracking

### Examples

- [Basic Usage](./examples/basic_usage.md) - Basic examples of using the API
- [Stepping](./examples/stepping.md) - Examples of stepping through the simulation
- [Weights](./examples/weights.md) - Examples of modifying factor weights

### Troubleshooting

- [Troubleshooting](./troubleshooting.md) - Common issues and solutions

## Key Components

The API consists of several key components:

1. **ZeroMQ Server** ([`zmq_server.rs`](../zmq_server.rs)) - Handles communication with clients
2. **Message Protocol** ([`message.rs`](../message.rs)) - Defines the message format for communication
3. **State Structures** ([`state.rs`](../state.rs)) - Defines the data structures for representing simulation state
4. **State Extraction** ([`extract.rs`](../extract.rs)) - Extracts state information from the simulation
5. **Factor Details Extraction** ([`factor_details.rs`](../factor_details.rs)) - Extracts detailed information about factor graphs
6. **Weight Updates** ([`weights.rs`](../weights.rs)) - Handles updates to factor weights
7. **Reset Handling** ([`reset.rs`](../reset.rs)) - Handles resetting the API state
8. **Plugin Integration** ([`plugin.rs`](../plugin.rs)) - Integrates the API with the Bevy application

## Python Client

The API includes a Python client ([`magics_client.py`](../../../../python_api/magics_client.py)) that provides a convenient interface for interacting with the API from Python applications. The client handles connection management, message serialization/deserialization, and provides helper methods for common operations.

## OpenAI Gym Integration

The API is designed to support integration with OpenAI Gym, allowing the Magics simulation to be used as an environment for reinforcement learning research. The integration is implemented in the [`magics_gym`](../../../../python_api/magics_gym/) package.

## Getting Started

To get started with the Magics API, see the [Getting Started](./getting_started.md) guide.
