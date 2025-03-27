# Magics API Documentation

This document provides detailed information about the Magics API, including the meaning and units of all values exposed through the API.

## Overview

The Magics API provides access to the state of the simulation, including agent states, factor graph details, and environment information. The API is designed to be used with the Python client library, which provides a simple interface for interacting with the simulation.

## Documentation Structure

For more detailed documentation, please see the [docs](./docs/) directory, which contains the following:

- [README](./docs/README.md) - Overview and navigation
- [Getting Started](./docs/getting_started.md) - Quick start guide
- [Core Concepts](./docs/core_concepts.md) - Explanation of key concepts
- [Commands Reference](./docs/commands.md) - Detailed information about available commands
- [Troubleshooting](./docs/troubleshooting.md) - Common issues and solutions

### Data Structures

- [Agent State](./docs/data_structures/agent_state.md) - Information about agent state representation
- [Environment State](./docs/data_structures/environment_state.md) - Information about environment state representation
- [Factor Graph](./docs/data_structures/factor_graph.md) - Information about factor graph representation
- [Collision Information](./docs/data_structures/collision_info.md) - Information about collision detection and tracking

### Examples

- [Basic Usage](./docs/examples/basic_usage.md) - Basic examples of using the API
- [Stepping](./docs/examples/stepping.md) - Examples of stepping through the simulation
- [Weights](./docs/examples/weights.md) - Examples of modifying factor weights

## Agent State

The `AgentState` object contains information about a single agent in the simulation.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `agent_id` | `int` | Unique identifier for the agent | - |
| `factorgraph_id` | `int` | Identifier for the agent's factor graph (same as agent_id) | - |
| `position` | `Vec2` | Current position of the agent | world units |
| `velocity` | `Vec2` | Current velocity of the agent | world units/s |
| `factor_graph_state` | `FactorGraphState` | State of the agent's factor graph | - |
| `connected_neighbors` | `List[int]` | IDs of connected neighbor agents | - |
| `mission_state` | `MissionState` | Current mission state (Idle, Active, Completed) | - |
| `planning_strategy` | `PlanningStrategy` | Strategy used for planning (OnlyLocal, RrtStar) | - |
| `radius` | `float` | Radius of the agent | world units |
| `communication_active` | `bool` | Whether the agent's communication is active | - |
| `communication_radius` | `float` | Radius of the agent's communication range | world units |
| `target_speed` | `float` | Target speed of the agent | world units/s |
| `current_waypoint_index` | `Optional[int]` | Index of the current waypoint | - |
| `next_waypoint` | `Optional[StateVectorInfo]` | Information about the next waypoint | - |
| `goal_point` | `Optional[Vec2]` | Position of the goal point | world units |
| `mission_progress` | `MissionProgress` | Information about mission progress | - |
| `factor_details` | `FactorDetails` | Detailed information about factor graph components | - |

## Factor Graph State

The `FactorGraphState` object contains information about the state of a factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `weights` | `FactorWeights` | Current weights of the factor graph | - |
| `variable_count` | `int` | Number of variables in the factor graph | - |
| `factor_count` | `int` | Number of factors in the factor graph | - |
| `messages_sent` | `MessageStats` | Statistics about messages sent from the factor graph | - |
| `messages_received` | `MessageStats` | Statistics about messages received by the factor graph | - |
| `factor_counts` | `FactorCounts` | Counts of different factor types | - |
| `factor_details` | `FactorDetails` | Detailed information about factor graph components | - |

## Factor Weights

The `FactorWeights` object contains the weights for different factor types in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `dynamic` | `float` | Weight for dynamic factors | - |
| `obstacle` | `float` | Weight for obstacle factors | - |
| `interrobot` | `float` | Weight for inter-robot factors | - |
| `tracking` | `float` | Weight for tracking factors | - |

## Message Statistics

The `MessageStats` object contains statistics about messages in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `internal` | `int` | Number of internal messages | - |
| `external` | `int` | Number of external messages | - |

## Factor Counts

The `FactorCounts` object contains counts of different factor types in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `obstacle` | `int` | Number of obstacle factors | - |
| `interrobot` | `int` | Number of inter-robot factors | - |
| `dynamic` | `int` | Number of dynamic factors | - |
| `tracking` | `int` | Number of tracking factors | - |

## Factor Details

