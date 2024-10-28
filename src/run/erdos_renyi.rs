use std::fs;

use hashbrown::HashSet;
use modular_decomposition::ModuleKind;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    fix_int::int,
    graph::{Label, generic::ImplGraph},
    run::{GenGraph, Graph, check},
};

const CONSIDER_PARALLEL_GRAPHS: bool = true;
// const CONSIDER_PARALLEL_GRAPHS: bool = false;

#[derive(Serialize)]
struct Results {
    sweep: Vec<Sweep>,
    consider_parallel_graphs: bool,
    densities: Vec<f64>,
}

// all f64 values are percentages w.r.t. the size
#[derive(Serialize, Default)]
struct Sweep {
    before_collapse_claw_free: Vec<f64>,
    before_collapse_simplicial: Vec<f64>, // and claw_free of course
    avg_collapsed_nodes: Vec<f64>,        // average
    claw_free: Vec<f64>,
    simplicial: Vec<f64>,
    num_samples: usize,
}

fn num_samples(_size: int) -> usize {
    // TODO: make this an actual appropriate function depending on the graph size
    80
}

pub fn run() {
    const MAX_SIZE: int = 20;
    const NUM_DENSITIES: usize = 30;

    let rng = &mut Pcg64::from_entropy();
    // let rng = &mut Pcg64::from_seed([0; 32]);

    let mut results = Results {
        sweep: Vec::with_capacity(MAX_SIZE as usize),
        consider_parallel_graphs: CONSIDER_PARALLEL_GRAPHS,
        densities: Vec::with_capacity(NUM_DENSITIES),
    };

    let start = 4;
    for _ in 0..start {
        results.sweep.push(Sweep::default());
    }

    let densities = (1..NUM_DENSITIES).map(|d| d as f64 / NUM_DENSITIES as f64);

    for size in 4..MAX_SIZE + 1 {
        let edge_pool = (0..size).flat_map(|i| (i + 1..size).map(move |j| (i, j)));
        let num_samples = num_samples(size);

        let mut sweep = Sweep {
            before_collapse_claw_free: Vec::with_capacity(NUM_DENSITIES),
            before_collapse_simplicial: Vec::with_capacity(NUM_DENSITIES),
            avg_collapsed_nodes: Vec::with_capacity(NUM_DENSITIES),
            claw_free: Vec::with_capacity(NUM_DENSITIES),
            simplicial: Vec::with_capacity(NUM_DENSITIES),
            num_samples,
        };

        for density in densities.clone() {
            let mut before_collapse_claw_free = 0;
            let mut before_collapse_simplicial = 0;
            let mut avg_collapsed_nodes = 0.;
            let mut claw_free = 0;
            let mut simplicial = 0;

            let mut i = 0;
            let mut _tries = 0;
            while i < num_samples {
                _tries += 1;
                println!("{:?}", _tries);

                // need to collect because we want to reuse the same edges, but the filter
                // depends on the random number generator
                let edges: Vec<(Label, Label)> =
                    edge_pool.clone().filter(|_| rng.gen::<f64>() < density).collect();

                if edges.is_empty() {
                    if CONSIDER_PARALLEL_GRAPHS {
                        before_collapse_simplicial += 1;
                        before_collapse_claw_free += 1;
                        // avg_collapsed_nodes += 0.;
                        claw_free += 1;
                        simplicial += 1;
                        i += 1;
                    } else {
                        continue;
                    }
                }

                let mut gen_graph =
                    GenGraph::from_edge_labels(edges.iter().copied()).unwrap();

                // note that the from_edge_labels does not ensure that the graph has
                // actually `size` nodes, since if a node does not appear in any edge, it
                // is not added to the graph
                let fill_up = if gen_graph.len() != size as usize {
                    // instead we could sample the subgraphs and append them to the
                    // results with the appropriate size, but I runs fast enough
                    if CONSIDER_PARALLEL_GRAPHS {
                        // need to fill up
                        let nodes = gen_graph.iter_labels().collect::<HashSet<_>>();
                        let mut fill_up = Vec::with_capacity(size as usize - nodes.len());
                        for node in 0..size {
                            if !nodes.contains(&node) {
                                gen_graph.add_labelled_node_symmetrically((node, []));
                                fill_up.push(node);
                            }
                        }
                        fill_up
                    } else {
                        continue;
                    }
                } else {
                    vec![]
                };

                let gen_tree = gen_graph.modular_decomposition();

                if !CONSIDER_PARALLEL_GRAPHS
                    && matches!(
                        gen_tree.graph.node_weight(gen_tree.root).unwrap(),
                        ModuleKind::Parallel
                    )
                {
                    continue;
                }

                i += 1;

                let gen_check = check::do_gen_check(&gen_graph, &gen_tree);
                if gen_check.claw_free {
                    before_collapse_claw_free += 1;
                }
                if gen_check.simplicial {
                    before_collapse_simplicial += 1;
                }

                let mut graph = Graph::from_edge_labels(edges).unwrap();
                if !fill_up.is_empty() {
                    for node in fill_up {
                        unsafe {
                            graph.add_labelled_node_symmetrically_unchecked((node, []))
                        };
                    }
                }
                let mut tree = graph.modular_decomposition();

                let len = graph.len();
                unsafe { graph.twin_collapse(&mut tree) };
                avg_collapsed_nodes += (len - graph.len()) as f64 / size as f64;

                let check = check::do_check(&graph, &tree);

                // #[cfg(debug_assertions)]
                // {
                //     gen_graph.twin_collapse(&mut gen_tree);
                //     assert_eq!(do_gen_check(&gen_graph, &gen_tree), check);
                // }

                if check.claw_free {
                    claw_free += 1;
                }
                if check.simplicial {
                    simplicial += 1;
                }
            }
            println!("{:?}", (size, density, _tries));

            sweep
                .before_collapse_claw_free
                .push(before_collapse_claw_free as f64 / num_samples as f64);
            sweep
                .before_collapse_simplicial
                .push(before_collapse_simplicial as f64 / num_samples as f64);
            sweep
                .avg_collapsed_nodes
                .push(avg_collapsed_nodes / num_samples as f64);
            sweep.claw_free.push(claw_free as f64 / num_samples as f64);
            sweep.simplicial.push(simplicial as f64 / num_samples as f64);
        }

        results.sweep.push(sweep);
    }

    results.densities.extend(densities);

    fs::write(
        format!("output/erdos_renyi_{CONSIDER_PARALLEL_GRAPHS}.json"),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}
