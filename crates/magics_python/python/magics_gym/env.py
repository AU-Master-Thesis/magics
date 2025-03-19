"""
OpenAI Gym environment for the Magics simulation.

This module provides an OpenAI Gym environment for the Magics simulation,
allowing reinforcement learning algorithms to interact with the simulation.
"""

import os
import sys
# Add the virtual environment path to sys.path
venv_path = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 
                         '.venv/lib/python3.10/site-packages')
sys.path.insert(0, venv_path)

import gym
import numpy as np
from gym import spaces
from typing import Dict, List, Optional, Tuple, Union, Any

try:
    import magics_api
except ImportError:
    raise ImportError(
        "Could not import magics_api. Make sure the magics_api module is installed "
        "and the simulation is running with the API plugin enabled."
    )


class MagicsEnv(gym.Env):
    """
    OpenAI Gym environment for the Magics simulation.

    This environment allows reinforcement learning algorithms to interact with
    the Magics simulation, controlling the factor graph weights to optimize
    path planning.

    Attributes:
        action_space: The space of possible actions.
        observation_space: The space of possible observations.
        metadata: Metadata for the environment.
    """

    metadata = {"render.modes": ["human"]}

    def __init__(self, max_episode_steps: int = 1000):
        """
        Initialize the Magics environment.

        Args:
            max_episode_steps: Maximum number of steps per episode.
        """
        super().__init__()

        # Activate the API
        magics_api.set_api_active(True)

        # Define action space (factor weights)
        # [dynamic, obstacle, interrobot, tracking]
        self.action_space = spaces.Box(
            low=np.array([0.1, 0.1, 0.1, 0.1]),
            high=np.array([10.0, 10.0, 10.0, 10.0]),
            dtype=np.float32,
        )

        # Define observation space
        # This is a Dict space with:
        # - agents: Dict of agent states
        # - environment: Dict of environment state
        self.observation_space = spaces.Dict(
            {
                "agents": spaces.Dict(
                    {
                        "positions": spaces.Box(
                            low=-np.inf, high=np.inf, shape=(100, 2), dtype=np.float32
                        ),
                        "velocities": spaces.Box(
                            low=-np.inf, high=np.inf, shape=(100, 2), dtype=np.float32
                        ),
                        "neighbors": spaces.Box(
                            low=0, high=100, shape=(100, 100), dtype=np.int32
                        ),
                    }
                ),
                "environment": spaces.Dict(
                    {
                        "obstacles": spaces.Box(
                            low=-np.inf, high=np.inf, shape=(100, 2), dtype=np.float32
                        ),
                        "boundaries": spaces.Box(
                            low=-np.inf, high=np.inf, shape=(2, 2), dtype=np.float32
                        ),
                    }
                ),
            }
        )

        self.max_episode_steps = max_episode_steps
        self.current_step = 0

    def step(self, action: np.ndarray) -> Tuple[Dict, float, bool, Dict]:
        """
        Take a step in the environment.

        Args:
            action: The action to take, which is a numpy array of factor weights.

        Returns:
            A tuple of (observation, reward, done, info).
        """
        # Set factor weights
        weights = {
            "dynamic": float(action[0]),
            "obstacle": float(action[1]),
            "interrobot": float(action[2]),
            "tracking": float(action[3]),
        }
        magics_api.set_factor_weights(weights, None)

        # Step the simulation
        magics_api.step()

        # Get the new state
        observation = self._get_observation()

        # Calculate reward
        reward = self._calculate_reward(observation)

        # Check if done
        self.current_step += 1
        done = self.current_step >= self.max_episode_steps

        # Additional info
        info = {}

        return observation, reward, done, info

    def reset(self) -> Dict:
        """
        Reset the environment.

        Returns:
            The initial observation.
        """
        # Reset the simulation
        # Note: This is a placeholder. The actual reset functionality
        # will depend on how the simulation is designed to be reset.
        magics_api.reset()

        # Reset step counter
        self.current_step = 0

        # Get the initial observation
        observation = self._get_observation()

        return observation

    def render(self, mode: str = "human") -> None:
        """
        Render the environment.

        Args:
            mode: The rendering mode.
        """
        # The simulation is already rendering, so this is a no-op
        pass

    def close(self) -> None:
        """
        Close the environment.
        """
        # Deactivate the API
        magics_api.set_api_active(False)

    def _get_observation(self) -> Dict:
        """
        Get the current observation.

        Returns:
            A dictionary containing the current observation.
        """
        # Get agent states
        agent_states = magics_api.get_agent_state()

        # Get environment state
        env_state = magics_api.get_environment_state()

        # Convert to numpy arrays
        max_agents = 100
        positions = np.zeros((max_agents, 2), dtype=np.float32)
        velocities = np.zeros((max_agents, 2), dtype=np.float32)
        neighbors = np.zeros((max_agents, max_agents), dtype=np.int32)

        # Fill in agent data
        agent_ids = []
        for i, (agent_id, agent) in enumerate(agent_states.items()):
            if i >= max_agents:
                break

            agent_ids.append(int(agent_id))
            positions[i] = agent["position"]
            velocities[i] = agent["velocity"]

            # Fill in neighbor data
            for neighbor in agent["connected_neighbors"]:
                if neighbor < max_agents:
                    neighbors[i, neighbor] = 1

        # Convert environment data
        max_obstacles = 100
        obstacles = np.zeros((max_obstacles, 2), dtype=np.float32)
        for i, obstacle in enumerate(env_state["obstacles"]):
            if i >= max_obstacles:
                break
            obstacles[i] = obstacle

        boundaries = np.array(
            [
                env_state["boundaries"]["min"],
                env_state["boundaries"]["max"],
            ],
            dtype=np.float32,
        )

        # Construct observation dictionary
        observation = {
            "agents": {
                "positions": positions,
                "velocities": velocities,
                "neighbors": neighbors,
            },
            "environment": {
                "obstacles": obstacles,
                "boundaries": boundaries,
            },
        }

        return observation

    def _calculate_reward(self, observation: Dict) -> float:
        """
        Calculate the reward based on the current observation.

        Args:
            observation: The current observation.

        Returns:
            The reward value.
        """
        # This is a placeholder reward function
        # In a real implementation, you would calculate a reward based on:
        # - Distance to goals
        # - Collision avoidance
        # - Path efficiency
        # - etc.

        # For now, we'll just return a constant reward
        return 0.0
