use rand::{Rng, distributions::Bernoulli, prelude::Distribution};

use super::{Commutator, Density};
use crate::fix_int::int;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MajoranaString {
    ops: Vec<bool>,
}

impl Commutator for MajoranaString {
    fn commute(&self, other: &Self) -> bool {
        let (abs_self, abs_other, inner) = self
            .ops
            .iter()
            .zip(other.ops.iter())
            .fold((false, false, false), |(s, o, i), (&x, &y)| {
                (s ^ x, o ^ y, i ^ (x && y))
            });
        (abs_self && abs_other) ^ inner
    }
}

#[derive(Debug)]
pub struct ElectronicStructure {
    operators: Vec<MajoranaString>,
}

impl ElectronicStructure {
    pub fn draw(density: Density, n: usize, rng: &mut impl Rng) -> Self {
        let density = density.0;
        let dist = Bernoulli::new(density).unwrap();

        let n_choose_2 = n * (n - 1) / 2;
        let n_choose_4 = n * (n - 1) * (n - 2) * (n - 3) / 24;

        let mut operators: Vec<MajoranaString> = Vec::with_capacity(
            ((n_choose_2 as f64 + n_choose_4 as f64) * density) as usize,
        );

        operators.extend(
            (0..n)
                .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
                .filter_map(|(i, j)| {
                    if dist.sample(rng) {
                        let mut ops = vec![false; n];
                        ops[i] = true;
                        ops[j] = true;
                        Some(MajoranaString { ops })
                    } else {
                        None
                    }
                }),
        );
        operators.extend(
            (0..n)
                .flat_map(|i| {
                    (i + 1..n).flat_map(move |j| {
                        (j + 1..n)
                            .flat_map(move |k| (k + 1..n).map(move |l| (i, j, k, l)))
                    })
                })
                .filter_map(|(i, j, k, l)| {
                    if dist.sample(rng) {
                        let mut ops = vec![false; n];
                        ops[i] = true;
                        ops[j] = true;
                        ops[k] = true;
                        ops[l] = true;
                        Some(MajoranaString { ops })
                    } else {
                        None
                    }
                }),
        );

        ElectronicStructure { operators }
    }

    pub fn get_graph(&self) -> Vec<(int, int)> {
        super::get_edges(&self.operators)
    }
}
