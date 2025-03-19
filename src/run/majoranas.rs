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
const NUM_SAMPLES: usize = 100; // per thread

const DENSITY_START: f64 = 0.00;
const DENSITY_END: f64 = 0.06;
// const DENSITY_END: f64 = 1.0;
const NUM_DENSITY_STEPS: usize = 200;
// const NUM_DENSITY_STEPS: usize = 100;

// these two have to be even
const SIZE_START: usize = 10;
const SIZE_END: usize = 18;
// const SIZE_START: usize = 2;
// const SIZE_END: usize = 10;
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
    before_claw_free: Vec<Vec<f64>>,
    after_claw_free: Vec<Vec<f64>>,
    before_simplicial: Vec<Vec<f64>>,
    after_simplicial: Vec<Vec<f64>>,
    collapsed: Vec<Vec<f64>>,
}

struct CountResults {
    before_claw_free: Vec<Vec<usize>>,
    after_claw_free: Vec<Vec<usize>>,
    before_simplicial: Vec<Vec<usize>>,
    after_simplicial: Vec<Vec<usize>>,
    collapsed: Vec<Vec<f64>>,
}

impl CountResults {
    fn init() -> Self {
        Self {
            before_claw_free: vec![vec![0; NUM_DENSITY_STEPS]; NUM_SIZES],
            after_claw_free: vec![vec![0; NUM_DENSITY_STEPS]; NUM_SIZES],
            before_simplicial: vec![vec![0; NUM_DENSITY_STEPS]; NUM_SIZES],
            after_simplicial: vec![vec![0; NUM_DENSITY_STEPS]; NUM_SIZES],
            collapsed: vec![vec![0.; NUM_DENSITY_STEPS]; NUM_SIZES],
        }
    }

    fn merge(results: Vec<Self>) -> Self {
        let mut ret = Self::init();
        for result in results {
            for i in 0..NUM_SIZES {
                let ret_before_claw_free = ret.before_claw_free.get_mut(i).unwrap();
                let before_claw_free = result.before_claw_free.get(i).unwrap();
                let ret_after_claw_free = ret.after_claw_free.get_mut(i).unwrap();
                let after_claw_free = result.after_claw_free.get(i).unwrap();
                let ret_before_simp = ret.before_simplicial.get_mut(i).unwrap();
                let before_simp = result.before_simplicial.get(i).unwrap();
                let ret_after_simp = ret.after_simplicial.get_mut(i).unwrap();
                let after_simp = result.after_simplicial.get(i).unwrap();
                let ret_collapsed = ret.collapsed.get_mut(i).unwrap();
                let collapsed = result.collapsed.get(i).unwrap();
                for j in 0..NUM_DENSITY_STEPS {
                    ret_before_claw_free[j] += before_claw_free[j];
                    ret_after_claw_free[j] += after_claw_free[j];
                    ret_before_simp[j] += before_simp[j];
                    ret_after_simp[j] += after_simp[j];
                    ret_collapsed[j] += collapsed[j];
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
            before_claw_free: vec![vec![0.0; NUM_DENSITY_STEPS]; NUM_SIZES],
            after_claw_free: vec![vec![0.0; NUM_DENSITY_STEPS]; NUM_SIZES],
            before_simplicial: vec![vec![0.0; NUM_DENSITY_STEPS]; NUM_SIZES],
            after_simplicial: vec![vec![0.0; NUM_DENSITY_STEPS]; NUM_SIZES],
            collapsed: vec![vec![0.0; NUM_DENSITY_STEPS]; NUM_SIZES],
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
            let ret_before_claw_free = ret.before_claw_free.get_mut(i).unwrap();
            let before_claw_free = results.before_claw_free.get(i).unwrap();
            let ret_after_claw_free = ret.after_claw_free.get_mut(i).unwrap();
            let after_claw_free = results.after_claw_free.get(i).unwrap();
            let ret_before_simp = ret.before_simplicial.get_mut(i).unwrap();
            let before_simp = results.before_simplicial.get(i).unwrap();
            let ret_after_simp = ret.after_simplicial.get_mut(i).unwrap();
            let after_simp = results.after_simplicial.get(i).unwrap();
            let ret_collapsed = ret.collapsed.get_mut(i).unwrap();
            let collapsed = results.collapsed.get(i).unwrap();
            for j in 0..NUM_DENSITY_STEPS {
                ret_before_claw_free[j] =
                    before_claw_free[j] as f64 / NUM_TOTAL_SAMPLES as f64;
                ret_after_claw_free[j] =
                    after_claw_free[j] as f64 / NUM_TOTAL_SAMPLES as f64;
                ret_before_simp[j] = before_simp[j] as f64 / NUM_TOTAL_SAMPLES as f64;
                ret_after_simp[j] = after_simp[j] as f64 / NUM_TOTAL_SAMPLES as f64;
                ret_collapsed[j] = collapsed[j] / NUM_TOTAL_SAMPLES as f64;
            }
        }
        ret
    }
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

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let mut ret = CountResults::init();

        for (size_idx, size) in (SIZE_START..=SIZE_END).step_by(2).enumerate() {
            println!("{:?}", size);
            let ret_before_claw_free = ret.before_claw_free.get_mut(size_idx).unwrap();
            let ret_after_claw_free = ret.after_claw_free.get_mut(size_idx).unwrap();
            let ret_before_simp = ret.before_simplicial.get_mut(size_idx).unwrap();
            let ret_after_simp = ret.after_simplicial.get_mut(size_idx).unwrap();
            let ret_collapsed = ret.collapsed.get_mut(size_idx).unwrap();

            for (density_idx, density) in densities.iter().copied().enumerate() {
                let d = Density::new(density);
                let mut before_claw_free = 0;
                let mut after_claw_free = 0;
                let mut before_simplicial = 0;
                let mut after_simplicial = 0;
                let mut collapsed = 0.;

                let mut i = 0;
                while i < NUM_SAMPLES {
                    // println!("{:?}", i);
                    let lattice = ElectronicStructure::draw(d, size, rng);

                    let mut graph =
                        GenGraph::from_edge_labels(lattice.get_graph()).unwrap();

                    if graph.is_empty() {
                        i += 1;
                        before_claw_free += 1;
                        after_claw_free += 1;
                        before_simplicial += 1;
                        after_simplicial += 1;
                        continue;
                    }

                    let orig_len = graph.len();
                    let mut tree = graph.modular_decomposition();

                    let check = check::do_gen_check(&graph, &tree);
                    if check.claw_free {
                        before_claw_free += 1;
                    }
                    if check.simplicial {
                        before_simplicial += 1;
                    }

                    graph.twin_collapse(&mut tree);
                    collapsed += (orig_len - graph.len()) as f64 / orig_len as f64;

                    let check = check::do_gen_check(&graph, &tree);
                    if check.claw_free {
                        after_claw_free += 1;
                    }
                    if check.simplicial {
                        after_simplicial += 1;
                    }

                    i += 1;
                }

                ret_before_claw_free[density_idx] = before_claw_free;
                ret_after_claw_free[density_idx] = after_claw_free;
                ret_before_simp[density_idx] = before_simplicial;
                ret_after_simp[density_idx] = after_simplicial;
                ret_collapsed[density_idx] = collapsed;
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
        // format!("output/e_structure_first_{id}.json"),
        format!("output/e_structure_second_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}
