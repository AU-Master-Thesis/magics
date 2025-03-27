# Modifying Factor Weights

This example demonstrates how to modify factor weights using the Magics API. Factor weights determine the influence of different factor types on the overall solution, allowing you to control the behavior of agents in the simulation.

## Factor Weight Basics

The factor graph in Magics uses several types of factors, each with an associated weight:

- **Dynamic Factors**: Model the dynamics of an agent, connecting variables at different time steps
- **Obstacle Factors**: Model the interaction between an agent and obstacles in the environment
- **Inter-Robot Factors**: Model the interaction between different agents
- **Tracking Factors**: Model the tracking of a predefined path

The weights of these factors determine their influence on the overall solution. Higher weights give more importance to the corresponding factor type.

## Setting System-Wide Weights

You can set system-wide weights that apply to all agents using the `set_factor_weights` method:

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
```

This sets the weights for all agents in the simulation. The weights are normalized, so the relative values are what matter.

## Setting Per-Agent Weights

You can also set weights for a specific agent using the `agent_id` parameter:

```python
from magics_client import MagicsClient

# Connect to the API
client = MagicsClient()

# Set factor weights for a specific agent
agent_id = 1
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0
}
client.set_factor_weights(weights, agent_id)
```

**Note**: Per-agent weights are currently not fully implemented in the API. The API structure exists, but the implementation is incomplete. This feature is planned for future updates.

## Weight Effects

The weights of different factor types have different effects on the behavior of agents:

- **Dynamic Weight**: Controls how closely the agent follows its predicted trajectory. Higher values make the agent more likely to follow a smooth, physically plausible trajectory.
- **Obstacle Weight**: Controls how strongly the agent avoids obstacles. Higher values make the agent more likely to avoid obstacles, even if it means deviating from its trajectory.
- **Inter-Robot Weight**: Controls how strongly the agent avoids other agents. Higher values make the agent more likely to avoid collisions with other agents, even if it means deviating from its trajectory.
- **Tracking Weight**: Controls how closely the agent follows its predefined path. Higher values make the agent more likely to stay on the path, even if it means getting closer to obstacles or other agents.

## Experimenting with Weights

You can experiment with different weight configurations to see how they affect the behavior of agents:

```python
from magics_client import MagicsClient
import time
import numpy as np
import matplotlib.pyplot as plt

# Connect to the API
client = MagicsClient()

# Activate the API if it's not already active
if not client.is_api_active():
    client.set_api_active(True)

# Set the number of iterations per step
client.set_iterations_per_step(5)

# Define different weight configurations to test
weight_configs = [
    {
        "name": "Balanced",
        "weights": {
            "dynamic": 1.0,
            "obstacle": 1.0,
            "interrobot": 1.0,
            "tracking": 1.0
        }
    },
    {
        "name": "Obstacle Avoidance",
        "weights": {
            "dynamic": 1.0,
            "obstacle": 2.0,
            "interrobot": 1.0,
            "tracking": 0.5
        }
    },
    {
        "name": "Path Following",
        "weights": {
            "dynamic": 1.0,
            "obstacle": 0.5,
            "interrobot": 0.5,
            "tracking": 2.0
        }
    },
    {
        "name": "Inter-Robot Avoidance",
        "weights": {
            "dynamic": 1.0,
            "obstacle": 0.5,
            "interrobot": 2.0,
            "tracking": 0.5
        }
    }
]

# Number of steps to run for each configuration
num_steps = 20

# Initialize arrays to store agent positions for each configuration
all_positions = {}

# Run the simulation with each weight configuration
for config in weight_configs:
    print(f"Testing weight configuration: {config['name']}")
    
    # Reset the simulation (not fully implemented yet)
    # client.reset()
    
    # Set the weights
    client.set_factor_weights(config['weights'])
    
    # Initialize arrays to store agent positions
    positions = {}
    
    # Get initial agent states
    agent_states = client.get_agent_state()
    for agent_id in agent_states:
        positions[agent_id] = []
    
    # Step through the simulation
    for i in range(num_steps):
        print(f"  Step {i+1}/{num_steps}")
        
        # Step the simulation
        client.step()
        
        # Get the updated agent states
        agent_states = client.get_agent_state()
        
        # Store agent positions
        for agent_id, agent in agent_states.items():
            positions[agent_id].append(agent['position'])
    
    # Convert positions to numpy arrays
    for agent_id in positions:
        positions[agent_id] = np.array(positions[agent_id])
    
    # Store positions for this configuration
    all_positions[config['name']] = positions

