# Troubleshooting

This document provides guidance on common issues and solutions when using the Magics API.

## Connection Issues

### Cannot Connect to the API

**Symptoms**: The client fails to connect to the API with a connection refused error.

**Possible Causes**:
- The Magics simulation is not running
- The API feature is not enabled in the simulation
- The API server is not listening on the expected port
- A firewall is blocking the connection

**Solutions**:
1. Ensure the Magics simulation is running with the API feature enabled
2. Check that the API server is listening on the expected port (default: 5555)
3. Verify that no firewall is blocking the connection
4. Try connecting to a different port if the default port is in use

```python
# Connect to a different port
client = MagicsClient(port=5556)
```

### Connection Timeout

**Symptoms**: The client times out when trying to connect to the API.

**Possible Causes**:
- The Magics simulation is running but the API server is not responding
- The network is slow or unreliable
- The timeout value is too low

**Solutions**:
1. Increase the timeout value
2. Check that the API server is running and responsive
3. Try connecting to a different port if the default port is in use

```python
# Increase the timeout to 10 seconds
client = MagicsClient(timeout=10000)
```

## API Activation Issues

### API Not Active

**Symptoms**: The client can connect to the API, but commands fail with an "API not active" error.

**Possible Causes**:
- The API is not activated in the simulation
- The API was deactivated by another client

**Solutions**:
1. Activate the API using the `set_api_active` method
2. Check that no other client is deactivating the API

```python
# Activate the API
client.set_api_active(True)
```

### API Activation Fails

**Symptoms**: The `set_api_active` method fails with an error.

**Possible Causes**:
- The API server is not running
- The API server is not responding
- The API server is in an inconsistent state

**Solutions**:
1. Restart the Magics simulation
2. Check that the API server is running and responsive
3. Try connecting to a different port if the default port is in use

## Stepping Issues

### Step Command Hangs

**Symptoms**: The `step` method hangs and does not return.

**Possible Causes**:
- The simulation is paused
- The simulation is in an inconsistent state
- The API server is not responding

**Solutions**:
1. Check that the simulation is not paused
2. Restart the Magics simulation
3. Increase the timeout value

```python
# Increase the timeout to 10 seconds
client = MagicsClient(timeout=10000)
```

### Step Command Times Out

**Symptoms**: The `step` method times out with a timeout error.

**Possible Causes**:
- The simulation is taking too long to complete the step
- The number of iterations per step is too high
- The simulation is in an inconsistent state

**Solutions**:
1. Decrease the number of iterations per step
2. Increase the timeout value
3. Restart the Magics simulation

```python
# Decrease the number of iterations per step
client.set_iterations_per_step(1)

# Increase the timeout to 10 seconds
client = MagicsClient(timeout=10000)
```

### Step Command Fails

**Symptoms**: The `step` method fails with an error.

**Possible Causes**:
- The API is not active
- The simulation is in an inconsistent state
- The API server is not responding

**Solutions**:
1. Activate the API using the `set_api_active` method
2. Restart the Magics simulation
3. Check that the API server is running and responsive

## State Extraction Issues

### Agent States Not Available

**Symptoms**: The `get_agent_state` method returns an empty dictionary.

**Possible Causes**:
- There are no agents in the simulation
- The API is not active
- The simulation is in an inconsistent state

**Solutions**:
1. Check that there are agents in the simulation
2. Activate the API using the `set_api_active` method
3. Restart the Magics simulation

### Environment State Not Available

**Symptoms**: The `get_environment_state` method fails with an error.

**Possible Causes**:
- The API is not active
- The simulation is in an inconsistent state
- The API server is not responding

**Solutions**:
1. Activate the API using the `set_api_active` method
2. Restart the Magics simulation
3. Check that the API server is running and responsive

## Weight Update Issues

### Weight Updates Not Applied

**Symptoms**: Weight updates using the `set_factor_weights` method do not affect the simulation.

**Possible Causes**:
- The API is not active
- The simulation is in an inconsistent state
- The weights are not being applied correctly

**Solutions**:
1. Activate the API using the `set_api_active` method
2. Restart the Magics simulation
3. Check that the weights are being set correctly

```python
# Set weights for all agents
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0
}
client.set_factor_weights(weights)
```

### Per-Agent Weight Updates Not Working

**Symptoms**: Per-agent weight updates using the `set_factor_weights` method with an `agent_id` parameter do not affect the simulation.

