use std::fs;

use crate::{
    exact,
    graph::{
        generic::{self, Adj, Pet},
        specialised::{self, Custom, IndexMap},
    },
};

mod all_of_them;
mod bricks;
mod chain;
pub mod check;
mod density_size_sweep;
mod erdos_renyi;
mod lin_sparse;
mod majoranas;
mod num_two_local_claws;
mod old_erdos_renyi;
mod removal_examples;
mod small_claw_free;
mod sparse;
mod two_local;
mod twod_square_lattice;

// TODO: roughly test which implementations is the fastest
type GenGraph = generic::Graph<Pet>;
type GenGraphAdj = generic::Graph<Adj>;
type Graph = specialised::Graph<Custom>;

pub fn run() {
    fs::create_dir_all("output").unwrap();
    // chain::run();
    // bricks::run();
    twod_square_lattice::run();
    // majoranas::run();
    // erdos_renyi::run();
    // all_of_them::run();
    // removal_examples::run();
    // two_local::run();
    // small_claw_free::run();
    // num_two_local_claws::run();
    // sparse::run();
    // lin_sparse::run();
    // exact::square_lattice::run();
    // exact::square_lattice::run_analyse();
}

fn uniform_values(
    value_start: f64,
    value_end: f64,
    num_steps: usize,
) -> impl Iterator<Item = f64> + Clone {
    let delta = (value_end - value_start) / (num_steps - 1) as f64;
    (0..num_steps).map(move |i| value_start + delta * (i as f64))
}

// // use std::collections::HashSet;

// use rand::SeedableRng;
// use rand_pcg::Pcg64;

// use crate::{
//     // graph::my_graph::MyGraph,
//     hamiltonian::{self, OperatorPool},
//     // playing_around,
// };

// pub fn run() {
//     // let n = 10;
//     // let amount = 1;
//     // let num_nodes = hamiltonian::num_ops(n);
//     // // println!("{:?}", num_nodes);
//     // let seed = 8;

//     // let mut pool = OperatorPool::new_with(n, Pcg64::seed_from_u64(seed));

//     // 'outer: for k in 4..=num_nodes {
//     //     // for k in 4..=10 {
//     //     let amount = std::cmp::min(amount, pool.num_distinct_k_sets(k));
//     //     println!("k: {k}; {amount}");
//     //     let samples = pool.draw_exact_distinct_sets(amount, k);
//     //     for sample in samples {
//     //         let reduced_graph = MyGraph::from_iter(sample).reduce();
//     //         if reduced_graph.has_claw() {
//     //             break 'outer;
//     //             // println!("{:?}", reduced_graph);
//     //             // reduced_graph.check_all();
//     //             // println!("hit k: {k}; {amount}");
//     //         }
//     //     }
//     // }

//     // // playing_around::find_reduction();
//     // // playing_around::simple_testing();
//     // // playing_around::claw();
// }
