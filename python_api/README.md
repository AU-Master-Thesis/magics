# Magics ZeroMQ Python API

This package provides a Python API for the Magics simulation using ZeroMQ for communication. It replaces the previous PyO3-based Python bindings with a more flexible and robust network-based approach.

## System Architecture

### Overview

```mermaid
graph TB
    subgraph "Rust Simulation"
        B[Bevy App] --> S[API State]
        S <--> Z[ZMQ Server]
    end
    
    subgraph "Python Client"
        C[MagicsClient] --> G[MagicsGym Environment]
        C <--> R[Reinforcement Learning Agent]
    end
    
    Z <--> C
    
    style B fill:#f9d77e,stroke:#333,stroke-width:2px
    style S fill:#a1c9f4,stroke:#333,stroke-width:2px
    style Z fill:#fbafe4,stroke:#333,stroke-width:2px
    style C fill:#8ce8ad,stroke:#333,stroke-width:2px
    style G fill:#8ce8ad,stroke:#333,stroke-width:2px
    style R fill:#c4c4c4,stroke:#333,stroke-width:2px
```

### Detailed Component Diagram

```mermaid
graph LR
    subgraph "Rust Simulation"
        subgraph "Bevy App"
            P[Plugin Systems]
            FA[Factor Graph]
            E[Environment]
            R[Robots]
        end
        
        subgraph "API Layer"
            AS[API State]
            AS --- FA
            AS --- E
            AS --- R
        end
        
        subgraph "ZMQ Server"
            ZT[Server Thread]
            ZS[Socket: REP]
        end
        
        P --> AS
        AS --> ZT
        ZT --> ZS
    end
    
    subgraph "Network"
        N{ZeroMQ Protocol}
    end
    
    subgraph "Python Client"
        subgraph "Client Layer"
            ZC[Socket: REQ]
            MC[MagicsClient]
        end
        
        subgraph "Environment"
            GE[MagicsEnv]
            OB[Observation Space]
            AC[Action Space]
            RW[Reward Function]
        end
        
        MC --> GE
        GE --> OB
        GE --> AC
        GE --> RW
    end
    
    ZS <--> N
    N <--> ZC
    ZC --> MC
    
    style P fill:#f9d77e,stroke:#333,stroke-width:2px
    style FA fill:#f9d77e,stroke:#333,stroke-width:2px
    style E fill:#f9d77e,stroke:#333,stroke-width:2px
    style R fill:#f9d77e,stroke:#333,stroke-width:2px
    
    style AS fill:#a1c9f4,stroke:#333,stroke-width:2px
    
    style ZT fill:#fbafe4,stroke:#333,stroke-width:2px
    style ZS fill:#fbafe4,stroke:#333,stroke-width:2px
    
    style N fill:#c4c4c4,stroke:#333,stroke-width:2px
    
    style ZC fill:#8ce8ad,stroke:#333,stroke-width:2px
    style MC fill:#8ce8ad,stroke:#333,stroke-width:2px
    
    style GE fill:#8ce8ad,stroke:#333,stroke-width:2px
    style OB fill:#d0e8ff,stroke:#333,stroke-width:2px
    style AC fill:#d0e8ff,stroke:#333,stroke-width:2px
    style RW fill:#d0e8ff,stroke:#333,stroke-width:2px
```

### Communication Sequence

