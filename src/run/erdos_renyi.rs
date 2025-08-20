use std::{env, fs};

use hashbrown::HashMap;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use super::{GenGraph, density_size_sweep};
use crate::{fix_int::int, graph::generic::ImplGraph, hamiltonian::Density, rand_helper};

const NUM_THREADS: usize = 10;
// const NUM_THREAD_SAMPLES: usize = 5000;
const NUM_THREAD_SAMPLES: usize = 30;

const SIZES: [usize; 2] = [20, 40];
const DENSITY_START: f64 = 0.00;
const DENSITY_END: f64 = 1.;
// const NUM_DENSITY_STEPS: usize = 100;
const NUM_DENSITY_STEPS: usize = 50;

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
    let sizes: Vec<usize> = SIZES.to_vec();

    let edge_pools = sizes
        .iter()
        .map(|&size| {
            let pool = (0..(size as u32) - 1)
                .flat_map(move |i| (i + 1..(size as u32)).map(move |j| (i, j)));
            (size, pool)
        })
        .collect::<HashMap<_, _>>();

    let get_graph = |density: Density, size: usize, rng: &mut Pcg64| -> GenGraph {
        let density = density.get();
        let edges: Vec<(int, int)> = edge_pools
            .get(&size)
            .unwrap()
            .clone()
            .filter(|_| rng.gen::<f64>() <= density)
            .collect();
        GenGraph::from_edge_labels(edges).unwrap()
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
        format!("output/erdos_renyi_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}
