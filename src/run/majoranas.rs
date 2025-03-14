use std::{env, fs, thread};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    graph::generic::ImplGraph,
    hamiltonian::{Density, electronic_structure::ElectronicStructure},
    rand_helper,
    run::{GenGraph, check},
};

// adjust to hpc_run ncpus (don't need extra thread for main, because it is not doing
// much)
const NUM_THREADS: usize = 50;
// const NUM_THREADS: usize = 10;
const NUM_SAMPLES: usize = 1000; // per thread
// const NUM_SAMPLES: usize = 20; // per thread

const DENSITY_START: f64 = 0.00;
// const DENSITY_END: f64 = 0.40;
const DENSITY_END: f64 = 1.00;
// const DENSITY_END: f64 = 0.20;
const NUM_DENSITY_STEPS: usize = 2000;
// const NUM_DENSITY_STEPS: usize = 40;

// these two have to be even
const SIZE_START: usize = 2;
const SIZE_END: usize = 10;
const NUM_SIZES: usize = (SIZE_END - SIZE_START) / 2 + 1;

const NUM_TOTAL_SAMPLES: usize = NUM_THREADS * NUM_SAMPLES;

fn get_densities() -> Vec<f64> {
    const DELTA: f64 = (DENSITY_END - DENSITY_START) / (NUM_DENSITY_STEPS - 1) as f64;
    (0..NUM_DENSITY_STEPS)
        .map(|i| DENSITY_START + DELTA * (i as f64))
        .collect()
}

#[derive(Debug, Serialize)]
struct Results {
    densities: Vec<f64>,
    sizes: Vec<usize>,
    seed: u64,
    num_samples: usize,
    // first index is the size, second is the density; averaged over the samples
    simplicial: Vec<Vec<f64>>,
}

struct CountResults {
    simplicial: Vec<Vec<usize>>,
}

impl CountResults {
    fn init() -> Self {
        Self {
            simplicial: vec![vec![0; NUM_DENSITY_STEPS]; NUM_SIZES],
        }
    }

    fn merge(results: Vec<Self>) -> Self {
        let mut ret = Self::init();
        for result in results {
            for i in 0..NUM_SIZES {
                let ret_simp = ret.simplicial.get_mut(i).unwrap();
                let result_simp = result.simplicial.get(i).unwrap();
                for j in 0..NUM_DENSITY_STEPS {
                    ret_simp[j] += result_simp[j];
                }
            }
        }
        ret
    }
}

impl Results {
    fn init() -> Self {
        Self {
            densities: Vec::new(),
            sizes: Vec::new(),
            seed: 0,
            num_samples: NUM_TOTAL_SAMPLES,
            simplicial: vec![vec![0.0; NUM_DENSITY_STEPS]; NUM_SIZES],
        }
    }

    fn normalise_merged_count_results(
        results: CountResults,
        densities: Vec<f64>,
        sizes: Vec<usize>,
        seed: u64,
    ) -> Self {
        let mut ret = Self::init();
        ret.densities = densities;
        ret.sizes = sizes;
        ret.seed = seed;

        for i in 0..NUM_SIZES {
            let ret_simp = ret.simplicial.get_mut(i).unwrap();
            let results_simp = results.simplicial.get(i).unwrap();
            for j in 0..NUM_DENSITY_STEPS {
                ret_simp[j] = results_simp[j] as f64 / NUM_TOTAL_SAMPLES as f64;
            }
        }
        ret
    }
}

pub fn periodic() {
    let id = env::args()
        .nth(1)
        .expect("id not provided")
        .parse::<usize>()
        .expect("id not a number");

    // let seed = 0;
    let seed = Pcg64::from_entropy().gen();

    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(Some(seed));
    let densities = get_densities();

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let mut ret = CountResults::init();

        for (size_idx, size) in (SIZE_START..=SIZE_END).step_by(2).enumerate() {
            println!("{:?}", size);
            let ret_simp = ret.simplicial.get_mut(size_idx).unwrap();
            for (density_idx, density) in densities.iter().copied().enumerate() {
                let d = Density::new(density);
                let mut simplicial = 0;

                let mut i = 0;
                while i < NUM_SAMPLES {
                    // println!("{:?}", i);
                    let lattice = ElectronicStructure::draw(d, size, rng);

                    let mut graph =
                        GenGraph::from_edge_labels(lattice.get_graph()).unwrap();

                    if graph.is_empty() {
                        i += 1;
                        simplicial += 1;
                        continue;
                    }

                    let mut tree = graph.modular_decomposition();
                    graph.twin_collapse(&mut tree);
                    let check = check::do_gen_check(&graph, &tree);

                    if check.simplicial {
                        simplicial += 1;
                    }

                    i += 1;
                }

                ret_simp[density_idx] = simplicial;
            }
        }
        println!("thread {id} finished");
        ret
    };

    let results: Vec<_> = thread::scope(|scope| {
        let handles: Vec<_> =
            (0..NUM_THREADS).map(|i| scope.spawn(move || job(i))).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let results = Results::normalise_merged_count_results(
        CountResults::merge(results),
        densities,
        (SIZE_START..=SIZE_END).step_by(2).collect(),
        seed,
    );

    fs::write(
        // format!("output/e_structure_{id}.json"),
        format!("output/e_structure_first_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}

pub fn run() {
    periodic()
}
