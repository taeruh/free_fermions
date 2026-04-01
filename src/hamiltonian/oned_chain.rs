use rand::Rng;

use super::{Density, Pauli};
use crate::fix_int::int;

type LocalOperator = super::LocalOperator<2, Pauli>;

#[derive(Debug)]
pub struct OpenChain {
    operators: Vec<LocalOperator>,
}

impl OpenChain {
    pub fn draw(density: Density, rng: &mut impl Rng) -> Self {
        const CHAIN_LENGTH : usize = 6;
        let density = density.0;
        let singles = super::draw_singles(density, rng);
        let doubles = super::draw_doubles(density, rng);

        let mut operators = Vec::with_capacity(singles.len() * 4 + doubles.len() * 3);
        for i in 1..CHAIN_LENGTH {
            for s in singles.iter() {
                operators.push(LocalOperator {
                    index: [i, 0],
                    operator_at_index: [*s, Pauli::X],
                });
            }
        }
        for i in 1..CHAIN_LENGTH-1 {
            for d in doubles.iter() {
                operators.push(LocalOperator {
                    index: [i, i + 1],
                    operator_at_index: [d.0, d.1],
                });
            }
        }

        Self { operators }
    }

    pub fn get_graph(&self) -> Vec<(int, int)> {
        super::get_edges(&self.operators)
    }
}
