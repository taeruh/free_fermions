import matplotlib.pyplot as plt
import matplotlib
import numpy as np
import scipy


def binom(n, k):
    return scipy.special.comb(n, k)


def _exp_claws(n, p):
    # return binom(n, 4) * 4 * (p**3) * (1 - p) ** 3
    return n * binom(n - 1, 3) * (p**3) * (1 - p) ** 3


def _inverted_second_moment_claws(n, p):
    p_ = 1 - p
    return (
        binom(n - 4, 4)
        + 4 * binom(n - 4, 3)
        + 3 * binom(n - 4, 2) / (2 * p * p_)
        + (n - 4) / 4 * (1 / (p_**3) + 3 / (p**2 * p_))
        + 1 / (4 * p**3 * p_**3)
    ) / binom(n, 4)


def _variance_claws(n, p):
    return exp_claws(n, p) ** 2 * inverted_second_moment_claws(n, p)


exp_claws = np.vectorize(_exp_claws)
inverted_second_moment_claws = np.vectorize(_inverted_second_moment_claws)
variance_claws = np.vectorize(_variance_claws)


# {{{ everything here is in the limit with some wild approximations


# this is not too bad for small p, but pretty bad for big p, since then the neighbourhoods
# are completely overcounting -> bound is way too low for large p (except when they are
# p is getting close to 1)
def _exp_simp_clique(n, p, k_max):
    ret = 0
    for k in range(1, k_max + 1):
        # ret += binom(n, k) * p ** (binom(k, 2) + k * binom((n - k) * p, 2))
        # in the above, we just put in an expectation value for the neighbourhood size;
        # the below is better, but we are still assuming independence of the
        # neighbourhoods
        neigh = 0
        for l in range(0, n - k + 1):
            neigh += (
                binom(n - k, l) * p**l * ((1 - p) ** (n - k - l)) * p ** binom(l, 2)
            )
        ret += binom(n, k) * p ** binom(k, 2) * neigh**k
    return ret


def _better_second_moment_cliques(n, p, k_max):
    correction = 0
    for k in range(1, k_max + 1):
        neigh = 0
        for l in range(0, n - k + 1):
            neigh += (
                binom(n - k, l) * p**l * ((1 - p) ** (n - k - l)) * p ** binom(l, 2)
            )
        correction += binom(n, k) * (p ** binom(k, 2) * neigh**k) ** 2
    exp = exp_simp_clique(n, p, k_max)
    return 1 / (1 + 1 / exp - correction / exp**2)


exp_simp_clique = np.vectorize(_exp_simp_clique)
better_second_moment_cliques = np.vectorize(_better_second_moment_cliques)


def first_moment(exp):
    return exp


def inverted_first_moment(exp):
    return 1 - exp


def second_moment(exp):
    return 1 / (1 + 1 / exp)


def inverted_second_moment(exp):
    return 1 / (exp + 1)


def prob_connected(n, p):
    c = n * p / np.log(n)
    return np.exp(-np.exp(-c))


# see
# https://people.maths.bris.ac.uk/~maajg/teaching/complexnets/connected-giantcompt.pdf
# especially start of section 3
def get_lower_bound(n, p):
    ln_n = np.log(n)
    # print(f"n = {n}")

    connected = prob_connected(n, p)

    claw_lower_connected = max(0, inverted_first_moment(exp_claws(n, p)))
    clique_lower_connected = better_second_moment_cliques(n, p, n)

    claw_lower_unconnected = 0
    clique_lower_unconnected = 0
    # many small components: naively assume that we have n/ln(n) components of size
    # ln(n) each
    if p < 1 / n:
        size = round(ln_n)
        num_components = round(n / ln_n)
        claw_lower_single = max(0, inverted_first_moment(exp_claws(size, p)))
        simp_lower_single = better_second_moment_cliques(size, p, size)
        claw_lower_unconnected = claw_lower_single**num_components
        clique_lower_unconnected = simp_lower_single**num_components
    # one big component: naively assume that it is of size 2/3 * n and the rest of
    # size ln(n)
    elif p < ln_n / n:
        size = round(2 / 3 * n)
        size_small = round(ln_n)
        num_small = round((n - size) / size_small)
        claw_lower_big = max(0, inverted_first_moment(exp_claws(size, p)))
        claw_lower_small = max(0, inverted_first_moment(exp_claws(size_small, p)))
        simp_lower_big = better_second_moment_cliques(size, p, size)
        simp_lower_small = better_second_moment_cliques(size_small, p, size_small)
        claw_lower_unconnected = claw_lower_big * (claw_lower_small**num_small)
        clique_lower_unconnected = simp_lower_big * (simp_lower_small**num_small)
        claw_lower_unconnected = max(0, inverted_first_moment(exp_claws(n, p)))
        clique_lower_unconnected = better_second_moment_cliques(n, p, n)
    # assume one component as approximation
    else:
        claw_lower_unconnected = claw_lower_connected
        clique_lower_unconnected = clique_lower_connected

    return (
        connected * claw_lower_connected * clique_lower_connected
        + (1 - connected) * claw_lower_unconnected * clique_lower_unconnected
    )


def get_upper_bound(n, p):
    ln_n = np.log(n)
    # print(f"n = {n}")

    connected = prob_connected(n, p)

    claw_upper_connected = 1 - 1 / inverted_second_moment_claws(n, p)
    clique_upper_connected = min(1, first_moment(exp_simp_clique(n, p, n)))

    claw_upper_unconnected = 0
    clique_upper_unconnected = 0
    # many small components: naively assume that we have n/ln(n) components of size
    # ln(n) each
    if p < 1 / n:
        size = round(ln_n)
        num_components = round(n / ln_n)
        claw_upper_single = 1 - 1 / inverted_second_moment_claws(size, p)
        simp_upper_single = min(1, first_moment(exp_simp_clique(size, p, size)))
        claw_upper_unconnected = claw_upper_single**num_components
        clique_upper_unconnected = simp_upper_single**num_components
    # one big component: naively assume that it is of size 2/3 * n and the rest of
    # size ln(n)
    elif p < ln_n / n:
        size = round(2 / 3 * n)
        size_small = round(ln_n)
        num_small = round((n - size) / size_small)
        claw_upper_big = 1 - 1 / inverted_second_moment_claws(size, p)
        claw_upper_small = 1 - 1 / inverted_second_moment_claws(size_small, p)
        simp_upper_big = min(1, first_moment(exp_simp_clique(size, p, size)))
        simp_upper_small = min(
            1, first_moment(exp_simp_clique(size_small, p, size_small))
        )
        claw_upper_unconnected = claw_upper_big * (claw_upper_small**num_small)
        clique_upper_unconnected = simp_upper_big * (simp_upper_small**num_small)
        claw_upper_unconnected = max(0, inverted_first_moment(exp_claws(n, p)))
        clique_upper_unconnected = better_second_moment_cliques(n, p, n)
    # assume one component as approximation
    else:
        claw_upper_unconnected = claw_upper_connected
        clique_upper_unconnected = clique_upper_connected

    return (
        connected * claw_upper_connected * clique_upper_connected
        + (1 - connected) * claw_upper_unconnected * clique_upper_unconnected
    )


def gnp_almost_surely_scf_get_n(p, threshold):
    print(f"p = {p}")
    n = 1
    while True:
        n += 1
        lower = get_lower_bound(n, p)
        if not lower >= threshold:
            break
    return n


def gnp_almost_surely_scf_get_threshold(n, p):
    return get_lower_bound(n, p)


# }}}
