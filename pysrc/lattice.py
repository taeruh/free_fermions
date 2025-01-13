#!/usr/bin/env python

import json
import matplotlib.pyplot as plt
import numpy as np


def main():
    with open(f"output/periodic_square_lattice.json") as f:
        data = json.load(f)

    for key, value in data.items():
        data[key] = np.array(value) * 100  # percentage

    paper_setup()

    fig = plt.figure(figsize=set_size(height_in_width=0.7))
    gs = fig.add_gridspec(1, 1)
    ax = fig.add_subplot(gs[0, 0])
    axr = ax.twinx()
    inset = fig.add_axes((0.56, 0.70, 0.35, 0.25))

    rc_colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]
    colors = [rc_colors[0], rc_colors[2], rc_colors[3], rc_colors[5]]
    linestyles = ["solid", "dashed", "solid", "dotted"]
    labels = [
        "scf",
        "collapse gain",
        r"$\Delta$ before collapse",
        r"$\Delta$ after collapse",
    ]

    ax.set_ylabel(labels[0], color=colors[0])
    axr.set_ylabel(labels[1], color=colors[1])
    ax.tick_params(axis="y", labelcolor=colors[0])
    axr.tick_params(axis="y", labelcolor=colors[1])
    axr.spines['left'].set_color(colors[0])  # axr's spines overdraws ax's spines ...
    axr.spines['right'].set_color(colors[1])
    inset.set_ylabel(r"cf - scf $\Delta$")

    densities = data["densities"]
    density_len = len(densities)
    density_step = 20
    density_ticks = [data["densities"][d] for d in range(0, density_len, density_step)]

    ax.plot(
        densities,
        data["after_simplicial"],
        label=labels[0],
        color=colors[0],
        linestyle=linestyles[0],
    )
    axr.plot(
        densities,
        data["after_simplicial"] - data["before_simplicial"],
        label=labels[1],
        color=colors[1],
        linestyle=linestyles[1],
    )
    inset.plot(
        densities,
        data["before_claw_free"] - data["before_simplicial"],
        label=labels[2],
        color=colors[2],
        linestyle=linestyles[2],
    )
    inset.plot(
        densities,
        data["after_claw_free"] - data["after_simplicial"],
        label=labels[3],
        color=colors[3],
        linestyle=linestyles[3],
    )

    for a in [ax, axr, inset]:
        a.grid()
        ymax = a.get_ylim()[1]
        a.set_ylim(0, ymax)
        if a != axr:
            a.set_xlabel("density")

    # legend dummies
    for i in range(1, 4):
        ax.plot([], [], color=colors[i], linestyle=linestyles[i], label=labels[i])

    handles, labels = ax.get_legend_handles_labels()
    ax.legend(handles, labels, loc=(0.75, 0.42))

    plt.subplots_adjust(top=0.98, bottom=0.08, left=0.06, right=0.935)
    plt.savefig(f"output/periodic_square_lattice.pdf")


def paper_setup():
    plt.style.use(
        [
            "./pysrc/styles/ownstandard.mplstyle",
            "./pysrc/styles/ownlatex.mplstyle",
            # "./pysrc/styles/owndark.mplstyle",
        ]
    )
    plt.rcParams.update(
        {
            "figure.figsize": [*set_size()],
            "font.size": 9,
            "lines.linewidth": 1.5,
        }
    )


# get default with \the\textwidth
# def set_size(
#     width_in_pt=483.0, height_in_width=1.0, scale=1.0
# ):  # quantum 10 pt two-col
def set_size(width_in_pt=510.0, height_in_width=1.0, scale=1.0):  # revtex 10pt two-col
    width_in_in = width_in_pt * scale / 72.27
    return (width_in_in, width_in_in * height_in_width)


if __name__ == "__main__":
    main()
