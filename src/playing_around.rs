use std::collections::HashSet;

use rand::SeedableRng;
use rand_pcg::Pcg64;

use crate::{
    graph::Graph,
    hamiltonian::{self, OperatorPool},
};

pub fn claw() {
    let n = 3;
    let seed = 8;

    let mut pool = OperatorPool::new_with(n, Pcg64::seed_from_u64(seed));

    let p = 0.21;
    let set_size = (pool.ops.len() as f64 * p).round() as usize;
    // println!("{:?}", set_size);

    let sample = pool.draw(set_size);

    let graph = Graph::from_iter(sample);
    let reduced_graph = graph.clone().reduce();

    // println!("{:?}", graph);
    // println!("{:?}", reduced_graph);
    for (node, neighbors) in reduced_graph.nodes.iter() {
        println!("{:?}: {:?}", node, neighbors);
    }

    reduced_graph.check_all();
}

pub fn simple_testing() {
    let n = 3;
    let seed = 8;

    let mut pool = OperatorPool::new_with(n, Pcg64::seed_from_u64(seed));

    let p = 0.02;
    let set_size = (pool.ops.len() as f64 * p).round() as usize;
    // println!("num ops: {}", pool.num_ops());
    println!("max amount: {:?}", pool.num_distinct_k_sets(set_size));
    let amount = 25;

    // let samples = pool.draw_exact_distinct_sets(amount, set_size);
    // let samples = pool.draw_distinct_sets(amount, set_size);
    let samples = pool.draw_sets(amount, set_size);

    let samples = samples.map(|sample| sample.collect::<Vec<_>>()).collect::<Vec<_>>();

    let set = HashSet::<_>::from_iter(samples.clone());

    println!("{:?}", samples.len());
    println!("{:?}", set.len());

    // for sample in samples {
    //     let graph = Graph::from(&sample[..]);
    //     let reduced_graph = graph.clone().reduce();
    //     println!("Hamiltonian:");
    //     for (i, op) in sample.iter().enumerate() {
    //         println!("{i}: {:?}; {:?}", op.index, op.data);
    //     }
    //     println!("\n{:?}\n", graph);
    //     println!("{:?}\n", reduced_graph);
    // }

    //
}

pub fn find_reduction() {
    let n = 3;
    let seed = 8;
    let num_examples = 3;

    let mut pool = OperatorPool::new_with(n, Pcg64::seed_from_u64(seed));

    for m in 2..=hamiltonian::num_ops(n) {
        let mut count = 0;
        for i in 0..200 {
            let sample = pool.draw(m).collect::<Vec<_>>();
            let graph = Graph::sorted_from(&sample.clone());
            let reduced_graph = graph.clone().reduce();
            if reduced_graph.nodes.len() != graph.nodes.len() {
                println!("reduction at sample {m};{i}");
                println!("{:?}", graph);
                println!("{:?}", reduced_graph);
                println!("diff: {}", graph.nodes.len() - reduced_graph.nodes.len());
                count += 1;
                if count == num_examples {
                    break;
                }
            }
        }
        println!("no reduction at sample {m}");
    }
}