**Possible Causes**:
- Per-agent weight updates are not fully implemented yet
- The agent ID is invalid
- The API is not active

**Solutions**:
1. Use system-wide weight updates instead
2. Check that the agent ID is valid
3. Activate the API using the `set_api_active` method

```python
# Set weights for all agents
weights = {
    "dynamic": 1.0,
    "obstacle": 1.0,
    "interrobot": 1.0,
    "tracking": 1.0
}
client.set_factor_weights(weights)
```

## Simulation Hz Issues

### Simulation Hz Not Applied

**Symptoms**: Changes to the simulation Hz using the `set_simulation_hz` method do not affect the simulation.

**Possible Causes**:
- The API is not active
- The simulation is in an inconsistent state
- The Hz value is invalid

**Solutions**:
1. Activate the API using the `set_api_active` method
2. Restart the Magics simulation
3. Check that the Hz value is valid (must be greater than 0)

```python
# Set the simulation Hz to 30
client.set_simulation_hz(30.0)
```

### Simulation Hz Too Low

**Symptoms**: The simulation runs too slowly.

**Possible Causes**:
- The simulation Hz is set too low
- The simulation is running on a slow computer
- The simulation is complex and requires more computational resources

**Solutions**:
1. Increase the simulation Hz
2. Simplify the simulation
3. Run the simulation on a faster computer

```python
# Set the simulation Hz to 60
client.set_simulation_hz(60.0)
```

### Simulation Hz Too High

**Symptoms**: The simulation runs too quickly or becomes unstable.

**Possible Causes**:
- The simulation Hz is set too high
- The simulation is complex and requires more computational resources
- The simulation is running on a slow computer

**Solutions**:
1. Decrease the simulation Hz
2. Simplify the simulation
3. Run the simulation on a faster computer

```python
# Set the simulation Hz to 30
client.set_simulation_hz(30.0)
```

## Client-Side Issues

### NumPy Array Conversion

**Symptoms**: NumPy array conversion fails with an error.

**Possible Causes**:
- NumPy is not installed
- The data is not in the expected format
- The data is missing or invalid

**Solutions**:
1. Install NumPy
2. Check that the data is in the expected format
3. Check that the data is not missing or invalid

```bash
pip install numpy
```

### Matplotlib Plotting

**Symptoms**: Matplotlib plotting fails with an error.

**Possible Causes**:
- Matplotlib is not installed
- The data is not in the expected format
- The data is missing or invalid

**Solutions**:
1. Install Matplotlib
2. Check that the data is in the expected format
3. Check that the data is not missing or invalid

```bash
pip install matplotlib
```

## Server-Side Issues

### API Server Crashes

**Symptoms**: The API server crashes or becomes unresponsive.

**Possible Causes**:
- The simulation is in an inconsistent state
- The API server is running out of memory
- The API server is encountering an unhandled exception

**Solutions**:
1. Restart the Magics simulation
2. Check the simulation logs for error messages
3. Reduce the complexity of the simulation

### API Server Performance

**Symptoms**: The API server is slow to respond.

**Possible Causes**:
- The simulation is complex and requires more computational resources
- The API server is handling too many requests
- The API server is running on a slow computer

**Solutions**:
1. Simplify the simulation
2. Reduce the frequency of API requests
3. Run the simulation on a faster computer

## Implementation Details

The API client is implemented in [`magics_client.py`](../../../../python_api/magics_client.py), which provides a convenient interface for interacting with the API.

The server-side implementation is in the following files:

- [`zmq_server.rs`](../zmq_server.rs): Handles incoming requests and dispatches them to the appropriate handler
- [`message.rs`](../message.rs): Defines the message protocol and serialization/deserialization
- [`state.rs`](../state.rs): Manages the API state and provides methods for accessing and modifying it
- [`extract.rs`](../extract.rs): Extracts state information from the simulation
- [`factor_details.rs`](../factor_details.rs): Extracts detailed information about factor graphs
- [`weights.rs`](../weights.rs): Handles updates to factor weights
- [`reset.rs`](../reset.rs): Handles resetting the API state
- [`plugin.rs`](../plugin.rs): Integrates the API with the Bevy application

## Related Documentation

- [Commands Reference](./commands.md): Information about all available API commands
- [Getting Started](./getting_started.md): Quick start guide for using the API
- [Core Concepts](./core_concepts.md): Explanation of key concepts in the API
- [Examples](./examples/): Examples of using the API for various tasks
