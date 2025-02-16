import click
import matplotlib.pyplot as plt
import numpy as np

import json

from plot_levels import plot_level


@click.command()
@click.option("--name", help="Name of level to plot")
@click.option("--id", help="ID of level to plot")
@click.option("--width", default=0.2, help="Width of segment to cut", type=float)
@click.option("--line_idx", default=0, help="Line to cut", type=int)
@click.option("--point_idx", default=0, help="Point to cut", type=int)
@click.argument(
    "input_file", default="../data/default_levels.json", type=click.Path(exists=True)
)
@click.help_option("--help", "-h")
def cut_segment(name, id, width, line_idx, point_idx, input_file):
    with open(input_file, "r") as f:
        levels = json.load(f)

    # Filter levels by name or id
    if name:
        levels = [level for level in levels if level["name"] == name]
    elif id:
        levels = [level for level in levels if level["id"] == id]
    else:
        raise ValueError("Either name or id must be provided")

    if not levels:
        raise ValueError("Level not found")

    level = levels[0]
    plot_level(level)

    lines = level["body"]["shape"]["lines"]
    line = lines[line_idx]
    points = (line[point_idx], line[point_idx + 1])

    # Confirm length of segment is long enough
    x1, y1 = points[0]["x"], points[0]["y"]
    x2, y2 = points[1]["x"], points[1]["y"]
    segment_length = ((x2 - x1) ** 2 + (y2 - y1) ** 2) ** 0.5
    segment_unit_vector = ((x2 - x1) / segment_length, (y2 - y1) / segment_length)

    if segment_length < width:
        raise ValueError("Segment is too short to cut")

    # Calculate midpoint of segment
    x_mid = (x1 + x2) / 2
    y_mid = (y1 + y2) / 2

    # Move half width in each direction about the midpoint
    x1_new = x_mid - width / 2 * segment_unit_vector[0]
    y1_new = y_mid - width / 2 * segment_unit_vector[1]
    x2_new = x_mid + width / 2 * segment_unit_vector[0]
    y2_new = y_mid + width / 2 * segment_unit_vector[1]

    # Inject new points into line
    new_point1 = {"x": x1_new, "y": y1_new}
    new_point2 = {"x": x2_new, "y": y2_new}

    # Cut line into two and add points to start and end
    new_line1 = line[: point_idx + 1] + [new_point1]
    new_line2 = [new_point2] + line[point_idx + 1 :]
    lines[line_idx] = new_line1
    lines.insert(line_idx + 1, new_line2)

    # Plot new lines
    plot_level(level)

    plt.show()

    print(json.dumps(lines, indent=4))


if __name__ == "__main__":
    cut_segment()