The `FactorDetails` object contains detailed information about factor graph components.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variables` | `List[VariableInfo]` | Information about variables in the factor graph | - |
| `obstacle_factors` | `List[ObstacleFactorInfo]` | Information about obstacle factors | - |
| `interrobot_factors` | `List[InterRobotFactorInfo]` | Information about inter-robot factors | - |
| `tracking_factors` | `List[TrackingFactorInfo]` | Information about tracking factors | - |
| `dynamic_factors` | `List[DynamicFactorInfo]` | Information about dynamic factors | - |

## Variable Information

The `VariableInfo` object contains information about a variable in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `index` | `int` | Index of the variable | - |
| `factorgraph_id` | `int` | ID of the factor graph this variable belongs to | - |
| `mean` | `List[float]` | Mean vector of the variable [x, y, vx, vy] | world units, world units/s |
| `covariance` | `List[float]` | Covariance matrix of the variable (4x4, flattened) | - |
| `estimated_position` | `List[float]` | Estimated position from the variable [x, y] | world units |
| `estimated_velocity` | `List[float]` | Estimated velocity from the variable [vx, vy] | world units/s |

## Obstacle Factor Information

The `ObstacleFactorInfo` object contains information about an obstacle factor in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variable_index` | `int` | Index of the variable this factor is connected to | - |
| `sdf_value` | `float` | SDF value at the position, ranges from 0.0 (free space) to 1.0 (obstacle) | - |
| `position` | `List[float]` | Position where the SDF value was measured [x, y] | world units |

## Inter-Robot Factor Information

The `InterRobotFactorInfo` object contains information about an inter-robot factor in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variable_index` | `int` | Index of the variable this factor is connected to | - |
| `external_robot_id` | `int` | ID of the external robot this factor connects to | - |
| `external_factorgraph_id` | `int` | ID of the external factor graph | - |
| `external_variable_index` | `int` | Index of the variable in the external robot's factor graph | - |
| `safety_distance` | `float` | Safety distance for collision avoidance | world units |
| `distance_between_variables` | `float` | Current distance between the two variables | world units |
| `active` | `bool` | Whether the factor is active | - |

## Tracking Factor Information

The `TrackingFactorInfo` object contains information about a tracking factor in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `variable_index` | `int` | Index of the variable this factor is connected to | - |
| `tracking_path` | `List[List[float]]` | Path that the robot is tracking, list of [x, y] points | world units |
| `tracking_index` | `int` | Current index in the tracking path | - |
| `projected_position` | `List[float]` | Projected position on the path [x, y] | world units |
| `path_deviation` | `float` | Path deviation measurement (normalized distance) | 0.0-1.0 |
| `distance_to_path` | `float` | Distance from robot to projected point on path | world units |

## Dynamic Factor Information

The `DynamicFactorInfo` object contains information about a dynamic factor in the factor graph.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `from_variable_index` | `int` | Index of the source variable | - |
| `to_variable_index` | `int` | Index of the destination variable | - |
| `delta_t` | `float` | Time step between the variables | seconds |

## Environment State

The `EnvironmentState` object contains information about the environment in the simulation.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `obstacles` | `List[Vec2]` | Positions of obstacles in the environment | world units |
| `boundaries` | `Tuple[Vec2, Vec2]` | Boundaries of the environment (min, max) | world units |
| `total_agents` | `int` | Total number of agents in the environment | - |
| `agent_density_map` | `Optional[List[float]]` | Optional agent density map | - |
| `sdf_resolution` | `Optional[Tuple[int, int]]` | Optional SDF resolution (width, height) | pixels |
| `world_size` | `Optional[Tuple[float, float]]` | Optional world size (width, height) | world units |

## Mission Progress

The `MissionProgress` object contains information about mission progress.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `started_at` | `float` | Time when the mission started | seconds |
| `finished_at` | `Optional[float]` | Time when the mission finished, if completed | seconds |
| `active_route` | `int` | Index of the active route | - |
| `total_routes` | `int` | Total number of routes | - |
| `total_waypoints` | `int` | Total number of waypoints | - |
| `remaining_waypoints` | `int` | Number of remaining waypoints | - |

## State Vector Information

The `StateVectorInfo` object contains information about a state vector (position and velocity).

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `position` | `Vec2` | Position component of the state vector | world units |
| `velocity` | `Vec2` | Velocity component of the state vector | world units/s |

## Mission State

The `MissionState` enum represents the current mission state of an agent.

| Value | Description |
|-------|-------------|
| `Idle` | Agent is idle, possibly waiting for waypoints |
| `Active` | Agent is actively following a route |
| `Completed` | Agent has completed its mission |

## Planning Strategy

The `PlanningStrategy` enum represents the planning strategy used by an agent.

| Value | Description |
|-------|-------------|
| `OnlyLocal` | Agent uses only local planning |
| `RrtStar` | Agent uses RRT* for global planning |

## Collision Information

The `CollisionInfo` object contains information about collisions for an agent.

| Field | Type | Description | Units |
|-------|------|-------------|-------|
| `robot_collisions_total` | `int` | Total number of collisions with other robots | - |
| `robot_collisions_delta` | `int` | Change in robot collisions since last state extraction | - |
| `environment_collisions_total` | `int` | Total number of collisions with the environment | - |
| `environment_collisions_delta` | `int` | Change in environment collisions since last state extraction | - |

The `robot_collisions_delta` and `environment_collisions_delta` fields are particularly useful for detecting when a collision has just occurred, as they represent the change in collision counts since the last state extraction.
