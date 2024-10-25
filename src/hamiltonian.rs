use std::{collections::HashSet, hash::Hash};

use bitvec::vec::BitVec;
use num::integer;
use rand::{
    RngCore, SeedableRng,
    distributions::Uniform,
    prelude::Distribution,
    seq::{
        SliceRandom,
        index::{self},
    },
};
use rand_pcg::Pcg64;

#[derive(Clone, Copy)]
pub struct Density(f64);

impl Density {
    pub fn new(f: f64) -> Self {
        assert!((0.0..=1.0).contains(&f));
        Self(f)
    }
}

// n * 3 single particle
// (n over 2) * 3^2 = (n * (n - 1) / 2) * 3^2 two particle
pub fn num_ops(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    n * 3 + n * (n - 1) / 2 * 3 * 3
}

pub trait Commutator {
    fn commute(&self, other: &Self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pauli {
    X,
    Y,
    Z,
}

#[doc = non_semantic_default!()]
impl Default for Pauli {
    fn default() -> Self {
        Pauli::X
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// indices go from 1 to n; we include single particle operators, setting their second
// index to 0 and the corresponding data entry to X (no need to introduce an additional
// enum I variant); this way, their second index never matches any other index; doing
// this introduces an overhead for the single particle, however, there are much more two
// particle operators, so this is probably better than introducing an enum to separate
// the two cases or using trait objects opertors
pub struct LocalOperator<const N: usize> {
    index: [usize; N],
    pauli: [Pauli; N],
}

impl<const N: usize> Commutator for LocalOperator<N> {
    fn commute(&self, other: &Self) -> bool {
        let mut res = false;
        for s in 0..N {
            for o in 0..N {
                if self.index[s] == other.index[o] && self.pauli[s] != other.pauli[o] {
                    res ^= true;
                }
            }
        }
        res
    }
}

pub struct OperatorString {
    len: usize,
    z: BitVec,
    x: BitVec,
}

impl Commutator for OperatorString {
    fn commute(&self, other: &Self) -> bool {
        debug_assert_eq!(self.len, other.len);
        let r = self.z.clone() & &other.x;
        r.count_ones() % 2 == 1
    }
}

impl OperatorString {
    pub fn len(&self) -> usize {
        self.len
    }

    // pub fn new(
}

/// # About the draw_.*sets methods
///
/// For enough samples, there shouldn't be a real difference, but we might see some
/// effects in edge cases. The ...
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperatorPool<O, R> {
    pub ops: Vec<O>,
    pub rng: R,
    pub len: usize,
}

impl<O, R> AsRef<[O]> for OperatorPool<O, R> {
    fn as_ref(&self) -> &[O] {
        &self.ops
    }
}

impl<R> OperatorPool<R> {
    fn new_pool(n: usize) -> Vec<LocalOperator> {
        let mut ops = Vec::with_capacity(num_ops(n));

        for i in 1..(n + 1) {
            // single particle
            ops.push(LocalOperator {
                index: [i, 0],
                pauli: [Pauli::X, Pauli::X],
            });
            ops.push(LocalOperator {
                index: [i, 0],
                pauli: [Pauli::Y, Pauli::X],
            });
            ops.push(LocalOperator {
                index: [i, 0],
                pauli: [Pauli::Z, Pauli::X],
            });

            // two particle
            for j in (1 + i)..(n + 1) {
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::X, Pauli::X],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::X, Pauli::Y],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::X, Pauli::Z],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::Y, Pauli::X],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::Y, Pauli::Y],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::Y, Pauli::Z],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::Z, Pauli::X],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::Z, Pauli::Y],
                });
                ops.push(LocalOperator {
                    index: [i, j],
                    pauli: [Pauli::Z, Pauli::Z],
                });
            }
        }

        debug_assert_eq!(ops.len(), num_ops(n));

        ops
    }

    pub fn new_with(n: usize, rng: R) -> Self {
        Self {
            ops: Self::new_pool(n),
            rng,
            len: num_ops(n),
        }
    }

    /// this is not performant, but we shouldn't need it in a hot loop
    pub fn resize(&mut self, n: usize) {
        self.ops = Self::new_pool(n);
    }

    /// The number of possible operators.
    pub fn num_ops(&self) -> usize {
        self.ops.len()
    }

    /// The number of distinct sets with `k` many operators.
    ///
    /// If `self.num_ops() > 67`, it might overflow.
    pub fn num_distinct_k_sets(&self, k: usize) -> usize {
        integer::binomial(self.len, k)
    }
}

