//! 2d square lattices

use rand::Rng;

use super::{Density, Pauli};
use crate::fix_int::int;

type LocalOperator = super::LocalOperator<2, Pauli>;

#[derive(Debug)]
pub struct PeriodicLattice {
    pub operators: Vec<LocalOperator>,
    pub is_2d: bool,
}

impl PeriodicLattice {
    // translational invariant (separable in x and y direction)
    pub fn draw(
        e_density: Density,
        n_density: Density,
        ee_density: Density,
        en_density: Density,
        rng: &mut impl Rng,
    ) -> Self {
        let electrons = super::draw_singles(e_density.0, rng);
        let nuclei = super::draw_singles(n_density.0, rng);
        let ee_horizontal = super::draw_doubles(ee_density.0, rng);
        let ee_vertical = super::draw_doubles(ee_density.0, rng);
        let en_interactions = super::draw_doubles(en_density.0, rng);

        let is_2d = !ee_horizontal.is_empty() && !ee_vertical.is_empty();

        let mut operators = Vec::with_capacity(
            electrons.len() * 9
                + nuclei.len() * 9
                + ee_horizontal.len() * 12
                + ee_vertical.len() * 12
                + en_interactions.len() * 9,
        );
        for i in 0..3 {
            for j in 0..3 {
                for p in electrons.iter() {
                    operators.push(LocalOperator {
                        index: [1 + i * 3 + j, 0],
                        operator_at_index: [*p, Pauli::X],
                    });
                }
                for p in nuclei.iter() {
                    operators.push(LocalOperator {
                        index: [10 + i * 3 + j, 0],
                        operator_at_index: [*p, Pauli::X],
                    });
                }
                for p in en_interactions.iter() {
                    operators.push(LocalOperator {
                        index: [1 + i * 3 + j, 10 + i * 3 + j],
                        operator_at_index: [p.0, p.1],
                    });
                }
            }
        }
        for i in 0..3 {
            for j in 0..3 {
                let site = 1 + i * 3 + j;
                let horizontal = 1 + i * 3 + (j + 1) % 3;
                let vertical = 1 + (i + 1) % 3 * 3 + j;
                for p in ee_horizontal.iter() {
                    operators.push(LocalOperator {
                        index: [site, horizontal],
                        operator_at_index: [p.0, p.1],
                    });
                }
                for p in ee_vertical.iter() {
                    operators.push(LocalOperator {
                        index: [site, vertical],
                        operator_at_index: [p.0, p.1],
                    });
                }
            }
        }

        Self { operators, is_2d }
    }

    pub fn get_graph(&self) -> Vec<(int, int)> {
        super::get_edges(&self.operators)
    }
}
