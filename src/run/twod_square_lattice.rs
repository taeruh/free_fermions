//! 2d square lattice

use std::{
    cmp, env, fs,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    graph::generic::{ImplGraph, algorithms::is_line_graph::SageProcess},
    hamiltonian::{Density, square_lattice::PeriodicLattice},
    rand_helper,
    run::{GenGraph, check},
};

// adjust to hpc_run ncpus (don't need extra thread for main, because it is not doing
// much)
// const NUM_THREADS: usize = 50;
const NUM_THREADS: usize = 10;
// const NUM_SAMPLES: usize = 10000; // per thread
const NUM_SAMPLES: usize = 100; // per thread

const FORCE_2D: bool = true;
// const FORCE_2D: bool = false;
// const DENSITY_START: f64 = 1. / 9.;
const DENSITY_START: f64 = 0.01;
const DENSITY_END: f64 = 0.60;
// const DENSITY_END: f64 = 1.00;
// const NUM_DENSITY_STEPS: usize = 2000;
const NUM_DENSITY_STEPS: usize = 20;

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
    seed: u64,
    num_samples: usize,
    // {{ averaged over the samples
    before_claw_free: Vec<f64>,
    before_simplicial: Vec<f64>,
    after_claw_free: Vec<f64>,
    after_simplicial: Vec<f64>,
    // relative to the number of nodes
    collapsed: Vec<f64>,
    // }}
    max_sc_size: usize,
}

struct CountResults {
    before_claw_free: Vec<usize>,
    before_simplicial: Vec<usize>,
    claw_free: Vec<usize>,
    simplicial: Vec<usize>,
    collapsed: Vec<f64>,
    max_sc_size: usize,
}

impl CountResults {
    fn init() -> Self {
        Self {
            before_claw_free: vec![0; NUM_DENSITY_STEPS],
            before_simplicial: vec![0; NUM_DENSITY_STEPS],
            claw_free: vec![0; NUM_DENSITY_STEPS],
            simplicial: vec![0; NUM_DENSITY_STEPS],
            collapsed: vec![0.0; NUM_DENSITY_STEPS],
            max_sc_size: 0,
        }
    }

    fn merge(results: Vec<Self>) -> Self {
        let mut ret = Self::init();
        for result in results {
            for i in 0..NUM_DENSITY_STEPS {
                ret.before_claw_free[i] += result.before_claw_free[i];
                ret.before_simplicial[i] += result.before_simplicial[i];
                ret.claw_free[i] += result.claw_free[i];
                ret.simplicial[i] += result.simplicial[i];
                ret.collapsed[i] += result.collapsed[i];
            }
            ret.max_sc_size = cmp::max(ret.max_sc_size, result.max_sc_size);
        }
        ret
    }
}

impl Results {
    fn init() -> Self {
        Self {
            densities: Vec::new(),
            seed: 0,
            num_samples: NUM_TOTAL_SAMPLES,
            before_claw_free: vec![0.0; NUM_DENSITY_STEPS],
            before_simplicial: vec![0.0; NUM_DENSITY_STEPS],
            after_claw_free: vec![0.0; NUM_DENSITY_STEPS],
            after_simplicial: vec![0.0; NUM_DENSITY_STEPS],
            collapsed: vec![0.0; NUM_DENSITY_STEPS],
            max_sc_size: 0,
        }
    }

    fn normalise_merged_count_results(
        results: CountResults,
        densities: Vec<f64>,
        seed: u64,
    ) -> Self {
        let mut ret = Self::init();
        ret.densities = densities;
        ret.seed = seed;
        ret.max_sc_size = results.max_sc_size;

        for i in 0..NUM_DENSITY_STEPS {
            ret.before_claw_free[i] =
                results.before_claw_free[i] as f64 / NUM_TOTAL_SAMPLES as f64;
            ret.before_simplicial[i] =
                results.before_simplicial[i] as f64 / NUM_TOTAL_SAMPLES as f64;
            ret.after_claw_free[i] =
                results.claw_free[i] as f64 / NUM_TOTAL_SAMPLES as f64;
            ret.after_simplicial[i] =
                results.simplicial[i] as f64 / NUM_TOTAL_SAMPLES as f64;
            ret.collapsed[i] = results.collapsed[i] / NUM_TOTAL_SAMPLES as f64;
        }
        ret
    }
}

