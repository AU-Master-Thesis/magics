# Core Concepts of the Magics API

This document explains the core concepts of the Magics API, providing a foundation for understanding how the API works and how to use it effectively.

## Architecture Overview

The Magics API is built on a client-server architecture using ZeroMQ for communication:

```
+----------------+        ZeroMQ        +----------------+
|                |  Request/Response     |                |
|  Python Client | <------------------>  | Magics Server  |
|                |                       |                |
+----------------+                       +----------------+
                                               |
                                               | Bevy ECS
                                               v
                                         +----------------+
                                         |                |
                                         |   Simulation   |
                                         |                |
                                         +----------------+
```

- **Server**: Implemented in Rust as part of the Magics simulation, the server exposes the simulation state and control mechanisms through a ZeroMQ REP socket.
- **Client**: Implemented in Python, the client connects to the server using a ZeroMQ REQ socket and provides a convenient API for interacting with the simulation.

## Gaussian Belief Propagation (GBP)

At the core of the Magics simulation is the Gaussian Belief Propagation (GBP) algorithm, which is used for multi-agent path planning. Understanding GBP is essential for effectively using the API, especially when modifying factor weights.

### Factor Graphs

A factor graph is a bipartite graph that represents the factorization of a function. In the context of Magics, factor graphs are used to represent the joint probability distribution over agent positions and velocities.

The factor graph consists of:

- **Variables**: Represent the state of an agent (position and velocity)
- **Factors**: Represent constraints or relationships between variables

### Factor Types

The Magics simulation uses several types of factors:

- **Dynamic Factors**: Model the dynamics of an agent, connecting variables at different time steps
- **Obstacle Factors**: Model the interaction between an agent and obstacles in the environment
- **Inter-Robot Factors**: Model the interaction between different agents
- **Tracking Factors**: Model the tracking of a predefined path

### Factor Weights

Each factor type has an associated weight that determines its influence on the overall solution. These weights can be modified through the API to change the behavior of the simulation.

## API State

The API maintains state information about the simulation, including:

- **Agent States**: Information about each agent in the simulation
- **Environment State**: Information about the environment, including obstacles and boundaries
- **Weight Updates**: Pending updates to factor weights

## Stepping Mechanism

The API provides a stepping mechanism that allows external control of the simulation's progression:

1. The client sends a `Step` command to the server
2. The server unpauses the simulation for a specified number of iterations
3. The simulation runs for the specified number of iterations
4. The server pauses the simulation and extracts the current state
5. The server sends a response to the client indicating that the step is complete

This stepping mechanism allows for controlled execution of the simulation, which is essential for reinforcement learning applications.

## Message Protocol

The API uses a JSON-based message protocol for communication between the client and server:

- **Request**: Contains a command and optional parameters
- **Response**: Contains a status, optional data, and optional error message

### Commands

The API supports several commands:

- `GetAgentState`: Get the state of all agents in the simulation
- `GetEnvironmentState`: Get the state of the environment
- `SetFactorWeights`: Set factor graph weights
- `Step`: Step the simulation forward by one frame
- `Reset`: Reset the simulation
- `IsApiActive`: Check if the API is active
- `SetApiActive`: Set the API active state
- `SetIterationsPerStep`: Set the number of iterations per step
- `GetSimulationHz`: Get the simulation Hz (frequency)
- `SetSimulationHz`: Set the simulation Hz (frequency)

## Data Structures

The API uses several data structures to represent the state of the simulation:

### Agent State

The `AgentState` structure contains information about a single agent in the simulation, including:

- Position and velocity
- Factor graph state
- Connected neighbors
- Mission state and progress
- Communication properties
- Collision information

### Environment State

The `EnvironmentState` structure contains information about the environment, including:

- Obstacle positions
- Environment boundaries
- Total number of agents
- Optional agent density map
- Optional SDF resolution
- Optional world size

### Factor Graph State

The `FactorGraphState` structure contains information about the state of a factor graph, including:

- Factor weights
- Variable and factor counts
- Message statistics
- Factor counts
- Detailed factor information

## Python Client

The Python client provides a convenient interface for interacting with the API, handling:

- Connection management
- Message serialization/deserialization
- Error handling
- Helper methods for common operations

## OpenAI Gym Integration

The API is designed to support integration with OpenAI Gym, allowing the Magics simulation to be used as an environment for reinforcement learning research. The integration includes:

- Observation space definition
- Action space definition
- Step, reset, and render methods
- Reward calculation

## Next Steps

Now that you understand the core concepts of the Magics API, you can:

- Explore the [commands reference](./commands.md) for detailed information about available commands
- Learn about the [data structures](./data_structures/) used in the API
- Check out the [examples](./examples/) for more advanced usage patterns
