#!/usr/bin/env nix-shell
#! nix-shell -i python3
#! nix-shell -p python3Packages.numpy
#! nix-shell -p python3Packages.scipy
#! nix-shell -p python3Packages.rich
#! nix-shell -p python3Packages.tabulate
#! nix-shell -p python3Packages.matplotlib
#! nix-shell -p python3Packages.toolz
#! nix-shell -p python3Packages.seaborn
#! nix-shell -p python3Packages.result
#! nix-shell -p python3Packages.pretty-errors
#! nix-shell -p python3Packages.seaborn
#! nix-shell -p python3Packages.catppuccin
#! nix-shell -p texliveFull

import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), 'scripts'))

import re
import json
import argparse
import itertools
from pathlib import Path
import collections
from concurrent.futures import ProcessPoolExecutor

import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

from rich import print, pretty
from typing import  Iterable
import pretty_errors
from catppuccin import PALETTE

# import .scripts.ldj
from ldj import ldj

# use LaTeX for text with matplotlib
plt.rcParams.update({
    "text.usetex": True,
    "font.family": "sans-serif",
    "font.sans-serif": "Helvetica",
})

sns.set_theme()
pretty.install()

RESULTS_DIR = Path('./experiments/circle')
assert RESULTS_DIR.is_dir() and RESULTS_DIR.exists()

flavor = PALETTE.latte.colors
# num-robots-10-seed-0.json
RE = re.compile(r"num-robots-(\d+)-seed-(\d+).json")

def flatten(lst: Iterable) -> list:
    return list(itertools.chain.from_iterable(lst))

def process_file(file):
    match = RE.match(file.name)
    assert match is not None
    num_robots = int(match.group(1))
    seed = int(match.group(2))

    with open(file, 'r') as file:
        data = json.load(file)

    distance_travelled_of_each_robot: list[float] = []
    ldj_of_each_robot: list[float] = []

    for _, robot_data in data['robots'].items():
        positions = np.array(robot_data['positions'])
        # print(f"{positions.shape=}")
        # assert positions.shape == (num_robots, 2)
        # n x 2 matrix
        # sum the length between each pair of points
        distance_travelled = np.sum(np.linalg.norm(np.diff(positions, axis=0), axis=1))
        # print(f"{distance_travelled=}")
        distance_travelled_of_each_robot.append(distance_travelled)
        mission = robot_data['mission']
        # mission = robot_data.get("mission", None)
        # if not mission:
        #     continue
        t_start: float = mission['started_at']
        t_final: float = mission['finished_at'] if mission['finished_at'] else mission['duration'] + t_start
        timestamps: np.ndarray = np.array([measurement['timestamp'] for measurement in robot_data['velocities']])
        velocities3d_bevy: np.ndarray = np.array([measurement['velocity'] for measurement in robot_data['velocities']])
        velocities = velocities3d_bevy[:, [0, 2]]

        metric = ldj(velocities, timestamps)
        ldj_of_each_robot.append(metric)

    makespan: float = data['makespan']
    return num_robots, distance_travelled_of_each_robot, makespan, ldj_of_each_robot

    # for robot_id, robot_data in data['robots'].items():
    #     # print(f"{robot_data=}")
    #     # sys.exit(0)
    #     mission = robot_data['mission']
    #     # mission = robot_data.get("mission", None)
    #     # if not mission:
    #     #     continue
    #     t_start: float = mission['started_at']
    #     t_final: float = mission['finished_at'] if mission['finished_at'] else mission['duration'] + t_start
    #     timestamps: np.ndarray = np.array([measurement['timestamp'] for measurement in robot_data['velocities']])
    #     velocities3d_bevy: np.ndarray = np.array([measurement['velocity'] for measurement in robot_data['velocities']])
    #     velocities = velocities3d_bevy[:, [0, 2]]
    #
    #     metric = ldj(velocities, timestamps)
    #     ldj_of_each_robot[robot_id] = metric


