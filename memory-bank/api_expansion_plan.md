# API Expansion Plan: Agent and Environment Information

This document outlines a comprehensive plan for expanding the information extracted from the Magics simulation for reinforcement learning purposes. It provides detailed implementation steps and pointers to relevant code locations.

## Overview

The goal is to enhance the API to expose more detailed information about agents and the environment, focusing on:

1. Factor graph variables and their states
2. Mission and planning information
3. Robot configuration details
4. Performance metrics
5. Obstacle and environment details

## Implementation Plan

### 1. Expand State Structures

#### 1.1 Enhance AgentState Structure

**File**: `crates/magics/src/api/state.rs`

```rust
pub struct AgentState {
    // Existing fields
    pub position: Vec2,
    pub velocity: Vec2,
    pub factor_graph_state: FactorGraphState,
    pub connected_neighbors: Vec<Entity>,
    
    // New fields
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
}

pub struct StateVectorInfo {
    pub position: Vec2,
    pub velocity: Vec2,
}

pub enum MissionState {
    Idle { waiting_for_waypoints: bool },
    Active,
    Completed,
}

pub struct MissionProgress {
    pub started_at: f64,
    pub finished_at: Option<f64>,
    pub active_route: usize,
    pub total_routes: usize,
    pub total_waypoints: usize,
    pub remaining_waypoints: usize,
}

pub struct FactorDetails {
    pub variables: Vec<VariableInfo>,
    pub obstacle_factors: Vec<ObstacleFactorInfo>,
    pub interrobot_factors: Vec<InterRobotFactorInfo>,
    pub tracking_factors: Vec<TrackingFactorInfo>,
    pub dynamic_factors: Vec<DynamicFactorInfo>,
}

pub struct VariableInfo {
    pub index: usize,
    pub mean: [f64; 4],  // [x, y, vx, vy]
    pub covariance: [f64; 16],  // 4x4 covariance matrix (flattened)
    pub estimated_position: [f32; 2],
    pub estimated_velocity: [f32; 2],
}

pub struct ObstacleFactorInfo {
    pub variable_index: usize,
    pub measurement: f64,  // SDF value
    pub position: [f32; 2],
}

pub struct InterRobotFactorInfo {
    pub variable_index: usize,
    pub external_robot_id: u32,
    pub external_variable_index: usize,
    pub safety_distance: f32,
    pub measurement: f64,
}

pub struct TrackingFactorInfo {
    pub variable_index: usize,
    pub tracking_path: Vec<[f32; 2]>,
    pub tracking_index: usize,
    pub path_deviation: f32,
}

pub struct DynamicFactorInfo {
    pub from_variable_index: usize,
    pub to_variable_index: usize,
    pub delta_t: f32,
}
```

#### 1.2 Enhance FactorGraphState Structure

**File**: `crates/magics/src/api/state.rs`

```rust
pub struct FactorGraphState {
    // Existing fields
    pub weights: FactorWeights,
    pub variable_count: usize,
    pub factor_count: usize,
    
    // New fields
    pub messages_sent: MessageStats,
    pub messages_received: MessageStats,
    pub factor_counts: FactorCounts,
}

pub struct MessageStats {
    pub internal: usize,
    pub external: usize,
}

pub struct FactorCounts {
    pub obstacle: usize,
    pub interrobot: usize,
    pub dynamic: usize,
    pub tracking: usize,
}
```

#### 1.3 Enhance EnvironmentState Structure

**File**: `crates/magics/src/api/state.rs`

```rust
pub struct EnvironmentState {
    // Existing fields
    pub obstacles: Vec<Vec2>,
    pub boundaries: (Vec2, Vec2),
    
    // New fields
    pub total_agents: usize,
    pub agent_density_map: Option<Vec<f32>>,  // Optional density map
    pub sdf_resolution: Option<(usize, usize)>,  // SDF image dimensions if available
    pub world_size: Option<(f64, f64)>,  // Width and height of the world
}
```

### 2. Update Message Protocol

#### 2.1 Update Serialized Structures

**File**: `crates/magics/src/api/message.rs`

