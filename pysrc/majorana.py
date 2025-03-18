#!/usr/bin/env python

import json
import matplotlib.pyplot as plt
import matplotlib
import numpy as np

data_dir = "output"
# data_dir = "results"
file = "e_structure_"


def main():
    first = Data("first_")
    second = Data("second_")
    data = [first, second]

    paper_setup()

    fig = plt.figure(figsize=set_size(height_in_width=1.6))
    factor = 10
    space = 3
    gs = fig.add_gridspec(4 * factor + space, 1)
    axs = [
        [fig.add_subplot(gs[0:factor, 0]), fig.add_subplot(gs[factor : 2 * factor, 0])],
        [
            fig.add_subplot(gs[2 * factor + space : 3 * factor + space, 0]),
            fig.add_subplot(gs[3 * factor + space : 4 * factor + space, 0]),
        ],
    ]

    ranges = [
        range(1, 5),
        range(0, 4),
    ]

    color_map = matplotlib.colormaps["plasma"]
    colors = [
        color_map(i)
        for i in np.linspace(0.0, 0.95, len(ranges[0]) + len(ranges[1]) - 1)
    ]
    print(len(colors))

    linestyles = [
        "dashed",
        "solid",
        "dotted",
    ]
    labels = [
        r"$p_{\mathrm{SCF}}$",
        r"$\Delta p_{\mathrm{SCF}}$",
    ]

    for i, (color_offset, orbit_range) in enumerate(
        zip([-1, len(ranges[0]) - 1], ranges)
    ):
        for j in orbit_range:
            axs[i][0].plot(
                data[i].densities,
                data[i].simplicial[j],
                label=f"$m = {round(data[i].sizes[j]/2)}$",
                linestyle=linestyles[0],
                color=colors[color_offset + j],
            )
        for j in orbit_range:
            axs[i][1].plot(
                data[i].densities,
                data[i].delta_simplicial[j],
                label=f"$m = {round(data[i].sizes[j]/2)}$",
                linestyle=linestyles[1],
                color=colors[color_offset + j],
            )
            axs[i][1].plot(
                data[i].densities,
                data[i].collapsed[j],
                label=f"$m = {round(data[i].sizes[j]/2)}$",
                linestyle=linestyles[2],
                color=colors[color_offset + j],
            )
        handles, labels = axs[i][0].get_legend_handles_labels()
        axs[i][0].legend(handles, labels, loc="upper right")

    axs[0][1].set_yticks([0, 50, 100])
    axs[0][1].set_yticklabels(["0", "50", "100"])

    # ymax = ao.get_ylim()[1]
    # ao.set_ylim(0, ymax)
    # for ax in [ao, au]:
    #     # ax.set_ylabel(label)
    #     ax.set_xlim(0, 1)
    #     ax.grid()
    #     ax.tick_params(axis="x", which="both", bottom=True, top=True, labelbottom=True)
    #     handles, labels = ax.get_legend_handles_labels()
    #     ax.legend(handles, labels, loc="upper right")

    # au.set_xlabel(r"$d$")

    plt.subplots_adjust(top=0.96, bottom=0.13, left=0.14, right=0.960)

    plt.savefig(f"output/e_structure.pdf")


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


class Data:
    def __init__(self, suffix: str):
        thisfile = f"{file}{suffix}"
        with open(f"{data_dir}/{thisfile}1.json") as f:
            data = json.load(f)

        self.densities = data["densities"]
        self.sizes = data["sizes"]
        self.density_len = len(self.densities)
        self.size_len = len(self.sizes)

        # num_sample_files = 20
        num_sample_files = 1
        num_total_samples = 0

        self.simplicial = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )
        before_simplicial = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )

        for i in range(1, num_sample_files + 1):
            try:
                with open(f"{data_dir}/{thisfile}{i}.json") as f:
                    data = json.load(f)
            except FileNotFoundError:
                print(f"File {file}{i} not found")
                continue
            num_samples = data["num_samples"]
            num_total_samples += num_samples
            self.simplicial += num_samples * np.array(data["after_simplicial"])
            before_simplicial += num_samples * np.array(data["before_simplicial"])

        self.simplicial /= num_total_samples
        before_simplicial /= num_total_samples
        self.delta_simplicial = (self.simplicial - before_simplicial) * 100
        self.collapsed = np.array(data["collapsed"]) * 100


# get default with \the\textwidth
# def set_size(
#     width_in_pt=483.0, height_in_width=1.0, scale=1.0
# ):  # quantum 10 pt two-col
# def set_size(width_in_pt=510.0, height_in_width=1.0, scale=1.0):  # revtex 10pt two-col
def set_size(width_in_pt=246.0, height_in_width=1.0, scale=1.0):  # revtex 10pt two-col
    width_in_in = width_in_pt * scale / 72.27
    return (width_in_in, width_in_in * height_in_width)


if __name__ == "__main__":
    main()
