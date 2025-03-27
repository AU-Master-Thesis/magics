# Environment State

The `EnvironmentState` structure contains information about the environment in the simulation. This document provides detailed information about all fields in the `EnvironmentState` structure, including their meaning, units, and relationships.

## Structure Overview

The `EnvironmentState` structure is defined in [`state.rs`](../../state.rs) and contains the following fields:

```rust
pub struct EnvironmentState {
    pub obstacles: Vec<Vec2>,
    pub boundaries: (Vec2, Vec2),
    pub total_agents: usize,
    pub agent_density_map: Option<Vec<f32>>,
    pub sdf_resolution: Option<(usize, usize)>,
    pub world_size: Option<(f64, f64)>,
}
```

## Field Descriptions

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `obstacles` | `Vec<Vec2>` | Positions of obstacles in the environment | world units |
| `boundaries` | `(Vec2, Vec2)` | Boundaries of the environment (min, max) | world units |
| `total_agents` | `usize` | Total number of agents in the environment | - |
| `agent_density_map` | `Option<Vec<f32>>` | Optional agent density map | - |
| `sdf_resolution` | `Option<(usize, usize)>` | Optional SDF resolution (width, height) | pixels |
| `world_size` | `Option<(f64, f64)>` | Optional world size (width, height) | world units |

### Obstacles

The `obstacles` field contains a list of positions of obstacles in the environment. Each position is a 2D vector with x and y coordinates in world units.

Obstacles are represented as points in the environment. In the simulation, obstacles are typically represented as spheres or cylinders with a radius, but the API only exposes their positions.

### Boundaries

The `boundaries` field contains the minimum and maximum coordinates of the environment. The first `Vec2` represents the minimum x and y coordinates, and the second `Vec2` represents the maximum x and y coordinates.

The boundaries define the extents of the environment in world units. Agents are typically constrained to stay within these boundaries.

### Total Agents

The `total_agents` field contains the total number of agents in the environment. This is a convenience field that provides the same information as the length of the agent states dictionary returned by the `GetAgentState` command.

### Agent Density Map

The `agent_density_map` field is an optional field that contains a density map of agents in the environment. If present, it is a flattened 2D array of floating-point values representing the density of agents at each point in the environment.

The density map can be used for visualization or for planning purposes, such as identifying areas with high agent density.

**Note**: This field is currently not populated in the implementation and is always `None`. It is reserved for future use.

### SDF Resolution

The `sdf_resolution` field is an optional field that contains the resolution of the Signed Distance Field (SDF) used for obstacle avoidance. If present, it is a tuple of width and height in pixels.

The SDF is a grid-based representation of the distance to the nearest obstacle at each point in the environment. The resolution determines the granularity of this grid.

**Note**: This field is currently not populated in the implementation and is always `None`. It is reserved for future use.

### World Size

The `world_size` field is an optional field that contains the size of the world in world units. If present, it is a tuple of width and height.

The world size defines the dimensions of the environment in world units. It is typically used in conjunction with the SDF resolution to convert between world coordinates and SDF grid coordinates.

**Note**: This field is currently not populated in the implementation and is always `None`. It is reserved for future use.

## Serialization

When the `EnvironmentState` is sent over the API, it is serialized into a JSON object with the following structure:

```json
{
  "obstacles": [[10.0, 20.0], [30.0, 40.0], ...],
  "boundaries": [[-100.0, -100.0], [100.0, 100.0]],
  "total_agents": 5,
  "agent_density_map": null,
  "sdf_resolution": null,
  "world_size": null
}
```

## Python Client

In the Python client, the `EnvironmentState` is represented as a dictionary with the same fields as the JSON object. The `obstacles` field is converted to a NumPy array for convenience, and the `boundaries` field is converted to a dictionary with `min` and `max` keys.

```python
env_state = client.get_environment_state()
obstacles = env_state["obstacles"]  # numpy.ndarray
boundaries = env_state["boundaries"]  # {"min": numpy.ndarray, "max": numpy.ndarray}
```

## Implementation Details

The `EnvironmentState` structure is populated in the `extract_state` function in [`extract.rs`](../../extract.rs), which extracts the state from the Bevy ECS components.

Currently, only the `obstacles`, `boundaries`, and `total_agents` fields are populated. The `agent_density_map`, `sdf_resolution`, and `world_size` fields are reserved for future use and are always `None`.

The serialization of the `EnvironmentState` is handled in the `From` implementation in [`message.rs`](../../message.rs), which converts the `EnvironmentState` to a `SerializedEnvironmentState` for transmission over the API.

## Example Usage

Here's an example of how to access environment state information using the Python client:

```python
from magics_client import MagicsClient
import numpy as np
import matplotlib.pyplot as plt

# Connect to the API
client = MagicsClient()

# Get environment state
env_state = client.get_environment_state()

# Print environment information
print(f"Number of obstacles: {len(env_state['obstacles'])}")
print(f"Environment boundaries: {env_state['boundaries']}")
print(f"Total agents: {env_state['total_agents']}")

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

# Get agent states
agent_states = client.get_agent_state()

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
```

## Future Enhancements

The following enhancements are planned for the `EnvironmentState` structure:

1. **Agent Density Map**: Implement the extraction of the agent density map from the simulation.
2. **SDF Resolution**: Implement the extraction of the SDF resolution from the environment configuration.
3. **World Size**: Implement the extraction of the world size from the environment configuration.

These enhancements will provide more detailed information about the environment, which can be useful for visualization and planning purposes.

## Related Data Structures

- [Agent State](./agent_state.md): Information about an agent in the simulation
- [Factor Graph](./factor_graph.md): Information about the factor graph state and details