```rust
pub struct SerializedAgentState {
    // Existing fields
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub factor_graph_state: SerializedFactorGraphState,
    pub connected_neighbors: Vec<u32>,
    
    // New fields
    pub mission_state: String,  // "Idle", "Active", "Completed"
    pub waiting_for_waypoints: Option<bool>,  // Only present if mission_state is "Idle"
    pub planning_strategy: String,  // "OnlyLocal", "RrtStar"
    pub radius: f32,
    pub communication_active: bool,
    pub communication_radius: f32,
    pub target_speed: f32,
    pub current_waypoint_index: Option<usize>,
    pub next_waypoint: Option<SerializedStateVector>,
    pub goal_point: Option<[f32; 2]>,
    pub mission_progress: SerializedMissionProgress,
    pub factor_details: SerializedFactorDetails,
}

pub struct SerializedStateVector {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

pub struct SerializedMissionProgress {
    pub started_at: f64,
    pub finished_at: Option<f64>,
    pub active_route: usize,
    pub total_routes: usize,
    pub total_waypoints: usize,
    pub remaining_waypoints: usize,
}

pub struct SerializedFactorGraphState {
    // Existing fields
    pub weights: FactorWeights,
    pub variable_count: usize,
    pub factor_count: usize,
    
    // New fields
    pub messages_sent: SerializedMessageStats,
    pub messages_received: SerializedMessageStats,
    pub factor_counts: SerializedFactorCounts,
}

pub struct SerializedMessageStats {
    pub internal: usize,
    pub external: usize,
}

pub struct SerializedFactorCounts {
    pub obstacle: usize,
    pub interrobot: usize,
    pub dynamic: usize,
    pub tracking: usize,
}

pub struct SerializedFactorDetails {
    pub variables: Vec<SerializedVariableInfo>,
    pub obstacle_factors: Vec<SerializedObstacleFactorInfo>,
    pub interrobot_factors: Vec<SerializedInterRobotFactorInfo>,
    pub tracking_factors: Vec<SerializedTrackingFactorInfo>,
    pub dynamic_factors: Vec<SerializedDynamicFactorInfo>,
}

pub struct SerializedVariableInfo {
    pub index: usize,
    pub mean: [f64; 4],
    pub covariance: [f64; 16],
    pub estimated_position: [f32; 2],
    pub estimated_velocity: [f32; 2],
}

pub struct SerializedObstacleFactorInfo {
    pub variable_index: usize,
    pub measurement: f64,
    pub position: [f32; 2],
}

pub struct SerializedInterRobotFactorInfo {
    pub variable_index: usize,
    pub external_robot_id: u32,
    pub external_variable_index: usize,
    pub safety_distance: f32,
    pub measurement: f64,
}

pub struct SerializedTrackingFactorInfo {
    pub variable_index: usize,
    pub tracking_path: Vec<[f32; 2]>,
    pub tracking_index: usize,
    pub path_deviation: f32,
}

pub struct SerializedDynamicFactorInfo {
    pub from_variable_index: usize,
    pub to_variable_index: usize,
    pub delta_t: f32,
}

pub struct SerializedEnvironmentState {
    // Existing fields
    pub obstacles: Vec<[f32; 2]>,
    pub boundaries: [[f32; 2]; 2],
    
    // New fields
    pub total_agents: usize,
    pub agent_density_map: Option<Vec<f32>>,
    pub sdf_resolution: Option<[usize; 2]>,
    pub world_size: Option<[f64; 2]>,
}
```

#### 2.2 Update From Implementations

**File**: `crates/magics/src/api/message.rs`

```rust
impl From<&crate::api::state::AgentState> for SerializedAgentState {
    fn from(state: &crate::api::state::AgentState) -> Self {
        // Implement conversion from internal AgentState to SerializedAgentState
        // Include all new fields
    }
}

impl From<&crate::api::state::EnvironmentState> for SerializedEnvironmentState {
    fn from(state: &crate::api::state::EnvironmentState) -> Self {
        // Implement conversion from internal EnvironmentState to SerializedEnvironmentState
        // Include all new fields
    }
}
```

### 3. Enhance State Extraction

#### 3.1 Update extract_state Function

**File**: `crates/magics/src/api/plugin.rs`

