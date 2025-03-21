# Magics Python API (DEPRECATED)

**WARNING: This crate is deprecated and will be replaced by a ZeroMQ-based implementation. Please see the `python_api` directory for the new implementation.**

This crate provides Python bindings for the Magics simulation using PyO3. It allows controlling the simulation from Python, particularly for reinforcement learning applications.

## Building

To build the Python bindings, you need to have the following installed:
- Rust toolchain (1.78+)
- Python (3.8+)
- maturin

You can install maturin using pip:

```bash
pip install maturin
```

Then, to build the Python bindings:

```bash
cd crates/magics_python
export RUSTFLAGS="--cfg feature=\"api\""
maturin develop --cargo-extra-args="--features magics/api"
```

This will build the Python bindings and install them in your current Python environment.

## Using the API

The Python API provides the following functions:

- `get_agent_state()`: Get the state of all agents in the simulation.
- `get_environment_state()`: Get the state of the environment.
- `set_factor_weights(weights, agent_id=None)`: Set the factor graph weights.
- `step()`: Step the simulation forward by one frame.
- `reset()`: Reset the simulation.
- `is_api_active()`: Check if the API is active.
- `set_api_active(active)`: Set the API active state.

Example usage:

```python
import magics_api

# Activate the API
magics_api.set_api_active(True)

# Get the state of all agents
agent_states = magics_api.get_agent_state()

# Get the state of the environment
env_state = magics_api.get_environment_state()

# Set factor weights
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0,
}
magics_api.set_factor_weights(weights)

# Step the simulation
magics_api.step()

# Deactivate the API
magics_api.set_api_active(False)
```

## OpenAI Gym Environment

This crate also provides an OpenAI Gym environment for the Magics simulation. The environment is defined in the `magics_gym` package.

To install the Gym environment:

```bash
cd crates/magics_python/python
pip install -e .
```

Example usage:

```python
import magics_gym

# Create the environment
env = magics_gym.MagicsEnv()

# Reset the environment
observation = env.reset()

# Sample a random action
action = env.action_space.sample()

# Take a step in the environment
observation, reward, done, info = env.step(action)

# Close the environment
env.close()
```

See the `examples` directory for more examples.

## Requirements

- The simulation must be running with the API plugin enabled.
- The Python bindings must be built and installed in your Python environment.
- For the Gym environment, you also need to install the `magics_gym` package.
