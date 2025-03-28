from pydoc import cli
import time
import enum
import os
from magics_client import MagicsClient
import json
import numpy as np


def convert_numpy_to_python(obj):
    """Convert numpy objects and custom classes to native Python types for JSON serialization."""
    if isinstance(obj, np.ndarray):
        return obj.tolist()
    elif isinstance(obj, np.integer):
        return int(obj)
    elif isinstance(obj, np.floating):
        return float(obj)
    elif isinstance(obj, dict):
        return {key: convert_numpy_to_python(value) for key, value in obj.items()}
    elif isinstance(obj, list) or isinstance(obj, tuple):
        return [convert_numpy_to_python(item) for item in obj]
    elif isinstance(obj, enum.Enum):
        # Handle enum objects by just returning their value
        return obj.value
    elif hasattr(obj, "__dict__"):
        # Handle custom objects by converting their __dict__ to a dictionary
        return {
            "type": obj.__class__.__name__,
            "data": convert_numpy_to_python(obj.__dict__),
        }
    else:
        # For anything else that might not be serializable, convert to string
        try:
            json.dumps(obj)
            return obj
        except (TypeError, OverflowError):
            return str(obj)


# Function to write JSON data to a file
def write_json_to_file(data, filename):
    """Write JSON data to a file."""
    # Create output directory if it doesn't exist
    os.makedirs("output", exist_ok=True)
    filepath = os.path.join("output", filename)
    with open(filepath, "w") as f:
        json.dump(convert_numpy_to_python(data), f, sort_keys=True, indent=4)
    print(f"Wrote data to {filepath}")


client = MagicsClient(timeout=10_000)  # 10 second timeout

if not client.is_api_active():
    client.set_api_active(True)


client.set_iterations_per_step(iterations=10)  # 5 iterations per step
# client.step()  # Step the simulation once to initialize
# client.step()  # Step the simulation once to initialize
# client.reset()  # Reset the simulation
# client.step()  # Step the simulation once to initialize
# client.step()  # Step the simulation once to initialize
# client.load_environment("CircleExperiment")  # Load the environment
# client.step()  # Step the simulation once to initialize
# client.step()  # Step the simulation once to initialize

print("Press Enter to step the simulation (Ctrl+C to exit)...")

try:
    while True:
        # Wait for Enter key press
        input()
        print("Stepping simulation...")
        client.step()
        print("Step complete.")
        # Get the state of all agents
        start = time.perf_counter()
        agent_states = client.get_agent_state()
        end = time.perf_counter()
        print(f"Agent states ({((end - start) * 1000):.2f} ms):")
        # print(json.dumps(convert_numpy_to_python(agent_states), sort_keys=True, indent=4))
        # Write agent states to file
        write_json_to_file(agent_states, "agent_states.json")

        # Get the state of the environment
        start = time.perf_counter()
        env_state = client.get_environment_state()
        end = time.perf_counter()
        print(f"Environment state ({((end - start) * 1000):.2f} ms):")
        # print(json.dumps(convert_numpy_to_python(env_state), sort_keys=True, indent=4))
        # Write environment state to file
        write_json_to_file(env_state, "environment_state.json")

        print()
except KeyboardInterrupt:
    print("\nExiting...")
finally:
    # Clean up
    client.close()
