use std::{env, fs, thread};

use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use super::{density_size_sweep::Results, GenGraph};
use crate::{
    graph::generic::{algorithms::is_line_graph::SageProcess, ImplGraph},
    hamiltonian::sparse::Sparse,
    rand_helper,
    run::{check, density_size_sweep::CountResults},
};

// const NUM_THREADS: usize = 10;
const NUM_THREADS: usize = 50;
// const NUM_THREAD_SAMPLES: usize = 1000;
const NUM_THREAD_SAMPLES: usize = 1000;

const SIZES: [usize; 2] = [5, 30];
const NUM_OPERATORS: usize = 15 + 1;

pub fn run() {
    let id = env::args()
        .nth(1)
        .expect("id not provided")
        .parse::<usize>()
        .expect("id not a number");

    let seed = Pcg64::from_entropy().gen();
    // let seed = Pcg64::seed_from_u64(0).gen();
    let seeds = rand_helper::generate_seeds::<NUM_THREADS>(Some(seed));

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let mut ret = CountResults::init(NUM_OPERATORS, SIZES.len());

        let mut sage_process = SageProcess::default();

        for (size_idx, &size) in SIZES.iter().enumerate() {
            println!("{size:?}");
            let ret_before_claw_free = ret.before_claw_free.get_mut(size_idx).unwrap();
            let ret_after_claw_free = ret.after_claw_free.get_mut(size_idx).unwrap();
            let ret_before_simp = ret.before_simplicial.get_mut(size_idx).unwrap();
            let ret_after_simp = ret.after_simplicial.get_mut(size_idx).unwrap();
            let ret_collapsed = ret.collapsed.get_mut(size_idx).unwrap();

            for (num_ops_idx, num_ops) in (0..NUM_OPERATORS).enumerate() {
                let mut before_claw_free = 0;
                let mut after_claw_free = 0;
                let mut before_simplicial = 0;
                let mut after_simplicial = 0;
                let mut collapsed = 0.;

                let mut i = 0;
                while i < NUM_THREAD_SAMPLES {
                    let mut graph =
                        GenGraph::from_edge_labels(Sparse::draw(num_ops, size, rng).get_graph())
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

                ret_before_claw_free[num_ops_idx] = before_claw_free;
                ret_after_claw_free[num_ops_idx] = after_claw_free;
                ret_before_simp[num_ops_idx] = before_simplicial;
                ret_after_simp[num_ops_idx] = after_simplicial;
                ret_collapsed[num_ops_idx] = collapsed;
            }
        }
        println!("thread {id} finished");
        ret
    };

    let results: Vec<_> = thread::scope(|scope| {
        let handles: Vec<_> = (0..NUM_THREADS)
            .map(|i| scope.spawn(move || job(i)))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    let results = Results::normalise_merged_count_results(
        CountResults::merge(results),
        (0..=NUM_OPERATORS).map(|e| e as f64).collect(),
        SIZES.to_vec(),
        seed,
        NUM_THREAD_SAMPLES * NUM_THREADS,
    );

    fs::write(
        format!("output/lin_sparse_{id}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}
