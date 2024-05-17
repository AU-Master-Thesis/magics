#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.matplotlib python3Packages.plotly python3Packages.polars

import json
import plotly.graph_objects as go
import sys
from pathlib import Path

def load_data(filepath):
    """ Load the JSON data from the given filepath. """
    with open(filepath, 'r') as file:
        data = json.load(file)
    return data

def plot_robot_paths(data):
    """ Plot the paths for each robot using their position data. """
    fig = go.Figure()

    for robot_id, robot_data in data['robots'].items():
        positions = robot_data['positions']
        x_coords = [pos[0] for pos in positions]
        y_coords = [pos[1] for pos in positions]

        fig.add_trace(go.Scatter(x=x_coords, y=y_coords, mode='lines+markers',
                                 name=f'Robot {robot_id} (Radius: {robot_data["radius"]})'))

    fig.update_layout(
        title=f'Paths of Robots in Environment: {data["environment"]}',
        xaxis_title='X Coordinate',
        yaxis_title='Y Coordinate',
        legend_title='Robot ID',
        xaxis = dict(
            scaleanchor = "y",
            scaleratio = 1
        )
        # width=800,
        # height=800
    )

    # Save the plot to an HTML file
    fig.write_html('robot-positions.html')
    print("Plot saved to 'robot-positions.html'.")

def main():
    # Accept file path from command line argument
    filepath = Path(sys.argv[1]) if len(sys.argv) > 1 else Path('export.json')
    data = load_data(filepath)

    # Plot the paths of each robot
    plot_robot_paths(data)

if __name__ == '__main__':
    main()
