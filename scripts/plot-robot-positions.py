#!/usr/bin/env nix-shell
#! nix-shell -i python3
#! nix-shell -p python3Packages.matplotlib
#! nix-shell -p python3Packages.plotly
#! nix-shell -p python3Packages.rich
#! nix-shell -p python3Packages.toolz
#! nix-shell -p python3Packages.seaborn
#! nix-shell -p python3Packages.result
#! nix-shell -p python3Packages.numpy

import json
import argparse
import plotly.graph_objects as go
import numpy as np
import sys
import subprocess
import shutil
from pathlib import Path
import matplotlib.pyplot as plt

from rich import print, inspect, pretty
pretty.install()

print(f"{sys.executable = }")
print(f"{sys.version = }")

def load_data(filepath):
    """ Load the JSON data from the given filepath. """
    with open(filepath, 'r') as file:
        data = json.load(file)
    return data

def plot_robot_paths_plotly(data, filepath="robot-positions.html"):
    """ Plot the paths for each robot using their position data. """
    fig = go.Figure()

    for robot_id, robot_data in data['robots'].items():
        positions = robot_data['positions']
        x_coords = [pos[0] for pos in positions]
        y_coords = [pos[1] for pos in positions]

        fig.add_trace(go.Scatter(x=x_coords, y=y_coords, mode='lines+markers',
                                 name=f'Robot {robot_id} (Radius: {robot_data["radius"]})'))

    # # Plot obstacles
    # for obstacle in data['obstacles']:
    #     if obstacle['type'] == 'Circle':
    #         circle = go.Scatter(x=[obstacle['center'][0]], y=[obstacle['center'][1]], mode='markers',
    #                             marker=dict(size=obstacle['radius']*20, symbol='circle-open'),
    #                             name='Obstacle Circle')
    #         fig.add_trace(circle)
    #     elif obstacle['type'] == 'Polygon':
    #         vertices = obstacle['vertices']
    #         x_coords = [vertex[0] for vertex in vertices] + [vertices[0][0]]
    #         y_coords = [vertex[1] for vertex in vertices] + [vertices[0][1]]
    #         fig.add_trace(go.Scatter(x=x_coords, y=y_coords, mode='lines+markers',
    #                                  name='Obstacle Polygon'))


    # Plot obstacles
    for entity, obstacle in data['obstacles'].items():
        # print(f"{obstacle=}")
        if obstacle['type'] == 'Circle':
            circle = go.Scatter(x=[obstacle['center'][0]], y=[obstacle['center'][1]], mode='markers',
                                marker=dict(size=obstacle['radius']*20, symbol='circle-open'),
                                name='Obstacle Circle')
            fig.add_trace(circle)
        elif obstacle['type'] == 'Polygon':
            vertices = obstacle['vertices']
            x_coords = [vertex[0] for vertex in vertices] + [vertices[0][0]]
            y_coords = [vertex[1] for vertex in vertices] + [vertices[0][1]]
            fig.add_trace(go.Scatter(x=x_coords, y=y_coords, mode='lines+markers',
                                     name='Obstacle Polygon'))
            fig.add_trace(go.Scatter(x=x_coords, y=y_coords, fill="toself",
                                     fillcolor="gray", line=dict(color="gray"),
                                     name='Obstacle Polygon'))

     # Plot obstacles
    for entity, obstacle in data['obstacles'].items():
        # print(f"{obstacle=}")
        if obstacle['type'] == 'Circle':
            center_x, center_y = obstacle['center']
            radius = obstacle['radius']
            theta = np.linspace(0, 2*np.pi, 100)
            x_circle = center_x + radius * np.cos(theta)
            y_circle = center_y + radius * np.sin(theta)
            fig.add_trace(go.Scatter(x=x_circle, y=y_circle, fill="toself",
                                     fillcolor="gray", line=dict(color="gray"),
                                     name='Obstacle Circle'))
        elif obstacle['type'] == 'Polygon':
            vertices = obstacle['vertices']
            x_coords = [vertex[0] for vertex in vertices] + [vertices[0][0]]
            y_coords = [vertex[1] for vertex in vertices] + [vertices[0][1]]
            fig.add_trace(go.Scatter(x=x_coords, y=y_coords, fill="toself",
                                     fillcolor="gray", line=dict(color="gray"),
                                     name='Obstacle Polygon'))

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


def plot_robot_paths_matplotlib(data, filepath="robot-positions.svg"):
    """ Plot the paths for each robot using their position data. """
    fig, ax = plt.subplots()

    # Plot robot paths
    for robot_id, robot_data in data['robots'].items():
        positions = robot_data['positions']
        x_coords = [pos[0] for pos in positions]
        y_coords = [pos[1] for pos in positions]
        color: str = robot_data['color']

        ax.plot(x_coords, y_coords, marker='o', color=color, label=f'Robot {robot_id} (Radius: {robot_data["radius"]})')

    # Plot obstacles
    for entity, obstacle in data['obstacles'].items():
        color = 'gray'

        for collision in data['collisions']['environment']:
            print(f"{entity=} {collision=}")
            if collision['obstacle'] == int(entity):
                color = 'red'
                break

        if obstacle['type'] == 'Circle':
            circle = plt.Circle(obstacle['center'], obstacle['radius'], color=color, fill=True, alpha=0.5)
            ax.add_patch(circle)
        elif obstacle['type'] == 'Polygon':
            vertices = obstacle['vertices']
            polygon = plt.Polygon(vertices, color=color, fill=True, alpha=0.5)
            ax.add_patch(polygon)

    for collision in data['collisions']['robots']:
        robot_a = collision['robot_a']
        robot_b = collision['robot_b']
        for aabb in collision['aabbs']:
            mins: list[float] = aabb['mins']
            maxs: list[float] = aabb['maxs']

            x_coords = [mins[0], maxs[0], maxs[0], mins[0], mins[0]]
            y_coords = [mins[1], mins[1], maxs[1], maxs[1], mins[1]]

            ax.plot(x_coords, y_coords, color='red', linewidth=2)


    ax.set_title(f'Paths of Robots in Environment: {data["scenario"]}')
    ax.set_xlabel('X Coordinate')
    ax.set_ylabel('Y Coordinate')
    ax.set_aspect('equal', adjustable='box')
    ax.legend()
    ax.grid(True)

    # Save the plot to an image file
    plt.savefig(filepath)
    plt.show()
    print(f"Plot saved to '{filepath}'.")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', type=Path, help="Input JSON file containing robot data")
    parser.add_argument('-o', '--output', type=Path, default="robot-positions.svg", help="Output HTML file")
    parser.add_argument('-p', '--plot', action='store_true')
    args = parser.parse_args()

    data = json.loads(args.input.read_text())

    # Plot the paths of each robot
    # output_html = "robot-positions.html"
    plot_robot_paths_plotly(data, args.output)
    plot_robot_paths_matplotlib(data, args.output)

    # Check if `xdg-open` exists in path, and if it does then open the plot with it
    if shutil.which('xdg-open') is not None and args.plot:
        subprocess.run(['xdg-open', args.output])


if __name__ == '__main__':
    main()
