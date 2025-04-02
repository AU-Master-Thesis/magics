#!/usr/bin/env python3
"""
Interactive Terminal Control for Magics Simulation
"""

import sys
import os
import json
import time
import enum
import traceback
import numpy as np
import zmq
from magics_client import MagicsClient, MagicsError, PlanningStrategy, MissionState

# --- Helper Functions (copied from stepping.py) ---


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
        # Handle enum objects by just returning their value or name
        return obj.name  # Use name for better readability in JSON
    elif hasattr(obj, "__dict__"):
        # Handle custom objects by converting their __dict__ to a dictionary
        # Filter out internal attributes if necessary
        data = {k: v for k, v in obj.__dict__.items() if not k.startswith("_")}
        return {
            "type": obj.__class__.__name__,
            "data": convert_numpy_to_python(data),
        }
    else:
        # For anything else that might not be serializable, convert to string
        try:
            json.dumps(obj)
            return obj
        except (TypeError, OverflowError):
            return str(obj)


def write_json_to_file(data, filename):
    """Write JSON data to a file."""
    # Create output directory if it doesn't exist
    output_dir = "output"
    os.makedirs(output_dir, exist_ok=True)
    filepath = os.path.join(output_dir, filename)
    try:
        with open(filepath, "w") as f:
            json.dump(convert_numpy_to_python(data), f, sort_keys=True, indent=4)
        print(f"Wrote data to {filepath}")
    except Exception as e:
        print(f"Error writing to {filepath}: {e}")


# --- Command Handlers ---


def handle_step(client: MagicsClient, args: list):
    """Steps the simulation forward N times (default 1)."""
    num_steps = 1
    if args:
        try:
            num_steps = int(args[0])
            if num_steps <= 0:
                print("Error: Number of steps must be positive.")
                return
        except ValueError:
            print("Error: Invalid number of steps provided. Stepping once.")
            num_steps = 1

    start_time = time.perf_counter()
    print(f"Stepping simulation {num_steps} time(s)...")
    for i in range(num_steps):
        client.step()
        # Optional: Add a small delay or progress indicator if needed for many steps
        # print(f"  Step {i+1}/{num_steps} complete.")
    end_time = time.perf_counter()
    print(
        f"Step{'s' if num_steps > 1 else ''} complete in {((end_time - start_time) * 1000):.2f} ms."
    )


def handle_reset(client: MagicsClient, args: list):
    """Resets the simulation."""
    client.reset()
    print("Simulation reset.")


def handle_iterations(client: MagicsClient, args: list):
    """Sets the number of iterations per step."""
    if not args:
        print("Usage: iterations <number>")
        return
    try:
        iterations = int(args[0])
        if iterations <= 0:
            print("Error: Number of iterations must be positive.")
            return
        client.set_iterations_per_step(iterations=iterations)
        print(f"Set iterations per step to {iterations}.")
    except ValueError:
        print("Error: Invalid number provided.")
    except MagicsError as e:
        print(f"API Error: {e}")


def handle_get_agent_state(client: MagicsClient, args: list):
    """Gets the current agent states and prints them."""
    start = time.perf_counter()
    agent_states = client.get_agent_state()
    end = time.perf_counter()
    print(f"Agent states ({((end - start) * 1000):.2f} ms):")
    print(json.dumps(convert_numpy_to_python(agent_states), sort_keys=True, indent=4))


def handle_save_agent_state(client: MagicsClient, args: list):
    """Gets the current agent states and saves them to output/agent_states.json."""
    start = time.perf_counter()
    agent_states = client.get_agent_state()
    end = time.perf_counter()
    print(f"Fetched agent states ({((end - start) * 1000):.2f} ms). Saving...")
    write_json_to_file(agent_states, "agent_states.json")


def handle_get_env_state(client: MagicsClient, args: list):
    """Gets the current environment state and prints it."""
    start = time.perf_counter()
    env_state = client.get_environment_state()
    end = time.perf_counter()
    print(f"Environment state ({((end - start) * 1000):.2f} ms):")
    print(json.dumps(convert_numpy_to_python(env_state), sort_keys=True, indent=4))


