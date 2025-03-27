# Collision Information

The `CollisionInfo` structure contains information about collisions for an agent in the simulation. This document provides detailed information about all fields in the `CollisionInfo` structure, including their meaning and how they are tracked.

## Structure Overview

The `CollisionInfo` structure is defined in [`state.rs`](../../state.rs) and contains the following fields:

```rust
pub struct CollisionInfo {
    pub robot_collisions_total: usize,
    pub robot_collisions_delta: usize,
    pub environment_collisions_total: usize,
    pub environment_collisions_delta: usize,
}
```

## Field Descriptions

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `robot_collisions_total` | `usize` | Total number of collisions with other robots | - |
| `robot_collisions_delta` | `usize` | Change in robot collisions since last state extraction | - |
| `environment_collisions_total` | `usize` | Total number of collisions with the environment | - |
| `environment_collisions_delta` | `usize` | Change in environment collisions since last state extraction | - |

### Robot Collisions

Robot collisions occur when an agent collides with another agent. The `robot_collisions_total` field contains the total number of robot collisions that have occurred for the agent since the simulation started.

The `robot_collisions_delta` field contains the number of new robot collisions that have occurred since the last time the state was extracted. This can be useful for detecting when a collision has just occurred.

### Environment Collisions

Environment collisions occur when an agent collides with an obstacle in the environment. The `environment_collisions_total` field contains the total number of environment collisions that have occurred for the agent since the simulation started.

The `environment_collisions_delta` field contains the number of new environment collisions that have occurred since the last time the state was extracted. This can be useful for detecting when a collision has just occurred.

## Implementation Details

Collision information is tracked in the `RobotRobotCollisions` and `RobotEnvironmentCollisions` resources in the Bevy ECS. These resources are updated by the collision detection systems in the simulation.

The `CollisionInfo` structure is populated in the `extract_state` function in [`extract.rs`](../../extract.rs), which extracts the collision information from these resources.

The `PreviousCollisionCounts` resource in [`plugin.rs`](../../plugin.rs) is used to track the previous collision counts for each agent, which allows the calculation of the delta values.

## Example Usage

Here's an example of how to access collision information using the Python client:

```python
from magics_client import MagicsClient

# Connect to the API
client = MagicsClient()

# Get agent states
agent_states = client.get_agent_state()

# Print collision information for each agent
for agent_id, agent in agent_states.items():
    collision_info = agent['collision_info']
    
    print(f"Agent {agent_id} Collisions:")
    print(f"  Robot collisions: {collision_info['robot_collisions_total']} (delta: {collision_info['robot_collisions_delta']})")
    print(f"  Environment collisions: {collision_info['environment_collisions_total']} (delta: {collision_info['environment_collisions_delta']})")
    
    # Check if the agent has collided with anything in the current step
    if collision_info['robot_collisions_delta'] > 0 or collision_info['environment_collisions_delta'] > 0:
        print(f"  Agent {agent_id} has collided!")
```

## Using Collision Information for Reinforcement Learning

Collision information can be particularly useful for reinforcement learning applications, where you might want to penalize agents for collisions. Here's an example of how you might use collision information to calculate a reward:

```python
from magics_client import MagicsClient
import numpy as np

# Connect to the API
client = MagicsClient()

# Set up reward parameters
collision_penalty = -10.0  # Penalty for collisions
goal_reward = 100.0        # Reward for reaching the goal
step_penalty = -0.1        # Small penalty for each step to encourage efficiency

# Step the simulation and calculate rewards
client.step()

# Get agent states
agent_states = client.get_agent_state()

# Calculate rewards for each agent
rewards = {}
for agent_id, agent in agent_states.items():
    # Start with the step penalty
    reward = step_penalty
    
    # Check for collisions
    collision_info = agent['collision_info']
    if collision_info['robot_collisions_delta'] > 0:
        reward += collision_penalty * collision_info['robot_collisions_delta']
    if collision_info['environment_collisions_delta'] > 0:
        reward += collision_penalty * collision_info['environment_collisions_delta']
    
    # Check if the agent has reached its goal
    if agent['mission_state']['type'] == 'Completed':
        reward += goal_reward
    
    rewards[agent_id] = reward
    print(f"Agent {agent_id} reward: {reward}")
```

## Serialization

When the `CollisionInfo` is sent over the API, it is serialized into a JSON object with the following structure:

```json
{
  "robot_collisions_total": 5,
  "robot_collisions_delta": 1,
  "environment_collisions_total": 3,
  "environment_collisions_delta": 0
}
```

## Python Client

In the Python client, the `CollisionInfo` is represented as a dictionary with the same fields as the JSON object:

```python
collision_info = agent_state["collision_info"]
robot_collisions_total = collision_info["robot_collisions_total"]
robot_collisions_delta = collision_info["robot_collisions_delta"]
environment_collisions_total = collision_info["environment_collisions_total"]
environment_collisions_delta = collision_info["environment_collisions_delta"]
```

## Related Data Structures

- [Agent State](./agent_state.md): Information about an agent in the simulation
- [Environment State](./environment_state.md): Information about the environment
