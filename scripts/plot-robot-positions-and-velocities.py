#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.plotly

import json
import plotly.graph_objects as go
import sys
from pathlib import Path

def load_data(filepath):
    """ Load the JSON data from the given filepath. """
    with open(filepath, 'r') as file:
        data = json.load(file)
    return data

def plot_robot_paths_with_velocities(data):
    """ Plot the paths and velocities for each robot. """
    fig = go.Figure()

    # Scaling factor for velocity arrows to make them visible but not overwhelming
    scale_factor = 0.1

    for robot_id, robot_data in data['robots'].items():
        positions = robot_data['positions']
        velocities = robot_data['velocities']

        x_coords = [pos[0] for pos in positions]
        y_coords = [pos[1] for pos in positions]

        # Add the robot's path to the plot
        fig.add_trace(go.Scatter(x=x_coords, y=y_coords, mode='lines+markers',
                                 name=f'Robot {robot_id} (Radius: {robot_data["radius"]})',
                                 line=dict(width=2)))

        # Add velocity arrows
        for pos, vel in zip(positions, velocities):
            fig.add_shape(type='line',
                          x0=pos[0], y0=pos[1], x1=pos[0] + vel[0] * scale_factor, y1=pos[1] + vel[1] * scale_factor,
                          line=dict(color='RoyalBlue', width=3),
                          xref='x', yref='y')

    fig.update_layout(
        title=f'Paths and Velocities of Robots in Environment: {data["environment"]}',
        xaxis_title='X Coordinate',
        yaxis_title='Y Coordinate',
        showlegend=True,
        legend_title='Robot ID'
    )

    print("writing to html ...")
    # Save the plot to an HTML file
    fig.write_html('robot-paths-velocities.html')
    print("Plot with velocities saved to 'robot-paths-velocities.html'.")

def main():
    # Accept file path from command line argument
    filepath = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('export.json')
    data = load_data(filepath)

    # Plot the paths of each robot with velocities
    plot_robot_paths_with_velocities(data)

if __name__ == '__main__':
    main()
