#!/usr/bin/env python

import json
import matplotlib.pyplot as plt
import numpy as np

data_dir = "output"
# data_dir = "results"
# file = "periodic_bricks_"
file = "periodic_bricks_full_"


def main():
    with open(f"{data_dir}/{file}0.json") as f:
        data = json.load(f)

    densities = data["densities"]
    density_len = len(densities)

    max_sc_size = 0

    results = {
        "before_claw_free": np.array(np.zeros(density_len)),
        "after_claw_free": np.array(np.zeros(density_len)),
        "before_simplicial": np.array(np.zeros(density_len)),
        "after_simplicial": np.array(np.zeros(density_len)),
        "collapsed": np.array(np.zeros(density_len)),
    }

    num_sample_files = 20
    num_total_samples = 0

    for i in range(num_sample_files):
        try:
            with open(f"{data_dir}/{file}{i}.json") as f:
                data = json.load(f)
        except FileNotFoundError:
            print(f"File {i} not found")
            continue
        try:
            max_sc_size = max(max_sc_size, data["max_sc_size"])
        except KeyError:
            pass
        num_samples = data["num_samples"]
        num_total_samples += num_samples
        for key, value in results.items():
            value += num_samples * np.array(data[key])

    print(f"num_total_samples: {num_total_samples}")
    for key, value in results.items():
        value *= 100.0 / num_total_samples  # percentage

    print(f"max_sc_size: {max_sc_size}")

    paper_setup()

    fig = plt.figure(figsize=set_size(height_in_width=0.7))
    gs = fig.add_gridspec(1, 1)
    ax = fig.add_subplot(gs[0, 0])
    axr = ax.twinx()
    inset = fig.add_axes((0.56, 0.63, 0.35, 0.25))

    rc_colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]
    colors = [rc_colors[0], rc_colors[2], rc_colors[3], rc_colors[5]]
    linestyles = [
        "solid",
        "dashed",
        "dotted",
        # "solid",
        # "dotted",
    ]
    labels = [
        r"scf",
        r"collapse gain",
        r"collapsed",
        # r"$\Delta$ before collapse",
        # r"$\Delta$ after collapse",
    ]

    ax.set_ylabel(labels[0], color=colors[0])
    axr.set_ylabel(labels[1], color=colors[1])
    ax.tick_params(axis="y", labelcolor=colors[0])
    axr.tick_params(axis="y", labelcolor=colors[1])
    axr.spines["left"].set_color(colors[0])  # axr's spines overdraws ax's spines ...
    axr.spines["right"].set_color(colors[1])

    density_step = 20
    density_ticks = [densities[d] for d in range(0, density_len, density_step)]

    ax.plot(
        densities,
        results["after_simplicial"],
        label=labels[0],
        color=colors[0],
        linestyle=linestyles[0],
    )
    axr.plot(
        densities,
        results["after_simplicial"] - results["before_simplicial"],
        label=labels[1],
        color=colors[1],
        linestyle=linestyles[1],
    )

    # inset.set_ylabel(r"cf - scf $\Delta$")
    # inset.plot(
    #     densities,
    #     results["before_claw_free"] - results["before_simplicial"],
    #     label=labels[2],
    #     color=colors[2],
    #     linestyle=linestyles[2],
    # )
    # inset.plot(
    #     densities,
    #     results["after_claw_free"] - results["after_simplicial"],
    #     label=labels[3],
    #     color=colors[3],
    #     linestyle=linestyles[3],
    # )

    print(
        f"before sum(|scf - cf|) = {np.abs(results["before_claw_free"] -
        results["before_simplicial"]).sum()}"
    )
    print(
        f"after  sum(|scf - cf|) = {np.abs(results["after_claw_free"] -
        results["after_simplicial"]).sum()}"
    )

    inset.set_ylabel(r"collapsed in ")
    inset.plot(
        densities,
        results["collapsed"],
        label=labels[2],
        color=colors[2],
        linestyle=linestyles[2],
    )

    for a in [ax, axr, inset]:
        a.grid()
        ymax = a.get_ylim()[1]
        a.set_ylim(0, ymax)
        if a != axr:
            a.set_xlabel("density")

    # legend dummies
    # for i in range(1, 4):
    for i in range(1, 3):
        ax.plot([], [], color=colors[i], linestyle=linestyles[i], label=labels[i])

    handles, labels = ax.get_legend_handles_labels()
    ax.legend(handles, labels, loc=(0.77, 0.40))

    # plt.subplots_adjust(top=0.98, bottom=0.08, left=0.06, right=0.935)
    plt.savefig(f"output/periodic_bricks.pdf")


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
