use rand::Rng;

use crate::fix_int::int;

#[derive(Clone, Copy)]
pub struct Density(f64);

impl Density {
    pub fn new(f: f64) -> Self {
        assert!((0.0..=1.0).contains(&f));
        Self(f)
    }
}

pub trait Commutator {
    fn commute(&self, other: &Self) -> bool;
}

// {{{
// indices go from 1 to n; we include single particle operators, setting their second
// index to 0 and the corresponding data entry to X (no need to introduce an additional
// enum I variant); this way, their second index never matches any other index; doing
// this introduces an overhead for the single particle, however, there are much more two
// particle operators, so this is probably better than introducing an enum to separate
// the two cases or using trait objects opertors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pauli {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalOperator<const N: usize> {
    index: [usize; N],
    pauli: [Pauli; N],
}
// }}}

#[doc = non_semantic_default!()]
impl Default for Pauli {
    fn default() -> Self {
        Pauli::X
    }
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

fn get_edges<const N: usize>(ops: &[LocalOperator<N>]) -> Vec<(int, int)> {
    let mut ret = Vec::new();
    if ops.is_empty() {
        return ret;
    }
    for i in 0..ops.len() - 1 {
        for j in i + 1..ops.len() {
            if ops[i].commute(&ops[j]) {
                ret.push((i as int, j as int));
            }
        }
    }
    ret
}

const SINGLES: [Pauli; 3] = [Pauli::X, Pauli::Y, Pauli::Z];
const DOUBLES: [(Pauli, Pauli); 9] = [
    (Pauli::X, Pauli::X),
    (Pauli::X, Pauli::Y),
    (Pauli::X, Pauli::Z),
    (Pauli::Y, Pauli::X),
    (Pauli::Y, Pauli::Y),
    (Pauli::Y, Pauli::Z),
    (Pauli::Z, Pauli::X),
    (Pauli::Z, Pauli::Y),
    (Pauli::Z, Pauli::Z),
];

fn draw_singles(density: f64, rng: &mut impl Rng) -> Vec<Pauli> {
    SINGLES
        .iter()
        .filter_map(|&p| {
            if rng.gen::<f64>() < density {
                Some(p)
            } else {
                None
            }
        })
        .collect()
}

fn draw_doubles(density: f64, rng: &mut impl Rng) -> Vec<(Pauli, Pauli)> {
    DOUBLES
        .iter()
        .filter_map(|&p| {
            if rng.gen::<f64>() < density {
                Some(p)
            } else {
                None
            }
        })
        .collect()
}

pub mod oned_chain;
pub mod square_lattice;
pub mod bricks;
