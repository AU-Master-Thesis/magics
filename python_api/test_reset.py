#!/usr/bin/env python3
"""
Test script for reset and load environment functionality in the Magics API.
This script tests the new implementation that waits for reset and load operations to complete.
"""

import time
import sys
from magics_client import MagicsClient

def main():
    # Connect to the Magics API
    client = MagicsClient()
    print("Connected to Magics API")
    
    # Test reset functionality
    print("\n=== Testing Reset ===")
    print("Resetting simulation...")
    start_time = time.time()
    client.reset()
    end_time = time.time()
    print(f"Reset completed in {end_time - start_time:.2f} seconds")
    
    # Test load environment functionality
    print("\n=== Testing Load Environment ===")
    
    # Define some test environments
    # These should be environments that exist in your config/scenarios directory
    environments = ["CircleExperiment", "JunctionTwoway"]
    print(f"Testing with environments: {environments}")
    
    # Load each environment and measure time
    for env_name in environments[:2]:  # Limit to first 2 environments for brevity
        print(f"\nLoading environment: {env_name}")
        start_time = time.time()
        client.load_environment(env_name)
        end_time = time.time()
        print(f"Environment '{env_name}' loaded in {end_time - start_time:.2f} seconds")
        
        # Get some state to verify the environment was loaded
        agent_states = client.get_agent_state()
        env_state = client.get_environment_state()
        print(f"Environment has {len(agent_states)} agents and {len(env_state.obstacles)} obstacles")
        
        # Wait a bit before loading the next environment
        time.sleep(1)
    
    print("\nAll tests completed successfully!")

if __name__ == "__main__":
    main()
