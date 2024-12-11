//! 2d square lattice

use std::{
    iter, ops,
    sync::{Arc, Mutex},
    thread,
};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use crate::{
    graph::generic::ImplGraph,
    hamiltonian::{Density, square_lattice::PeriodicLattice},
    rand_helper,
    run::{GenGraph, check},
};

const NUM_THREADS: usize = 8;
const NUM_SAMPLES: usize = 20; // per thread

const DENSITY_START: f64 = 0.08;
const DENSITY_END: f64 = 0.09;
const NUM_DENSITY_STEPS: usize = 5;

const NUM_TOTAL_SAMPLES: usize = NUM_THREADS * NUM_SAMPLES;

fn get_densities() -> Vec<f64> {
    const DELTA: f64 = (DENSITY_END - DENSITY_START) / (NUM_DENSITY_STEPS - 1) as f64;
    (0..NUM_DENSITY_STEPS)
        .map(|i| DENSITY_START + DELTA * (i as f64))
        .collect()
}

#[derive(Default, Debug)]
struct Results {
    densities: Vec<f64>,
    // {{ averaged over the samples
    before_claw_free: [f64; NUM_DENSITY_STEPS],
    before_simplicial: [f64; NUM_DENSITY_STEPS],
    after_claw_free: [f64; NUM_DENSITY_STEPS],
    after_simplicial: [f64; NUM_DENSITY_STEPS],
    // relative to the number of nodes
    collapsed: [f64; NUM_DENSITY_STEPS],
    // }}
}

#[derive(Default)]
struct CountResults {
    before_claw_free: [usize; NUM_DENSITY_STEPS],
    before_simplicial: [usize; NUM_DENSITY_STEPS],
    claw_free: [usize; NUM_DENSITY_STEPS],
    simplicial: [usize; NUM_DENSITY_STEPS],
    collapsed: [f64; NUM_DENSITY_STEPS],
}

impl CountResults {
    fn merge(results: Vec<Self>) -> Self {
        let mut ret = Self::default();
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
    fn normalise_merged_count_results(
        results: CountResults,
        densities: Vec<f64>,
    ) -> Self {
        let mut ret = Self {
            densities,
            ..Default::default()
        };
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
}
impl Notification {
    fn new(densities: impl Iterator<Item = f64>) -> Self {
        Self {
            remaining: densities.map(|d| (d, NUM_THREADS)).collect(),
        }
    }
    fn update(&mut self, density_index: usize) {
        self.remaining[density_index].1 -= 1;
        println!("{:?}", self.remaining);
    }
}

pub fn periodic() {
    // let seed = None;
    let seed = Some(0);

    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(seed);
    let densities = get_densities();
    let notification = Arc::new(Mutex::new(Notification::new(densities.iter().copied())));

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let notification = notification.clone();
        let mut ret = CountResults::default();

        for (density_idx, density) in densities.iter().copied().enumerate() {
            let e_density = Density::new(density);
            let n_density = Density::new(density);
            let ee_density = Density::new(density);
            let en_density = Density::new(density);
            let mut before_claw_free = 0;
            let mut before_simplicial = 0;
            let mut claw_free = 0;
            let mut simplicial = 0;
            let mut collapsed = 0.0;

            let mut i = 0;
            while i < NUM_SAMPLES {
                let lattice = PeriodicLattice::draw(
                    e_density, n_density, ee_density, en_density, rng,
                );
                let mut graph = GenGraph::from_edge_labels(lattice.get_graph()).unwrap();

                if graph.is_empty() {
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

    let results =
        Results::normalise_merged_count_results(CountResults::merge(results), densities);

    println!("{:?}", results);
}

fn _periodic() {
    // let rng = &mut Pcg64::from_entropy();
    // let init = rng.gen();
    // println!("{:?}", init);
    // let rng = &mut Pcg64::seed_from_u64(init);
    let rng = &mut Pcg64::seed_from_u64(0);

    let e_density = Density::new(0.1);
    let n_density = Density::new(0.1);
    let ee_density = Density::new(0.1);
    let en_density = Density::new(0.1);

    let lattice =
        PeriodicLattice::draw(e_density, n_density, ee_density, en_density, rng);
    let mut graph = GenGraph::from_edge_labels(lattice.get_graph()).unwrap();
    let mut tree = graph.modular_decomposition();

    let num_nodes = graph.iter_labels().count();
    let check = check::do_gen_check(&graph, &tree);

    println!("{:?}", num_nodes);
    println!("{:?}", check);

    graph.twin_collapse(&mut tree);

    let num_nodes = graph.iter_labels().count();
    let check = check::do_gen_check(&graph, &tree);
    println!("{:?}", num_nodes);
    println!("{:?}", check);
}

pub fn run() {
    periodic()
}
