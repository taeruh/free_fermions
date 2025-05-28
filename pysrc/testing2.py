#!/usr/bin/env python

# trying to estimate the likelihood of independent sets in the k-local model

import matplotlib.pyplot as plt
import numpy as np
import scipy


def binom(n, k):
    return scipy.special.comb(n, k)


def edge_probability(n, k):
    ret = 0
    p = 2 * k / (3 * n)
    for j in range(1, k + 1, 2):
        ret += binom(k, j) * (p**j) * ((1 - p) ** (k - j))
    return ret


def no_edge(n, k, num_vertices):
    p = edge_probability(n, k)
    return (1 - p) ** num_vertices


# 4 set has at least one independent vertex
def g(n, k):
    p = edge_probability(n, k)
    p_ = 1 - p
    a = 4 * (p_**3) * (p**3 + 3 * p**2 * p_)  # only one independent
    b = binom(4, 2) * p * p_**5
    c = p_ ** binom(4, 2)  # three independent -> all independent
    return a + b + c


# 4 set has claw upper bound
def c(n, k):
    p = edge_probability(n, k)
    return 4 * (p**3) * (1 - p) ** 3


def f(n, k, num_vertices):
    # return 1 - binom(num_vertices, 4) * (1 - g(n, k))
    return 1 - binom(num_vertices, 4) * c(n, k)


def lim_exp_simp_clique(n, p, k_max):
    ret = 0
    for k in range(1, k_max + 1):
        # ret += binom(n, k) * p ** (binom(k, 2) + k * binom((n - k) * p, 2))
        neigh = 0
        for l in range(0, n - k + 1):
            neigh += (
                binom(n - k, l) * p**l * ((1 - p) ** (n - k - l)) * p ** binom(l, 2)
            )
        ret += binom(n, k) * p ** binom(k, 2) * neigh**k
    return ret


def lim_exp_claws(n, p):
    return binom(n, 4) * 4 * (p**3) * (1 - p) ** 3


def lower_simp_clique(n, p, k_max):
    return max(0, u(lim_exp_simp_clique(n, p, k_max)))


def upper_simp_clique(n, p, k_max):
    # return min(1, lim_exp_simp_clique(n, p, k_max))  # does strange things here
    return lim_exp_simp_clique(n, p, k_max)


def lower_not_claws(n, p):
    return max(0, 1 - lim_exp_claws(n, p))


def upper_not_claws(n, p):
    # return min(1, v(lim_exp_claws(n, p)))  # does strange things here
    return v(lim_exp_claws(n, p))


def u(x):
    return 1 / (1 + 1 / x)


def v(x):
    return 1 / (1 + x)


np_no_edge = np.vectorize(no_edge)
np_f = np.vectorize(f)

np_lower_simp_clique = np.vectorize(lower_simp_clique)
np_upper_simp_clique = np.vectorize(upper_simp_clique)
np_lower_not_claws = np.vectorize(lower_not_claws)
np_upper_not_claws = np.vectorize(upper_not_claws)
np_lim_exp_simp_clique = np.vectorize(lim_exp_simp_clique)


def main():
    fig = plt.figure()
    gs = fig.add_gridspec(1, 1)
    ax = fig.add_subplot(gs[0, 0])

    n = 40
    k = 2

    # print(edge_probability(n, k))
    # print(c(n, k))
    # print(binom(10, 4))

    eps = 0.00000000001
    xlim = 0.06
    # factor =0.000000000000000000000001
    factor = 1
    d = factor * np.linspace(eps, xlim - eps, 3000)
    num_vertices = d * 3**k * binom(n, k)

    # print(num_vertices)

    # y = np_f(n, k, num_vertices)
    # c = 0.5
    # last1 = 0
    # for i, x in enumerate(y):
    #     if x > 1 - c:
    #         last1 = num_vertices[i]
    #     else:
    #         break
    # print(last1)

    # ax.plot(d, np_f(n, k, num_vertices))
    # ax.plot(num_vertices, np_f(n, k, num_vertices))
    # ax.plot(d, num_vertices)

    # n = 20
    # kmax = n
    # x = np.linspace(0, 1, 50)
    # # ax.plot(x, np_lower_not_claws(n, x), label="lower not claws")
    # ax.plot(x, np_upper_not_claws(n, x), label="upper not claws")
    # # ax.plot(x, np_lower_simp_clique(n, x, kmax), label="lower simplicial clique")
    # ax.plot(x, np_upper_simp_clique(n, x, kmax), label="upper simplicial clique")
    # # ax.plot(x, np_lower_not_claws(n, x) * np_lower_simp_clique(n, x, kmax), label="lower")
    # ax.plot(x, np_upper_not_claws(n, x) * np_upper_simp_clique(n, x, kmax), label="upper")

    print(binom(1, 2))

    n = 10
    kmax = n
    x = np.linspace(0, 1, 1000)
    y = u(np_lim_exp_simp_clique(n, x, kmax))
    # y = np_lim_exp_simp_clique(n, x, 2)
    ax.plot(x, y)

    ax.set_xlim(0, 1)
    # ax.set_ylim(0, 50)

    plt.savefig("output/testing.pdf")


if __name__ == "__main__":
    main()