```mermaid
sequenceDiagram
    participant RL as Reinforcement Learning Agent
    participant Gym as MagicsEnv (Gymnasium)
    participant Client as MagicsClient
    participant ZMQ as ZMQ REQ/REP
    participant Server as ZMQ Server
    participant Sim as Bevy Simulation
    
    RL->>Gym: reset()
    Gym->>Client: client.reset()
    Client->>ZMQ: Send "Reset" command
    ZMQ->>Server: Request
    Server->>Sim: Reset simulation
    Sim->>Server: Acknowledge
    Server->>ZMQ: Response
    ZMQ->>Client: Success
    Client->>Gym: Get initial state
    Gym->>RL: Initial observation
    
    loop Training
        RL->>Gym: step(action)
        Gym->>Client: set_factor_weights(weights)
        Client->>ZMQ: Send "SetFactorWeights" command
        ZMQ->>Server: Request
        Server->>Sim: Update weights
        Sim->>Server: Acknowledge
        Server->>ZMQ: Response
        ZMQ->>Client: Success
        
        Gym->>Client: step()
        Client->>ZMQ: Send "Step" command
        ZMQ->>Server: Request
        Server->>Sim: Step simulation
        Note over Sim: Physics Update
        Sim->>Server: Completed
        Server->>ZMQ: Response
        ZMQ->>Client: Success
        
        Gym->>Client: get_agent_state()
        Client->>ZMQ: Send "GetAgentState" command
        ZMQ->>Server: Request
        Server->>Sim: Retrieve state
        Sim->>Server: Agent states
        Server->>ZMQ: Response with data
        ZMQ->>Client: Agent states
        
        Gym->>Client: get_environment_state()
        Client->>ZMQ: Send "GetEnvironmentState" command
        ZMQ->>Server: Request
        Server->>Sim: Retrieve environment
        Sim->>Server: Environment state
        Server->>ZMQ: Response with data
        ZMQ->>Client: Environment state
        
        Client->>Gym: Combined state data
        Gym->>Gym: Compute reward
        Gym->>RL: observation, reward, terminated, truncated, info
    end
    
    RL->>Gym: close()
    Gym->>Client: close()
    Client->>ZMQ: Cleanup
    Client->>Client: Terminate
```

### Python Class Structure

```mermaid
classDiagram
    class MagicsClient {
        -context: zmq.Context
        -socket: zmq.Socket
        +__init__(host, port, timeout)
        +_send_request(command, **parameters)
        +get_agent_state()
        +get_environment_state()
        +set_factor_weights(weights, agent_id)
        +step()
        +reset()
        +is_api_active()
        +set_api_active(active)
        +close()
    }
    
    class MagicsError {
        <<Exception>>
    }
    
    class MagicsEnv {
        -client: MagicsClient
        -agent_states: dict
        -environment_state: dict
        -steps: int
        -max_steps: int
        +__init__(host, port, render_mode)
        -_define_observation_space()
        -_update_state()
        -_get_observation()
        -_compute_reward()
        -_is_done()
        +step(action)
        +reset(seed, options)
        +render()
        +close()
    }
    
    class gym.Env {
        <<Interface>>
        +observation_space
        +action_space
        +step(action)
        +reset()
        +render()
        +close()
    }
    
    MagicsClient -- MagicsError : throws
    MagicsEnv --|> gym.Env : implements
    MagicsEnv *-- MagicsClient : uses
```

## Contents

- `magics_client.py`: Direct ZeroMQ client for communicating with the Magics simulation
- `magics_gym`: OpenAI Gymnasium environment for reinforcement learning applications
- `example.py`: Example usage of both the direct client and the Gym environment
- `setup.py`: Installation script for the package

## Installation

### Prerequisites

- Python 3.8 or higher
- The Magics simulation running with the ZeroMQ API enabled
- Required Python packages: numpy, pyzmq, gymnasium

### Installing

Clone this repository and install the package:

```bash
# Install dependencies
pip install numpy pyzmq gymnasium

# Install the package in development mode
cd python_api
pip install -e .
```

## Usage

### Starting the Magics Simulation

Before using the API, you need to start the Magics simulation with the API feature enabled:

```bash
cd /path/to/magics
cargo run --features api
```

### Direct ZeroMQ Client

The `MagicsClient` class provides direct access to the Magics API:

```python
from magics_client import MagicsClient

# Connect to the Magics server
client = MagicsClient(host="localhost", port=5555)

# Activate the API
client.set_api_active(True)

# Get the state of all agents
agent_states = client.get_agent_state()

# Get the state of the environment
env_state = client.get_environment_state()

# Set factor weights
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0,
}
client.set_factor_weights(weights)

# Step the simulation
client.step()

# Close the connection
client.close()
```

### OpenAI Gym Environment

The `MagicsEnv` class provides an OpenAI Gymnasium environment for reinforcement learning:

```python
from magics_gym import MagicsEnv

# Create the environment
env = MagicsEnv()

# Reset the environment
observation, info = env.reset()

# Sample a random action
action = env.action_space.sample()

# Take a step in the environment
observation, reward, terminated, truncated, info = env.step(action)

# Close the environment
env.close()
```

