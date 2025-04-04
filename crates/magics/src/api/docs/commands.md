# Magics API Commands Reference

This document provides detailed information about all the commands available in the Magics API, including their parameters and return values.

## Command Overview

The Magics API supports the following commands:

| Command | Description |
|---------|-------------|
| `GetAgentState` | Get the state of all agents in the simulation |
| `GetEnvironmentState` | Get the state of the environment |
| `SetFactorWeights` | Set factor graph weights |
| `Step` | Step the simulation forward by one frame |
| `Reset` | Reset the simulation |
| `IsApiActive` | Check if the API is active |
| `SetApiActive` | Set the API active state |
| `SetIterationsPerStep` | Set the number of iterations per step |
| `GetSimulationHz` | Get the simulation Hz (frequency) |
| `SetSimulationHz` | Set the simulation Hz (frequency) |
| `RemoveAgent` | Remove an agent from the simulation |
| `SpawnAgent` | Spawn a new agent in the simulation |

## Command Details

### GetAgentState

Get the state of all agents in the simulation.

**Parameters**: None

**Returns**: A dictionary mapping agent IDs to agent states. Each agent state contains:
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

**Example**:
```python
agent_states = client.get_agent_state()
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### GetEnvironmentState

Get the state of the environment.

**Parameters**: None

**Returns**: An environment state object containing:
- `obstacles`: Positions of obstacles in the environment
- `boundaries`: Boundaries of the environment (min, max)
- `total_agents`: Total number of agents in the environment
- `agent_density_map`: Optional agent density map
- `sdf_resolution`: Optional SDF resolution (width, height)
- `world_size`: Optional world size (width, height)

**Example**:
```python
env_state = client.get_environment_state()
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### SetFactorWeights

Set factor graph weights.

**Parameters**:
- `weights`: Dictionary mapping factor names to weights
  - `dynamic`: Weight for dynamic factors
  - `obstacle`: Weight for obstacle factors
  - `interrobot`: Weight for inter-robot factors
  - `tracking`: Weight for tracking factors
- `agent_id`: Optional agent ID for per-agent weights

**Returns**: None

**Example**:
```python
# Set weights for all agents
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0
}
client.set_factor_weights(weights)

# Set weights for a specific agent
client.set_factor_weights(weights, agent_id=1)
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`weights.rs`](../weights.rs) (weight application), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### Step

Step the simulation forward by one frame.

**Parameters**: None

**Returns**: None

**Example**:
```python
client.step()
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`plugin.rs`](../plugin.rs) (stepping mechanism), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### Reset

Reset the simulation. Optionally accepts a seed for reproducible randomness.

**Parameters**:
- `seed` (optional): An unsigned 64-bit integer (`u64`) to seed the random number generator used during the reset process. If provided, the reset will be deterministic for that seed. If omitted, the reset will use random initialization.

**Returns**: None

**Example**:
```python
client.reset()
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`reset.rs`](../reset.rs) (reset handling), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### IsApiActive

Check if the API is active.

**Parameters**: None

**Returns**: Boolean indicating whether the API is active

**Example**:
```python
is_active = client.is_api_active()
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`state.rs`](../state.rs) (API state management), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### SetApiActive

Set the API active state.

**Parameters**:
- `active`: Boolean indicating whether to activate the API

**Returns**: None

**Example**:
```python
client.set_api_active(True)
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`state.rs`](../state.rs) (API state management), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### SetIterationsPerStep

Set the number of iterations per step.

**Parameters**:
- `iterations`: The number of iterations per step (must be greater than 0)

**Returns**: None

**Example**:
```python
client.set_iterations_per_step(10)
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`state.rs`](../state.rs) (API state management), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### GetSimulationHz

Get the simulation Hz (frequency).

**Parameters**: None

**Returns**: The current simulation Hz (frequency)

**Example**:
```python
hz = client.get_simulation_hz()
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`state.rs`](../state.rs) (API state management), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### SetSimulationHz

Set the simulation Hz (frequency).

**Parameters**:
- `hz`: The new Hz value (must be greater than 0)

**Returns**: None

**Example**:
```python
client.set_simulation_hz(60.0)
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`state.rs`](../state.rs) (API state management), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### RemoveAgent

Remove an agent from the simulation immediately.

**Parameters**:
- `agent_id`: The ID (u32) of the agent to remove. This corresponds to the `agent_id` field in `AgentState`, which is the Bevy `Entity` index.