```rust
fn extract_state(
    api_state: Res<ApiState>,
    robots: Query<(
        Entity,
        &Transform,
        &StateVector,
        &FactorGraph,
        &RobotConnections,
        &Radius,
        &RadioAntenna,
        &Mission,
        &PlanningStrategy,
        &T0,
    )>,
    obstacles: Query<&Transform, With<ObstacleMarker>>,
    config: Res<Config>,
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
) {
    // Only extract state if API is active
    if !api_state.is_active() {
        return;
    }

    // Extract agent states
    if let Ok(mut agent_states) = api_state.agent_states.write() {
        agent_states.clear();

        for (entity, transform, state_vector, factor_graph, connections, radius, antenna, mission, planning_strategy, t0) in robots.iter() {
            let position = Vec2::new(transform.translation.x, transform.translation.z);
            let velocity = state_vector.velocity();

            // Extract factor graph state
            let factor_graph_state = extract_factor_graph_state(factor_graph, config.as_ref());

            // Extract connected neighbors
            let connected_neighbors = connections.robots_connected_with.iter().copied().collect();

            // Extract mission state
            let mission_state = mission.state;
            
            // Extract planning strategy
            let planning_strategy = *planning_strategy;
            
            // Extract radius
            let radius = radius.0;
            
            // Extract communication info
            let communication_active = antenna.active;
            let communication_radius = antenna.radius;
            
            // Extract target speed
            let target_speed = config.robot.target_speed.get();
            
            // Extract waypoint info
            let current_waypoint_index = mission.current_waypoint_index();
            let next_waypoint = mission.next_waypoint().map(|wp| StateVectorInfo {
                position: wp.position(),
                velocity: wp.velocity(),
            });
            let goal_point = mission.taskpoints.last().map(|wp| wp.position());
            
            // Extract mission progress
            let mission_progress = MissionProgress {
                started_at: mission.started_at(),
                finished_at: mission.finished_at(),
                active_route: mission.active_route,
                total_routes: mission.routes.len(),
                total_waypoints: mission.taskpoints.len(),
                remaining_waypoints: mission.taskpoints.len() - mission.active_route,
            };
            
            // Extract factor details
            let factor_details = extract_factor_details(factor_graph);

            agent_states.insert(entity, AgentState {
                position,
                velocity,
                factor_graph_state,
                connected_neighbors,
                mission_state,
                planning_strategy,
                radius,
                communication_active,
                communication_radius,
                target_speed,
                current_waypoint_index,
                next_waypoint,
                goal_point,
                mission_progress,
                factor_details,
            });
        }
    }

    // Extract environment state
    if let Ok(mut env_state) = api_state.environment_state.write() {
        let mut obstacle_positions = Vec::new();

        for transform in obstacles.iter() {
            obstacle_positions.push(Vec2::new(transform.translation.x, transform.translation.z));
        }

        // Get environment boundaries from config or use default
        let boundaries = (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));
        
        // Get total agents count
        let total_agents = robots.iter().count();
        
        // Get SDF resolution if available
        let sdf_resolution = None;  // This would need to be extracted from the environment config
        
        // Get world size if available
        let world_size = None;  // This would need to be extracted from the environment config

        *env_state = EnvironmentState {
            obstacles: obstacle_positions,
            boundaries,
            total_agents,
            agent_density_map: None,  // This would need to be calculated
            sdf_resolution,
            world_size,
        };
    }
}

fn extract_factor_graph_state(factor_graph: &FactorGraph, config: &Config) -> FactorGraphState {
    let weights = FactorWeights {
        dynamic: config.gbp.sigma_factor_dynamics as f32,
        obstacle: config.gbp.sigma_factor_obstacle as f32,
        interrobot: config.gbp.sigma_factor_interrobot as f32,
        tracking: config.gbp.sigma_factor_tracking as f32,
    };
    
    let node_count = factor_graph.node_count();
    let factor_count = factor_graph.factor_count();
    let messages_sent = factor_graph.messages_sent();
    let messages_received = factor_graph.messages_received();
    
    FactorGraphState {
        weights,
        variable_count: node_count.variables,
        factor_count: node_count.factors,
        messages_sent: MessageStats {
            internal: messages_sent.internal,
            external: messages_sent.external,
        },
        messages_received: MessageStats {
            internal: messages_received.internal,
            external: messages_received.external,
        },
        factor_counts: FactorCounts {
            obstacle: factor_count.obstacle,
            interrobot: factor_count.interrobot,
            dynamic: factor_count.dynamic,
            tracking: factor_count.tracking,
        },
    }
}

fn extract_factor_details(factor_graph: &FactorGraph) -> FactorDetails {
    let mut variables = Vec::new();
    let mut obstacle_factors = Vec::new();
    let mut interrobot_factors = Vec::new();
    let mut tracking_factors = Vec::new();
    let mut dynamic_factors = Vec::new();
    
    // Extract variable information
    for (index, variable) in factor_graph.variables() {
        let mean = variable.belief.mean.to_vec();
        let covariance = variable.belief.covariance.to_vec();
        let estimated_position = variable.estimated_position();
        let estimated_velocity = variable.estimated_velocity();
        
        variables.push(VariableInfo {
            index: index.into(),
            mean: [mean[0], mean[1], mean[2], mean[3]],
            covariance: covariance.try_into().unwrap_or([0.0; 16]),
            estimated_position: [estimated_position[0], estimated_position[1]],
            estimated_velocity: [estimated_velocity[0], estimated_velocity[1]],
        });
    }
    
    // Extract obstacle factor information
    for (index, factor) in factor_graph.factors() {
        if let Some(obstacle) = factor.kind.try_as_obstacle_ref() {
            let last_measurement = obstacle.last_measurement();
            
            obstacle_factors.push(ObstacleFactorInfo {
                variable_index: factor_graph.factor_neighbours(index.into())
                    .unwrap()
                    .next()
                    .map(|v| v.node_index().index())
                    .unwrap_or(0),
                measurement: last_measurement.value,
                position: [last_measurement.pos.x, last_measurement.pos.y],
            });
        } else if let Some(interrobot) = factor.kind.try_as_inter_robot_ref() {
            interrobot_factors.push(InterRobotFactorInfo {
                variable_index: factor_graph.factor_neighbours(index.into())
                    .unwrap()
                    .next()
                    .map(|v| v.node_index().index())
                    .unwrap_or(0),
                external_robot_id: interrobot.external_variable.factorgraph_id.index(),
                external_variable_index: interrobot.external_variable.variable_index.into(),
                safety_distance: interrobot.safety_radius,
                measurement: 0.0,  // This would need to be extracted from the factor
            });
        } else if let Some(tracking) = factor.kind.try_as_tracking_ref() {
            tracking_factors.push(TrackingFactorInfo {
                variable_index: factor_graph.factor_neighbours(index.into())
                    .unwrap()
                    .next()
                    .map(|v| v.node_index().index())
                    .unwrap_or(0),
                tracking_path: tracking.tracking_path()
                    .iter()
                    .map(|p| [p.x, p.y])
                    .collect(),
                tracking_index: tracking.tracking_index(),
                path_deviation: 0.0,  // This would need to be calculated
            });
        } else if let Some(dynamic) = factor.kind.try_as_dynamic_ref() {
            let neighbours: Vec<_> = factor_graph.factor_neighbours(index.into())
                .unwrap()
                .map(|v| v.node_index().index())
                .collect();
            
            if neighbours.len() >= 2 {
                dynamic_factors.push(DynamicFactorInfo {
                    from_variable_index: neighbours[0],
                    to_variable_index: neighbours[1],
                    delta_t: dynamic.delta_t(),
                });
            }
        }
    }
    
    FactorDetails {
        variables,
        obstacle_factors,
        interrobot_factors,
        tracking_factors,
        dynamic_factors,
    }
}
```

