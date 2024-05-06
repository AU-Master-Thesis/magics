#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.plotly

import json
import plotly.graph_objects as go
import sys
from pathlib import Path

def load_data(filepath: Path):
    """ Load the JSON data from the given filepath. """
    with open(filepath, 'r') as file:
        data = json.load(file)
    return data

def plot_robot_velocities(data):
    """ Plot the velocities for each robot over time. """
    fig = go.Figure()

    for robot_id, robot_data in data['robots'].items():
        velocities = robot_data['velocities']
        time_steps = list(range(len(velocities)))  # Assuming constant time steps

        # Calculating the magnitude of velocity
        velocity_magnitudes = [((v[0]**2 + v[1]**2)**0.5) for v in velocities]

        fig.add_trace(go.Scatter(x=time_steps, y=velocity_magnitudes, mode='lines+markers',
                                 name=f'Robot {robot_id} (Radius: {robot_data["radius"]})'))

    fig.update_layout(
        title=f'Velocities of Robots in Environment: {data["environment"]}',
        xaxis_title='Time Step',
        yaxis_title='Velocity Magnitude',
        legend_title='Robot ID'
    )

    # Save the plot to an HTML file
    fig.write_html('robot-velocities.html')
    print("Velocity plot saved to 'robot-velocities.html'.")

def main():
    # Accept file path from command line argument
    filepath = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('export.json')
    data = load_data(filepath)

    # Plot the velocities of each robot
    plot_robot_velocities(data)

if __name__ == '__main__':
    main()
