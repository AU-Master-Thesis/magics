# Agent State

The `AgentState` structure contains comprehensive information about a single agent in the simulation. This document provides detailed information about all fields in the `AgentState` structure, including their meaning, units, and relationships.

## Structure Overview

The `AgentState` structure is defined in [`state.rs`](../../state.rs) and contains the following fields:

```rust
pub struct AgentState {
    pub agent_id: u32,
    pub factorgraph_id: u32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub factor_graph_state: FactorGraphState,
    pub connected_neighbors: Vec<Entity>,
    pub mission_state: MissionState,
    pub planning_strategy: PlanningStrategy,
    pub radius: f32,
    pub communication_active: bool,
    pub communication_radius: f32,
    pub target_speed: f32,
    pub current_waypoint_index: Option<usize>,
    pub next_waypoint: Option<StateVectorInfo>,
    pub goal_point: Option<Vec2>,
    pub mission_progress: MissionProgress,
    pub factor_details: FactorDetails,
    pub collision_info: CollisionInfo,
}
```

## Field Descriptions

### Basic Information

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `agent_id` | `u32` | Unique identifier for the agent | - |
| `factorgraph_id` | `u32` | Identifier for the agent's factor graph (same as agent_id) | - |
| `position` | `Vec2` | Current position of the agent | world units |
| `velocity` | `Vec2` | Current velocity of the agent | world units/s |
| `radius` | `f32` | Radius of the agent | world units |

### Factor Graph Information

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `factor_graph_state` | `FactorGraphState` | State of the agent's factor graph | - |
| `factor_details` | `FactorDetails` | Detailed information about factor graph components | - |

See [Factor Graph](./factor_graph.md) for more information about the factor graph state and details.

### Connectivity Information

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `connected_neighbors` | `Vec<Entity>` | IDs of connected neighbor agents | - |
| `communication_active` | `bool` | Whether the agent's communication is active | - |
| `communication_radius` | `f32` | Radius of the agent's communication range | world units |

### Mission Information

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `mission_state` | `MissionState` | Current mission state | - |
| `planning_strategy` | `PlanningStrategy` | Strategy used for planning | - |
| `target_speed` | `f32` | Target speed of the agent | world units/s |
| `current_waypoint_index` | `Option<usize>` | Index of the current waypoint | - |
| `next_waypoint` | `Option<StateVectorInfo>` | Information about the next waypoint | - |
| `goal_point` | `Option<Vec2>` | Position of the goal point | world units |
| `mission_progress` | `MissionProgress` | Information about mission progress | - |

### Collision Information

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `collision_info` | `CollisionInfo` | Information about collisions | - |

See [Collision Information](./collision_info.md) for more information about collision detection and tracking.

## Mission State

The `MissionState` enum represents the current mission state of an agent:

```rust
pub enum MissionState {
    Idle { waiting_for_waypoints: bool },
    Active,
    Completed,
}
```

| Value | Description |
|-------|-------------|
| `Idle` | Agent is idle, possibly waiting for waypoints |
| `Active` | Agent is actively following a route |
| `Completed` | Agent has completed its mission |

## Planning Strategy

The `PlanningStrategy` enum represents the planning strategy used by an agent:

```rust
pub enum PlanningStrategy {
    OnlyLocal,
    RrtStar,
}
```

| Value | Description |
|-------|-------------|
| `OnlyLocal` | Agent uses only local planning |
| `RrtStar` | Agent uses RRT* for global planning |

## State Vector Information

The `StateVectorInfo` structure contains information about a state vector (position and velocity):

```rust
pub struct StateVectorInfo {
    pub position: Vec2,
    pub velocity: Vec2,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `position` | `Vec2` | Position component of the state vector | world units |
| `velocity` | `Vec2` | Velocity component of the state vector | world units/s |

## Mission Progress

The `MissionProgress` structure contains information about mission progress:

```rust
pub struct MissionProgress {
    pub started_at: f64,
    pub finished_at: Option<f64>,
    pub active_route: usize,
    pub total_routes: usize,
    pub total_waypoints: usize,
    pub remaining_waypoints: usize,
}
```

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `started_at` | `f64` | Time when the mission started | seconds |
| `finished_at` | `Option<f64>` | Time when the mission finished, if completed | seconds |
| `active_route` | `usize` | Index of the active route | - |
| `total_routes` | `usize` | Total number of routes | - |
| `total_waypoints` | `usize` | Total number of waypoints | - |
| `remaining_waypoints` | `usize` | Number of remaining waypoints | - |

## Serialization

When the `AgentState` is sent over the API, it is serialized into a JSON object with the following structure:

```json
{
  "agent_id": 1,
  "factorgraph_id": 1,
  "position": [10.0, 20.0],
  "velocity": [1.0, 0.0],
  "factor_graph_state": { ... },
  "connected_neighbors": [2, 3],
  "mission_state": {
    "type": "Active"
  },
  "planning_strategy": "OnlyLocal",
  "radius": 0.5,
  "communication_active": true,
  "communication_radius": 5.0,
  "target_speed": 1.0,
  "current_waypoint_index": 2,
  "next_waypoint": {
    "position": [15.0, 25.0],
    "velocity": [1.0, 0.0]
  },
  "goal_point": [50.0, 50.0],
  "mission_progress": { ... },
  "factor_details": { ... },
  "collision_info": { ... }
}
```

## Python Client

In the Python client, the `AgentState` is represented as a dictionary with the same fields as the JSON object. The `position`, `velocity`, and `goal_point` fields are converted to NumPy arrays for convenience.

```python
agent_state = client.get_agent_state()[agent_id]
position = agent_state["position"]  # numpy.ndarray
velocity = agent_state["velocity"]  # numpy.ndarray
```

## Implementation Details

The `AgentState` structure is populated in the `extract_state` function in [`extract.rs`](../../extract.rs), which extracts the state from the Bevy ECS components.

The serialization of the `AgentState` is handled in the `From` implementation in [`message.rs`](../../message.rs), which converts the `AgentState` to a `SerializedAgentState` for transmission over the API.

## Example Usage

Here's an example of how to access agent state information using the Python client:

```python
from magics_client import MagicsClient
import numpy as np

# Connect to the API
client = MagicsClient()

# Get agent states
agent_states = client.get_agent_state()

# Print information about each agent
for agent_id, agent in agent_states.items():
    print(f"Agent {agent_id}:")
    print(f"  Position: {agent['position']}")
    print(f"  Velocity: {agent['velocity']}")
    print(f"  Mission state: {agent['mission_state']['type']}")
    
    # Check if the agent has a goal
    if agent['goal_point'] is not None:
        print(f"  Goal: {agent['goal_point']}")
        
        # Calculate distance to goal
        distance = np.linalg.norm(agent['position'] - agent['goal_point'])
        print(f"  Distance to goal: {distance:.2f}")
    
    # Check if the agent has collided with anything
    collision_info = agent['collision_info']
    if collision_info['robot_collisions_delta'] > 0:
        print(f"  New robot collisions: {collision_info['robot_collisions_delta']}")
    if collision_info['environment_collisions_delta'] > 0:
        print(f"  New environment collisions: {collision_info['environment_collisions_delta']}")
```

## Related Data Structures

- [Factor Graph](./factor_graph.md): Information about the factor graph state and details
- [Collision Information](./collision_info.md): Information about collision detection and tracking
- [Environment State](./environment_state.md): Information about the environment
