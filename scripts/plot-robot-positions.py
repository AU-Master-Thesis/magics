#!/usr/bin/env nix-shell
#! nix-shell -i python3
#! nix-shell -p python311
#! nix-shell -p python3Packages.matplotlib
#! nix-shell -p qt5.qtbase
#! nix-shell -p qt5.qtwayland
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
import matplotlib
import matplotlib.pyplot as plt
matplotlib.use('Agg')  # Use the Agg backend (no GUI)

from rich import print, inspect, pretty
pretty.install()

# print(f"{sys.executable = }")
# print(f"{sys.version = }")

# sys.exit(0)

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

        positions = np.array(robot_data['positions'])
        total_distance: float = np.sum(np.linalg.norm(np.diff(positions, axis=1), axis=0))

        fig.add_trace(go.Scatter(x=x_coords, y=y_coords, mode='lines+markers',
                                 name=f'Robot {robot_id} (Radius: {robot_data["radius"]}, Distance: {total_distance:.2f})'))

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
    fig, ax = plt.subplots(figsize=(12, 10))

    # Plot robot paths
    n: int = 0
    for robot_id, robot_data in data['robots'].items():
        positions = robot_data['positions']
        # match positions.shape:
        #     case (_, 2):
        #         pass
        #     case _:
        #         print(f"wrong shape {positions.shape=}")
        #         sys.exit(1)

        # for i in range(len(positions) - 1, -1, -1):
        #     point = positions[i]
        #     if abs(point[0]) > 95 or abs(point[1]) > 55:
        #         _ = positions.pop()

        x_coords = [pos[0] for pos in positions]
        y_coords = [pos[1] for pos in positions]
        color: str = robot_data['color']

        distance_traveled: float = np.sum(np.linalg.norm(np.diff(positions, axis=0), axis=1))

        waypoints = []
        mission = robot_data['mission']
        for route in mission['routes']:
            waypoints.append(route['waypoints'][0])
            for wp in route['waypoints'][1:]:
                waypoints.append(wp)

        waypoints = np.array(waypoints)
        waypoints = np.squeeze(waypoints)

        for ix in [0, -1]:
            x =waypoints[ix][0]
            xlimit = 95
            if abs(x) > xlimit:
                sign: int = -1 if x < 0.0 else 1
                x = sign * xlimit
                waypoints[ix][0] = x

            ylimit = 60
            y =waypoints[ix][1]
            if abs(y) > ylimit:
                sign: int = -1 if y < 0.0 else 1
                y = sign * ylimit
                waypoints[ix][1] = y


                # [-50, 50]

        ax.plot([pos[0] for pos in waypoints], [pos[1] for pos in waypoints], color='black')

        def accumulated_distance(points):
            # Compute pairwise Euclidean distances between successive points
            distances = np.sum(np.sqrt(np.sum(np.diff(points, axis=0)**2, axis=1)))
            return distances

        # print(f"{robot_id=} {distance_travelled=}")

        ax.plot(x_coords, y_coords, marker='o', markersize=2, color=color, label=f'Robot {robot_id} (Radius: {robot_data["radius"]} Distance Traveled: {distance_traveled})')
        n += 1
        if n == 12:
            break

    # Plot obstacles
    for entity, obstacle in data['obstacles'].items():
        color = 'gray'

        for collision in data['collisions']['environment']:
            # print(f"{entity=} {collision=}")
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

    # for collision in data['collisions']['robots']:
    #     robot_a = collision['robot_a']
    #     robot_b = collision['robot_b']
    #     for aabb in collision['aabbs']:
    #         mins: list[float] = aabb['mins']
    #         maxs: list[float] = aabb['maxs']

    #         x_coords = [mins[0], maxs[0], maxs[0], mins[0], mins[0]]
    #         y_coords = [mins[1], mins[1], maxs[1], maxs[1], mins[1]]

    #         ax.plot(x_coords, y_coords, color='red', linewidth=2)


    ax.set_title(f'Paths of Robots in Environment: {data["scenario"]}')
    ax.set_xlabel('X Coordinate')
    ax.set_ylabel('Y Coordinate')
    ax.set_aspect('equal', adjustable='box')
    # ax.legend()
    # ax.grid(True)

    # Save the plot to an image file
    plt.savefig(filepath)
    plt.tight_layout()
    plt.show()
    print(f"Plot saved to '{filepath}'.")

def main():

    # positions = np.array([
    #     [0, 0],
    #     [1, 0],
    #     [1, 2],
    # ])
    #
    # distance_traveled: float = np.sum(np.linalg.norm(np.diff(positions, axis=0), axis=1))
    # print(f"{distance_traveled=}")
    #
    # sys.exit(0)

    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', type=Path, help="Input JSON file containing robot data")
    parser.add_argument('-o', '--output', type=Path, default="robot-positions.svg", help="Output HTML file")
    parser.add_argument('-p', '--plot', action='store_true')
    args = parser.parse_args()

    data = json.loads(args.input.read_text())

    # Plot the paths of each robot
    # output_html = "robot-positions.html"
    # plot_robot_paths_plotly(data, args.output)
    plot_robot_paths_matplotlib(data, args.output)

    if shutil.which('timg') is not None and args.plot:
        print(f"input: {args.input}")
        subprocess.run(['timg', '--center', args.output])

    # Check if `xdg-open` exists in path, and if it does then open the plot with it
    # if shutil.which('xdg-open') is not None and args.plot:
    #     subprocess.run(['xdg-open', args.output])


if __name__ == '__main__':
    main()
