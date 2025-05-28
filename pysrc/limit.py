#!/usr/bin/env python3

import matplotlib.pyplot as plt
import matplotlib
import numpy as np
import scipy


from bounds import *


# n k-local spins edge probability, i.e., the p for gnp graphs
def edge_probability(n, k):
    ret = 0
    p = 2 * k / (3 * n)
    for j in range(1, k + 1, 2):
        ret += binom(k, j) * (p**j) * ((1 - p) ** (k - j))
    return ret


def klocal_calculations():
    n = 10
    d = 0.01
    k = 2

    gnp = {"n": round(d * binom(n, 2) * k**3), "p": edge_probability(n, k)}
    print(gnp)
    print(np.log(gnp["n"]) / gnp["n"])

    print(prob_connected(gnp["n"], gnp["p"]))
    print(gnp_almost_surely_scf_get_threshold(gnp["n"], gnp["p"]))

    threshold = 0.9

    fig = plt.figure()
    gs = fig.add_gridspec(1, 1)
    ax = fig.add_subplot(gs[0, 0])

    eps = 0.01
    p = np.linspace(0 + eps, 1 - eps, 300)
    y = [gnp_almost_surely_scf_get_n(p_, threshold) for p_ in p]

    ax.plot(p, y)

    plt.savefig("output/limit.pdf")


if __name__ == "__main__":
    klocal_calculations()
