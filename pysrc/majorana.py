#!/usr/bin/env python

import json
import matplotlib.pyplot as plt
import matplotlib
import numpy as np

data_dir = "output"
# data_dir = "results"
file = "e_structure_"

# suffixes = ["first_", "second_"]
suffixes = ["", "second_"]


def main():
    first = Data(suffixes[0])
    second = Data(suffixes[0])

    paper_setup()

    fig = plt.figure(figsize=set_size(height_in_width=0.8))
    gs = fig.add_gridspec(2, 1)
    axf = fig.add_subplot(gs[0, 0])
    axs = fig.add_subplot(gs[1, 0])

    color_map = matplotlib.colormaps["plasma"]
    colors = [
        color_map(i)
        for i in np.linspace(0.01, 0.80, first.size_len + second.size_len - 1)
    ]

    label = r"$p_{\mathrm{SCF}}$"

    for i in range(first.size_len):
        axf.plot(
            first.densities,
            first.results[i],
            label=f"$n = {first.sizes[i]}$",
            color=colors[i],
        )

    for i in range(second.size_len):
        axs.plot(
            first.densities,
            first.results[i],
            label=f"$n = {second.sizes[i]}$",
            color=colors[i + first.size_len - 1],
        )

    ymax = axf.get_ylim()[1]
    for ax in [axf, axs]:
        ax.set_ylabel(label)
        ax.grid()
        ax.tick_params(axis="x", which="both", bottom=True, top=True, labelbottom=True)
        ax.set_ylim(0, ymax)
        handles, labels = ax.get_legend_handles_labels()
        ax.legend(handles, labels, loc="upper right")

    axf.set_xlim(0, 1)
    axs.set_xlim(0, 0.2)

    axs.set_xlabel(r"$d$")


    plt.subplots_adjust(top=0.96, bottom=0.13, left=0.14, right=0.970)

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
        with open(f"{data_dir}/{file}{suffix}0.json") as f:
            data = json.load(f)

        self.densities = data["densities"]
        self.sizes = data["sizes"]
        self.density_len = len(self.densities)
        self.size_len = len(self.sizes)

        num_sample_files = 20
        self.num_total_samples = 0

        self.results = np.array(
            np.tile(0, (self.size_len, self.density_len)), dtype=float
        )

        for i in range(num_sample_files):
            try:
                with open(f"{data_dir}/{file}{i}.json") as f:
                    data = json.load(f)
            except FileNotFoundError:
                print(f"File {i} not found")
                continue
            num_samples = data["num_samples"]
            self.num_total_samples += num_samples
            self.results += num_samples * np.array(data["simplicial"])

        self.results /= self.num_total_samples


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