# Plot agent trajectories for each configuration
fig, axs = plt.subplots(2, 2, figsize=(15, 15))
axs = axs.flatten()

for i, config in enumerate(weight_configs):
    ax = axs[i]
    positions = all_positions[config['name']]
    
    for agent_id, pos in positions.items():
        ax.plot(pos[:, 0], pos[:, 1], '-o', label=f"Agent {agent_id}")
        ax.plot(pos[0, 0], pos[0, 1], 'go', markersize=10)  # Start position
        ax.plot(pos[-1, 0], pos[-1, 1], 'ro', markersize=10)  # End position
    
    ax.grid(True)
    ax.set_title(f"{config['name']} Weights")
    ax.set_xlabel('X')
    ax.set_ylabel('Y')
    ax.axis('equal')

plt.tight_layout()
plt.show()
```

This example tests four different weight configurations and plots the resulting agent trajectories for each configuration.

## Adaptive Weight Adjustment

You can also implement adaptive weight adjustment based on the agent's state:

```python
from magics_client import MagicsClient
import numpy as np

# Connect to the API
client = MagicsClient()

# Activate the API if it's not already active
if not client.is_api_active():
    client.set_api_active(True)

# Set the number of iterations per step
client.set_iterations_per_step(5)

# Number of steps to run
num_steps = 50

# Step through the simulation with adaptive weights
for i in range(num_steps):
    print(f"Step {i+1}/{num_steps}")
    
    # Get the current agent states
    agent_states = client.get_agent_state()
    
    # Adjust weights for each agent based on its state
    for agent_id, agent in agent_states.items():
        # Get the agent's position and velocity
        position = agent['position']
        velocity = agent['velocity']
        
        # Get collision information
        collision_info = agent['collision_info']
        
        # Get factor graph state
        factor_graph_state = agent['factor_graph_state']
        
        # Default weights
        weights = {
            "dynamic": 1.0,
            "obstacle": 1.0,
            "interrobot": 1.0,
            "tracking": 1.0
        }
        
        # Adjust obstacle weight based on recent collisions
        if collision_info['environment_collisions_delta'] > 0:
            # Increase obstacle weight if the agent has recently collided with an obstacle
            weights['obstacle'] = 2.0
        
        # Adjust interrobot weight based on recent collisions
        if collision_info['robot_collisions_delta'] > 0:
            # Increase interrobot weight if the agent has recently collided with another robot
            weights['interrobot'] = 2.0
        
        # Adjust tracking weight based on distance to goal
        if agent['goal_point'] is not None:
            # Calculate distance to goal
            goal = np.array(agent['goal_point'])
            distance_to_goal = np.linalg.norm(position - goal)
            
            # Increase tracking weight as the agent gets closer to the goal
            if distance_to_goal < 10.0:
                weights['tracking'] = 1.5
        
        # Set the adjusted weights for this agent
        client.set_factor_weights(weights, agent_id)
    
    # Step the simulation
    client.step()
```

This example adjusts the weights for each agent based on its current state, such as recent collisions and distance to the goal.

## Implementation Details

The weight update functionality is implemented in the following files:

- [`weights.rs`](../../weights.rs): Contains the `apply_weight_updates` system that applies weight updates to factor graphs
- [`state.rs`](../../state.rs): Contains the `WeightUpdate` structure that represents a weight update request
- [`zmq_server.rs`](../../zmq_server.rs): Handles the `SetFactorWeights` command and adds weight update requests to the API state

Currently, only system-wide weight updates are fully implemented. Per-agent weight updates are planned for future updates.

## Related Documentation

- [Commands Reference](../commands.md): Information about the `SetFactorWeights` command and other API commands
- [Factor Graph](../data_structures/factor_graph.md): Information about the factor graph state and details
- [Agent State](../data_structures/agent_state.md): Information about the agent state structure
