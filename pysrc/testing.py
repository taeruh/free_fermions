#!/usr/bin/env python

import matplotlib.pyplot as plt
import numpy as np


def choose_two(n):
    return n * (n - 1) / 2


def choose_three(n):
    return n * (n - 1) * (n - 2) / 6


def upper_bound_single_vertex_is_not_simplicial_in_claw_free_graph(n, p):
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


# def upper_bound_two_vertices_are_not_simplicial_in_claw_free_graph(n, p):
#     p_ = 1 - p
#     pass  # todo


def lower_bound_single_vertex_is_simplicial_in_claw_free_graph(n, p):
    return max(
        0.0, 1 - upper_bound_single_vertex_is_not_simplicial_in_claw_free_graph(n, p)
    )


def _lower_bound_expected_number_single_vertex_is_simplicial_in_claw_free_graph(n, p):
    return n * lower_bound_single_vertex_is_simplicial_in_claw_free_graph(n, p)


lower_bound_expected_number_single_vertex_is_simplicial_in_claw_free_graph = np.vectorize(
    _lower_bound_expected_number_single_vertex_is_simplicial_in_claw_free_graph
)


def main():
    fig = plt.figure()
    gs = fig.add_gridspec(1, 1)
    ax = fig.add_subplot(gs[0, 0])

    n = 10
    # n = 5
    eps = 0.0001
    p = np.linspace(eps, 1 - eps, 300)

    ax.plot(
        p,
        # lower_bound_expected_number_single_vertex_is_simplicial_in_claw_free_graph(n, p),
        n * upper_bound_single_vertex_is_not_simplicial_in_claw_free_graph(n, p),
    )

    ax.set_xlim(0, 1)
    ax.set_ylim(0, n)

    plt.savefig("output/testing.pdf")


if __name__ == "__main__":
    main()
