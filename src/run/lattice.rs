//! 2d square lattice

use std::{
    env, fs,
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use itertools::Itertools;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    graph::generic::ImplGraph,
    hamiltonian::{Density, bricks::Bricks},
    rand_helper,
    run::{GenGraph, check},
};

// adjust to hpc_run ncpus (don't need extra thread for main, because it is not doing
// much)
const NUM_THREADS: usize = 50;
const NUM_SAMPLES: usize = 100000; // per thread

const DENSITY_START: f64 = 1. / 9.;
const DENSITY_END: f64 = 0.40;
const NUM_DENSITY_STEPS: usize = 2000;

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
}

struct CountResults {
    before_claw_free: Vec<usize>,
    before_simplicial: Vec<usize>,
    claw_free: Vec<usize>,
    simplicial: Vec<usize>,
    collapsed: Vec<f64>,
}

impl CountResults {
    fn init() -> Self {
        Self {
            before_claw_free: vec![0; NUM_DENSITY_STEPS],
            before_simplicial: vec![0; NUM_DENSITY_STEPS],
            claw_free: vec![0; NUM_DENSITY_STEPS],
            simplicial: vec![0; NUM_DENSITY_STEPS],
            collapsed: vec![0.0; NUM_DENSITY_STEPS],
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

    // let seed = 0;
    let seed = Pcg64::from_entropy().gen();

    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(Some(seed));
    let densities = get_densities();
    let notification = Arc::new(Mutex::new(Notification::new(densities.iter().copied())));

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let notification = notification.clone();
        let mut ret = CountResults::init();

        for (density_idx, density) in densities.iter().copied().enumerate() {
            let (e1d, e2d, e3d, e4d, e5d) =
                (0..5).map(|_| Density::new(density)).collect_tuple().unwrap();
            let mut before_claw_free = 0;
            let mut before_simplicial = 0;
            let mut claw_free = 0;
            let mut simplicial = 0;
            let mut collapsed = 0.0;

            let mut i = 0;
            while i < NUM_SAMPLES {
                // println!("{:?}", i);
                let lattice = Bricks::draw(e1d, e2d, e3d, e4d, e5d, rng);

                let mut graph = GenGraph::from_edge_labels(lattice.get_graph()).unwrap();

                if graph.is_empty() {
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
                }

                // TODO: (maybe) in the connected case, we could switch to the specialised
                // algorithms, however, at the moment I'm doing a first order simplicial
                // check (in do_gen_check) which is not yet implement for the specialised
                // representation

                graph.twin_collapse(&mut tree);
                collapsed += (orig_len - graph.len()) as f64 / orig_len as f64;

                let check = check::do_gen_check(&graph, &tree);
                if check.claw_free {
                    claw_free += 1;
                }
                if check.simplicial {
                    simplicial += 1;
                }

                i += 1;
            }

            ret.before_claw_free[density_idx] = before_claw_free;
            ret.before_simplicial[density_idx] = before_simplicial;
            ret.claw_free[density_idx] = claw_free;
            ret.simplicial[density_idx] = simplicial;
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

    fs::write(
        format!("output/periodic_bricks_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}

pub fn run() {
    periodic()
}
