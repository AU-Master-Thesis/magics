# Basic API Usage

This example demonstrates the basic usage of the Magics API for common operations such as connecting to the API, getting agent and environment states, and controlling the simulation.

## Connecting to the API

To use the Magics API, you first need to create a `MagicsClient` instance:

```python
from magics_client import MagicsClient

# Connect to the API (default: localhost:5555)
client = MagicsClient()

# Or specify a custom host and port
# client = MagicsClient(host="localhost", port=5555)
```

The `MagicsClient` constructor takes the following parameters:

- `host`: The hostname or IP address of the Magics server (default: "localhost")
- `port`: The port number of the Magics server (default: 5555)
- `timeout`: The timeout in milliseconds for receiving responses (default: 5000)

## Activating the API

By default, the API may not be active. You can check and activate it:

```python
# Check if the API is active
is_active = client.is_api_active()
print(f"API active: {is_active}")

# Activate the API if it's not already active
if not is_active:
    client.set_api_active(True)
    print("API activated")
```

## Getting Agent States

You can get the state of all agents in the simulation:

```python
# Get agent states
agent_states = client.get_agent_state()

# Print the number of agents
print(f"Number of agents: {len(agent_states)}")

# Print information about each agent
for agent_id, agent in agent_states.items():
    print(f"Agent {agent_id}:")
    print(f"  Position: {agent['position']}")
    print(f"  Velocity: {agent['velocity']}")
    print(f"  Mission state: {agent['mission_state']['type']}")
    
    # Check if the agent has a goal
    if agent['goal_point'] is not None:
        print(f"  Goal: {agent['goal_point']}")
```

The `agent_states` variable is a dictionary mapping agent IDs to agent state dictionaries. Each agent state dictionary contains the following fields:

- `agent_id`: Unique identifier for the agent
- `factorgraph_id`: Identifier for the agent's factor graph
- `position`: Current position of the agent [x, y]
- `velocity`: Current velocity of the agent [vx, vy]
- `factor_graph_state`: State of the agent's factor graph
- `connected_neighbors`: IDs of connected neighbor agents
- `mission_state`: Current mission state (Idle, Active, Completed)
- `planning_strategy`: Strategy used for planning (OnlyLocal, RrtStar)
- `radius`: Radius of the agent
- `communication_active`: Whether the agent's communication is active
- `communication_radius`: Radius of the agent's communication range
- `target_speed`: Target speed of the agent
- `current_waypoint_index`: Index of the current waypoint
- `next_waypoint`: Information about the next waypoint
- `goal_point`: Position of the goal point
- `mission_progress`: Information about mission progress
- `factor_details`: Detailed information about factor graph components
- `collision_info`: Information about collisions

See [Agent State](../data_structures/agent_state.md) for more information about the agent state structure.

## Getting Environment State

You can get the state of the environment:

```python
# Get environment state
env_state = client.get_environment_state()

# Print environment information
print(f"Number of obstacles: {len(env_state['obstacles'])}")
print(f"Environment boundaries: {env_state['boundaries']}")
print(f"Total agents: {env_state['total_agents']}")
```

The `env_state` variable is a dictionary containing the following fields:

- `obstacles`: Positions of obstacles in the environment
- `boundaries`: Boundaries of the environment (min, max)
- `total_agents`: Total number of agents in the environment
- `agent_density_map`: Optional agent density map
- `sdf_resolution`: Optional SDF resolution (width, height)
- `world_size`: Optional world size (width, height)

See [Environment State](../data_structures/environment_state.md) for more information about the environment state structure.

## Stepping the Simulation

You can step the simulation forward:

```python
# Set the number of iterations per step
client.set_iterations_per_step(10)

# Step the simulation
client.step()

# Get the updated agent states
agent_states = client.get_agent_state()
```

See [Stepping](./stepping.md) for more information about stepping through the simulation.

## Setting Factor Weights

You can modify the factor weights used in the simulation:

```python
# Set factor weights for all agents
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0
}
client.set_factor_weights(weights)
```

See [Weights](./weights.md) for more information about modifying factor weights.

## Getting and Setting Simulation Hz

You can get and set the simulation Hz (frequency):

```python
# Get the current simulation Hz
hz = client.get_simulation_hz()
print(f"Current simulation Hz: {hz}")

# Set the simulation Hz to 30
client.set_simulation_hz(30.0)
```

