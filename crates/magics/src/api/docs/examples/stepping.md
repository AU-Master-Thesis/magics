# Stepping Through the Simulation

This example demonstrates how to step through the simulation using the Magics API. Stepping allows you to advance the simulation one frame at a time, which is useful for reinforcement learning and other applications where you need fine-grained control over the simulation.

## Basic Stepping

The simplest way to step through the simulation is to use the `step` method of the `MagicsClient` class:

```python
from magics_client import MagicsClient

# Connect to the API
client = MagicsClient()

# Activate the API if it's not already active
if not client.is_api_active():
    client.set_api_active(True)

# Step the simulation
client.step()

# Get the updated agent states
agent_states = client.get_agent_state()
```

Each call to `step` advances the simulation by one frame. By default, each frame consists of 2 iterations of the simulation loop.

## Configuring Iterations Per Step

You can configure the number of iterations per step using the `set_iterations_per_step` method:

```python
# Set the number of iterations per step to 10
client.set_iterations_per_step(10)

# Step the simulation (will run 10 iterations)
client.step()
```

Increasing the number of iterations per step can be useful for making larger time steps, but it may also reduce the granularity of control.

## Stepping with a Loop

You can step through the simulation in a loop to observe the agents' behavior over time:

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

# Number of steps to run
num_steps = 20

# Initialize arrays to store agent positions
positions = {}

# Get initial agent states
agent_states = client.get_agent_state()
for agent_id in agent_states:
    positions[agent_id] = []

# Step through the simulation
for i in range(num_steps):
    print(f"Step {i+1}/{num_steps}")
    
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

# Plot agent trajectories
plt.figure(figsize=(10, 10))
for agent_id, pos in positions.items():
    plt.plot(pos[:, 0], pos[:, 1], '-o', label=f"Agent {agent_id}")
    plt.plot(pos[0, 0], pos[0, 1], 'go', markersize=10)  # Start position
    plt.plot(pos[-1, 0], pos[-1, 1], 'ro', markersize=10)  # End position

plt.grid(True)
plt.legend()
plt.title('Agent Trajectories')
plt.xlabel('X')
plt.ylabel('Y')
plt.axis('equal')
plt.show()
```

This example steps through the simulation 20 times, storing the position of each agent at each step. It then plots the trajectories of the agents.

## Stepping with Collision Detection

You can use the collision information to detect when agents collide with each other or with obstacles:

```python
from magics_client import MagicsClient

# Connect to the API
client = MagicsClient()

# Activate the API if it's not already active
if not client.is_api_active():
    client.set_api_active(True)

# Set the number of iterations per step
client.set_iterations_per_step(5)

# Number of steps to run
num_steps = 20

# Step through the simulation
for i in range(num_steps):
    print(f"Step {i+1}/{num_steps}")
    
    # Step the simulation
    client.step()
    
    # Get the updated agent states
    agent_states = client.get_agent_state()
    
    # Check for collisions
    for agent_id, agent in agent_states.items():
        collision_info = agent['collision_info']
        
        if collision_info['robot_collisions_delta'] > 0:
            print(f"Agent {agent_id} collided with another robot!")
        
        if collision_info['environment_collisions_delta'] > 0:
            print(f"Agent {agent_id} collided with the environment!")
```

This example steps through the simulation 20 times, checking for collisions at each step.

## Stepping with Goal Detection

You can use the mission state to detect when agents reach their goals:

```python
from magics_client import MagicsClient

# Connect to the API
client = MagicsClient()

# Activate the API if it's not already active
if not client.is_api_active():
    client.set_api_active(True)

# Set the number of iterations per step
client.set_iterations_per_step(5)

# Step until all agents reach their goals or a maximum number of steps is reached
max_steps = 100
step_count = 0

while step_count < max_steps:
    step_count += 1
    print(f"Step {step_count}/{max_steps}")
    
    # Step the simulation
    client.step()
    
    # Get the updated agent states
    agent_states = client.get_agent_state()
    
    # Check if all agents have reached their goals
    all_completed = True
    for agent_id, agent in agent_states.items():
        if agent['mission_state']['type'] != 'Completed':
            all_completed = False
            break
    
    if all_completed:
        print(f"All agents reached their goals after {step_count} steps!")
        break