struct Notification {
    remaining: Vec<(f64, usize)>,
    start_time: Instant,
}
impl Notification {
    fn new(densities: impl Iterator<Item = f64>) -> Self {
        Self {
            remaining: densities.map(|d| (d, NUM_THREADS)).collect(),
            start_time: Instant::now(),
        }
    }
    fn update(&mut self, density_index: usize) {
        self.remaining[density_index].1 -= 1;
        // println!(
        //     "{:?}: {:?}",
        //     (Instant::now() - self.start_time).as_secs_f64() / 3600.,
        //     self.remaining
        // );
    }
}

pub fn periodic() {
    let id = env::args()
        .nth(1)
        .expect("id not provided")
        .parse::<usize>()
        .expect("id not a number");

    let seed = Pcg64::from_entropy().gen();

    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(Some(seed));
    let densities = get_densities();
    let notification = Arc::new(Mutex::new(Notification::new(densities.iter().copied())));

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let notification = notification.clone();
        let mut ret = CountResults::init();

        let mut sage_process = SageProcess::default();

        for (density_idx, density) in densities.iter().copied().enumerate() {
            // println!("{:?}", density);
            // let (ed, nd, eed, end) =
            //     (0..4).map(|_| Density::new(density)).collect_tuple().unwrap();
            let (ed, nd, eed, end) = (
                Density::new(0.),
                Density::new(0.),
                Density::new(density),
                Density::new(density),
            );
            let mut before_claw_free = 0;
            let mut before_simplicial = 0;
            let mut after_claw_free = 0;
            let mut after_simplicial = 0;
            let mut collapsed = 0.0;

            let mut i = 0;
            while i < NUM_SAMPLES {
                // println!("{:?}", i);
                let lattice = PeriodicLattice::draw(ed, nd, eed, end, rng);

                if FORCE_2D && !lattice.is_2d {
                    continue;
                }

                let mut graph = GenGraph::from_edge_labels(lattice.get_graph()).unwrap();

                if graph.is_empty() {
                    before_claw_free += 1;
                    before_simplicial += 1;
                    // collapsed += 0;
                    after_claw_free += 1;
                    after_simplicial += 1;
                    i += 1;
                    continue;
                }

                let orig_len = graph.len();
                let mut tree = graph.modular_decomposition();

                // // don't do this; non-connected frustration graphs does not necessarily
                // // mean the problem is trivial
                // if matches!(
                //     tree.graph.node_weight(tree.root).unwrap(),
                //     ModuleKind::Parallel
                // ) {
                //     continue;
                // }

                let check = check::do_gen_check(&graph, &tree);
                if check.claw_free {
                    before_claw_free += 1;
                }
                if check.simplicial {
                    before_simplicial += 1;
                    ret.max_sc_size = cmp::max(ret.max_sc_size, check.sc_size);
                }

                // TODO: (maybe) in the connected case, we could switch to the specialised
                // algorithms, however, at the moment I'm doing a first order simplicial
                // check (in do_gen_check) which is not yet implement for the specialised
                // representation

                graph.twin_collapse(&mut tree, &mut sage_process);
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

            ret.before_claw_free[density_idx] = before_claw_free;
            ret.before_simplicial[density_idx] = before_simplicial;
            ret.claw_free[density_idx] = after_claw_free;
            ret.simplicial[density_idx] = after_simplicial;
            ret.collapsed[density_idx] = collapsed;

            notification.lock().unwrap().update(density_idx);
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
        seed,
    );

    let force_2d = if FORCE_2D { "force_2d" } else { "" };
    fs::write(
        format!("output/periodic_square_lattice_{force_2d}_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}

pub fn run() {
    periodic()
}
