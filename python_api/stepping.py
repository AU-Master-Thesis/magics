import time
import enum
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
    elif hasattr(obj, '__dict__'):
        # Handle custom objects by converting their __dict__ to a dictionary
        return {"type": obj.__class__.__name__, "data": convert_numpy_to_python(obj.__dict__)}
    else:
        # For anything else that might not be serializable, convert to string
        try:
            json.dumps(obj)
            return obj
        except (TypeError, OverflowError):
            return str(obj)

client = MagicsClient()

if not client.is_api_active():
    client.set_api_active(True)

client.set_iterations_per_step(iterations=2)  # 5 iterations per step

print("Press Enter to step the simulation (Ctrl+C to exit)...")

try:
    while True:
        # Wait for Enter key press
        input()
        print("Stepping simulation...")
        client.step()
        print("Step complete.")
        # Get the state of all agents
        agent_states = client.get_agent_state()
        print("Agent states:")
        print(json.dumps(convert_numpy_to_python(agent_states), sort_keys=True, indent=4))
        # Get the state of the environment
        env_state = client.get_environment_state()
        print("Environment state:")
        print(json.dumps(convert_numpy_to_python(env_state), sort_keys=True, indent=4))
        print()
except KeyboardInterrupt:
    print("\nExiting...")
finally:
    # Clean up
    client.close()
