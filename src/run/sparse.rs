use std::{env, fs, thread};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use super::{GenGraph, density_size_sweep::Results};
use crate::{
    graph::generic::{algorithms::is_line_graph::SageProcess, ImplGraph},
    hamiltonian::sparse::Sparse,
    rand_helper,
    run::{check, density_size_sweep::CountResults},
};

const NUM_THREADS: usize = 10;
const NUM_THREAD_SAMPLES: usize = 10;

const SIZES: [usize; 2] = [10, 15];
const FACTOR_START: f64 = 0.0;
const FACTOR_END: f64 = 0.0001;
const NUM_FACTOR_STEPS: usize = 100;

pub fn run() {
    let id = env::args()
        .nth(1)
        .expect("id not provided")
        .parse::<usize>()
        .expect("id not a number");

    let seed = Pcg64::from_entropy().gen();
    // let seed = Pcg64::seed_from_u64(0).gen();
    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(Some(seed));

    let factor_iter = super::uniform_values(FACTOR_START, FACTOR_END, NUM_FACTOR_STEPS);

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let mut ret = CountResults::init(NUM_FACTOR_STEPS, SIZES.len());

        let mut sage_process = SageProcess::default();

        for (size_idx, &size) in SIZES.iter().enumerate() {
            println!("{:?}", size);
            let ret_before_claw_free = ret.before_claw_free.get_mut(size_idx).unwrap();
            let ret_after_claw_free = ret.after_claw_free.get_mut(size_idx).unwrap();
            let ret_before_simp = ret.before_simplicial.get_mut(size_idx).unwrap();
            let ret_after_simp = ret.after_simplicial.get_mut(size_idx).unwrap();
            let ret_collapsed = ret.collapsed.get_mut(size_idx).unwrap();

            let mut last_real_factor = usize::MAX;
            let size_power = size.pow(5);

            for (factor_idx, factor) in factor_iter.clone().enumerate() {
                let real_factor = (factor * size_power as f64).floor() as usize;

                if real_factor == last_real_factor {
                    ret_before_claw_free[factor_idx] =
                        ret_before_claw_free[factor_idx - 1];
                    ret_after_claw_free[factor_idx] = ret_after_claw_free[factor_idx - 1];
                    ret_before_simp[factor_idx] = ret_before_simp[factor_idx - 1];
                    ret_after_simp[factor_idx] = ret_after_simp[factor_idx - 1];
                    ret_collapsed[factor_idx] = ret_collapsed[factor_idx - 1];
                    println!("skipped: {:?}", last_real_factor);
                    continue;
                } else {
                    last_real_factor = real_factor;
                    println!("real_factor: {:?}", last_real_factor);
                }

                let mut before_claw_free = 0;
                let mut after_claw_free = 0;
                let mut before_simplicial = 0;
                let mut after_simplicial = 0;
                let mut collapsed = 0.;

                let mut i = 0;
                while i < NUM_THREAD_SAMPLES {
                    // let mut graph = get_graph(d, *size, rng);
                    let mut graph = GenGraph::from_edge_labels(
                        Sparse::draw(real_factor, size, rng).get_graph(),
                    )
                    .unwrap();

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

                ret_before_claw_free[factor_idx] = before_claw_free;
                ret_after_claw_free[factor_idx] = after_claw_free;
                ret_before_simp[factor_idx] = before_simplicial;
                ret_after_simp[factor_idx] = after_simplicial;
                ret_collapsed[factor_idx] = collapsed;
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
        factor_iter.collect(),
        SIZES.to_vec(),
        seed,
        NUM_THREAD_SAMPLES * NUM_THREADS,
    );

    fs::write(
        format!("output/sparse_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}
