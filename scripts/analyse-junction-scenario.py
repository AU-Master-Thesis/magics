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


import json
import sys
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

# use LaTeX for text with matplotlib
plt.rcParams.update({
    "text.usetex": True,
    "font.family": "sans-serif",
    "font.sans-serif": "Helvetica",
})

sns.set_theme()
pretty.install()

RESULTS_DIR = Path('./experiments/junction')
assert RESULTS_DIR.is_dir() and RESULTS_DIR.exists()

flavor = PALETTE.latte.colors

def flatten(lst: Iterable) -> list:
    return list(itertools.chain.from_iterable(lst))

def extract_data_from_file(file_path):
    with open(file_path, 'r') as file:
        data = json.load(file)
    # Assuming the JSON structure and extracting necessary information
    # Replace 'key' with actual key names in your JSON files

    durations: list[float] = []
    bins: list[int] = []


    goal_areas: list = [v for _, v in data['goal_areas'].items()]
    # print(f"{goal_areas=}")

    makespan = data['makespan']
    num_of_robots_reached_goal: int = sum((len(goal_area['history']) for goal_area in goal_areas))
    num_robots: int = len(data['robots'])
    # print(f"{num_of_robots_reached_goal=}, {num_robots=} {makespan=}")

    robot_ids: set[str] = set(data['robots'].keys())
    robots_not_reached_goal: set[str] = robot_ids - set(flatten((goal_area['history'].keys() for goal_area in goal_areas)))
    # print(f"{robots_not_reached_goal=}")

    # print the start time of the robots not reached goal
    # for entity, robot_data in data['robots'].items():
    #     if entity in robots_not_reached_goal:
    #         print(f"{entity=} {robot_data['mission']['started_at']=}")


    # start_at: float = 6.7
    ignore_after: float = 50.0
    num_robots_reached_goal: int = 0
    num_robots_after_ten_secs: int = 0
    for entity, robot_data in data['robots'].items():
        started_at: float = robot_data['mission']['started_at']
        if started_at >= ignore_after:
        # if started_at <= start_at:
            continue

        reached_goal: bool = False
        for goal_area in goal_areas:
            # print(f"{goal_area=}")
            for key, reached_at in goal_area['history'].items():
                if entity == key:
                    reached_goal = True
                    break

        if not reached_goal:
            continue

        num_robots_reached_goal += 1
        num_robots_after_ten_secs += 1

    # print(f"{num_robots=} {num_robots_reached_goal=} {num_robots_after_ten_secs=}")


    # t: float = makespan -  start_at
    t: float = ignore_after
    return (1 / (t / num_robots_reached_goal))


    num_robo
    # for entity, robot_data in data['robots'].items():
    #     started_at: float = robot_data['mission']['started_at']
    #     reached_goal_at: float | None = None
    #     for goal_area in goal_areas:
    #         # print(f"{goal_area=}")
    #         for key, reached_at in goal_area['history'].items():
    #             if entity == key:
    #                 reached_goal_at = reached_at
    #                 break
    #
    #     if reached_goal_at is None:
    #         continue
    #
    #     duration = reached_goal_at - started_at
    #     durations.append(duration)

        # if started_at < 6.67:
        #     continue

        # finished_at: float | None = robot_data['mission']['finished_at']
        # if finished_at is None:
        #     continue
        #
        # bin: int = math.floor(finished_at)
        #
        # if bin >= len(bins):
        #     bins += [0 for _ in range(bin - len(bins) + 1)]
        #
        # bins[bin] += 1
        #

        # duration = finished_at - started_at
        # durations.append(duration)

    # bins = list(itertools.dropwhile(lambda x: x == 0, bins))
    # bins = bins[:-1]

    # return durations
    # return bins

def process_file(file):
    qin_value = float(file.stem.split('-')[1])
    extracted_value = extract_data_from_file(file)
    return qin_value, extracted_value

def main():
    print(f"{sys.executable = }")
    print(f"{sys.version = }")

    aggregated_data = collections.defaultdict(list)

    with ProcessPoolExecutor() as executor:
        results = executor.map(process_file, RESULTS_DIR.glob('qin-*.json'))

    # Aggregate results in a single-threaded manner to avoid data races
    for Qin, extracted_value in results:
        aggregated_data[Qin].append(extracted_value)

    xs: list[float] = []
    ys: list[float] = []
    for Qin, values in sorted(aggregated_data.items(), key=lambda x: x[0]):
        avg = sum(values) / len(values)
        ys.append(avg)
        xs.append(Qin)
        print(f"{Qin=} {avg=}")

    xs.insert(0, 0.)
    ys.insert(0, 0.)

    # plt.plot(xs, xs, color=flavor.red.hex)
    # plt.plot(range(0, 8), range(0, 8), linestyle='--', dashes=(10, 5), color=flavor.overlay2.hex, legend='Ideal')
    # plt.plot([0., 7.], [0., 7.], linestyle='--', dashes=(10, 5), color=flavor.overlay2.hex, label='Ideal')
    x_max, y_max = (8., 8.)
    plt.plot([0., x_max], [0., y_max], linestyle='--', dashes=(10, 5), color=flavor.overlay2.hex, label="$Q_{in} = Q_{out}$")
    plt.plot(xs[:-1], ys[:-1], marker='o', color=flavor.lavender.hex, label='Average flowrate over $50s$')
    plt.plot(xs[-2:], ys[-2:], marker='x', linestyle='--', color=flavor.lavender.hex)
    plt.xlabel(r'Input Flowrate $Q_{in}$ [robots/s]', fontsize=12)
    plt.ylabel(r'Output Flowrate $Q_{out}$ [robots/s]', fontsize=12)
    plt.tick_params(axis='both', which='major', labelsize=10)
    plt.ylim(0, 7)
    plt.xlim(0, 7)
    plt.xticks(np.arange(0, x_max + 0.5, 0.5))
    plt.yticks(np.arange(0, y_max + 0.5, 0.5))
    # plt.aspect(1 / 1.414) # A4 paper
    plt.gca().set_aspect(1 / 1.414) # A4 paper
    legend = plt.legend(borderpad=0.5, framealpha=0.8, frameon=True)
    # legend = plt.legend(loc='upper left', bbox_to_anchor=(1.05, 1), borderaxespad=0.)
    # legend.get_frame().set_facecolor('lightgrey')  # Change background color
    legend.get_frame().set_facecolor(flavor.surface0.hex)  # Change background color
    # plt.legend()
    plt.savefig('qin-vs-qout.svg')
    plt.show()

    return 0

if __name__ == '__main__':
    sys.exit(main())
