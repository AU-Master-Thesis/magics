"""
Basic example of using the Magics Gym environment.

This script demonstrates how to use the Magics Gym environment with a simple
random agent.
"""

import os
import sys
# Add the virtual environment path to sys.path
venv_path = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), '.venv/lib/python3.10/site-packages')
sys.path.insert(0, venv_path)

import numpy as np
import magics_gym

# Create the environment
env = magics_gym.MagicsEnv()

# Reset the environment
observation = env.reset()

# Run for 100 steps
for i in range(100):
    print(f"Step {i}")
    
    # Sample a random action
    action = env.action_space.sample()
    
    # Take a step in the environment
    observation, reward, done, info = env.step(action)
    
    # Print the reward
    print(f"Reward: {reward}")
    
    # Check if the episode is done
    if done:
        print("Episode finished")
        break

# Close the environment
env.close()
