#!/usr/bin/env python

import json
import matplotlib.pyplot as plt


def main():
    with open("output/erdos_renyi.json") as f:
        data = json.load(f)

    fig = plt.figure()
    gs = fig.add_gridspec(4, 1)
    acs = []
    for i in range(4):
        acs.append(fig.add_subplot(gs[i, 0]))

    for i, d in enumerate(
        [
            "before_collapse_claw_free",
            "claw_free",
            "before_collapse_simplicial",
            "simplicial",
        ]
    ):
        dat = []
        for size in range(4, 11):
            dat.append(data["sweep"][size][d])
        acs[i].imshow(dat)

    plt.savefig("output/erdos_renyi.pdf")


if __name__ == "__main__":
    main()
