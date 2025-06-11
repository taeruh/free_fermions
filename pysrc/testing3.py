#!/usr/bin/env python


import matplotlib.pyplot as plt
import numpy as np

from bounds import *


def full_fraction(n, p):
    numerator = 0
    for f in range(0, n - 1 + 1):
        numerator += ((n - 1) * p) ** f / fact(f) * p ** binom(f, 2)
    denominator = 0
    for f in range(0, n - 2 + 1):
        fact_f = ((n - 2) * p) ** f
        for s in range(0, f + 1):
            fact_s = fact_f / fact(s)
            for l in range(0, f - s + 1):
                m = f - s - l
                factor = fact_s / (fact(l) * fact(m))
                denominator += (
                    factor
                    * p ** (binom(l, 2) + binom(m, 2) + s * f - binom(s, 2))
                    * (1 - p) ** (n - 2 - s)
                )
    return numerator - denominator


def diff(n, p):
    ret = 0
    for f in range(0, n - 2 + 1):
        shared_factor = ((n - 2) * p) ** f / fact(f)
        numerator = p ** binom(f, 2)
        denominator = 0
        fact_f = fact(f)
        for s in range(0, f + 1):
            fact_s = fact_f / fact(s)
            for l in range(0, f - s + 1):
                m = f - s - l
                factor = fact_s / (fact(l) * fact(m))
                denominator += (
                    factor
                    * p ** (binom(l, 2) + binom(m, 2) + s * f - binom(s, 2))
                    * (1 - p) ** (n - 2 - s)
                )
        # print("s", shared_factor)
        # print(shared_factor*(numerator - denominator))
        ret += shared_factor * (numerator - denominator)
    print("x")
    # print()
    # print()
    return ret


np_diff = np.vectorize(diff)
np_full_fraction = np.vectorize(full_fraction)


def this(p):
    return p + np.log(2 - p)


np_this = np.vectorize(this)


def that(n, p):
    ret = 0
    for l in range(0, n - 1 + 1):
        ret += binom(n - 1, l) * p**l * (1 - p) ** (n - 1 - l) * p ** binom(l, 2)
    return (1 - p) ** ((n - 2) / 2) - ret


np_that = np.vectorize(that)


def main():
    fig = plt.figure()
    gs = fig.add_gridspec(1, 1)
    ax = fig.add_subplot(gs[0, 0])

    # p = np.linspace(0, 0.000000002, 200)
    p = np.linspace(0, 0.9, 200)

    n = 50

    # y = np_diff(n, p)
    # y = np_full_fraction(n, p)
    y = np_this(p)
    # y = np_that(n, p)

    min = np.min(y)
    print("min", min)
    max = np.max(y)
    print("max", max)

    ax.plot(p, y)

    # ax.set_xlim(0, 0.9)
    # ax.set_ylim(0, 100.1)

    plt.savefig("output/testing.pdf")


if __name__ == "__main__":
    main()