if step_count >= max_steps:
    print(f"Maximum number of steps ({max_steps}) reached without all agents reaching their goals.")
```

This example steps through the simulation until all agents reach their goals or a maximum number of steps is reached.

## Stepping with Reinforcement Learning

Stepping is particularly useful for reinforcement learning applications, where you need to take actions and observe the results:

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

# Define reward parameters
collision_penalty = -10.0
goal_reward = 100.0
step_penalty = -0.1

# Define a simple policy (random actions)
def random_policy(agent_states):
    actions = {}
    for agent_id in agent_states:
        # Random weights between 0.5 and 1.5
        weights = {
            "dynamic": 0.5 + np.random.rand(),
            "obstacle": 0.5 + np.random.rand(),
            "interrobot": 0.5 + np.random.rand(),
            "tracking": 0.5 + np.random.rand()
        }
        actions[agent_id] = weights
    return actions

# Run an episode
max_steps = 100
step_count = 0
total_rewards = {}

# Get initial agent states
agent_states = client.get_agent_state()
for agent_id in agent_states:
    total_rewards[agent_id] = 0.0

while step_count < max_steps:
    step_count += 1
    print(f"Step {step_count}/{max_steps}")
    
    # Get the current agent states
    agent_states = client.get_agent_state()
    
    # Check if all agents have reached their goals
    all_completed = True
    for agent_id, agent in agent_states.items():
        if agent['mission_state']['type'] != 'Completed':
            all_completed = False
            break
    
    if all_completed:
        print(f"All agents reached their goals after {step_count} steps!")
        break
    
    # Choose actions based on the policy
    actions = random_policy(agent_states)
    
    # Apply actions (set factor weights for each agent)
    for agent_id, weights in actions.items():
        client.set_factor_weights(weights, agent_id)
    
    # Step the simulation
    client.step()
    
    # Get the updated agent states
    new_agent_states = client.get_agent_state()
    
    # Calculate rewards
    for agent_id, agent in new_agent_states.items():
        reward = step_penalty
        
        # Penalty for collisions
        collision_info = agent['collision_info']
        if collision_info['robot_collisions_delta'] > 0:
            reward += collision_penalty * collision_info['robot_collisions_delta']
        if collision_info['environment_collisions_delta'] > 0:
            reward += collision_penalty * collision_info['environment_collisions_delta']
        
        # Reward for reaching the goal
        if agent['mission_state']['type'] == 'Completed':
            reward += goal_reward
        
        total_rewards[agent_id] += reward
        print(f"Agent {agent_id} reward: {reward:.2f}, total: {total_rewards[agent_id]:.2f}")

print("Episode completed.")
for agent_id, reward in total_rewards.items():
    print(f"Agent {agent_id} total reward: {reward:.2f}")
```

This example demonstrates a simple reinforcement learning loop, where a random policy is used to choose actions (factor weights) for each agent. The agents are rewarded for reaching their goals and penalized for collisions.

## Simulation Hz

You can also control the simulation Hz (frequency) using the `set_simulation_hz` method:

```python
# Get the current simulation Hz
hz = client.get_simulation_hz()
print(f"Current simulation Hz: {hz}")

# Set the simulation Hz to 30
client.set_simulation_hz(30.0)
```

The simulation Hz determines the time step size for the simulation. A higher Hz means smaller time steps, which can lead to more accurate simulation but may also be slower.

## Implementation Details

The stepping mechanism is implemented in the following files:

- [`plugin.rs`](../../plugin.rs): Contains the `process_step_request`, `monitor_fixed_update`, and `complete_step_in_fixed_update` systems that handle stepping
- [`state.rs`](../../state.rs): Contains the `ApiState` resource that tracks step requests and iterations
- [`zmq_server.rs`](../../zmq_server.rs): Handles the `Step` command and waits for the step to complete

When a step is requested, the server unpauses the simulation for the specified number of iterations, then pauses it again and extracts the current state.

## Related Documentation

- [Commands Reference](../commands.md): Information about the `Step` command and other API commands
- [Agent State](../data_structures/agent_state.md): Information about the agent state structure
- [Environment State](../data_structures/environment_state.md): Information about the environment state structure
- [Collision Information](../data_structures/collision_info.md): Information about collision detection and tracking
