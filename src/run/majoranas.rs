use std::{env, fs};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use super::density_size_sweep;
use crate::{
    graph::generic::ImplGraph,
    hamiltonian::{Density, electronic_structure::ElectronicStructure},
    rand_helper,
    run::GenGraph,
};

// adjust to hpc_run ncpus (don't need extra thread for main, because it is not doing
// much)
const NUM_THREADS: usize = 50;
// const NUM_THREADS: usize = 10;
// const NUM_THREAD_SAMPLES: usize = 5000; // per thread
const NUM_THREAD_SAMPLES: usize = 2; // per thread

const DENSITY_START: f64 = 0.00;
const DENSITY_END: f64 = 1.0;
// const DENSITY_END: f64 = 0.06;
// const NUM_DENSITY_STEPS: usize = 2000;
const NUM_DENSITY_STEPS: usize = 50;

// these two have to be even
const SIZE_START: usize = 4;
const SIZE_END: usize = 10;
// const SIZE_START: usize = 10;
// const SIZE_END: usize = 16;

fn get_densities() -> Vec<f64> {
    super::uniform_values(DENSITY_START, DENSITY_END, NUM_DENSITY_STEPS).collect()
}

fn get_graph(density: Density, size: usize, rng: &mut Pcg64) -> GenGraph {
    let lattice = ElectronicStructure::draw(density, size, rng);
    GenGraph::from_edge_labels(lattice.get_graph()).unwrap()
}

pub fn run() {
    let id = env::args()
        .nth(1)
        .expect("id not provided")
        .parse::<usize>()
        .expect("id not a number");

    // let seed = 0;
    let seed = Pcg64::from_entropy().gen();

    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(Some(seed));

    let densities = get_densities();
    let sizes = (SIZE_START..=SIZE_END).step_by(2).collect();

    let results = density_size_sweep::sweep(
        seed,
        seeds.to_vec(),
        densities,
        sizes,
        NUM_THREAD_SAMPLES,
        get_graph,
    );

    fs::write(
        format!("output/e_structure_first_{id}.json"),
        // format!("output/e_structure_second_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}
