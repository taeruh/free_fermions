#!/usr/bin/env python

import matplotlib.pyplot as plt
import matplotlib
import numpy as np
import scipy

from data_density_sizes import Data
from plot_helper import paper_setup, set_size

data_dir = "output"
# data_dir = "results"
file = "erdos_renyi_"


def choose_two(n):
    return n * (n - 1) / 2


def choose_four(n):
    return n * (n - 1) * (n - 2) * (n - 3) / 24


def binomial(n, k):
    return scipy.special.comb(n, k)


def four_set_has_claw(density):
    return 4 * density**3 * (1 - density) ** 3


def is_simplicial_exponent(density, set_size, size):
    return choose_two(set_size) + choose_two((size - set_size) * density)
    # return choose_two(set_size) + set_size * choose_two((size - set_size) * density)
    # return choose_two(set_size) + set_size * (size - set_size) * density
    # return set_size * ((set_size - 1) / 2 + (size - set_size) * density)


def set_is_simplicial_clique(density, set_size, size):
    return density ** is_simplicial_exponent(density, set_size, size)


def _upper_claw_bound(density):
    return 1 - four_set_has_claw(density)


def _lower_claw_bound(density, size):
    return 1 - choose_four(size) * four_set_has_claw(density)


def _upper_simplicial_bound(density, size):
    ret = 0
    # for set_size in range(1, int(np.min([size, 5]))):
    for set_size in range(1, size + 1):
        ret += binomial(size, set_size) * set_is_simplicial_clique(
            density, set_size, size
        )
    return ret


def _lower_simplicial_bound(density, size):
    return set_is_simplicial_clique(density, size, size)


# from Perkins paper "The Typical Structure Of Dense Claw-free Graphs"
def _limit_claw_free(density, size):
    def r(density):
        transition_point = (3 - np.sqrt(5)) / 2
        if density < transition_point:
            return - np.log2(1 - density)
        else:
            return - 0.5 * np.log2(density)
    return np.exp2(- choose_two(size) * r(density))


lower_claw_bound = np.vectorize(_lower_claw_bound)
upper_claw_bound = np.vectorize(_upper_claw_bound)
upper_simplicial_bound = np.vectorize(_upper_simplicial_bound)
lower_simplicial_bound = np.vectorize(_lower_simplicial_bound)
limit_claw_free = np.vectorize(_limit_claw_free)


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

    orbit_range = range(0, 2)
    color_offset = 0

    color_map = matplotlib.colormaps["plasma"]
    colors = [
        color_map(i)
        for i in np.linspace(0.0, 0.95, len(orbit_range) + len(orbit_range) - 1)
    ]

    linestyles = ["dashed", "solid", "dotted", "dashed"]
    labels = [
        r"$p_{\mathrm{SCF}}$",
        r"$\Delta p_{\mathrm{SCF}}$",
        r"$\Delta \Xi$",
        r"bounds",
    ]
    bounds_width = 0.5

    for j in orbit_range:
        axs[0].plot(
            data.densities,
            data.simplicial[j],
            # data.before_simplicial[j],
            label=f"$n = {data.sizes[j]}$",
            linestyle=linestyles[0],
            color=colors[color_offset + j],
        )
        # axs[0].plot(
        #     data.densities,
        #     lower_claw_bound(data.densities, data.sizes[j]),
        #     linestyle=linestyles[3],
        #     color=colors[color_offset + j],
        #     linewidth=bounds_width,
        # )
        # axs[0].plot(
        #     data.densities,
        #     upper_simplicial_bound(data.densities, data.sizes[j]),
        #     linestyle=linestyles[3],
        #     color=colors[color_offset + j],
        #     linewidth=bounds_width,
        # )
        # axs[0].plot(
        #     data.densities,
        #     limit_claw_free(data.densities, data.sizes[j]),
        #     linestyle=linestyles[2],
        #     color=colors[color_offset + j],
        #     linewidth=bounds_width,
        # )
        axs[1].plot(
            data.densities,
            data.delta_simplicial[j],
            linestyle=linestyles[1],
            color=colors[color_offset + j],
        )
        axs[1].plot(
            data.densities,
            data.collapsed[j],
            linestyle=linestyles[2],
            color=colors[color_offset + j],
        )

    # n = 20
    # p = 0.9
    # x = [k for k in range(1, n + 1)]
    # y = [is_simplicial_exponent(p, k, n) for k in x]
    # axs[1].plot(x, y)

    handles, this_labels = axs[0].get_legend_handles_labels()
    axs[0].legend(handles, this_labels, loc="upper right")

    # ymax = ao.get_ylim()[1]
    # ao.set_ylim(0, ymax)
    for ax in axs:
        ax.set_ylim(0, ax.get_ylim()[1])
        # ax.set_ylim(0, 1)
        ax.grid()
        ax.tick_params(axis="x", which="both", bottom=True, top=True)
        ax.set_xlim(0, data.densities[-1])
        # ax.set_xlim(0, 0.1)
        # handles, labels = ax.get_legend_handles_labels()
        # ax.legend(handles, labels, loc="upper right")

    axs[0].set_ylim(0, 1 + 0.05)

    axs[1].set_xlabel(r"$d$")
    axs[1].xaxis.set_label_coords(0.5, -0.14)
    axs[1].set_ylabel(r"[\%]")

    # ax.set_yticks([0.5, 1.0])
    # ax.set_yticklabels(["0.5", "1.0"])
    axs[0].set_ylabel(labels[0])
    axs[0].tick_params(axis="x", labelbottom=False)

    # axs[1].set_ylim(0, 66)

    axs[1].plot([], [], color="black", linestyle=linestyles[0], label=labels[0])
    axs[1].plot([], [], color="black", linestyle=linestyles[1], label=labels[1])
    axs[1].plot([], [], color="black", linestyle=linestyles[2], label=labels[2])
    # axs[1].plot(
    #     [],
    #     [],
    #     color="black",
    #     linestyle=linestyles[3],
    #     label=labels[3],
    #     linewidth=bounds_width,
    # )
    handles, labels = axs[1].get_legend_handles_labels()
    handles = handles[-4:]
    labels = labels[-4:]
    axs[1].legend(handles, labels, loc="upper right")

    plt.subplots_adjust(top=0.98, bottom=0.12, left=0.12, right=0.950)

    plt.savefig(f"output/erdos_renyi.pdf")


if __name__ == "__main__":
    main()
