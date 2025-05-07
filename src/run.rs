use std::fs;

use crate::graph::{
    generic::{self, Pet},
    specialised::{self, Custom, IndexMap},
};

mod all_of_them;
mod bricks;
mod chain;
mod check;
mod erdos_renyi;
mod klocal;
mod majoranas;
mod removal_examples;
mod twod_square_lattice;
mod density_size_sweep;

// TODO: roughly test which implementations is the fastest
type GenGraph = generic::Graph<Pet>;
type Graph = specialised::Graph<Custom>;

pub fn run() {
    fs::create_dir_all("output").unwrap();
    // chain::run();
    // bricks::run();
    // twod_square_lattice::run();
    // majoranas::run();
    // erdos_renyi::run();
    // all_of_them::run();
    // removal_examples::run();
    klocal::run();
}

fn uniform_densities(
    density_start: f64,
    density_end: f64,
    num_density_steps: usize,
) -> Vec<f64> {
    let delta = (density_end - density_start) / (num_density_steps - 1) as f64;
    (0..num_density_steps)
        .map(|i| density_start + delta * (i as f64))
        .collect()
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
