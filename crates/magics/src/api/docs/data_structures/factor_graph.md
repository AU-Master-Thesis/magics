# Factor Graph

The factor graph is a core component of the Magics simulation, representing the probabilistic model used for path planning. This document provides detailed information about the factor graph state and details exposed through the API.

## Factor Graph State

The `FactorGraphState` structure contains information about the state of a factor graph:

```rust
pub struct FactorGraphState {
    pub weights: FactorWeights,
    pub variable_count: usize,
    pub factor_count: usize,
    pub messages_sent: MessageStats,
    pub messages_received: MessageStats,
    pub factor_counts: FactorCounts,
    pub factor_details: FactorDetails,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `weights` | `FactorWeights` | Current weights of the factor graph | - |
| `variable_count` | `usize` | Number of variables in the factor graph | - |
| `factor_count` | `usize` | Number of factors in the factor graph | - |
| `messages_sent` | `MessageStats` | Statistics about messages sent from the factor graph | - |
| `messages_received` | `MessageStats` | Statistics about messages received by the factor graph | - |
| `factor_counts` | `FactorCounts` | Counts of different factor types | - |
| `factor_details` | `FactorDetails` | Detailed information about factor graph components | - |

## Factor Weights

The `FactorWeights` structure contains the weights for different factor types in the factor graph:

```rust
pub struct FactorWeights {
    pub dynamic: f32,
    pub obstacle: f32,
    pub interrobot: f32,
    pub tracking: f32,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `dynamic` | `f32` | Weight for dynamic factors | - |
| `obstacle` | `f32` | Weight for obstacle factors | - |
| `interrobot` | `f32` | Weight for inter-robot factors | - |
| `tracking` | `f32` | Weight for tracking factors | - |

These weights determine the influence of each factor type on the overall solution. Higher weights give more importance to the corresponding factor type.

## Message Statistics

The `MessageStats` structure contains statistics about messages in the factor graph:

```rust
pub struct MessageStats {
    pub internal: usize,
    pub external: usize,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `internal` | `usize` | Number of internal messages | - |
| `external` | `usize` | Number of external messages | - |

Internal messages are sent between nodes within the same factor graph, while external messages are sent between nodes in different factor graphs (i.e., between different agents).

## Factor Counts

The `FactorCounts` structure contains counts of different factor types in the factor graph:

```rust
pub struct FactorCounts {
    pub obstacle: usize,
    pub interrobot: usize,
    pub dynamic: usize,
    pub tracking: usize,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `obstacle` | `usize` | Number of obstacle factors | - |
| `interrobot` | `usize` | Number of inter-robot factors | - |
| `dynamic` | `usize` | Number of dynamic factors | - |
| `tracking` | `usize` | Number of tracking factors | - |

## Factor Details

The `FactorDetails` structure contains detailed information about factor graph components:

```rust
pub struct FactorDetails {
    pub variables: Vec<VariableInfo>,
    pub obstacle_factors: Vec<ObstacleFactorInfo>,
    pub interrobot_factors: Vec<InterRobotFactorInfo>,
    pub tracking_factors: Vec<TrackingFactorInfo>,
    pub dynamic_factors: Vec<DynamicFactorInfo>,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variables` | `Vec<VariableInfo>` | Information about variables in the factor graph | - |
| `obstacle_factors` | `Vec<ObstacleFactorInfo>` | Information about obstacle factors | - |
| `interrobot_factors` | `Vec<InterRobotFactorInfo>` | Information about inter-robot factors | - |
| `tracking_factors` | `Vec<TrackingFactorInfo>` | Information about tracking factors | - |
| `dynamic_factors` | `Vec<DynamicFactorInfo>` | Information about dynamic factors | - |

### Variable Information

The `VariableInfo` structure contains information about a variable in the factor graph:

```rust
pub struct VariableInfo {
    pub index: usize,
    pub factorgraph_id: u32,
    pub mean: [f64; 4],
    pub covariance: [f64; 16],
    pub estimated_position: [f64; 2],
    pub estimated_velocity: [f64; 2],
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `index` | `usize` | Index of the variable | - |
| `factorgraph_id` | `u32` | ID of the factor graph this variable belongs to | - |
| `mean` | `[f64; 4]` | Mean vector of the variable [x, y, vx, vy] | world units, world units/s |
| `covariance` | `[f64; 16]` | Covariance matrix of the variable (4x4, flattened) | - |
| `estimated_position` | `[f64; 2]` | Estimated position from the variable [x, y] | world units |
| `estimated_velocity` | `[f64; 2]` | Estimated velocity from the variable [vx, vy] | world units/s |

The mean vector represents the estimated state of the variable, which includes position (x, y) and velocity (vx, vy). The covariance matrix represents the uncertainty in this estimate.

### Obstacle Factor Information

The `ObstacleFactorInfo` structure contains information about an obstacle factor in the factor graph:

```rust
pub struct ObstacleFactorInfo {
    pub variable_index: usize,
    pub sdf_value: f64,
    pub position: [f32; 2],
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variable_index` | `usize` | Index of the variable this factor is connected to | - |
| `sdf_value` | `f64` | SDF value at the position, ranges from 0.0 (free space) to 1.0 (obstacle) | - |
| `position` | `[f32; 2]` | Position where the SDF value was measured [x, y] | world units |

Obstacle factors model the interaction between an agent and obstacles in the environment. The SDF (Signed Distance Field) value represents the distance to the nearest obstacle, normalized to the range [0, 1].

### Inter-Robot Factor Information

The `InterRobotFactorInfo` structure contains information about an inter-robot factor in the factor graph:

```rust
pub struct InterRobotFactorInfo {
    pub variable_index: usize,
    pub external_robot_id: u32,
    pub external_factorgraph_id: u32,
    pub external_variable_index: usize,
    pub safety_distance: f32,
    pub distance_between_variables: f64,
    pub active: bool,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variable_index` | `usize` | Index of the variable this factor is connected to | - |
| `external_robot_id` | `u32` | ID of the external robot this factor connects to | - |
| `external_factorgraph_id` | `u32` | ID of the external factor graph | - |
| `external_variable_index` | `usize` | Index of the variable in the external robot's factor graph | - |
| `safety_distance` | `f32` | Safety distance for collision avoidance | world units |
| `distance_between_variables` | `f64` | Current distance between the two variables | world units |
| `active` | `bool` | Whether the factor is active | - |

Inter-robot factors model the interaction between different agents. They help agents avoid collisions with each other by maintaining a safety distance.

### Tracking Factor Information

The `TrackingFactorInfo` structure contains information about a tracking factor in the factor graph:

```rust
pub struct TrackingFactorInfo {
    pub variable_index: usize,
    pub tracking_path: Vec<[f32; 2]>,
    pub tracking_index: usize,
    pub projected_position: [f32; 2],
    pub path_deviation: f32,
    pub distance_to_path: f64,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variable_index` | `usize` | Index of the variable this factor is connected to | - |
| `tracking_path` | `Vec<[f32; 2]>` | Path that the robot is tracking, list of [x, y] points | world units |
| `tracking_index` | `usize` | Current index in the tracking path | - |
| `projected_position` | `[f32; 2]` | Projected position on the path [x, y] | world units |
| `path_deviation` | `f32` | Path deviation measurement (normalized distance) | 0.0-1.0 |
| `distance_to_path` | `f64` | Distance from robot to projected point on path | world units |

Tracking factors model the tracking of a predefined path. They help agents follow a specified trajectory by minimizing the distance to the path.

### Dynamic Factor Information

The `DynamicFactorInfo` structure contains information about a dynamic factor in the factor graph:

```rust
pub struct DynamicFactorInfo {
    pub from_variable_index: usize,
    pub to_variable_index: usize,
    pub delta_t: f32,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `from_variable_index` | `usize` | Index of the source variable | - |
| `to_variable_index` | `usize` | Index of the destination variable | - |
| `delta_t` | `f32` | Time step between the variables | seconds |

Dynamic factors model the dynamics of an agent, connecting variables at different time steps. They ensure that the agent's motion follows a physically plausible trajectory.

## Implementation Details

The factor graph state is populated in the `extract_state` function in [`extract.rs`](../../extract.rs), which extracts the state from the Bevy ECS components.

The factor details are extracted in the `extract_factor_details` function in [`factor_details.rs`](../../factor_details.rs), which extracts detailed information about factor graph components.

## Example Usage

Here's an example of how to access factor graph information using the Python client:

```python
from magics_client import MagicsClient
import numpy as np

# Connect to the API
client = MagicsClient()

# Get agent states
agent_states = client.get_agent_state()

# Print factor graph information for each agent
for agent_id, agent in agent_states.items():
    factor_graph_state = agent['factor_graph_state']
    factor_details = agent['factor_details']
    
    print(f"Agent {agent_id} Factor Graph:")
    print(f"  Variables: {factor_graph_state['variable_count']}")
    print(f"  Factors: {factor_graph_state['factor_count']}")
    
    # Print factor weights
    weights = factor_graph_state['weights']
    print(f"  Weights:")
    print(f"    Dynamic: {weights['dynamic']}")
    print(f"    Obstacle: {weights['obstacle']}")
    print(f"    Interrobot: {weights['interrobot']}")
    print(f"    Tracking: {weights['tracking']}")
    
    # Print message statistics
    messages_sent = factor_graph_state['messages_sent']
    messages_received = factor_graph_state['messages_received']
    print(f"  Messages:")
    print(f"    Sent: {messages_sent['internal']} internal, {messages_sent['external']} external")
    print(f"    Received: {messages_received['internal']} internal, {messages_received['external']} external")
    
    # Print factor counts
    factor_counts = factor_graph_state['factor_counts']
    print(f"  Factor Counts:")
    print(f"    Obstacle: {factor_counts['obstacle']}")
    print(f"    Interrobot: {factor_counts['interrobot']}")
    print(f"    Dynamic: {factor_counts['dynamic']}")
    print(f"    Tracking: {factor_counts['tracking']}")
    
    # Print variable information
    variables = factor_details['variables']
    print(f"  Variables ({len(variables)}):")
    for i, var in enumerate(variables):
        if i < 3:  # Print only the first 3 variables
            print(f"    Variable {var['index']}:")
            print(f"      Position: {var['estimated_position']}")
            print(f"      Velocity: {var['estimated_velocity']}")
    
    # Print obstacle factor information
    obstacle_factors = factor_details['obstacle_factors']
    print(f"  Obstacle Factors ({len(obstacle_factors)}):")
    for i, factor in enumerate(obstacle_factors):
        if i < 3:  # Print only the first 3 factors
            print(f"    Factor {i}:")
            print(f"      Variable: {factor['variable_index']}")
            print(f"      SDF Value: {factor['sdf_value']}")
            print(f"      Position: {factor['position']}")
```

## Modifying Factor Weights

You can modify the factor weights using the `set_factor_weights` method of the Python client:

```python
from magics_client import MagicsClient

# Connect to the API
client = MagicsClient()

# Set factor weights for all agents
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0
}
client.set_factor_weights(weights)

# Set factor weights for a specific agent
agent_id = 1
client.set_factor_weights(weights, agent_id)
```

## Related Data Structures

- [Agent State](./agent_state.md): Information about an agent in the simulation
- [Environment State](./environment_state.md): Information about the environment
