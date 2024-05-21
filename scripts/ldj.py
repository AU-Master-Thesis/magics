#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3Packages.numpy python3Packages.scipy python3Packages.rich

import json
import sys
import argparse
from pathlib import Path

import numpy as np
from scipy.integrate import simpson
from rich import print, inspect, pretty

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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('json_file', type=Path)
    args = parser.parse_args()

    data = json.loads(args.json_file.read_text())

    for robot_id, robot_data in data['robots'].items():
        route = robot_data['route']
        t_start: float = route['started_at']
        t_final: float = route['finished_at'] if route['finished_at'] else route['duration'] + t_start
        timestamps: np.ndarray = np.array([measurement['timestamp'] for measurement in robot_data['velocities']])
        velocities3d_bevy: np.ndarray = np.array([measurement['velocity'] for measurement in robot_data['velocities']])
        velocities = velocities3d_bevy[:, [0, 2]]

        metric = ldj(velocities, timestamps)
        print(f"{robot_id} t_start: {t_start:.3f} t_final: {t_final:.3f} #velocities: {len(velocities)} LDJ: {metric:.3f}")

if __name__ == '__main__':
    main()
