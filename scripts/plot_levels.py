import click
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import tabulate

import json


def plot_level(level):
    # Plot level
    plt.figure()
    lines = level["body"]["shape"]["lines"]
    for idx, line in enumerate(lines):
        line_x = [point["x"] for point in line]
        line_y = [point["y"] for point in line]
        plt.plot(line_x, line_y, label=idx)
        offset_angle = 2.0 * np.pi * idx / len(lines)
        offset = (10 * np.cos(offset_angle), 10 * np.sin(offset_angle))
        for idx, point in enumerate(line):
            plt.annotate(
                idx,
                (point["x"], point["y"]),
                textcoords="offset pixels",
                xytext=offset,
            )
    plt.title(level["name"])
    plt.axis("equal")
    plt.legend()
    plt.grid()


@click.command()
@click.option("--name", help="Name of level to plot")
@click.option("--id", help="ID of level to plot")
@click.argument(
    "input_file", default="../data/default_levels.json", type=click.Path(exists=True)
)
@click.help_option("--help", "-h")
def plot_levels(name, id, input_file):
    with open(input_file, "r") as f:
        levels = json.load(f)

    # Filter levels by name or id
    if name:
        levels = [level for level in levels if level["name"] == name]

    if id:
        levels = [level for level in levels if level["id"] == id]

    # Print level names and IDs as table with pandas
    df = pd.DataFrame(levels)
    print(
        tabulate.tabulate(df[["id", "name"]], headers="keys", tablefmt="fancy_outline")
    )

    for level in levels:
        plot_level(level)

    plt.show()


if __name__ == "__main__":
    plot_levels()
