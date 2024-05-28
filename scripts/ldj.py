#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.numpy python3Packages.scipy python3Packages.rich python3Packages.tabulate python3Packages.matplotlib

import sys
import statistics
import json
import argparse
from pathlib import Path

import numpy as np
from scipy.integrate import simpson
import matplotlib.pyplot as plt
from rich import print, inspect, pretty
from tabulate import tabulate

pretty.install()

print(f"{sys.executable = }")
print(f"{sys.version = }")

def ldj(velocities: np.ndarray, timesteps: np.ndarray) -> float:
    """ Calculate the Log Dimensionless Jerk (LDJ) metric. """
    assert len(velocities) > 0
    assert velocities.shape == (len(velocities), 2)
    assert len(timesteps) == len(velocities)
    assert np.all(np.diff(timesteps) > 0)
    assert timesteps.shape == (len(timesteps),)

    t_start: float = timesteps[0]
    t_final: float = timesteps[-1]
    assert t_start < t_final

    # dt: float = (t_final - t_start) / len(velocities)
    dt: float = np.mean(np.diff(timesteps))
    # dt: float = np.mean(timesteps)
    vx = velocities[:, 0]
    vy = velocities[:, 1]
    # Estimate acceleration components
    ax = np.gradient(vx, dt)
    ay = np.gradient(vy, dt)

    # Estimate jerk components
    jx = np.gradient(ax, dt)
    jy = np.gradient(ay, dt)

    # Square of the jerk magnitude
    squared_jerk = jx**2 + jy**2

    time_samples = np.linspace(t_start, t_final, len(velocities))

    # Numerical integration of the squared jerk using Simpson's rule
    integral_squared_jerk = simpson(squared_jerk, x=time_samples)

    # LDJ calculation
    v_max = np.max(np.sqrt(vx**2 + vy**2))  # Max speed (magnitude of velocity vector)

    ldj = -np.log((t_final - t_start)**3 / v_max**2 * integral_squared_jerk)

    return ldj


def plot_ldj(ldjs):
    plt.boxplot(ldjs)
    plt.title('Log Dimensionless Jerk by Robots')
    plt.ylabel('LDJ')
    plt.xlabel('Robots')
    plt.ylim( -25, 0)
    num_measurements = len(ldjs)
    plt.xticks([1], [str(num_measurements)])

    plt.show()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('-i', '--input', type=Path)
    parser.add_argument('-p', '--plot', action='store_true')
    args = parser.parse_args()

    data = json.loads(args.input.read_text())

    ldj_of_each_robot: dict[int, float] = {}
    for robot_id, robot_data in data['robots'].items():
        # print(f"{robot_data=}")
        # sys.exit(0)
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
        ldj_of_each_robot[robot_id] = metric
        # print(f"{robot_id} t_start: {t_start:.3f} t_final: {t_final:.3f} #velocities: {len(velocities)} LDJ: {metric:.3f}")

    headers = ['Robot ID', 'LDJ']
    table = [[robot_id, f"{ldj:.3f}"] for robot_id, ldj in ldj_of_each_robot.items()]
    tabulate_opts = dict(
        tablefmt="mixed_outline",
        showindex="always"
    )
    # print(tabulate(table, headers, tablefmt="mixed_outline"))
    print(tabulate(table, headers, **tabulate_opts))

    mean: float = statistics.mean(ldj_of_each_robot.values())

    mean: float = statistics.mean(ldj_of_each_robot.values())
    median: float = statistics.median(ldj_of_each_robot.values())
    largest: float = max(ldj_of_each_robot.values())
    smallest: float = min(ldj_of_each_robot.values())
    variance: float = statistics.variance(ldj_of_each_robot.values())
    stdev: float = statistics.stdev(ldj_of_each_robot.values())
    N: int = len(ldj_of_each_robot)

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

    if args.plot:
        plot_ldj(ldj_of_each_robot.values())


if __name__ == '__main__':
    main()