**Returns**: None

**Example**:
```python
# Remove agent with ID 5
client.remove_agent(agent_id=5)
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`plugin.rs`](../plugin.rs) (removal system), [`state.rs`](../state.rs) (request queue), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### SpawnAgent

Spawn a new agent in the simulation.

**Parameters**:
- `initial_position`: List `[x, z]` representing the initial position.
- `goal_position`: List `[x, z]` representing the goal position.
- `initial_velocity` (optional): List `[vx, vz]` for initial velocity. Defaults to `[0.0, 0.0]`.
- `radius` (optional): Float for the agent's radius. Defaults to the value in the simulation config (`config.robot.radius`).
- `planning_strategy` (optional): String `"OnlyLocal"` or `"RrtStar"`. Defaults to `"OnlyLocal"`.
- `target_speed` (optional): Float for the agent's target speed. Defaults to the value in the simulation config (`config.robot.target_speed`).
- `weights` (optional): Dictionary for custom factor weights (see `SetFactorWeights`). Defaults to simulation config values.

**Returns**: The ID (u32) of the newly spawned agent.

**Example**:
```python
new_agent_id = client.spawn_agent(
    initial_position=[10.0, 5.0],
    goal_position=[-5.0, -8.0],
    radius=0.6,
    target_speed=1.2
)
print(f"Spawned agent with ID: {new_agent_id}")
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`plugin.rs`](../plugin.rs) (spawn system), [`state.rs`](../state.rs) (request/result queues), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)

### GetCurrentScenario

Get the name of the currently loaded scenario.

**Parameters**: None

**Returns**: The name of the current scenario as a string, or `null` if no scenario is loaded or the information is unavailable.

**Example**:
```python
scenario_name = client.get_current_scenario()
if scenario_name:
    print(f"Current scenario: {scenario_name}")
else:
    print("No scenario currently loaded.")
```

**Implementation**: [`zmq_server.rs`](../zmq_server.rs) (server-side), [`state.rs`](../state.rs) (API state management), [`plugin.rs`](../plugin.rs) (state update system), [`magics_client.py`](../../../../python_api/magics_client.py) (client-side)


## Message Protocol

All commands are sent using the ZeroMQ REQ-REP pattern. The client sends a request message and waits for a response from the server.

### Request Format

The request message is a JSON object with the following structure:

```json
{
  "command": {
    "command": "CommandName",
    "parameters": {
      "param1": "value1",
      "param2": "value2"
    }
  },
  "request_id": "unique-request-id"
}
```

- `command`: The command object
  - `command`: The name of the command to execute
  - `parameters`: Optional parameters for the command
- `request_id`: Optional unique identifier for the request

### Response Format

The response message is a JSON object with the following structure:

```json
{
  "status": "Success",
  "data": {
    "type": "DataType",
    "content": { ... }
  },
  "error": null,
  "request_id": "unique-request-id"
}
```

- `status`: The status of the response ("Success" or "Error")
- `data`: Optional data returned by the command
  - `type`: The type of data returned
  - `content`: The actual data content
- `error`: Optional error message (if status is "Error")
- `request_id`: The request ID from the original request

## Error Handling

If a command fails, the server will return an error response with a status of "Error" and an error message in the `error` field.

The Python client will raise a `MagicsError` exception with the error message if a command fails.

**Example**:
```python
try:
    client.set_simulation_hz(-1.0)  # Invalid Hz value
except MagicsError as e:
    print(f"Error: {e}")
```

## Implementation Details

The command handling is implemented in the following files:

- [`zmq_server.rs`](../zmq_server.rs): Handles incoming requests and dispatches them to the appropriate handler
- [`message.rs`](../message.rs): Defines the message protocol and serialization/deserialization
- [`state.rs`](../state.rs): Manages the API state and provides methods for accessing and modifying it
- [`extract.rs`](../extract.rs): Extracts state information from the simulation
- [`factor_details.rs`](../factor_details.rs): Extracts detailed information about factor graphs
- [`weights.rs`](../weights.rs): Handles updates to factor weights
- [`reset.rs`](../reset.rs): Handles resetting the API state
- [`plugin.rs`](../plugin.rs): Integrates the API with the Bevy application
- [`magics_client.py`](../../../../python_api/magics_client.py): Provides a Python client for interacting with the API