The simulation Hz determines the time step size for the simulation. A higher Hz means smaller time steps, which can lead to more accurate simulation but may also be slower.

## Closing the Connection

When you're done, you should close the connection:

```python
client.close()
```

## Complete Example

Here's a complete example that connects to the API, gets agent and environment states, steps the simulation, and closes the connection:

```python
from magics_client import MagicsClient
import numpy as np
import matplotlib.pyplot as plt

# Connect to the API
client = MagicsClient()

try:
    # Activate the API if it's not already active
    if not client.is_api_active():
        client.set_api_active(True)
        print("API activated")
    
    # Get the current simulation Hz
    hz = client.get_simulation_hz()
    print(f"Current simulation Hz: {hz}")
    
    # Get initial agent states
    agent_states = client.get_agent_state()
    print(f"Number of agents: {len(agent_states)}")
    
    # Get environment state
    env_state = client.get_environment_state()
    print(f"Number of obstacles: {len(env_state['obstacles'])}")
    print(f"Environment boundaries: {env_state['boundaries']}")
    
    # Set the number of iterations per step
    client.set_iterations_per_step(10)
    
    # Step the simulation
    print("Stepping simulation...")
    client.step()
    
    # Get updated agent states
    agent_states = client.get_agent_state()
    
    # Print positions of all agents
    for agent_id, agent in agent_states.items():
        print(f"Agent {agent_id} position: {agent['position']}")
    
    # Plot the environment
    plt.figure(figsize=(10, 10))
    
    # Plot boundaries
    min_bounds = env_state['boundaries']['min']
    max_bounds = env_state['boundaries']['max']
    plt.xlim(min_bounds[0], max_bounds[0])
    plt.ylim(min_bounds[1], max_bounds[1])
    
    # Plot obstacles
    obstacles = env_state['obstacles']
    plt.scatter(obstacles[:, 0], obstacles[:, 1], color='red', label='Obstacles')
    
    # Plot agents
    agent_positions = np.array([agent['position'] for agent in agent_states.values()])
    plt.scatter(agent_positions[:, 0], agent_positions[:, 1], color='blue', label='Agents')
    
    plt.grid(True)
    plt.legend()
    plt.title('Environment State')
    plt.xlabel('X')
    plt.ylabel('Y')
    plt.axis('equal')
    plt.show()
    
finally:
    # Close the connection
    client.close()
    print("Connection closed")
```

This example connects to the API, gets agent and environment states, steps the simulation, and plots the environment with agents and obstacles.

## Error Handling

The `MagicsClient` class raises a `MagicsError` exception if a command fails:

```python
from magics_client import MagicsClient, MagicsError

# Connect to the API
client = MagicsClient()

try:
    # Try to set an invalid Hz value
    client.set_simulation_hz(-1.0)
except MagicsError as e:
    print(f"Error: {e}")
finally:
    client.close()
```

## Implementation Details

The API client is implemented in [`magics_client.py`](../../../../python_api/magics_client.py), which provides a convenient interface for interacting with the API.

The server-side implementation is in the following files:

- [`zmq_server.rs`](../../zmq_server.rs): Handles incoming requests and dispatches them to the appropriate handler
- [`message.rs`](../../message.rs): Defines the message protocol and serialization/deserialization
- [`state.rs`](../../state.rs): Manages the API state and provides methods for accessing and modifying it
- [`extract.rs`](../../extract.rs): Extracts state information from the simulation
- [`factor_details.rs`](../../factor_details.rs): Extracts detailed information about factor graphs
- [`weights.rs`](../../weights.rs): Handles updates to factor weights
- [`reset.rs`](../../reset.rs): Handles resetting the API state
- [`plugin.rs`](../../plugin.rs): Integrates the API with the Bevy application

## Related Documentation

- [Commands Reference](../commands.md): Information about all available API commands
- [Agent State](../data_structures/agent_state.md): Information about the agent state structure
- [Environment State](../data_structures/environment_state.md): Information about the environment state structure
- [Factor Graph](../data_structures/factor_graph.md): Information about the factor graph state and details
- [Collision Information](../data_structures/collision_info.md): Information about collision detection and tracking
- [Stepping](./stepping.md): Information about stepping through the simulation
- [Weights](./weights.md): Information about modifying factor weights
