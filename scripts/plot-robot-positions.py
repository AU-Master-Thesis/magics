#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.matplotlib python3Packages.plotly python3Packages.polars

import json
import argparse
import plotly.graph_objects as go
import sys
import subprocess
import shutil
from pathlib import Path

def load_data(filepath):
    """ Load the JSON data from the given filepath. """
    with open(filepath, 'r') as file:
        data = json.load(file)
    return data

def plot_robot_paths(data, filepath="robot-positions.html"):
    """ Plot the paths for each robot using their position data. """
    fig = go.Figure()

    for robot_id, robot_data in data['robots'].items():
        positions = robot_data['positions']
        x_coords = [pos[0] for pos in positions]
        y_coords = [pos[1] for pos in positions]

        fig.add_trace(go.Scatter(x=x_coords, y=y_coords, mode='lines+markers',
                                 name=f'Robot {robot_id} (Radius: {robot_data["radius"]})'))

    fig.update_layout(
        title=f'Paths of Robots in Environment: {data["scenario"]}',
        xaxis_title='X Coordinate',
        yaxis_title='Y Coordinate',
        legend_title='Robot ID',
        xaxis = dict(
            scaleanchor = "y",
            scaleratio = 1
        )
    )

    # Save the plot to an HTML file
    fig.write_html(filepath)
    print(f"Plot saved to '{filepath}'.")
    # print("Plot saved to 'robot-positions.html'.")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', type=Path, help="Input JSON file containing robot data")
    parser.add_argument('-o', '--output', type=Path, default="robot-positions.html", help="Output HTML file")
    args = parser.parse_args()

    data = json.loads(args.input.read_text())

    # Plot the paths of each robot
    # output_html = "robot-positions.html"
    plot_robot_paths(data, args.output)

    # Check if `xdg-open` exists in path, and if it does then open the plot with it
    if shutil.which('xdg-open') is not None:
        subprocess.run(['xdg-open', args.output])


if __name__ == '__main__':
    main()
