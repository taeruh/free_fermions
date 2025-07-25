//! 2d shifted bricks: (without the ``ignore, rust wants to run compile that ...)
//! ```ignore
//! -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
//!   |           |           |           |           |           |
//! -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
//!      |           |           |           |           |        
//! -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
//!         |           |           |           |           |
//! -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
//!            |           |           |           |           |
//! -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
//!   |           |           |           |           |           |
//! -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- -- --
//! ```
//!
//! we do it periodic again; 2 rows are sufficient -> check: draw the line graph of it to
//! see how big cliques and where the claws could be
//! ```ignore
//!                 |                           |
//! row 0  --6------7------0--e1--1--e2--2--e3--3--e4--4------5--
//!                        |                           |
//!                        e5                          |
//!                        |                           |
//! row 1         --6------7------0------1------2------3------4------5--
//!                               |                           |
//! ```

use itertools::Itertools;
use rand::{Rng, seq::SliceRandom};

use super::{Density, Pauli};
use crate::{fix_int::int, hamiltonian::DOUBLES};

type LocalOperator = super::LocalOperator<2, Pauli>;

#[derive(Debug)]
pub struct Bricks {
    pub operators: Vec<LocalOperator>,
}

impl Bricks {
    pub fn draw(
        e1_density: Density,
        e2_density: Density,
        e3_density: Density,
        e4_density: Density,
        e5_density: Density,
        rng: &mut impl Rng,
    ) -> Self {
        // // basic connected (frustration graph) example
        // let e1 = [(Pauli::Z, Pauli::Z)];
        // let e2 = [(Pauli::X, Pauli::X), (Pauli::Y, Pauli::Y)];
        // let e3 = [(Pauli::Z, Pauli::Z)];
        // let e4 = [(Pauli::X, Pauli::X)];
        // let e5 = [(Pauli::Y, Pauli::Y)];
        // let num_ops = e1.len() + e2.len() + e3.len() + e4.len() + e5.len();

        // // simple example where a three-sibling sets collapses and before it wasn't
        // claw-free but afterwards it is the 3-edge path (in 4 disconnected components)
        // let e1 = [(Pauli::X, Pauli::Y), (Pauli::X, Pauli::Z)];
        // let e2 = [(Pauli::X, Pauli::Z), (Pauli::Z, Pauli::Y)];
        // let e3 = [(Pauli::Z, Pauli::Y)];
        // let e4 = [(Pauli::Y, Pauli::Z)];
        // let e5 = [(Pauli::Z, Pauli::Y)];
        // let num_ops = e1.len() + e2.len() + e3.len() + e4.len() + e5.len();

        let mut num_ops = 0;
        let (e1, e2, e3, e4, e5) =
            [e1_density, e2_density, e3_density, e4_density, e5_density]
                .into_iter()
                .enumerate()
                .map(|(i, density)| {
                    let d = density.0;
                    assert!(d >= 1. / 9., "density {i} too low");
                    let mut e = super::draw_doubles(d, rng);
                    while e.is_empty() {
                        e = super::draw_doubles(d, rng);
                    }
                    num_ops += e.len();
                    e
                })
                .collect_tuple()
                .unwrap();

        let mut operators = Vec::with_capacity(num_ops * 4);

        for i in 0..2 {
            let row = i * 8;
            for j in 0..2 {
                let col = j * 4;
                for p in e5.iter() {
                    operators.push(LocalOperator {
                        index: [row + col, ((row + 8) % 16) + ((col + 7) % 8)],
                        pauli: [p.0, p.1],
                    });
                }
                for p in e1.iter() {
                    operators.push(LocalOperator {
                        index: [row + col, row + col + 1],
                        pauli: [p.0, p.1],
                    });
                }
                for p in e2.iter() {
                    operators.push(LocalOperator {
                        index: [row + col + 1, row + col + 2],
                        pauli: [p.0, p.1],
                    });
                }
                for p in e3.iter() {
                    operators.push(LocalOperator {
                        index: [row + col + 2, row + col + 3],
                        pauli: [p.0, p.1],
                    });
                }
                for p in e4.iter() {
                    operators.push(LocalOperator {
                        index: [row + col + 3, row + (col + 4) % 8],
                        pauli: [p.0, p.1],
                    });
                }
            }
        }

        Self { operators }
    }

    pub fn get_graph(&self) -> Vec<(int, int)> {
        super::get_edges(&self.operators)
    }
}
