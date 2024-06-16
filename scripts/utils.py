
# rgba in [0, 255] -> [0, 1]
import collections
import colorsys
from dataclasses import dataclass
import itertools
import json
from typing import Iterable, List

import numpy as np
from ldj import ldj


rgba_to_float = lambda rgba: [x / 255 for x in rgba]
# rgba tuple (r, g, b, a) in [0, 1], hue in [0, 1]
# rgba_add_hue = lambda rgba, hue_offset: (*sns.color_palette([rgba], 1, 1)[0], rgba[3])
def adjust_hue(rgba, hue_offset):
    """
    Adjusts the hue of an RGBA color.
    rgba: A tuple of (r, g, b, a) where each component is between 0 and 1.
    hue_offset: Hue offset in degrees (0 to 360).
    Returns: A new RGBA color with adjusted hue.
    """
    # Convert hue offset from degrees to a scale of 0-1
    hue_offset = hue_offset / 360.0

    # Unpack the RGBA values
    r, g, b, a = rgba

    # Convert RGB to HSV
    h, s, v = colorsys.rgb_to_hsv(r, g, b)

    # Adjust hue, ensuring it wraps around
    h = (h + hue_offset) % 1.0

    # Convert back to RGB
    r, g, b = colorsys.hsv_to_rgb(h, s, v)

    # Return the new color with the original alpha value
    return (r, g, b, a)

def set_alpha(rgba, alpha):
    return (*rgba[:3], alpha)

def lighten(rgba, factor):
    return (*[x + (1 - x) * factor for x in rgba[:3]], rgba[3])
    # return ([x + (1 - x) * factor for x in rgba[:3]])

def hex_to_rgba(hex):
    return rgba_to_float([int(hex[i:i+2], 16) for i in (1, 3, 5)])

# lighten hex
def lighten_hex(hex, factor):
    rgba = hex_to_rgba(hex)
    return rgba_to_hex(lighten(rgba, factor))

def darken(rgba, factor):
    return (*[x - x * factor for x in rgba[:3]], rgba[3])

def rgba_to_hex(rgba):
    return '#' + ''.join(f'{int(x * 255):02x}' for x in rgba[:3])


def flatten(lst: Iterable) -> list:
    return list(itertools.chain.from_iterable(lst))

def sliding_window(iterable: Iterable, n: int):
    "Collect data into overlapping fixed-length chunks or blocks."
    # sliding_window('ABCDEFG', 4) → ABCD BCDE CDEF DEFG
    iterator = iter(iterable)
    window = collections.deque(itertools.islice(iterator, n - 1), maxlen=n)
    for x in iterator:
        window.append(x)
        yield tuple(window)

def process_file(file, RE):
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

@dataclass
class LinePoints:
    start: List[float]
    end: List[float]

def line_from_line_segment(x1: float, y1: float, x2: float, y2: float) -> Line:
    a = (y2 - y1) / (x2 - x1)
    b = y1 - (a * x1)
    return Line(a=a, b=b)

# def projection_onto_line(point: np.ndarray, line: Line) -> np.ndarray:
#     assert len(point) == 2
#     x1, y1 = point
#     xp: float = (x1 + line.a * (y1 - line.b)) / (line.a * line.a + 1)
#     projection = np.array([xp, line.a * xp + line.b])
#     assert len(projection) == 2

#     return projection

def projection_onto_line(point: np.ndarray, line: LinePoints) -> np.ndarray:
    start = np.array(line.start)
    end = np.array(line.end)

    line_vector = end - start

    # projection in rust:
    # &current_start + (&x_pos - &current_start).dot(&line) / &line.dot(&line) * &line;
    projection = start + np.dot(point - start, line_vector) / np.dot(line_vector, line_vector) * line_vector
    return projection



def line_is_valid(line_point: LinePoints, point: np.ndarray) -> bool:
    lp1 = np.array(line_point.start)
    lp2 = np.array(line_point.end)

    v1 = point - lp1
    v2 = point - lp2

    return abs(np.arctan2(v1[0], v1[1]) - np.arctan2(v2[0], v2[1])) >= np.pi / 2

def closest_projection_onto_line_segments(point: np.ndarray, lines: list[LinePoints]) -> np.ndarray:
    assert len(point) == 2

    # lines: List[Line] = [line_from_line_segment(*line_points[0].start, *line_points[0].end) for line in line_points]

    # sort out lines that the point is not between
    valid_lines = [
        line
        for line in lines
        if line_is_valid(line, point)
    ]

    # print(f"{valid_lines=}")

    valid_lines = valid_lines if valid_lines else lines
    projections = [projection_onto_line(point, line) for line in valid_lines]
    closest_projection = min(projections, key=lambda proj: np.linalg.norm(proj - point))

    min_distance = np.linalg.norm(closest_projection - point)

    if min_distance > 10:
        valid_lines = lines
        projections = [projection_onto_line(point, line) for line in valid_lines]
        closest_projection = min(projections, key=lambda proj: np.linalg.norm(proj - point))

    return closest_projection