### 4. Update Python Client

#### 4.1 Update MagicsClient Class

**File**: `python_api/magics_client.py`

```python
class MagicsClient:
    # Existing methods...
    
    def get_agent_state(self) -> Dict[str, Dict[str, Any]]:
        """
        Get the state of all agents in the simulation.
        
        Returns:
            Dictionary mapping agent IDs to agent states with enhanced information
        """
        data = self._send_request("GetAgentState")
        
        # Check data format
        if not data:
            raise MagicsError("No data returned from get_agent_state")
        
        if "type" not in data or data["type"] != "AgentStates":
            raise MagicsError(f"Invalid response format for get_agent_state: {data}")
        
        # Process agent states
        result = {}
        for agent_id, state in data["content"].items():
            # Convert positions and velocities to numpy arrays
            if "position" in state:
                state["position"] = np.array(state["position"], dtype=np.float32)
            if "velocity" in state:
                state["velocity"] = np.array(state["velocity"], dtype=np.float32)
            
            # Convert next_waypoint if present
            if "next_waypoint" in state and state["next_waypoint"]:
                state["next_waypoint"]["position"] = np.array(state["next_waypoint"]["position"], dtype=np.float32)
                state["next_waypoint"]["velocity"] = np.array(state["next_waypoint"]["velocity"], dtype=np.float32)
            
            # Convert goal_point if present
            if "goal_point" in state and state["goal_point"]:
                state["goal_point"] = np.array(state["goal_point"], dtype=np.float32)
            
            # Process factor details
            if "factor_details" in state:
                fd = state["factor_details"]
                
                # Process variables
                if "variables" in fd:
                    for var in fd["variables"]:
                        var["mean"] = np.array(var["mean"], dtype=np.float64)
                        var["covariance"] = np.array(var["covariance"], dtype=np.float64).reshape(4, 4)
                        var["estimated_position"] = np.array(var["estimated_position"], dtype=np.float32)
                        var["estimated_velocity"] = np.array(var["estimated_velocity"], dtype=np.float32)
                
                # Process obstacle factors
                if "obstacle_factors" in fd:
                    for factor in fd["obstacle_factors"]:
                        factor["position"] = np.array(factor["position"], dtype=np.float32)
                
                # Process tracking factors
                if "tracking_factors" in fd:
                    for factor in fd["tracking_factors"]:
                        factor["tracking_path"] = np.array(factor["tracking_path"], dtype=np.float32)
            
            result[agent_id] = state
        
        return result
    
    def get_environment_state(self) -> Dict[str, Any]:
        """
        Get the state of the environment.
        
        Returns:
            Dictionary containing enhanced environment state
        """
        data = self._send_request("GetEnvironmentState")
        
        # Check data format
        if not data:
            raise MagicsError("No data returned from get_environment_state")
        
        if "type" not in data or data["type"] != "EnvironmentState":
            raise MagicsError(f"Invalid response format for get_environment_state: {data}")
        
        # Process environment state
        result = data["content"]
        
        # Convert obstacles to numpy arrays
        if "obstacles" in result:
            result["obstacles"] = np.array(result["obstacles"], dtype=np.float32)
        
        # Convert boundaries to numpy arrays
        if "boundaries" in result:
            result["boundaries"] = {
                "min": np.array(result["boundaries"][0], dtype=np.float32),
                "max": np.array(result["boundaries"][1], dtype=np.float32)
            }
        
        # Convert agent_density_map if present
        if "agent_density_map" in result and result["agent_density_map"]:
            result["agent_density_map"] = np.array(result["agent_density_map"], dtype=np.float32)
        
        # Convert sdf_resolution if present
        if "sdf_resolution" in result and result["sdf_resolution"]:
            result["sdf_resolution"] = np.array(result["sdf_resolution"], dtype=np.int32)
        
        # Convert world_size if present
        if "world_size" in result and result["world_size"]:
            result["world_size"] = np.array(result["world_size"], dtype=np.float64)
        
        return result
    
    # Helper methods for accessing specific information
    
    def get_agent_factor_graph_variables(self, agent_id: str) -> List[Dict[str, Any]]:
        """
        Get the factor graph variables for a specific agent.
        
        Args:
            agent_id: ID of the agent
            
        Returns:
            List of variable information dictionaries
        """
        agent_states = self.get_agent_state()
        if agent_id not in agent_states:
            raise MagicsError(f"Agent {agent_id} not found")
        
        if "factor_details" not in agent_states[agent_id] or "variables" not in agent_states[agent_id]["factor_details"]:
            raise MagicsError(f"Factor graph variables not available for agent {agent_id}")
        
        return agent_states[agent_id]["factor_details"]["variables"]
    
    def get_agent_obstacle_factors(self, agent_id: str) -> List[Dict[str, Any]]:
        """
        Get the obstacle factors for a specific agent.
        
        Args:
            agent_id: ID of the agent
            
        Returns:
            List of obstacle factor information dictionaries
        """
        agent_states = self.get_agent_state()
        if agent_id not in agent_states:
            raise MagicsError(f"Agent {agent_id} not found")
        
        if "factor_details" not in agent_states[agent_id] or "obstacle_factors" not in agent_states[agent_id]["factor_details"]:
            raise MagicsError(f"Obstacle factors not available for agent {agent_id}")
        
        return agent_states[agent_id]["factor_details"]["obstacle_factors"]
    
    def get_agent_interrobot_factors(self, agent_id: str) -> List[Dict[str, Any]]:
        """
        Get the inter-robot factors for a specific agent.
        
        Args:
            agent_id: ID of the agent
            
        Returns:
            List of inter-robot factor information dictionaries
        """
        agent_states = self.get_agent_state()
        if agent_id not in agent_states:
            raise MagicsError(f"Agent {agent_id} not found")
        
        if "factor_details" not in agent_states[agent_id] or "interrobot_factors" not in agent_states[agent_id]["factor_details"]:
            raise MagicsError(f"Inter-robot factors not available for agent {agent_id}")
        
        return agent_states[agent_id]["factor_details"]["interrobot_factors"]
    
    def get_agent_tracking_factors(self, agent_id: str) -> List[Dict[str, Any]]:
        """
        Get the tracking factors for a specific agent.
        
        Args:
            agent_id: ID of the agent
            
        Returns:
            List of tracking factor information dictionaries
        """
        agent_states = self.get_agent_state()
        if agent_id not in agent_states:
            raise MagicsError(f"Agent {agent_id} not found")
        
        if "factor_details" not in agent_states[agent_id] or "tracking_factors" not in agent_states[agent_id]["factor_details"]:
            raise MagicsError(f"Tracking factors not available for agent {agent_id}")
        
        return agent_states[agent_id]["factor_details"]["tracking_factors"]
    
    def get_agent_dynamic_factors(self, agent_id: str) -> List[Dict[str, Any]]:
        """
        Get the dynamic factors for a specific agent.
        
        Args:
            agent_id: ID of the agent
            
        Returns:
            List of dynamic factor information dictionaries
        """
        agent_states = self.get_agent_state()
        if agent_id not in agent_states:
            raise MagicsError(f"Agent {agent_id} not found")
        
        if "factor_details" not in agent_states[agent_id] or "dynamic_factors" not in agent_states[agent_id]["factor_details"]:
            raise MagicsError(f"Dynamic factors not available for agent {agent_id}")
        
        return agent_states[agent_id]["factor_details"]["dynamic_factors"]
    
    def get_agent_mission_info(self, agent_id: str) -> Dict[str, Any]:
        """
        Get the mission information for a specific agent.
        
        Args:
            agent_id: ID of the agent
            
        Returns:
            Dictionary containing mission information
        """
        agent_states = self.get_agent_state()
        if agent_id not in agent_states:
            raise MagicsError(f"Agent {agent_id} not found")
        
        return {
            "mission_state": agent_states[agent_id].get("mission_state"),
            "waiting_for_waypoints": agent_states[agent_id].get("waiting_for_waypoints"),
            "planning_strategy": agent_states[agent_id].get("planning_strategy"),
            "current_waypoint_index": agent_states[agent_id].get("current_waypoint_index"),
            "next_waypoint": agent_states[agent_id].get("next_waypoint"),
            "goal_point": agent_states[agent_id].get("goal_point"),
            "mission_progress": agent_states[agent_id].get("mission_progress"),
        }
```

