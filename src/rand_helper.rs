use hashbrown::HashSet;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Generate N different seeds with the [ChaCha20Rng] generator.
pub fn generate_seeds<const N: usize>(seed: Option<u64>) -> [u64; N] {
    let rng = &mut if let Some(seed) = seed {
        ChaCha20Rng::seed_from_u64(seed)
    } else {
        ChaCha20Rng::from_entropy()
    };

    let mut seeds = HashSet::<u64>::with_capacity(N);
    while seeds.len() < N {
        let seed = rng.gen();
        if !seeds.contains(&seed) {
            seeds.insert(seed);
        }
    }

    debug_assert_eq!(seeds.len(), N);

    let mut ret = [0; N];
    for (r, s) in ret.iter_mut().zip(seeds.into_iter()) {
        *r = s;
    }
    ret
}
