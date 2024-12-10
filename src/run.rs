use std::fs;

use crate::graph::{
    generic::{self, Pet},
    specialised::{self, Custom, IndexMap},
};

mod check;
mod erdos_renyi;
mod all_of_them;
mod chain;
mod lattice;
mod removal_examples;

// TODO: roughly test which implementations is the fastest
type GenGraph = generic::Graph<Pet>;
type Graph = specialised::Graph<Custom>;

pub fn run() {
    fs::create_dir_all("output").unwrap();
    // chain::run();
    lattice::run();
    // erdos_renyi::run();
    // all_of_them::run();
    // removal_examples::run();
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