### 5. Update Test Script

#### 5.1 Update test_zmq_requests.py

**File**: `python_api/test_zmq_requests.py`

```python
def test_get_agent_state(socket):
    """Test the GetAgentState command with enhanced information."""
    print("\n=== Testing GetAgentState with Enhanced Information ===")
    response = send_request(socket, "GetAgentState")
    
    if response:
        if response.get("status") == "Success":
            print("✅ Command succeeded")
            
            data = response.get("data")
            if data and data.get("type") == "AgentStates":
                agents = data.get("content", {})
                print(f"Received data for {len(agents)} agents")
                
                # Print some agent details if available
                for agent_id, state in list(agents.items())[:1]:  # Show first agent in detail
                    print(f"Agent {agent_id}:")
                    if "position" in state:
                        print(f"  Position: {state['position']}")
                    if "velocity" in state:
                        print(f"  Velocity: {state['velocity']}")
                    if "mission_state" in state:
                        print(f"  Mission State: {state['mission_state']}")
                    if "planning_strategy" in state:
                        print(f"  Planning Strategy: {state['planning_strategy']}")
                    if "radius" in state:
                        print(f"  Radius: {state['radius']}")
                    if "communication_active" in state:
                        print(f"  Communication Active: {state['communication_active']}")
                    if "communication_radius" in state:
                        print(f"  Communication Radius: {state['communication_radius']}")
                    if "target_speed" in state:
                        print(f"  Target Speed: {state['target_speed']}")
                    ...



