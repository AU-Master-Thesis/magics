#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.numpy python3Packages.rich python3Packages.matplotlib python3Packages.tabulate

import json
import sys
import argparse
from pathlib import Path
import statistics

import numpy as np
import matplotlib.pyplot as plt
from rich import print, inspect, pretty
from tabulate import tabulate
# import asciichartpy as ac


pretty.install()

print(f"{sys.executable = }")
print(f"{sys.version = }")

def plot_distances(distances):
    plt.boxplot(distances)
    plt.title('Distance Travelled by Robots')
    plt.ylabel('Distance')
    plt.xlabel('Robots')
    plt.show()

def distance_travelled(positions: np.ndarray) -> float:
    assert len(positions) > 0
    assert positions.shape == (len(positions), 2)

    diffs = np.diff(positions, axis=0)
    distances = np.linalg.norm(diffs, axis=1)
    total_distance = np.sum(distances)
    return total_distance


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('json_file', type=Path)
    parser.add_argument('-b', '--best-possible', type=float, default=None)
    args = parser.parse_args()

    data = json.loads(args.json_file.read_text())

    distance_travelled_of_each_robot: dict[int, float] = {}
    for robot_id, robot_data in data['robots'].items():
        positions = np.array([p for p in robot_data['positions']])

        d = distance_travelled(positions)
        if args.best_possible is not None:
            d = max(d, args.best_possible)
        distance_travelled_of_each_robot[robot_id] = d

    headers = ['Robot ID', 'Distance Travelled (m)']
    table = [[robot_id, f"{d:.3f}"] for robot_id, d in distance_travelled_of_each_robot.items()]
    tabulate_opts = dict(
        tablefmt="mixed_outline",
        showindex="always"
    )
    # print(tabulate(table, headers, tablefmt="mixed_outline"))
    print(tabulate(table, headers, **tabulate_opts))

    mean: float = statistics.mean(distance_travelled_of_each_robot.values())
    median: float = statistics.median(distance_travelled_of_each_robot.values())
    largest: float = max(distance_travelled_of_each_robot.values())
    smallest: float = min(distance_travelled_of_each_robot.values())
    variance: float = statistics.variance(distance_travelled_of_each_robot.values())
    stdev: float = statistics.stdev(distance_travelled_of_each_robot.values())
    N: int = len(distance_travelled_of_each_robot)

    stats = dict(
        robots=N,
        mean=mean,
        median=median,
        largest=largest,
        smallest=smallest,
        variance=variance,
        stdev=stdev
    )

    print(tabulate([stats], headers="keys", **tabulate_opts))

    # plot_distances(distance_travelled_of_each_robot.values())

if __name__ == '__main__':
    main()
