import time
from magics_client import MagicsClient

client = MagicsClient()

if not client.is_api_active():
    client.set_api_active(True)

client.set_iterations_per_step(iterations=10)  # 5 iterations per step

print("Press Enter to step the simulation (Ctrl+C to exit)...")

try:
    while True:
        # Wait for Enter key press
        input()
        print("Stepping simulation...")
        client.step()
        print("Step complete.")
except KeyboardInterrupt:
    print("\nExiting...")
finally:
    # Clean up
    client.close()
