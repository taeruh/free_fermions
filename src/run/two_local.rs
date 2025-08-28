use std::{env, fs};

use hashbrown::HashMap;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use super::{GenGraph, density_size_sweep};
use crate::{
    graph::generic::ImplGraph,
    hamiltonian::{
        Density,
        two_local::{self, TwoLocal},
    },
    rand_helper,
};

// const NUM_THREADS: usize = 10;
const NUM_THREADS: usize = 50;
const NUM_THREAD_SAMPLES: usize = 10000;
// const NUM_THREAD_SAMPLES: usize = 50;

const SIZES: [usize; 3] = [10, 20, 30];
const DENSITY_START: f64 = 0.00;
const DENSITY_END: f64 = 0.06;
const NUM_DENSITY_STEPS: usize = 500;
// const NUM_DENSITY_STEPS: usize = 40;

pub fn run() {
    let id = env::args()
        .nth(1)
        .expect("id not provided")
        .parse::<usize>()
        .expect("id not a number");

    let seed = Pcg64::from_entropy().gen();
    // let seed = Pcg64::seed_from_u64(0).gen();
    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(Some(seed));

    let densities =
        super::uniform_values(DENSITY_START, DENSITY_END, NUM_DENSITY_STEPS);
    let sizes = SIZES.to_vec();

    let pools = sizes
        .iter()
        .map(|&size| {
            let pool = two_local::init_pool(size);
            (size, pool)
        })
        .collect::<HashMap<_, _>>();

    let get_graph = |density: Density, size: usize, rng: &mut Pcg64| -> GenGraph {
        let lattice = TwoLocal::draw(density, pools.get(&size).unwrap().iter(), rng);
        GenGraph::from_edge_labels(lattice.get_graph()).unwrap()
    };

    let results = density_size_sweep::sweep(
        seed,
        seeds.to_vec(),
        densities.collect(),
        sizes,
        NUM_THREAD_SAMPLES,
        get_graph,
    );

    fs::write(
        format!("output/two_local_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}
