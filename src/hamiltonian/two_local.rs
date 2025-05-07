use rand::Rng;

use super::{Density, SINGLES};
use crate::{
    fix_int::int,
    hamiltonian::{DOUBLES, Pauli},
};

type LocalOperator = super::LocalOperator<2, Pauli>;

#[derive(Debug)]
pub struct TwoLocal {
    operators: Vec<LocalOperator>,
}

// for small n and and especially small k this is fine; for bigger we probably do not want
// to collect and if they are really big it has to be done completely differently, e.g.,
// encode it somehow into numbers that we can directly draw with rand::seq::index::sample
pub fn init_pool(n: usize) -> Vec<LocalOperator> {
    (1..n)
        .flat_map(|i| {
            ((i + 1)..(n + 1))
                .flat_map(move |j| {
                    DOUBLES.into_iter().map(move |p| LocalOperator {
                        index: [i, j],
                        pauli: [p.0, p.1],
                    })
                })
                .chain(SINGLES.into_iter().map(move |p| LocalOperator {
                    index: [i, 0],
                    pauli: [p, Pauli::X],
                }))
        })
        .chain(SINGLES.into_iter().map(move |p| LocalOperator {
            index: [n, 0],
            pauli: [p, Pauli::X],
        }))
        .collect::<Vec<_>>()
}

impl TwoLocal {
    pub fn draw<'a>(
        density: Density,
        pool: impl Iterator<Item = &'a LocalOperator>,
        rng: &mut impl Rng,
    ) -> Self {
        let density = density.0;
        let ops = super::draw_from_iter(density, rng, pool);
        Self { operators: ops }
    }

    pub fn get_graph(&self) -> Vec<(int, int)> {
        super::get_edges(&self.operators)
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn this_test() {
        todo!()
    }
}