## API Reference

### MagicsClient

- `__init__(host="localhost", port=5555, timeout=5000)`: Initialize the client with the given host, port, and timeout
- `get_agent_state()`: Get the state of all agents in the simulation
- `get_environment_state()`: Get the state of the environment
- `set_factor_weights(weights, agent_id=None)`: Set factor graph weights
- `step()`: Step the simulation forward by one frame
- `reset()`: Reset the simulation
- `is_api_active()`: Check if the API is active
- `set_api_active(active)`: Set the API active state
- `close()`: Close the connection

#### Agent State

The agent state returned by `get_agent_state()` includes the following fields:

- `position`: 2D position of the agent [x, y]
- `velocity`: 2D velocity of the agent [vx, vy]
- `factor_graph_state`: State of the agent's factor graph
- `connected_neighbors`: List of connected neighbor IDs
- `mission_state`: Current mission state of the agent
  - `IDLE`: Agent is idle, possibly waiting for waypoints
  - `ACTIVE`: Agent is actively following a route
  - `COMPLETED`: Agent has completed its mission
- `planning_strategy`: Planning strategy used by the agent
  - `ONLY_LOCAL`: Agent uses only local planning
  - `RRT_STAR`: Agent uses RRT* for global planning
- `radius`: Radius of the agent
- `communication_active`: Whether the agent's communication is active
- `communication_radius`: Communication radius of the agent
- `target_speed`: Target speed of the agent
- `current_waypoint_index`: Index of the current waypoint
- `next_waypoint`: Information about the next waypoint
- `goal_point`: Position of the goal point
- `mission_progress`: Mission progress information
- `factor_details`: Detailed information about factor graph components

### MagicsEnv

The `MagicsEnv` class implements the OpenAI Gymnasium interface:

- `__init__(host="localhost", port=5555, render_mode=None)`: Initialize the environment
- `step(action)`: Take a step in the environment
- `reset(seed=None, options=None)`: Reset the environment
- `render()`: Render the environment
- `close()`: Close the environment

#### Observation Space

The observation space is a dictionary with the following structure:

```python
{
    "agents": {
        "positions": Box(shape=(num_agents, 2)),  # Agent positions
        "velocities": Box(shape=(num_agents, 2)),  # Agent velocities
        "connectivity": Box(shape=(num_agents, num_agents)),  # Connectivity matrix
        "planning_strategies": MultiDiscrete([2] * num_agents),  # 0=OnlyLocal, 1=RrtStar
        "mission_states": MultiDiscrete([3] * num_agents),  # 0=Idle, 1=Active, 2=Completed
        "waiting_for_waypoints": MultiBinary(num_agents)  # Whether agents are waiting for waypoints
    },
    "obstacles": Box(shape=(num_obstacles, 2)),  # Obstacle positions
    "boundaries": Box(shape=(2, 2))  # Environment boundaries [min, max]
}
```

#### Action Space

The action space is a Box with shape (4,) representing the factor weights:
- `[dynamic, obstacle, interrobot, tracking]`

Each weight can range from 0.1 to 10.0.

## Comparison with PyO3-based API

This ZeroMQ-based API offers several advantages over the previous PyO3-based API:

1. **Network transparency**: The API can be used from any machine, not just the one running the simulation
2. **Language independence**: The protocol is language-agnostic, enabling easy implementation of clients in other languages
3. **Process isolation**: The simulation and client run in separate processes, improving stability
4. **Easier debugging**: The network layer makes it easier to debug API calls
5. **More scalable**: The architecture supports multiple clients connecting to the same simulation
6. **No compilation dependencies**: No need for PyO3 and Rust toolchain to install the Python package
7. **Modern Gymnasium interface**: Uses the latest Gymnasium API for reinforcement learning applications

## Running the Example

An example script is provided to demonstrate the use of both the direct client and the Gym environment:

```bash
python example.py
```

This script:
1. Connects to the Magics simulation
2. Gets the state of the environment and agents
3. Sets factor weights and steps the simulation
4. Creates a Gym environment and runs it for a few steps with random actions
