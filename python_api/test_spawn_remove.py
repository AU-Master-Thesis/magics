"""
Test script for Magics API spawn_agent and remove_agent functionality.
"""

from encodings.punycode import T
import random
import time
import zmq  # Add this import
from magics_client import MagicsClient, MagicsError, FactorWeightsDict

# --- Configuration ---
HOST = "localhost"
PORT = 5555
TIMEOUT = 10000  # Increased timeout for potentially longer operations like spawn


# --- Helper Function ---
def print_agent_summary(agent_states):
    """Prints a summary of current agents."""
    if not agent_states:
        print("  No agents found.")
        return
    print(f"  Found {len(agent_states)} agents:")
    for agent_id, state in agent_states.items():
        pos = state.get("position", "[N/A]")
        vel = state.get("velocity", "[N/A]")
        print(f"    - Agent {agent_id}: Pos={pos}, Vel={vel}")


# --- Main Test Logic ---
if __name__ == "__main__":
    client = None
    spawned_agent_id = None
    try:
        print(f"Connecting to Magics server at tcp://{HOST}:{PORT}...")
        client = MagicsClient(host=HOST, port=PORT, timeout=TIMEOUT)
        print("Connected successfully.")

        # 1. Get initial state
        print("\n--- Getting initial agent state ---")
        initial_states = client.get_agent_state()
        print_agent_summary(initial_states)
        initial_agent_ids = set(initial_states.keys())

        # 2. Spawn a new agent
        print("\n--- Spawning a new agent ---")
        # get random positions between 100 and -100
        spawn_pos = [random.uniform(-100, 100), random.uniform(-100, 100)]
        # get random goal positions between 100 and -100
        goal_pos = [random.uniform(-100, 100), random.uniform(-100, 100)]
        # spawn_pos = [0.0, 5.0]
        # goal_pos = [50.0, -50.0]
        print(f"  Spawning agent at {spawn_pos} with goal {goal_pos}")

        # Example with optional parameters
        custom_weights: FactorWeightsDict = {
            "dynamic": 0.1,
            "obstacle": 0.005,
            "interrobot": 0.0005,
            "tracking": 0.1,
        }

        spawned_agent_id = client.spawn_agent(
            initial_position=spawn_pos,
            goal_position=goal_pos,
            initial_velocity=[0.0, 0.0],
            radius=2.0,
            planning_strategy="OnlyLocal",  # or "RrtStar"
            target_speed=10.0,
            weights=custom_weights,
        )
        print(f"  Successfully spawned agent with ID: {spawned_agent_id}")

        # 3. Step simulation a bit
        steps = 5
        print(f"\n--- Stepping simulation ({steps} steps) ---")
        for i in range(steps):
            client.step()
            print(f"  Step {i + 1} completed.")
            time.sleep(0.1)  # Small delay for clarity

        # 4. Verify agent exists
        print("\n--- Verifying spawned agent state ---")
        states_after_spawn = client.get_agent_state()
        print_agent_summary(states_after_spawn)
        if str(spawned_agent_id) in states_after_spawn:
            print(f"  Agent {spawned_agent_id} confirmed in simulation state.")
        else:
            print(f"  ERROR: Agent {spawned_agent_id} not found after spawning!")
            raise MagicsError(
                f"Agent {spawned_agent_id} not found after spawning!"
            )  # Optional: raise error
        input()
        # 5. Remove the spawned agent
        print(f"\n--- Removing agent {spawned_agent_id} ---")
        client.remove_agent(agent_id=spawned_agent_id)
        print(f"  Remove command sent for agent {spawned_agent_id}.")

        # 6. Step simulation again
        print("\n--- Stepping simulation (1 step) after removal ---")
        client.step()
        print("  Step completed.")

        # 7. Verify agent is removed
        print("\n--- Verifying agent removal ---")
        states_after_removal = client.get_agent_state()
        print_agent_summary(states_after_removal)
        if str(spawned_agent_id) not in states_after_removal:
            print(f"  Agent {spawned_agent_id} successfully removed from active state.")
        else:
            print(
                f"  ERROR: Agent {spawned_agent_id} still found after removal request!"
            )
            # raise MagicsError(f"Agent {spawned_agent_id} still found after removal request!") # Optional: raise error

        # Optional: Check if the number of agents returned to original count (if no others were removed/added)
        # current_agent_ids = set(states_after_removal.keys())
        # expected_agent_ids = initial_agent_ids
        # if current_agent_ids == expected_agent_ids:
        #     print("  Agent count returned to initial state.")
        # else:
        #     print(f"  Warning: Agent count mismatch. Initial: {len(initial_agent_ids)}, Final: {len(current_agent_ids)}")

        # print("\n--- Test Completed Successfully ---")

    except MagicsError as e:
        print(f"\n--- Magics API Error ---")
        print(f"  Error: {e}")
    except zmq.ZMQError as e:
        print(f"\n--- ZeroMQ Error ---")
        print(f"  Error connecting or communicating with the server: {e}")
        print(f"  Is the Magics simulation running with the API enabled?")
    except Exception as e:
        print(f"\n--- An Unexpected Error Occurred ---")
        print(f"  Error: {e}")
    finally:
        if client:
            print("\nClosing connection.")
            client.close()
