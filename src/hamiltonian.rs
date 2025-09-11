use std::ops::BitAnd;

use bitvec::vec::BitVec;
use rand::Rng;

use crate::fix_int::int;

#[derive(Clone, Copy)]
pub struct Density(f64);

impl Density {
    pub fn new(f: f64) -> Self {
        assert!((0.0..=1.0).contains(&f));
        Self(f)
    }

    pub fn get(&self) -> f64 {
        self.0
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

#[allow(dead_code)]
#[derive(Debug)]
enum FullPauli {
    I,
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalOperator<const N: usize, Op> {
    pub index: [usize; N],
    pub pauli: [Op; N],
}
// }}}

#[doc = non_semantic_default!()]
impl Default for Pauli {
    fn default() -> Self {
        Pauli::X
    }
}

impl Commutator for Pauli {
    fn commute(&self, other: &Self) -> bool {
        self == other
    }
}

impl<const N: usize, Op: Commutator> Commutator for LocalOperator<N, Op> {
    fn commute(&self, other: &Self) -> bool {
        let mut anticommute = false;
        for s in 0..N {
            for o in 0..N {
                if self.index[s] == other.index[o] && !self.pauli[s].commute(&other.pauli[o]) {
                    anticommute ^= true;
                }
            }
        }
        !anticommute
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PauliString {
    n: usize,
    z: BitVec,
    x: BitVec,
}

impl Commutator for PauliString {
    fn commute(&self, other: &Self) -> bool {
        let zx = self.z.clone().bitand(&other.x);
        let xz = self.x.clone().bitand(&other.z);
        (zx.count_ones() + xz.count_ones()) % 2 == 0
    }
}

impl PauliString {
    #[allow(dead_code)]
    pub fn from_paulis(n: usize, paulis: Vec<LocalOperator<1, Pauli>>) -> Self {
        let mut z = BitVec::repeat(false, n);
        let mut x = BitVec::repeat(false, n);
        for op in paulis {
            match op.pauli[0] {
                Pauli::X => x.set(op.index[0], true),
                Pauli::Y => {
                    x.set(op.index[0], true);
                    z.set(op.index[0], true);
                }
                Pauli::Z => z.set(op.index[0], true),
            }
        }
        Self { n, z, x }
    }

    pub fn from_bit_strings(max_bit: usize, z: usize, x: usize) -> Self {
        Self {
            n: max_bit,
            z: bitvec_from_usize(max_bit, z),
            x: bitvec_from_usize(max_bit, x),
        }
    }

    #[allow(dead_code)]
    pub fn draw_as_paulis(&self) {
        let mut paulis = Vec::with_capacity(self.n);
        for i in 0..self.n {
            let z = *self.z.get(i).unwrap();
            let x = *self.x.get(i).unwrap();
            if z && x {
                paulis.push(FullPauli::Y);
            } else if z {
                paulis.push(FullPauli::Z);
            } else if x {
                paulis.push(FullPauli::X);
            } else {
                paulis.push(FullPauli::I);
            }
        }
        println!("{paulis:?}");
    }
}

fn bitvec_from_usize(max_bit: usize, bits: usize) -> BitVec {
    let mut ret = BitVec::repeat(false, max_bit);
    for i in 0..max_bit {
        if (bits >> i) & 1 == 1 {
            ret.set(i, true);
        }
    }
    ret
}

pub fn get_edges<Op: Commutator>(ops: &[Op]) -> Vec<(int, int)> {
    let mut ret = Vec::new();
    if ops.is_empty() {
        return ret;
    }
    for i in 0..ops.len() - 1 {
        for j in i + 1..ops.len() {
            if !ops[i].commute(&ops[j]) {
                ret.push((i as int, j as int));
            }
        }
    }
    ret
}

const SINGLES: [Pauli; 3] = [Pauli::X, Pauli::Y, Pauli::Z];
pub const DOUBLES: [(Pauli, Pauli); 9] = [
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

pub fn draw_from_iter<'a, I: Copy + 'a>(
    density: f64,
    rng: &mut impl Rng,
    iter: impl Iterator<Item = &'a I>,
) -> Vec<I> {
    iter.filter_map(|&p| {
        if rng.gen::<f64>() < density {
            Some(p)
        } else {
            None
        }
    })
    .collect()
}

fn draw_singles(density: f64, rng: &mut impl Rng) -> Vec<Pauli> {
    draw_from_iter(density, rng, SINGLES.iter())
}

fn draw_doubles(density: f64, rng: &mut impl Rng) -> Vec<(Pauli, Pauli)> {
    draw_from_iter(density, rng, DOUBLES.iter())
}

pub mod bricks;
pub mod electronic_structure;
pub mod oned_chain;
pub mod sparse;
pub mod square_lattice;
pub mod two_local;
