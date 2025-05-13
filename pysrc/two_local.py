#!/usr/bin/env python

import matplotlib.pyplot as plt
import matplotlib
import numpy as np

from data_density_sizes import Data
from plot_helper import paper_setup, set_size

data_dir = "output"
# data_dir = "results"
file = "two_local_"


def main():
    data = Data(data_dir, file)

    paper_setup()

    fig = plt.figure(figsize=set_size(height_in_width=0.7))
    gs = fig.add_gridspec(2, 1)
    axs = [
        fig.add_subplot(gs[0, 0]),
        fig.add_subplot(gs[1, 0]),
    ]
    gs.update(hspace=0.005)

    orbit_range = range(0, 3)
    color_offset = 0

    color_map = matplotlib.colormaps["plasma"]
    colors = [
        color_map(i)
        for i in np.linspace(0.0, 0.95, len(orbit_range) + len(orbit_range) - 1)
    ]

    linestyles = [
        "dashed",
        "solid",
        "dotted",
    ]
    labels = [
        r"$p_{\mathrm{SCF}}$",
        r"$\Delta p_{\mathrm{SCF}}$",
        r"$\Delta \Xi$",
    ]

    for j in orbit_range:
        axs[0].plot(
            data.densities,
            data.simplicial[j],
            label=f"$m = {round(data.sizes[j]/2)}$",
            linestyle=linestyles[0],
            color=colors[color_offset + j],
        )
    for j in orbit_range:
        axs[1].plot(
            data.densities,
            data.delta_simplicial[j],
            label=f"$m = {round(data.sizes[j]/2)}$",
            linestyle=linestyles[1],
            color=colors[color_offset + j],
        )
        axs[1].plot(
            data.densities,
            data.collapsed[j],
            label=f"$m = {round(data.sizes[j]/2)}$",
            linestyle=linestyles[2],
            color=colors[color_offset + j],
        )
    handles, this_labels = axs[0].get_legend_handles_labels()
    axs[0].legend(handles, this_labels, loc="upper right")

    # ymax = ao.get_ylim()[1]
    # ao.set_ylim(0, ymax)
    for ax in axs:
        ax.set_ylim(0, ax.get_ylim()[1])
        ax.grid()
        ax.tick_params(axis="x", which="both", bottom=True, top=True)
        ax.set_xlim(0, data.densities[-1])
        # handles, labels = ax.get_legend_handles_labels()
        # ax.legend(handles, labels, loc="upper right")

    axs[1].set_xlabel(r"$d$")
    axs[1].xaxis.set_label_coords(0.5, -0.14)
    axs[1].set_ylabel(r"[\%]")

    # ax.set_yticks([0.5, 1.0])
    # ax.set_yticklabels(["0.5", "1.0"])
    axs[0].set_ylabel(labels[0])
    axs[0].tick_params(axis="x", labelbottom=False)

    axs[1].set_ylim(0, 66)

    axs[1].plot([], [], color="black", linestyle=linestyles[0], label=labels[0])
    axs[1].plot([], [], color="black", linestyle=linestyles[1], label=labels[1])
    axs[1].plot([], [], color="black", linestyle=linestyles[2], label=labels[2])
    handles, labels = axs[1].get_legend_handles_labels()
    handles = handles[-3:]
    labels = labels[-3:]
    axs[1].legend(handles, labels, loc="upper right")

    plt.subplots_adjust(top=0.98, bottom=0.12, left=0.12, right=0.950)

    plt.savefig(f"output/klocal.pdf")


if __name__ == "__main__":
    main()
