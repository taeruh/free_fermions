#!/usr/bin/env python

import json
import matplotlib.pyplot as plt
import numpy as np
import scipy

# data_dir = "output"
data_dir = "results"
file = "periodic_bricks_"


def main():
    with open(f"{data_dir}/exact_bricks_coefficients.json") as f:
        coefficients = json.load(f)

    max_ops = len(coefficients["scf"])
    # print(max_ops)  # should be 5*9=27

    def norm_factor(k):
        return scipy.special.comb(5 * 9, k) - (
            # inclusion-exclusion principle: we have 5 pots, each with 9 elements; we
            # have to correct for the cases where not all pots are missing at least
            # one element; for i=1,...,5 let A_i be the set of cases where we draw the
            # k elements such that pot i is empty; then we want to calculate the union
            # of (A_i)_i -> use inclusion-exclusion principle on those sets
            5 * scipy.special.comb(4 * 9, k)
            - 10 * scipy.special.comb(3 * 9, k)
            + 10 * scipy.special.comb(2 * 9, k)
            - 5 * scipy.special.comb(1 * 9, k)
            + 1 * scipy.special.comb(0 * 9, k)
        )

    def exact_scf(density):
        ret = 0.0
        norm = 0.0  # normalization factor since we were throwing away non-2d cases
        for k, coeff in enumerate(coefficients["scf"]):
            ret += coeff * density**k * (1 - density) ** (max_ops - k)
            norm += norm_factor(k) * density**k * (1 - density) ** (max_ops - k)
        return ret / norm

    def exact_dscf(density):
        ret = 0.0
        norm = 0.0
        for k, coeff in enumerate(coefficients["dscf"]):
            ret += coeff * density**k * (1 - density) ** (max_ops - k)
            norm += norm_factor(k) * density**k * (1 - density) ** (max_ops - k)
        return ret / norm


    with open(f"{data_dir}/{file}1.json") as f:
        data = json.load(f)

    densities = data["densities"]
    density_len = len(densities)
    # print(densities[0])

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

    for i in range(1, num_sample_files + 1):
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

    for key, value in results.items():
        # value *= 100.0 / num_total_samples  # percentage
        value /= num_total_samples

    # print(f"max_sc_size: {max_sc_size}")

    paper_setup()

    fig = plt.figure(figsize=set_size(height_in_width=0.8))
    gs = fig.add_gridspec(2, 1)
    ax = fig.add_subplot(gs[0, 0])
    axl = fig.add_subplot(gs[1, 0])
    gs.update(hspace=0.005)

    rc_colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]
    colors = [rc_colors[4], rc_colors[3], rc_colors[2], "black", "black"]
    linestyles = [
        "dashed",
        "solid",
        "dotted",
        (0, (2.5, 1.5, 1, 1, 1, 1.5)),
        (0, (2.5, 1.5, 1, 1, 1, 1.5)),
    ]
    labels = [
        r"$p_{\mathrm{SCF}}$",
        r"$\Delta p_{\mathrm{SCF}}$",
        r"$\Delta \Xi$",
        r"$p_{\mathrm{SCF}}$ exact",
        r"$\Delta p_{\mathrm{SCF}}$ exact",
    ]

    ax.set_ylabel(labels[0])
    ax.plot(
        densities,
        results["after_simplicial"],
        label=labels[0],
        color=colors[0],
        linestyle=linestyles[0],
    )

    exact = np.array([exact_scf(d) for d in densities])
    ax.plot(
        densities,
        exact,
        label=labels[3],
        color=colors[3],
        linestyle=linestyles[3],
    )

    axl.set_ylabel(r"[\%]")
    axl.plot(
        densities,
        (results["after_simplicial"] - results["before_simplicial"]) * 100,
        label=labels[1],
        color=colors[1],
        linestyle=linestyles[1],
    )
    axl.plot(
        densities,
        results["collapsed"] * 100,
        label=labels[2],
        color=colors[2],
        linestyle=linestyles[2],
    )

    exact = np.array([exact_dscf(d) for d in densities])
    axl.plot(
        densities,
        exact * 100,
        label=labels[4],
        color=colors[4],
        linestyle=linestyles[4],
    )

    # print(
    #     f"before sum(|scf - cf|) = {np.abs(results["before_claw_free"] -
    #     results["before_simplicial"]).sum()}"
    # )
    # print(
    #     f"after  sum(|scf - cf|) = {np.abs(results["after_claw_free"] -
    #     results["after_simplicial"]).sum()}"
    # )

    for a in [ax, axl]:
        a.grid()
        ymax = a.get_ylim()[1]
        a.set_ylim(0, ymax)
        a.set_xlim(0, densities[-1])
    axl.set_xlabel(r"$p$")
    ax.tick_params(axis="x", which="both", bottom=True, top=True, labelbottom=False)
    axl.tick_params(axis="x", which="both", top=True)

    handles, labels = axl.get_legend_handles_labels()
    axl.legend(handles, labels, loc="upper right")
    handles, labels = ax.get_legend_handles_labels()
    ax.legend(handles, labels, loc="upper right")

    # axl.set_yticks([0.0, 2.5, 5.0])
    plt.subplots_adjust(top=0.96, bottom=0.13, left=0.14, right=0.970)

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
# def set_size(width_in_pt=510.0, height_in_width=1.0, scale=1.0):  # revtex 10pt two-col
def set_size(width_in_pt=246.0, height_in_width=1.0, scale=1.0):  # revtex 10pt two-col
    width_in_in = width_in_pt * scale / 72.27
    return (width_in_in, width_in_in * height_in_width)


if __name__ == "__main__":
    main()
