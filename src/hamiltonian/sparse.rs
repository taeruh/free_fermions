use rand::{Rng, SeedableRng, seq::index};
use rand_pcg::Pcg64;

use super::PauliString;
use crate::fix_int::int;

pub struct Sparse {
    pub ops: Vec<PauliString>,
}

impl Sparse {
    pub fn draw<'a>(num_operators: usize, num_spins: usize, rng: &mut impl Rng) -> Self {
        assert!(num_spins <= 31); // otherwise the shifting we do below will overflow

        let max_code = (1 << (2 * num_spins)) - 1;

        // index::sample does not allow equality (I guess because they want to prevent
        // unnecessary sampling if we sample everything anyways ...); we should allow this
        // but we don't need it because we never sample that much
        assert!(num_operators < max_code + 1);

        let mut ops = Vec::with_capacity(num_operators);
        println!("{:?}", max_code);

        for code in index::sample(rng, max_code, num_operators) {
            let x = code >> num_spins;
            let z = code;
            ops.push(PauliString::from_bit_strings(num_spins, z, x));
        }

        Sparse { ops }
    }

    pub fn get_graph(&self) -> Vec<(int, int)> {
        super::get_edges(&self.ops)
    }
}
