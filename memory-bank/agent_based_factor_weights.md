# Agent-Based Factor Weights Implementation

## Overview

This document outlines the implementation of per-agent factor weights in the Magics simulation. The goal is to allow individual agents to have different factor weights, rather than system-wide weights that apply to all agents. This enables more flexible and tailored behavior for each agent, and potentially better performance in heterogeneous multi-agent systems.

## Current State

We have successfully implemented the necessary components for supporting per-agent factor weights:

1. Added a `factor_weights` getter method in `FactorGraph` to access the current weights
2. Added an `update_factor_weights` method in `FactorGraph` to update weights and propagate changes
3. Added helper methods to update specific factor types (dynamic, obstacle, interrobot, tracking)
4. Updated the UI code to use the getter method when displaying weights
5. The API state contains a `weight_requests` field that stores pending weight updates
6. The `apply_weight_updates` system processes these weight updates and applies them to agents
7. Created a Python example script to demonstrate using the per-agent weight setting API

## Implementation Details

### FactorGraph Methods

The `FactorGraph` struct now has these key methods:

```rust
// Get current factor weights
pub fn factor_weights(&self) -> &crate::api::state::FactorWeights;

// Update factor weights and propagate changes to existing factors
pub fn update_factor_weights(&mut self, weights: crate::api::state::FactorWeights);

// Helper methods for specific factor types
fn update_dynamic_factor_weights(&mut self, weight: f32);
fn update_obstacle_factor_weights(&mut self, weight: f32);
fn update_interrobot_factor_weights(&mut self, weight: f32);
fn update_tracking_factor_weights(&mut self, weight: f32);
```

### API Integration

The API weight update system:

```rust
pub fn apply_weight_updates(
    api_state: Res<ApiState>,
    mut robots: Query<(Entity, &mut FactorGraph)>,
    mut config: ResMut<Config>,
) {
    // ... code that processes weight updates from API ...
    
    // System-wide update
    // or
    // Agent-specific update using factor_graph.update_factor_weights(update.weights)
}
```

The Python client has a method to set factor weights:

```python
def set_factor_weights(
    self, weights: Dict[str, float], agent_id: Optional[int] = None
) -> None:
    """
    Set factor graph weights.

    Args:
        weights: Dictionary mapping factor names to weights
        agent_id: Optional agent ID for per-agent weights
    """
    self._send_request("SetFactorWeights", weights=weights, agent_id=agent_id)
```

## Usage Example

We've created a Python example script (`agent_weights_example.py`) that demonstrates how to:

1. Connect to the Magics simulation
2. Get current agent states and factor weights
3. Set new weights for a specific agent or system-wide
4. Verify the weight changes were applied

This script serves as a practical demonstration of the per-agent weight setting capabilities.

## Remaining Work

While the core implementation is complete, there are a few potential enhancements:

1. **UI Improvements**: Add UI controls for setting per-agent weights in the simulation interface
2. **Weight Visualization**: Visual indicators showing different agent weights in the simulation
3. **Automatic Tuning**: Mechanisms to automatically adjust weights based on agent performance
4. **Presets and Profiles**: Create preset weight profiles for different agent behaviors
5. **Persistence**: Save and load agent-specific weight configurations

## Benefits

The per-agent factor weights provide several advantages:

1. **Heterogeneous Agent Behavior**: Agents can have different navigation styles and priorities
2. **Role-Based Optimization**: Specialized weights for agents with different roles
3. **Dynamic Adaptation**: Weights can be adjusted in response to changing conditions
4. **Testing and Experimentation**: Easier comparison of different weight configurations
5. **Reinforcement Learning Integration**: Better support for learned agent-specific policies

## Testing

The implementation can be tested using:

1. The provided Python example script
2. The UI controls for system-wide weight changes
3. Direct API calls in custom scripts or applications
