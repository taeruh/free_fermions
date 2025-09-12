use itertools::Itertools;
use petgraph::Undirected;

use super::check;
#[allow(unused_imports)]
use crate::{
    fix_int::int,
    graph::{generic::ImplGraph, Label, Node},
    run::GenGraph,
};

#[allow(dead_code)]
pub fn run() {
    let size = 5;
    let total_num_edges = size * (size - 1) / 2;
    let edge_pool = (0..size).flat_map(|i| (i + 1..size).map(move |j| (i as int, j as int)));

    let mut results = vec![(0, 0); total_num_edges + 1];
    let mut naive_results = vec![0; total_num_edges + 1];

    for edges in edge_pool.powerset() {
        let len = edges.len();
        results[len].0 += 1;
        if len == 0 {
            results[0].1 += 1;
            naive_results[0] += 1;
            continue;
        }

        // let graph = GenGraph::from_edge_labels(edges).unwrap();
        // let tree = graph.modular_decomposition();
        // let check = check::do_gen_check(&graph, &tree);
        // if check.claw_free {
        //     results[len].1 += 1;
        // }
        // let mut naive_claw_free = true;
        // 'outer: for a in 0..size {
        //     for b in a + 1..size {
        //         for c in b + 1..size {
        //             for d in c + 1..size {
        //                 if check::contains_claw(
        //                     &graph, a as Node, b as Node, c as Node, d as Node,
        //                 ) {
        //                     naive_claw_free = false;
        //                     break 'outer;
        //                 }
        //             }
        //         }
        //     }
        // }
        // if naive_claw_free {
        //     naive_results[len] += 1;
        // }
        // assert_eq!(results[len].1, naive_results[len],);

        let graph = petgraph::Graph::<(), (), Undirected, int>::from_edges(&edges);
        let _node_count = graph.node_count();
        // if node_count < 5 {
        //     println!("{:?}", edges);
        //     println!("{:?}", graph);
        // }
        let mut naive_claw_free = true;
        'outer: for a in 0..size {
            for b in a + 1..size {
                for c in b + 1..size {
                    for d in c + 1..size {
                        if check::pet_contains_claw(
                            &graph,
                            (a as int).into(),
                            (b as int).into(),
                            (c as int).into(),
                            (d as int).into(),
                        ) {
                            naive_claw_free = false;
                            break 'outer;
                        }
                    }
                }
            }
        }
        if naive_claw_free {
            results[len].1 += 1;
        }
    }

    let results = results
        .into_iter()
        .enumerate()
        .map(|(i, r)| (i as f64 / total_num_edges as f64, r.1, r.0))
        .collect::<Vec<_>>();

    println!("{:?}", results);

    let num_graphs = results.iter().map(|r| r.2).sum::<usize>();
    assert_eq!(num_graphs, 2usize.pow(total_num_edges as u32));
}
