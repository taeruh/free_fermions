use rand::{
    seq::SliceRandom,
    RngCore,
    SeedableRng,
};
use rand_pcg::Pcg64;

// n * 3 single particle
// (n over 2) * 3^2 = (n * (n - 1) / 2) * 3^2 two particle
fn num_ops(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    n * 3 + n * (n - 1) / 2 * 3 * 3
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
// indices go from 1 to n; we include single particle operators, setting their second
// index to 0 and the corresponding data entry to X (no need to introduce an additional
// enum I variant); this way, their second index never matches any other index; doing
// this introduces an overhead for the single particle, however, there are much more two
// particle operators, so this is probably better than introducing an enum to separate
// the two cases or using trait objects opertors
pub struct Operator {
    index: [usize; 2],
    data: [Pauli; 2],
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperatorPool<R> {
    pub ops: Vec<Operator>,
    pub rng: R,
}

impl<R> AsRef<[Operator]> for OperatorPool<R> {
    fn as_ref(&self) -> &[Operator] {
        &self.ops
    }
}

impl<R> OperatorPool<R> {
    fn new_pool(n: usize) -> Vec<Operator> {
        let mut ops = Vec::with_capacity(num_ops(n));

        for i in 1..(n + 1) {
            // single particle
            ops.push(Operator {
                index: [i, 0],
                data: [Pauli::X, Pauli::X],
            });
            ops.push(Operator {
                index: [i, 0],
                data: [Pauli::Y, Pauli::X],
            });
            ops.push(Operator {
                index: [i, 0],
                data: [Pauli::Z, Pauli::X],
            });

            // two particle
            for j in (1 + i)..(n + 1) {
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::X, Pauli::X],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::X, Pauli::Y],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::X, Pauli::Z],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::Y, Pauli::X],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::Y, Pauli::Y],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::Y, Pauli::Z],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::Z, Pauli::X],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::Z, Pauli::Y],
                });
                ops.push(Operator {
                    index: [i, j],
                    data: [Pauli::Z, Pauli::Z],
                });
            }
        }

        debug_assert_eq!(ops.len(), num_ops(n));

        ops
    }

    pub fn new_with(n: usize, rng: R) -> Self {
        Self { ops: Self::new_pool(n), rng }
    }

    /// this is not performant, but we shouldn't need it in a hot loop
    pub fn resize(&mut self, n: usize) {
        self.ops = Self::new_pool(n);
    }
}

impl<R: RngCore> OperatorPool<R> {
    pub fn draw(&mut self, amount: usize) -> impl Iterator<Item = &Operator> {
        self.ops.choose_multiple(&mut self.rng, amount)
    }
}

impl OperatorPool<Pcg64> {
    pub fn new(n: usize) -> Self {
        Self::new_with(n, Pcg64::from_entropy())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    // those tests here are pretty trivial, it's just to make sure that we don't break
    // anything accidentally in the future because we think we can do it smarter, but
    // actually can't

    #[test]
    fn correct_pool() {
        fn equal<A: AsRef<[Operator]>, B: AsRef<[Operator]>>(a: A, b: B) {
            assert_eq!(
                HashSet::<_>::from_iter(a.as_ref()),
                HashSet::from_iter(b.as_ref())
            );
        }

        macro_rules! operator {
            ($i1:expr, $p1:ident, $i2:expr, $p2:ident) => {
                Operator {
                    index: [$i1, $i2],
                    data: [
                        $crate::hamiltonian::Pauli::$p1,
                        $crate::hamiltonian::Pauli::$p2,
                    ],
                }
            };
        }
        macro_rules! single_operator {
            ($i:expr, $p:ident) => {
                operator!($i, $p, 0, X)
            };
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
