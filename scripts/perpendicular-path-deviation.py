#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.numpy python3Packages.scipy python3Packages.rich python3Packages.tabulate python3Packages.matplotlib python3Packages.toolz python3Packages.seaborn python3Packages.result

import dataclasses
import statistics
import json
import sys
import argparse
import itertools
from pathlib import Path
from dataclasses import dataclass
import collections

import numpy as np
from scipy.integrate import simpson
import matplotlib.pyplot as plt
from rich import print, inspect, pretty
from tabulate import tabulate
import toolz
from toolz.curried import get
from typing import Generator, Iterable
# from matplotlib.patches import Polygon

pretty.install()

def sliding_window(iterable: Iterable, n: int):
    "Collect data into overlapping fixed-length chunks or blocks."
    # sliding_window('ABCDEFG', 4) → ABCD BCDE CDEF DEFG
    iterator = iter(iterable)
    window = collections.deque(itertools.islice(iterator, n - 1), maxlen=n)
    for x in iterator:
        window.append(x)
        yield tuple(window)

@dataclass
class Line:
    a: float
    b: float

def line_from_line_segment(x1: float, y1: float, x2: float, y2: float) -> Line:
    a = (y2 - y1) / (x2 - x1)
    b = y1 - (a * x1)
    return Line(a=a, b=b)

def projection_onto_line(point: np.ndarray, line: Line) -> np.ndarray:
    assert len(point) == 2
    x1, y1 = point
    xp: float = (x1 + line.a * (y1 - line.b)) / (line.a * line.a + 1)
    projection = np.array([xp, line.a * xp + line.b])
    assert len(projection) == 2

    return projection


def closest_projection_onto_line_segments(point: np.ndarray, lines: list[Line]) -> np.ndarray:
    assert len(point) == 2
    projections = [projection_onto_line(point, line) for line in lines]
    closest_projection = min(projections, key=lambda proj: np.linalg.norm(proj - point))

    return closest_projection


def main():
    print(f"{sys.executable = }")
    print(f"{sys.version = }")

    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', type=Path)
    parser.add_argument('-p', '--plot', action='store_true')
    args = parser.parse_args()

    data = json.loads(args.input.read_text())

    rmses: list[float] = []
    rmse_of_each_robot: dict[str, float] = {}

    for robot_id, robot_data in data['robots'].items():
        color: str = robot_data['color']
        positions = np.array([p for p in robot_data['positions']])
        mission = robot_data['mission']
        waypoints = []
        for route in mission['routes']:
            waypoints.append(route['waypoints'][0])
            for wp in route['waypoints'][1:]:
                waypoints.append(wp)

        # print(waypoints)

        # waypoints = np.array([wp for wp in (route['waypoints'] for route in mission['routes'])])
        waypoints = np.array(waypoints)

        waypoints = np.squeeze(waypoints)

        x_coords = []
        y_coords = []
        projection_x_coords = []
        projection_y_coords = []

        lines: list[Line] = [line_from_line_segment(*start, *end) for start, end in sliding_window(waypoints, 2)]
        closest_projections = [closest_projection_onto_line_segments(p, lines) for p in positions]

        # plot a solid lines of all waypoints
        # plt.plot(waypoints[:, 0], waypoints[:, 1], linestyle='solid', color=color)
        plt.plot(waypoints[:, 0], waypoints[:, 1], linestyle='solid', color=color, label=f'Robot {robot_id}')

        # plot a dashed line from each point to closest projection
        for p, cp in zip(positions, closest_projections):
            # plt.plot([p[0], cp[0]], [p[1], cp[1]], linestyle='dashed', color='red')
            x_coords.append(p[0])
            y_coords.append(p[1])
            projection_x_coords.append(cp[0])
            projection_y_coords.append(cp[1])

        error: float = np.sum(np.linalg.norm(positions - closest_projections, axis=1))
        rmse: float = np.sqrt(error / len(positions))

        for ((p1, cp1), (p2, cp2)) in sliding_window(zip(positions, closest_projections), 2):
            # plt.fill([p1[0], cp1[0], cp2[0], p2[0]], [p1[1], cp1[1],  cp2[1], p2[1]], color=color, alpha=0.3)
            # plt.fill([p])
            xs = [p1[0], p2[0], cp1[0], cp2[0]]
            ys = [p1[1], p2[1], cp1[1], cp2[1]]
            # xs = [p2[0], p1[0], cp2[0], cp1[0]]
            # ys = [p2[1], p1[1], cp2[1], cp1[1]]
            # xs = [cp1[0], cp2[0], p2[0], p1[0]]
            # ys = [cp1[1], cp2[1], p2[1], p1[1]]
            # xs = [cp2[0], cp1[0], p2[0], p1[0]]
            # ys = [cp2[1], cp1[1], p2[1], p1[1]]
            # xs = [cp2[0], cp1[0], p2[0], p1[0]]
            # ys = [cp2[1], cp1[1], p2[1], p1[1]]
            plt.fill(xs, ys, color=color, alpha=0.3)



        # print(f"{robot_id} RMSE: {rmse:.3f}")
        rmse_of_each_robot[robot_id] = rmse

    mean: float = statistics.mean(rmse_of_each_robot.values())
    median: float = statistics.median(rmse_of_each_robot.values())
    largest: float = max(rmse_of_each_robot.values())
    smallest: float = min(rmse_of_each_robot.values())
    variance: float = statistics.variance(rmse_of_each_robot.values())
    stdev: float = statistics.stdev(rmse_of_each_robot.values())
    N: int = len(rmse_of_each_robot)

    stats = dict(
        robots=N,
        mean=mean,
        median=median,
        largest=largest,
        smallest=smallest,
        variance=variance,
        stdev=stdev
    )

    headers = ['Robot ID', 'RMSE']
    table = [[robot_id, f"{rmse:.3f}"] for robot_id, rmse in rmse_of_each_robot.items()]
    tabulate_opts = dict(
        tablefmt="mixed_outline",
        showindex="always"
    )
    # print(tabulate(table, headers, tablefmt="mixed_outline"))
    print(tabulate(table, headers, **tabulate_opts))

    print(tabulate([stats], headers="keys", **tabulate_opts))



    if args.plot:
        plt.grid(True)
        plt.xlabel("x")
        plt.ylabel("y")
        plt.legend()
        plt.show()


if __name__ == '__main__':
    main()
