#!/usr/bin/env python

import json
import matplotlib.pyplot as plt
import numpy as np


def main():
    with open(f"output/periodic_square_lattice.json") as f:
        data = json.load(f)

    fig = plt.figure()
    gs = fig.add_gridspec(4, 2)
    acs = []
    for i in range(4):
        acs.append([])
        for j in range(2):
            acs[i].append(fig.add_subplot(gs[i, j]))

    densities = data["densities"]
    density_len = len(densities)
    density_step = 20
    density_ticks = [data["densities"][d] for d in range(0, density_len, density_step)]

    dat = {}
    for i, j, d in [
        (0, 0, "before_claw_free"),
        (0, 1, "after_claw_free"),
        (1, 0, "before_simplicial"),
        (1, 1, "after_simplicial"),
    ]:
        acs[i][j].plot(densities, data[d])

    # acs[2][0].imshow(
    #     dat["before_collapse_claw_free"] - dat["before_collapse_simplicial"]
    # )
    # acs[2][1].imshow(dat["claw_free"] - dat["simplicial"])

    # da = []
    # for size in range(size_start, size_end + 1):
    #     da.append(data["sweep"][size]["avg_collapsed_nodes"])
    # acs[3][0].imshow(da)

    # da = []
    # length = int(dat["before_collapse_claw_free"].shape[1] // 2)
    # for size in range(len(dat["before_collapse_claw_free"])):
    #     d = []
    #     for i in range(length):
    #         print(size, i)
    #         d.append(
    #             dat["before_collapse_claw_free"][size][-i]
    #             - dat["before_collapse_claw_free"][size][i]
    #         )
    #     da.append(d)
    # acs[3][1].imshow(da)

    plt.savefig(f"output/periodic_square_lattice.pdf")


if __name__ == "__main__":
    main()
