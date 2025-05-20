#!/usr/bin/env python

import matplotlib.pyplot as plt
import numpy as np
import scipy


def choose_two(n):
    return n * (n - 1) / 2


def choose_three(n):
    return choose_two(n) * (n - 2) / 3


def choose_four(n):
    return choose_three(n) * (n - 3) / 4


def choose_five(n):
    return n * (n - 1) * (n - 2) * (n - 3) * (n - 4) / 120


def upper_bound_single_is_not_simplicial(n, p):
    p_ = 1 - p
    return choose_three(n - 1) * (  # number of potential non-compatible 3set neighbours
        1
        - (  # probability that a 3set is compatible with the vertex ->
            p_**3
            + 3 * p * p_**2
            + 3 * p**2 * p_
            + p**3
            * (  # clique options / all options (independent set not allowed)
                p**3 / (p**3 + 3 * p**2 * p_ + 3 * p * p_**2)
            )
        )
    )


def upper_bound_double_is_not_simplicial(n, p):
    # return 1.0
    return 0.0


def upper_bound_triple_is_not_simplicial(n, p):
    # return 1.0
    return 0.0


def upper_bound_quadruple_is_not_simplicial(n, p):
    # return 1.0
    return 0.0


def upper_bound_quintuple_is_not_simplicial(n, p):
    # return 1.0
    return 0.0


def upper_bound_expected_non_simplicial_singles(n, p):
    return n * upper_bound_single_is_not_simplicial(n, p)


def upper_bound_expected_singles_and_doubles_are_not_simplicial(n, p):
    return n * upper_bound_single_is_not_simplicial(n, p) + choose_two(
        n
    ) * upper_bound_double_is_not_simplicial(n, p)


def lower_bound_expected_simplicial_singles(n, p):
    return max(0.0, n - upper_bound_expected_non_simplicial_singles(n, p))


def lower_bound_expected_singles_and_doubles_are_simplicial(n, p):
    return max(
        0.0,
        n
        + choose_two(n)
        - upper_bound_expected_singles_and_doubles_are_not_simplicial(n, p),
    )


def _lower_bound_has_single_or_double_simplicial(n, p):
    # return lower_bound_expected_simplicial_singles(n, p) ** 2 / n**2
    return (
        lower_bound_expected_singles_and_doubles_are_simplicial(n, p) ** 2
        / (n + choose_two(n)) ** 2
    )


def _lower_bound_has_simplicial(n, p):
    return (
        max(
            0.0,
            n * (1 - upper_bound_single_is_not_simplicial(n, p))
            + choose_two(n) * (1 - upper_bound_double_is_not_simplicial(n, p)),
            # + choose_three(n) * (1 - upper_bound_triple_is_not_simplicial(n, p))
            # + choose_four(n) * (1 - upper_bound_quadruple_is_not_simplicial(n, p))
            # + choose_five(n) * (1 - upper_bound_quintuple_is_not_simplicial(n, p)),
        )
        ** 2
        / (
            n
            + choose_two(n)
            # + choose_three(n)
            # + choose_four(n)
            # + choose_five(n)
            #
        )
        ** 2
    )


lower_bound_has_single_or_double_simplicial = np.vectorize(
    _lower_bound_has_single_or_double_simplicial
)
lower_bound_has_simplicial = np.vectorize(_lower_bound_has_simplicial)


def binom(n, k):
    return scipy.special.comb(n, k)


def _lower_bound(n, p, max_k):
    sum = 0
    for k in range(1, max_k + 1):
        sum += (
            binom(n, k)
            * p ** binom(k, 2)
            * (1 - binom(n - k, 2) * ((1 - p) * (1 - (1 - p**2) ** k)))
        )
    return max(0.0, sum)


def _trivial_upper_bound(n, _, max_k):
    sum = 0
    for k in range(1, max_k + 1):
        sum += binom(n, k)
    return sum


def _exact(n, p, max_k):
    sum = 0
    for k in range(1, max_k + 1):
        inner_sum = 0
        for l in range(0, n - k + 1):
            inner_sum += p**l * (1 - p) ** (n - k - l) * p ** binom(l, 2)
        sum += binom(n, k) * p ** binom(k, 2) * inner_sum
    return sum


lower_bound = np.vectorize(_lower_bound)
trivial_upper_bound = np.vectorize(_trivial_upper_bound)
exact = np.vectorize(_exact)


def main():
    fig = plt.figure()
    gs = fig.add_gridspec(1, 1)
    ax = fig.add_subplot(gs[0, 0])

    n = 20
    # n = 5
    eps = 0.0001
    p = np.linspace(eps, 1 - eps, 300)

    k = 1
    ax.plot(p, trivial_upper_bound(n, p, k))
    ax.plot(p, exact(n, p, k))
    # ax.plot(p, exact(n, p, k) / trivial_upper_bound(n, p, k))

    ax.set_xlim(0, 1)
    ax.set_ylim(0, n)

    plt.savefig("output/testing.pdf")


if __name__ == "__main__":
    main()
