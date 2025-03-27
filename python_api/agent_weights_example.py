#!/usr/bin/env python3
"""
Example script demonstrating how to set per-agent factor weights in Magics.

This script allows you to set different factor weights for different agents
in the simulation, which can be useful for testing different behaviors
or optimizing performance for specific scenarios.
"""

import time
import json
from magics_client import MagicsClient
import numpy as np

# Connect to the simulation
client = MagicsClient(timeout=10_000)  # 10 second timeout

if not client.is_api_active():
    print("Activating API...")
    client.set_api_active(True)

# Get the current state to see all agents
agent_states = client.get_agent_state()
print(f"Found {len(agent_states)} agents in the simulation")

for agent_id, state in agent_states.items():
    print(f"Agent {agent_id}: Position {state['position']}")
    # Print current factor weights for each agent
    if 'factor_graph_state' in state and 'weights' in state['factor_graph_state']:
        weights = state['factor_graph_state']['weights']
        print(f"  Current weights: dynamic={weights['dynamic']}, obstacle={weights['obstacle']}, "
              f"interrobot={weights['interrobot']}, tracking={weights['tracking']}")

# Ask which agent to modify
agent_choice = input("\nEnter agent ID to modify (leave empty for system-wide change): ")
agent_id_to_modify = None if agent_choice == "" else int(agent_choice)

# Ask which factor weights to modify
print("\nEnter new weights (leave empty to keep current value):")
dynamic_input = input("Dynamic factor weight: ")
obstacle_input = input("Obstacle factor weight: ")
interrobot_input = input("Interrobot factor weight: ")
tracking_input = input("Tracking factor weight: ")

# Prepare weights dictionary
weights = {}
if dynamic_input:
    weights["dynamic"] = float(dynamic_input)
if obstacle_input:
    weights["obstacle"] = float(obstacle_input)
if interrobot_input:
    weights["interrobot"] = float(interrobot_input)
if tracking_input:
    weights["tracking"] = float(tracking_input)

if weights:
    # Apply the weight update
    if agent_id_to_modify is None:
        print("\nUpdating system-wide factor weights...")
    else:
        print(f"\nUpdating factor weights for agent {agent_id_to_modify}...")
    
    client.set_factor_weights(weights, agent_id_to_modify)
    print("Weight update sent successfully")
    
    # Step the simulation to apply changes
    print("\nStepping simulation to apply changes...")
    client.step()
    print("Step complete")
    
    # Get updated state
    updated_states = client.get_agent_state()
    if agent_id_to_modify is None:
        print("\nUpdated weights for all agents:")
        for agent_id, state in updated_states.items():
            if 'factor_graph_state' in state and 'weights' in state['factor_graph_state']:
                weights = state['factor_graph_state']['weights']
                print(f"Agent {agent_id}: dynamic={weights['dynamic']}, obstacle={weights['obstacle']}, "
                      f"interrobot={weights['interrobot']}, tracking={weights['tracking']}")
    else:
        if str(agent_id_to_modify) in updated_states:
            state = updated_states[str(agent_id_to_modify)]
            if 'factor_graph_state' in state and 'weights' in state['factor_graph_state']:
                weights = state['factor_graph_state']['weights']
                print(f"Updated weights for agent {agent_id_to_modify}:")
                print(f"  dynamic={weights['dynamic']}, obstacle={weights['obstacle']}, "
                      f"interrobot={weights['interrobot']}, tracking={weights['tracking']}")
else:
    print("No weights specified, nothing to update")

# Clean up
client.close()
print("Connection closed")
