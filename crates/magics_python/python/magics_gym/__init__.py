"""
Magics Gym - OpenAI Gym environment for the Magics simulation.

This package provides an OpenAI Gym environment for the Magics simulation,
allowing reinforcement learning algorithms to interact with the simulation.
"""

import os
import sys
# Add the virtual environment path to sys.path
venv_path = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), 
                         '.venv/lib/python3.10/site-packages')
sys.path.insert(0, venv_path)

from magics_gym.env import MagicsEnv

__all__ = ["MagicsEnv"]