def handle_save_env_state(client: MagicsClient, args: list):
    """Gets the current environment state and saves it to output/environment_state.json."""
    start = time.perf_counter()
    env_state = client.get_environment_state()
    end = time.perf_counter()
    print(f"Fetched environment state ({((end - start) * 1000):.2f} ms). Saving...")
    write_json_to_file(env_state, "environment_state.json")


def handle_help(client: MagicsClient, args: list):
    """Prints this help message."""
    print("Available commands:")
    print("  step (s) [N]     : Step the simulation forward N times (default: 1).")
    print("  reset (r)        : Reset the simulation.")
    print("  iterations (i) N : Set the number of GBP iterations per step to N.")
    print("  get_agent_state (gas) : Get and print current agent states.")
    print(
        "  save_agent_state (sas): Get and save current agent states to output/agent_states.json."
    )
    print("  get_env_state (ges)   : Get and print current environment state.")
    print(
        "  save_env_state (ses)  : Get and save current environment state to output/environment_state.json."
    )
    print("  help (h)         : Show this help message.")
    print("  quit (q) / exit  : Exit the controller.")


def handle_quit(client: MagicsClient, args: list):
    """Exits the controller."""
    print("Exiting...")
    return False  # Signal to exit the main loop


# --- Command Mapping ---

commands = {
    "step": handle_step,
    "s": handle_step,
    "reset": handle_reset,
    "r": handle_reset,
    "iterations": handle_iterations,
    "i": handle_iterations,
    "get_agent_state": handle_get_agent_state,
    "gas": handle_get_agent_state,
    "save_agent_state": handle_save_agent_state,
    "sas": handle_save_agent_state,
    "get_env_state": handle_get_env_state,
    "ges": handle_get_env_state,
    "save_env_state": handle_save_env_state,
    "ses": handle_save_env_state,
    "help": handle_help,
    "h": handle_help,
    "quit": handle_quit,
    "q": handle_quit,
    "exit": handle_quit,
}

# --- Main Execution ---


def main():
    client = None
    try:
        print("Connecting to Magics server...")
        # Increased timeout for potentially long operations
        client = MagicsClient(timeout=10_000)
        client.set_iterations_per_step(iterations=1)
        print("Connected.")

        if not client.is_api_active():
            print("API not active, activating...")
            client.set_api_active(True)
            print("API activated.")
        else:
            print("API is active.")

        print("\nMagics Terminal Controller")
        print("Type 'help' for a list of commands.")

        running = True
        while running:
            try:
                raw_input = input("Magics> ").strip()

                # Default action: step once if input is empty
                if not raw_input:
                    handle_step(client, ['1']) # Simulate 'step 1'
                    continue

                parts = raw_input.split()
                command_name = parts[0].lower()
                command_args = parts[1:]

                if command_name in commands:
                    handler = commands[command_name]
                    result = handler(client, command_args)
                    if result == False:  # Check specifically for False to signal exit
                        running = False
                else:
                    print(
                        f"Unknown command: '{command_name}'. Type 'help' for options."
                    )

            except KeyboardInterrupt:
                print("\nCtrl+C detected.")
                running = False  # Allow finally block to clean up
            except MagicsError as e:
                print(f"Magics API Error: {e}")
            except Exception as e:
                print(f"An unexpected error occurred: {e}")
                traceback.print_exc()  # Print full traceback for debugging

    except MagicsError as e:
        print(f"Failed to connect or initialize: {e}")
    except zmq.ZMQError as e:
        print(f"ZeroMQ Error: {e}. Is the Magics server running?")
    except Exception as e:
        print(f"An unexpected startup error occurred: {e}")
        traceback.print_exc()
    finally:
        if client:
            print("Closing connection...")
            client.close()
            print("Connection closed.")


if __name__ == "__main__":
    main()