#[derive(Debug)]
pub struct OperatorIter<'l> {
    ops: &'l Vec<LocalOperator>,
    ind: std::collections::hash_set::IntoIter<usize>,
}

impl<'l> Iterator for OperatorIter<'l> {
    type Item = &'l LocalOperator;
    fn next(&mut self) -> Option<Self::Item> {
        self.ind.next().map(|i| &self.ops[i])
    }
}

pub fn draw_with_rate<'l>(
    ops: &'l [LocalOperator],
    rng: &mut impl RngCore,
    dist: &impl Distribution<f64>,
    acceptance_probability: Density,
) -> Vec<&'l LocalOperator> {
    ops.iter()
        .filter(|_| dist.sample(rng) < acceptance_probability.0)
        .collect()
}

impl<R: RngCore> OperatorPool<R> {
    pub fn draw_set_with_probability(
        &mut self,
        density: Density,
    ) -> Vec<&LocalOperator> {
        let dist = Uniform::new_inclusive(0.0, 1.0);
        draw_with_rate(&self.ops, &mut self.rng, &dist, density)
    }

    pub fn draw_multiple_sets_with_probability(
        &mut self,
        density: Density,
        amount: usize,
    ) -> Vec<Vec<&LocalOperator>> {
        let dist = Uniform::new_inclusive(0.0, 1.0);
        let mut res = Vec::with_capacity(amount);
        for _ in 0..amount {
            res.push(draw_with_rate(&self.ops, &mut self.rng, &dist, density));
        }
        res
    }

    /// Draw `amount` many distinct operators from the pool.
    pub fn draw(&mut self, amount: usize) -> impl Iterator<Item = &LocalOperator> {
        self.ops.choose_multiple(&mut self.rng, amount)
    }

    /// Draw `amount` many distinct operator sets, each with distinct operators.
    ///
    /// # Important!
    ///
    /// This function will loop endlessly if `amount` is larger than the number of
    /// possible distinct sets, i.e., if `amount > self.num_distinct_k_sets(set_size)`
    /// (this is checked, in debug mode, if `self.num_ops <= 67`, which is true for less
    /// then 5 qubits).
    pub fn draw_exact_distinct_sets(
        &mut self,
        amount: usize,
        set_size: usize,
    ) -> impl Iterator<Item = impl Iterator<Item = &LocalOperator>> {
        #[cfg(debug_assertions)]
        if self.num_ops() <= 67 {
            assert!(
                amount <= self.num_distinct_k_sets(set_size),
                "amount > self.num_distinct_k_sets(set_size); would loop endlessly"
            );
        }

        let mut sets = Vec::<HashSet<_>>::with_capacity(amount);
        let mut len = 0;
        'outer: while len < amount {
            let set =
                HashSet::from_iter(index::sample(&mut self.rng, self.len, set_size));
            for s in sets.iter() {
                if *s == set {
                    continue 'outer;
                }
            }
            sets.push(set);
            len += 1;
        }

