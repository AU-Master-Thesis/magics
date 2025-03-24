"""
Magics Gym Environment

This module provides an OpenAI Gym environment for the Magics simulation.
"""

import gymnasium as gym
import numpy as np
from gymnasium import spaces
import sys
import os

# Add parent directory to path to import magics_client
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from magics_client import MagicsClient, MagicsError


class MagicsEnv(gym.Env):
    """
    OpenAI Gym environment for the Magics simulation.
    
    This environment implements the OpenAI Gym interface to control the
    Magics simulation through the ZeroMQ API.
    """
    
    metadata = {"render_modes": ["human", "rgb_array"]}
    
    def __init__(self, host="localhost", port=5555, render_mode=None):
        """
        Initialize the environment.
        
        Args:
            host: Hostname or IP address of the Magics server
            port: Port number of the Magics server
            render_mode: Rendering mode, either "human" or "rgb_array"
        """
        super().__init__()
        
        # Connect to the Magics server
        self.client = MagicsClient(host=host, port=port)
        
        # Activate the API
        try:
            # Check if API is already active
            if not self.client.is_api_active():
                self.client.set_api_active(True)
                print("API activated successfully")
        except MagicsError as e:
            print(f"Error checking API status: {e}")
            # Try to activate anyway
            self.client.set_api_active(True)
            print("API activation attempted")
        
        # Initialize state caches
        self.agent_states = {}
        self.environment_state = {}
        
        # Get initial states to determine observation space
        self._update_state()
        
        # Define observation space based on the structure of the states
        self._define_observation_space()
        
        # Define action space as factor graph weights
        # Each action is a set of weights for the factor graph:
        # [dynamic, obstacle, interrobot, tracking]
        self.action_space = spaces.Box(
            low=0.1,
            high=10.0,
            shape=(4,),
            dtype=np.float32
        )
        
        # Store render mode
        self.render_mode = render_mode
        
        # Episode tracking
        self.steps = 0
        self.max_steps = 1000  # Maximum steps per episode
    
    def _define_observation_space(self):
        """Define observation space based on the state structure."""
        # Calculate the number of agents and obstacles
        num_agents = len(self.agent_states)
        num_obstacles = len(self.environment_state.get("obstacles", []))
        
        # Define observation space as a dictionary of:
        # - Agent positions and velocities
        # - Obstacle positions
        # - Boundaries
        agent_space = spaces.Dict({
            "positions": spaces.Box(
                low=-np.inf, high=np.inf, shape=(num_agents, 2), dtype=np.float32
            ),
            "velocities": spaces.Box(
                low=-np.inf, high=np.inf, shape=(num_agents, 2), dtype=np.float32
            ),
            "connectivity": spaces.Box(
                low=0, high=1, shape=(num_agents, num_agents), dtype=np.int32
            ),
            # Add planning strategy (0=OnlyLocal, 1=RrtStar)
            "planning_strategies": spaces.MultiDiscrete([2] * num_agents),
            # Add mission state (0=Idle, 1=Active, 2=Completed)
            "mission_states": spaces.MultiDiscrete([3] * num_agents),
            # Add waiting_for_waypoints flag for Idle state
            "waiting_for_waypoints": spaces.MultiBinary(num_agents)
        })
        
        obstacle_space = spaces.Box(
            low=-np.inf, high=np.inf, shape=(num_obstacles, 2), dtype=np.float32
        )
        
        boundary_space = spaces.Box(
            low=-np.inf, high=np.inf, shape=(2, 2), dtype=np.float32
        )
        
        self.observation_space = spaces.Dict({
            "agents": agent_space,
            "obstacles": obstacle_space,
            "boundaries": boundary_space
        })
    
    def _update_state(self):
        """Update the internal state from the simulation."""
        self.agent_states = self.client.get_agent_state()
        self.environment_state = self.client.get_environment_state()
    
    def _get_observation(self):
        """
        Convert the current state to an observation.
        
        Returns:
            Dictionary containing the observation
        """
        # Get agent information
        agent_ids = sorted(self.agent_states.keys())
        num_agents = len(agent_ids)
        
        # Initialize arrays
        positions = np.zeros((num_agents, 2), dtype=np.float32)
        velocities = np.zeros((num_agents, 2), dtype=np.float32)
        connectivity = np.zeros((num_agents, num_agents), dtype=np.int32)
        planning_strategies = np.zeros(num_agents, dtype=np.int32)
        mission_states = np.zeros(num_agents, dtype=np.int32)
        waiting_for_waypoints = np.zeros(num_agents, dtype=np.int32)
        
        # Fill arrays with agent data
        for i, agent_id in enumerate(agent_ids):
            agent_state = self.agent_states[agent_id]
            positions[i] = agent_state["position"]
            velocities[i] = agent_state["velocity"]
            
            # Set connectivity matrix
            if "connected_neighbors" in agent_state:
                for neighbor_id in agent_state["connected_neighbors"]:
                    if neighbor_id in agent_ids:
                        j = agent_ids.index(neighbor_id)
                        connectivity[i, j] = 1
            
            # Process planning strategy
            if "planning_strategy" in agent_state:
                from magics_client import PlanningStrategy
                strategy = agent_state["planning_strategy"]
                if isinstance(strategy, str):
                    # Handle string representation
                    if strategy == "RrtStar":
                        planning_strategies[i] = 1
                else:
                    # Handle enum representation
                    if strategy == PlanningStrategy.RRT_STAR:
                        planning_strategies[i] = 1
            
            # Process mission state
            if "mission_state" in agent_state:
                from magics_client import MissionState
                mission_state = agent_state["mission_state"]
                
                if isinstance(mission_state, dict):
                    # Handle structured mission state
                    if "type" in mission_state:
                        mission_type = mission_state["type"]
                        if mission_type == MissionState.ACTIVE:
                            mission_states[i] = 1
                        elif mission_type == MissionState.COMPLETED:
                            mission_states[i] = 2
                        elif mission_type == MissionState.IDLE:
                            mission_states[i] = 0
                            # Check if waiting for waypoints
                            if mission_state.get("waiting_for_waypoints", False):
                                waiting_for_waypoints[i] = 1
                    elif "Idle" in mission_state:
                        mission_states[i] = 0
                        # Check if waiting for waypoints
                        if mission_state["Idle"].get("waiting_for_waypoints", False):
                            waiting_for_waypoints[i] = 1
                elif isinstance(mission_state, str):
                    # Handle string representation
                    if mission_state == "Active":
                        mission_states[i] = 1
                    elif mission_state == "Completed":
                        mission_states[i] = 2
        
        # Get obstacle and boundary information
        obstacles = self.environment_state.get("obstacles", np.zeros((0, 2), dtype=np.float32))
        boundaries = np.array([
            self.environment_state.get("boundaries", {}).get("min", [-100.0, -100.0]),
            self.environment_state.get("boundaries", {}).get("max", [100.0, 100.0])
        ], dtype=np.float32)
        
        # Return observation dictionary
        return {
            "agents": {
                "positions": positions,
                "velocities": velocities,
                "connectivity": connectivity,
                "planning_strategies": planning_strategies,
                "mission_states": mission_states,
                "waiting_for_waypoints": waiting_for_waypoints
            },
            "obstacles": obstacles,
            "boundaries": boundaries
        }
    
    def _compute_reward(self):
        """
        Compute the reward based on the current state.
        
        The reward is computed based on:
        - Distance to target locations
        - Collisions (negative reward)
        - Success reaching target (positive reward)
        
        Returns:
            Scalar reward value
        """
        # This is a simple example reward function
        # In a real application, this would be much more sophisticated
        
        # For now, we'll just use a simple reward function based on agent progress
        # Positive reward for movement, negative reward for collisions
        
        # Get current positions and velocities
        positions = np.array([state["position"] for state in self.agent_states.values()])
        velocities = np.array([state["velocity"] for state in self.agent_states.values()])
        
        # Calculate movement reward: reward for agents moving toward their goals
        # In this simple version, we just reward overall speed
        movement_reward = np.mean(np.linalg.norm(velocities, axis=1))
        
        # Calculate collision penalty
        collision_penalty = 0.0
        
        # Simple collision detection between agents
        if len(positions) > 1:
            # Calculate distances between agents
            dists = np.linalg.norm(positions[:, None, :] - positions[None, :, :], axis=2)
            
            # Set diagonal to a large value to ignore self-distances
            np.fill_diagonal(dists, 1000.0)
            
            # Count collisions (agents too close to each other)
            collisions = np.sum(dists < 1.0)
            collision_penalty = -10.0 * collisions
        
        # Calculate obstacle avoidance reward
        obstacle_penalty = 0.0
        if "obstacles" in self.environment_state and len(self.environment_state["obstacles"]) > 0:
            obstacles = self.environment_state["obstacles"]
            
            # Calculate minimum distance to obstacles for each agent
            for pos in positions:
                dists_to_obstacles = np.linalg.norm(obstacles - pos, axis=1)
                min_dist = np.min(dists_to_obstacles)
                
                # Penalize being too close to obstacles
                if min_dist < 1.0:
                    obstacle_penalty -= 5.0 * (1.0 - min_dist)
        
        # Combine rewards and penalties
        reward = movement_reward + collision_penalty + obstacle_penalty
        
        return reward
    
    def _is_done(self):
        """
        Check if the episode is done.
        
        The episode is done if:
        - Maximum steps have been reached
        - All agents have reached their targets
        - A collision has occurred
        
        Returns:
            Boolean indicating if the episode is done
        """
        # Check if maximum steps have been reached
        if self.steps >= self.max_steps:
            return True
        
        # In a real application, we would also check for:
        # - All agents reaching their targets
        # - Major collisions
        
        return False
    
    def step(self, action):
        """
        Take a step in the environment.
        
        Args:
            action: Action to take, as an array of 4 weights:
                   [dynamic, obstacle, interrobot, tracking]
        
        Returns:
            observation: The current observation
            reward: The reward for the action
            terminated: Whether the episode is terminated
            truncated: Whether the episode was truncated
            info: Additional information
        """
        # Convert action to weights
        weights = {
            "dynamic": float(action[0]),
            "obstacle": float(action[1]),
            "interrobot": float(action[2]),
            "tracking": float(action[3])
        }
        
        # Apply weights to the simulation
        self.client.set_factor_weights(weights)
        
        # Step the simulation
        self.client.step()
        
        # Update internal state
        self._update_state()
        
        # Get observation
        observation = self._get_observation()
        
        # Compute reward
        reward = self._compute_reward()
        
        # Check if done
        self.steps += 1
        terminated = self._is_done()
        truncated = False
        
        # Additional info
        info = {}
        
        return observation, reward, terminated, truncated, info
    
    def reset(self, seed=None, options=None):
        """
        Reset the environment.
        
        Args:
            seed: Random seed
            options: Additional options
        
        Returns:
            observation: The initial observation
            info: Additional information
        """
        # Reset RNG if seed is provided
        if seed is not None:
            np.random.seed(seed)
        
        # Reset the simulation
        self.client.reset()
        
        # Reset step counter
        self.steps = 0
        
        # Update internal state
        self._update_state()
        
        # Get initial observation
        observation = self._get_observation()
        
        # Additional info
        info = {}
        
        return observation, info
    
    def render(self):
        """
        Render the environment.
        
        Returns:
            If render_mode is "rgb_array", an RGB array of the scene is returned.
            If render_mode is "human", nothing is returned and the environment is rendered to a window.
        """
        # The simulation is already being rendered by the Bevy application
        # This method could be extended to capture screenshots from the simulation
        if self.render_mode == "rgb_array":
            # In a real implementation, you would capture a screenshot from the simulation
            # For now, we'll just return a blank image
            return np.zeros((600, 800, 3), dtype=np.uint8)
        return None
    
    def close(self):
        """
        Close the environment.
        """
        # Deactivate the API
        self.client.set_api_active(False)
        
        # Close the client
        self.client.close()
