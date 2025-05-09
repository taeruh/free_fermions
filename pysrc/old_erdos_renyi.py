#!/usr/bin/env python

import json
import matplotlib.pyplot as plt
import numpy as np

CONSIDER_PARALLEL_GRAPHS = "true"
# CONSIDER_PARALLEL_GRAPHS = "false"


def main():
    with open(f"output/erdos_renyi_{CONSIDER_PARALLEL_GRAPHS}.json") as f:
        data = json.load(f)

    fig = plt.figure()
    gs = fig.add_gridspec(4, 2)
    acs = []
    for i in range(4):
        acs.append([])
        for j in range(2):
            acs[i].append(fig.add_subplot(gs[i, j]))

    size_start = 4
    size_end = 4
    size_len = size_end + 1 - size_start
    size_step = 2
    size_ticks = [s for s in range(size_start, size_end + 1, size_step)]

    density_len = len(data["densities"])
    density_step = 20
    density_ticks = [data["densities"][d] for d in range(0, density_len, density_step)]

    for i in range(4):
        for j in range(2):
            acs[i][j].set_xticks(range(0, density_len, density_step))
            acs[i][j].set_xticklabels(density_ticks)
            acs[i][j].set_yticks(range(0, size_len, size_step))
            acs[i][j].set_yticklabels(size_ticks)

    dat = {}
    for i, j, d in [
        (0, 0, "before_collapse_claw_free"),
        (0, 1, "claw_free"),
        (1, 0, "before_collapse_simplicial"),
        (1, 1, "simplicial"),
    ]:
        da = []
        for size in range(size_start, size_end + 1):
            da.append(data["sweep"][size][d])
        acs[i][j].imshow(da)
        dat[d] = np.array(da)
    acs[2][0].imshow(
        dat["before_collapse_claw_free"] - dat["before_collapse_simplicial"]
    )
    acs[2][1].imshow(dat["claw_free"] - dat["simplicial"])

    da = []
    for size in range(size_start, size_end + 1):
        da.append(data["sweep"][size]["avg_collapsed_nodes"])
    acs[3][0].imshow(da)

    da = []
    length = int(dat["before_collapse_claw_free"].shape[1] // 2)
    for size in range(len(dat["before_collapse_claw_free"])):
        d = []
        for i in range(length):
            print(size, i)
            d.append(
                dat["before_collapse_claw_free"][size][-i]
                - dat["before_collapse_claw_free"][size][i]
            )
        da.append(d)
    acs[3][1].imshow(da)

    plt.savefig(f"output/erdos_renyi_{CONSIDER_PARALLEL_GRAPHS}.pdf")


if __name__ == "__main__":
    main()
