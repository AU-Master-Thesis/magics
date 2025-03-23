#!/usr/bin/env python3
"""
Example script for using the Magics API.

This script demonstrates how to use both the direct ZeroMQ client and
the OpenAI Gym environment.
"""

import time
import numpy as np
from magics_client import MagicsClient, MagicsError
from magics_gym import MagicsEnv


def test_direct_api():
    """Test the direct ZeroMQ API."""
    print("Testing direct ZeroMQ API...")
    
    # Create a client
    client = MagicsClient()
    
    try:
        # Check if API is active
        try:
            api_active = client.is_api_active()
            print(f"API active: {api_active}")
            
            # If not active, activate it
            if not api_active:
                print("Activating API...")
                client.set_api_active(True)
                print("API activated successfully")
        except MagicsError as e:
            print(f"Error checking API status: {e}")
            print("Attempting to activate API anyway...")
            client.set_api_active(True)
            print("API activation attempted")
        
        # Get environment state
        try:
            env_state = client.get_environment_state()
            print(f"Environment has {len(env_state.get('obstacles', []))} obstacles")
            print(f"Environment boundaries: {env_state.get('boundaries', {})}")
        except MagicsError as e:
            print(f"Error getting environment state: {e}")
        
        # Get agent states
        try:
            agent_states = client.get_agent_state()
            print(f"There are {len(agent_states)} agents in the simulation")
            
            # For each agent, print position and velocity
            for agent_id, state in agent_states.items():
                print(f"Agent {agent_id}:")
                print(f"  Position: {state['position']}")
                print(f"  Velocity: {state['velocity']}")
                print(f"  Connected to {len(state.get('connected_neighbors', []))} neighbors")
        except MagicsError as e:
            print(f"Error getting agent states: {e}")
        
        # Set factor weights
        print("Setting factor weights...")
        weights = {
            "dynamic": 1.0,
            "obstacle": 5.0,
            "interrobot": 2.0,
            "tracking": 1.0
        }
        try:
            client.set_factor_weights(weights)
            print("Factor weights set successfully")
        except MagicsError as e:
            print(f"Error setting factor weights: {e}")
        
        # Step the simulation a few times
        for i in range(5):
            print(f"Stepping simulation ({i+1}/5)...")
            try:
                client.step()
                print("Step completed successfully")
                
                # Get updated agent states
                agent_states = client.get_agent_state()
                
                # Print updated positions
                for agent_id, state in agent_states.items():
                    print(f"Agent {agent_id} position: {state['position']}")
            except MagicsError as e:
                print(f"Error during step {i+1}: {e}")
            
            time.sleep(0.5)  # Wait for the simulation to complete the step
    
    finally:
        # Clean up
        print("Cleaning up...")
        client.close()


def test_gym_env():
    """Test the OpenAI Gym environment."""
    print("\nTesting OpenAI Gym environment...")
    
    # Create environment
    try:
        env = MagicsEnv()
        
        try:
            # Reset the environment
            observation, info = env.reset()
            print(f"Initial observation:")
            print(f"  Number of agents: {len(observation['agents']['positions'])}")
            print(f"  Number of obstacles: {len(observation['obstacles'])}")
            
            # Run for a few steps with random actions
            for i in range(5):
                print(f"Step {i+1}/5...")
                
                # Sample a random action
                action = env.action_space.sample()
                print(f"  Action: {action}")
                
                try:
                    # Take a step
                    observation, reward, terminated, truncated, info = env.step(action)
                    
                    # Print results
                    print(f"  Reward: {reward}")
                    print(f"  Terminated: {terminated}")
                    print(f"  Agent positions:")
                    for j, pos in enumerate(observation["agents"]["positions"]):
                        print(f"    Agent {j}: {pos}")
                    
                    # Break if the episode is done
                    if terminated or truncated:
                        print("Episode finished early")
                        break
                except Exception as e:
                    print(f"Error during step {i+1}: {e}")
        
        finally:
            # Clean up
            env.close()
            print("Gym environment closed")
    except Exception as e:
        print(f"Error creating Gym environment: {e}")


if __name__ == "__main__":
    # Test direct API
    try:
        test_direct_api()
    except Exception as e:
        print(f"Error in direct API test: {e}")
    
    # Test Gym environment
    try:
        test_gym_env()
    except Exception as e:
        print(f"Error in Gym environment test: {e}")
    
    print("\nExample completed.")
