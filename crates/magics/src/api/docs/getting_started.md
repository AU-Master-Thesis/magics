# Getting Started with the Magics API

This guide will help you get started with using the Magics API to interact with the Magics simulation.

## Prerequisites

- Magics simulation running with the API enabled
- Python 3.7 or later
- ZeroMQ Python bindings (`pyzmq`)
- NumPy

You can install the required Python packages using pip:

```bash
pip install pyzmq numpy
```

## Installation

The Magics Python client is located in the `python_api` directory. You can install it using pip:

```bash
cd python_api
pip install -e .
```

This will install the `magics_client` package in development mode, allowing you to make changes to the client code without reinstalling.

## Basic Usage

### Connecting to the API

To connect to the Magics API, you need to create a `MagicsClient` instance:

```python
from magics_client import MagicsClient

# Connect to the API (default: localhost:5555)
client = MagicsClient()

# Or specify a custom host and port
# client = MagicsClient(host="localhost", port=5555)
```

### Activating the API

By default, the API may not be active. You can check and activate it:

```python
# Check if the API is active
is_active = client.is_api_active()
print(f"API active: {is_active}")

# Activate the API if it's not already active
if not is_active:
    client.set_api_active(True)
```

### Getting Agent States

You can get the state of all agents in the simulation:

```python
# Get agent states
agent_states = client.get_agent_state()

# Print the number of agents
print(f"Number of agents: {len(agent_states)}")

# Print the position of the first agent
first_agent_id = list(agent_states.keys())[0]
first_agent = agent_states[first_agent_id]
print(f"Agent {first_agent_id} position: {first_agent['position']}")
```

### Getting Environment State

You can get the state of the environment:

```python
# Get environment state
env_state = client.get_environment_state()

# Print the number of obstacles
print(f"Number of obstacles: {len(env_state['obstacles'])}")

# Print the environment boundaries
print(f"Environment boundaries: {env_state['boundaries']}")
```

### Stepping the Simulation

You can step the simulation forward:

```python
# Set the number of iterations per step
client.set_iterations_per_step(10)

# Step the simulation
client.step()

# Get the updated agent states
agent_states = client.get_agent_state()
```

### Setting Factor Weights

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

# Set factor weights for a specific agent
agent_id = 1
client.set_factor_weights(weights, agent_id)
```

### Closing the Connection

When you're done, you should close the connection:

```python
client.close()
```

## Complete Example

Here's a complete example that connects to the API, gets agent and environment states, steps the simulation, and closes the connection:

```python
from magics_client import MagicsClient
import time

# Connect to the API
client = MagicsClient()

try:
    # Activate the API if it's not already active
    if not client.is_api_active():
        client.set_api_active(True)
        print("API activated")
    
    # Set the number of iterations per step
    client.set_iterations_per_step(10)
    
    # Get initial agent states
    agent_states = client.get_agent_state()
    print(f"Number of agents: {len(agent_states)}")
    
    # Get environment state
    env_state = client.get_environment_state()
    print(f"Number of obstacles: {len(env_state['obstacles'])}")
    
    # Step the simulation
    print("Stepping simulation...")
    client.step()
    
    # Get updated agent states
    agent_states = client.get_agent_state()
    
    # Print positions of all agents
    for agent_id, agent in agent_states.items():
        print(f"Agent {agent_id} position: {agent['position']}")
    
finally:
    # Close the connection
    client.close()
    print("Connection closed")
```

## Next Steps

Now that you know the basics of using the Magics API, you can:

- Learn about the [core concepts](./core_concepts.md) of the API
- Explore the [commands reference](./commands.md) for detailed information about available commands
- Check out the [examples](./examples/) for more advanced usage patterns
- Learn about the [data structures](./data_structures/) used in the API

For more information about specific aspects of the API, see the relevant documentation pages.