        sets.into_iter().map(|e| OperatorIter {
            ops: &self.ops,
            ind: e.into_iter(),
        })
    }

    /// Draw up to `amount` many distinct operators sets, each with distinct operators.
    pub fn draw_distinct_sets(
        &mut self,
        amount: usize,
        set_size: usize,
    ) -> impl Iterator<Item = impl Iterator<Item = &LocalOperator>> {
        let mut sets = Vec::<HashSet<_>>::new();
        'outer: for _ in 0..amount {
            let set =
                HashSet::from_iter(index::sample(&mut self.rng, self.len, set_size));
            for s in sets.iter() {
                if *s == set {
                    continue 'outer;
                }
            }
            sets.push(set);
        }

        sets.into_iter().map(|e| OperatorIter {
            ops: &self.ops,
            ind: e.into_iter(),
        })
    }

    /// Draw `amount` many operators sets, each with distinct operators.
    pub fn draw_sets(
        &mut self,
        amount: usize,
        set_size: usize,
    ) -> impl Iterator<Item = impl Iterator<Item = &LocalOperator>> {
        let mut sets = Vec::with_capacity(amount);
        for _ in 0..amount {
            sets.push(self.ops.choose_multiple(&mut self.rng, set_size));
        }
        sets.into_iter()
    }
}

impl OperatorPool<Pcg64> {
    pub fn new(n: usize) -> Self {
        Self::new_with(n, Pcg64::from_entropy())
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;

    use super::*;

    macro_rules! operator {
        ($i1:expr, $p1:ident, $i2:expr, $p2:ident) => {
            Operator {
                index: [$i1, $i2],
                data: [$crate::hamiltonian::Pauli::$p1, $crate::hamiltonian::Pauli::$p2],
            }
        };
    }
    pub(crate) use operator;
    macro_rules! single_operator {
        ($i:expr, $p:ident) => {
            operator!($i, $p, 0, X)
        };
    }
    pub(crate) use single_operator;

    // those tests here are pretty trivial, it's just to make sure that we don't break
    // anything accidentally in the future because we think we can do it smarter, but
    // actually can't

    #[test]
    fn correct_pool() {
        fn equal<A: AsRef<[LocalOperator]>, B: AsRef<[LocalOperator]>>(a: A, b: B) {
            assert_eq!(
                HashSet::<_>::from_iter(a.as_ref()),
                HashSet::from_iter(b.as_ref())
            );
        }

        macro_rules! singles {
            ($i:expr) => {
                [
                    single_operator!($i, X),
                    single_operator!($i, Y),
                    single_operator!($i, Z),
                ]
            };
        }
        macro_rules! doubles {
            ($i1:expr, $i2:expr) => {
                [
                    operator!($i1, X, $i2, X),
                    operator!($i1, X, $i2, Y),
                    operator!($i1, X, $i2, Z),
                    operator!($i1, Y, $i2, X),
                    operator!($i1, Y, $i2, Y),
                    operator!($i1, Y, $i2, Z),
                    operator!($i1, Z, $i2, X),
                    operator!($i1, Z, $i2, Y),
                    operator!($i1, Z, $i2, Z),
                ]
            };
        }

        equal(OperatorPool::new(0), []);
        equal(OperatorPool::new(1), singles!(1));
        equal(OperatorPool::new(2), {
            let mut res = Vec::new();
            for i in 1..3 {
                res.extend_from_slice(&singles!(i));
            }
            res.extend_from_slice(&doubles!(1, 2));
            res
        });
        equal(OperatorPool::new(3), {
            let mut res = Vec::new();
            for i in 1..4 {
                res.extend_from_slice(&singles!(i));
            }
            for (i, j) in [(1, 2), (1, 3), (2, 3)].iter() {
                res.extend_from_slice(&doubles!(*i, *j));
            }
            res
        })
    }

    #[test]
    fn distinct() {
        let n = 7;
        let max = num_ops(n);
        let mut pool = OperatorPool::new(n);

        let mut check = move |amount| {
            assert_eq!(
                HashSet::<_>::from_iter(pool.draw(amount)).len(),
                if amount > max { max } else { amount }
            );
        };

        check(0);
        check(1);
        check(2);
        check(max - 2);
        check(max - 1);
        check(max);
        check(max + 1);
    }
}