def main():
    print(f"{sys.executable = }")
    print(f"{sys.version = }")

    aggregated_data_distance_travelled = collections.defaultdict(list)

    with ProcessPoolExecutor() as executor:
        results = executor.map(process_file, RESULTS_DIR.glob('*.json'))

    # Aggregate results in a single-threaded manner to avoid data races
    aggregated_data_distance_travelled: dict[int, list[float]] = collections.defaultdict(list)
    aggregated_data_makespan: dict[int, list[float]] = collections.defaultdict(list)
    aggregated_data_ldj: dict[int, list[float]] = collections.defaultdict(list)

    for num_robots, distance_travelled_for_each_robot, makespan, ldj_for_each_robot in results:
        aggregated_data_distance_travelled[num_robots].extend(distance_travelled_for_each_robot)
        aggregated_data_makespan[num_robots].append(makespan)
        aggregated_data_ldj[num_robots].extend(ldj_for_each_robot)

    data = [aggregated_data_distance_travelled[key] for key in sorted(aggregated_data_distance_travelled.keys())]
    labels = sorted(aggregated_data_distance_travelled.keys())

    a4_ratio = 1 / 1.414
    fig, ax = plt.subplots(figsize=(8 * a4_ratio, 8))

# showmeans=False, showfliers=False,
#                 medianprops={"color": "white", "linewidth": 0.5},
#                 boxprops={"facecolor": "C0", "edgecolor": "white",
#                           "linewidth": 0.5},
#                 whiskerprops={"color": "C0", "linewidth": 1.5},
#                 capprops={"color": "C0", "linewidth": 1.5})
    boxplot_opts = dict(
        showmeans=False, showfliers=True,
        # medianprops={"color": flavor.blue.hex, "linewidth": 0.5},
        medianprops=dict(color=flavor.blue.hex, linewidth=0.5),
        boxprops=dict(linestyle='-', linewidth=2, color=flavor.lavender.hex),
        whiskerprops=dict(color=flavor.lavender.hex, linewidth=1.5),
        capprops=dict(color=flavor.lavender.hex, linewidth=1.5),
        flierprops=dict(marker='D', color=flavor.lavender.hex, markersize=8)
    )

    ax.boxplot(data, labels=labels, **boxplot_opts)

    # violin_parts = plt.violinplot(data, showmeans=False, showmedians=True)

    ax.set_xlabel(r'Number of Robots $N_R$', fontsize=12)
    ax.set_ylabel(r'Distance Travelled $[m]$', fontsize=12)
    ax.tick_params(axis='both', which='major', labelsize=10)

    ax.set_ylim(0, 250)
    # ax.set_aspect(1 / 1.414) # A4 paper

    # Draw optimal line
    ax.axhline(y=100, color=flavor.overlay2.hex, linestyle='--', linewidth=1.5)

    plt.tight_layout()
    plt.savefig('circle-experiment-distance-travelled.svg')

    # fig, ax = plt.subplots(figsize=(8 * a4_ratio, 8))

    a4_width = 8.27
    a4_height = 11.69
    fig, ax = plt.subplots(figsize=(a4_width, a4_height))
    data = [aggregated_data_makespan[key] for key in sorted(aggregated_data_makespan.keys())]
    labels = sorted(aggregated_data_makespan.keys())
    data = [np.mean(makespan) for makespan in data]
    ax.plot(labels, data, marker='o', color=flavor.lavender.hex, label='Circle Scenario')
    ax.set_ylim(0, 200)
    # ax.set_aspect(1 / 1.414) # A4 paper
    # ax.boxplot(data, labels=labels, flierprops=dict(marker='D', color='r', markersize=8))

    ax.set_xlabel(r'Number of Robots $N_R$', fontsize=12)
    ax.set_ylabel(r'Makespan $[s]$', fontsize=12)
    ax.tick_params(axis='both', which='major', labelsize=10)
    legend = ax.legend(borderpad=0.5, framealpha=0.8, frameon=True)
    legend.get_frame().set_facecolor(flavor.surface0.hex)  # Change background color
    # plt.tight_layout()
    plt.savefig('circle-experiment-makespan.svg')

    fig, ax = plt.subplots(figsize=(8 * a4_ratio, 8))
    data = [aggregated_data_ldj[key] for key in sorted(aggregated_data_ldj.keys())]
    labels = sorted(aggregated_data_ldj.keys())
    ax.boxplot(data, labels=labels, **boxplot_opts)

    ax.set_ylim(-25, 0)
    # ax.set_aspect(1 / 1.414) # A4 paper
    ax.set_xlabel(r'Number of Robots $N_R$', fontsize=12)
    ax.set_ylabel(r'Log Dimensionless Jerk $[m/s^3]$', fontsize=12)
    ax.tick_params(axis='both', which='major', labelsize=10)

    plt.tight_layout()
    plt.savefig('circle-experiment-ldj.svg')

    plt.show()


if __name__ == '__main__':
    sys.exit(main())
