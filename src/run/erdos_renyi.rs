use std::fs;

use modular_decomposition::ModuleKind;
use petgraph::Direction;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use serde::Serialize;

use crate::{
    fix_int::int,
    graph::{
        Label,
        algorithms::modular_decomposition::Tree,
        generic::{self, ImplGraph, Pet, algorithms::claw_free::ClawFree},
        specialised::{self, Custom},
    },
};

type GenGraph = generic::Graph<Pet>;
type Graph = specialised::Graph<Custom>;

const CONSIDER_PARALLEL_GRAPHS: bool = true;

#[derive(Serialize)]
struct Results {
    sweep: Vec<Sweep>,
    consider_parallel_graphs: bool,
}

// all f64 values are percentages w.r.t. the size
#[derive(Serialize, Default)]
struct Sweep {
    density: Vec<f64>,
    before_collapse_claw_free: Vec<f64>,
    before_collapse_simplicial: Vec<f64>, // and claw_free of course
    avg_collapsed_nodes: Vec<f64>,        // average
    claw_free: Vec<f64>,
    simplicial: Vec<f64>,
    num_samples: usize,
}

fn num_samples(_size: int) -> usize {
    // TODO: make this an actual appropriate function depending on the graph size
    20
}

pub fn run() {
    const MAX_SIZE: int = 10;
    const NUM_DENSITIES: usize = 20;

    let rng = &mut Pcg64::from_entropy();
    // let rng = &mut Pcg64::from_seed([0; 32]);

    let mut results = Results {
        sweep: Vec::with_capacity(MAX_SIZE as usize),
        consider_parallel_graphs: CONSIDER_PARALLEL_GRAPHS,
    };

    let start = 4;
    for _ in 0..start {
        results.sweep.push(Sweep::default());
    }

    for size in 4..MAX_SIZE + 1 {
        let edge_pool = (0..size).flat_map(|i| (i + 1..size).map(move |j| (i, j)));
        let num_samples = num_samples(size);

        let mut sweep = Sweep {
            density: Vec::with_capacity(NUM_DENSITIES),
            before_collapse_claw_free: Vec::with_capacity(NUM_DENSITIES),
            before_collapse_simplicial: Vec::with_capacity(NUM_DENSITIES),
            avg_collapsed_nodes: Vec::with_capacity(NUM_DENSITIES),
            claw_free: Vec::with_capacity(NUM_DENSITIES),
            simplicial: Vec::with_capacity(NUM_DENSITIES),
            num_samples,
        };

        for d in 1..NUM_DENSITIES {
            let density = d as f64 / (NUM_DENSITIES) as f64;
            let mut before_collapse_claw_free = 0;
            let mut before_collapse_simplicial = 0;
            let mut avg_collapsed_nodes = 0.;
            let mut claw_free = 0;
            let mut simplicial = 0;

            let mut i = 0;
            while i < num_samples {
                // need to collect because we want to reuse the same edges, but the filter
                // depends on the random number generator
                let edges: Vec<(Label, Label)> =
                    edge_pool.clone().filter(|_| rng.gen::<f64>() < density).collect();

                let mut gen_graph =
                    GenGraph::from_edge_labels(edges.iter().copied()).unwrap();

                // note that the from_edge_labels does not ensure that the graph has
                // actually `size` nodes, since if a node does not appear in any edge, it
                // is not added to the graph
                if gen_graph.len() != size as usize {
                    // instead we could sample the subgraphs and append them to the
                    // results with the appropriate size, but I runs fast enough
                    continue;
                }

                let mut gen_tree = gen_graph.modular_decomposition();

                if !CONSIDER_PARALLEL_GRAPHS
                    && matches!(
                        gen_tree.graph.node_weight(gen_tree.root).unwrap(),
                        ModuleKind::Parallel
                    )
                {
                    continue;
                }

                i += 1;

                let gen_check = do_gen_check(&gen_graph, &gen_tree);
                if gen_check.claw_free {
                    before_collapse_claw_free += 1;
                }
                if gen_check.simplicial {
                    before_collapse_simplicial += 1;
                }

                let mut graph = Graph::from_edge_labels(edges).unwrap();
                debug_assert!(!graph.is_empty());
                let mut tree = graph.modular_decomposition();

                let len = graph.len();
                unsafe { graph.twin_collapse(&mut tree) };
                avg_collapsed_nodes += (len - graph.len()) as f64 / size as f64;

                let check = do_check(&graph, &tree);

                #[cfg(debug_assertions)]
                {
                    gen_graph.twin_collapse(&mut gen_tree);
                    assert_eq!(do_gen_check(&gen_graph, &gen_tree), check);
                }

                if check.claw_free {
                    claw_free += 1;
                }
                if check.simplicial {
                    simplicial += 1;
                }
            }

            sweep.density.push(density);
            sweep
                .before_collapse_claw_free
                .push(before_collapse_claw_free as f64 / i as f64);
            sweep
                .before_collapse_simplicial
                .push(before_collapse_simplicial as f64 / i as f64);
            sweep.avg_collapsed_nodes.push(avg_collapsed_nodes / i as f64);
            sweep.claw_free.push(claw_free as f64 / i as f64);
            sweep.simplicial.push(simplicial as f64 / i as f64);
        }

        results.sweep.push(sweep);
    }

    fs::write("output/erdos_renyi.json", serde_json::to_string_pretty(&results).unwrap())
        .unwrap();
}

// Default: bool defaults to false
#[derive(Default, PartialEq, Debug)]
struct Check {
    claw_free: bool,
    simplicial: bool,
    #[cfg(debug_assertions)]
    parallel: bool,
}

fn do_gen_check(graph: &GenGraph, tree: &Tree) -> Check {
    let mut ret = Check::default();
    if matches!(graph.is_claw_free(tree), ClawFree::Yes) {
        ret.claw_free = true;
    }
    if let ModuleKind::Parallel = tree.graph.node_weight(tree.root).unwrap() {
        #[cfg(debug_assertions)]
        {
            ret.parallel = true;
        }
        if !ret.claw_free {
            return ret;
        }
        ret.simplicial = true;
        for subgraph in tree
            .graph
            .neighbors_directed(tree.root, Direction::Outgoing)
            .map(|child| graph.subgraph(&tree.module_nodes(child, None)))
        {
            let subtree = subgraph.modular_decomposition();
            // we already checked that the whole graph is claw-free, but we required that
            // each subgraph is simplicial (each one has to be solved independently)
            if !subgraph
                .simplicial(&subtree, Some(&ClawFree::Yes))
                .unwrap()
                .into_iter()
                .flatten()
                .any(|clique| !clique.is_empty())
            {
                ret.simplicial = false;
                break;
            }
        }
    } else if ret.claw_free
        && graph
            .simplicial(tree, Some(&ClawFree::Yes))
            .unwrap()
            .into_iter()
            .flatten()
            .any(|clique| !clique.is_empty())
    {
        ret.simplicial = true;
    }
    ret
}

fn do_check(graph: &Graph, tree: &Tree) -> Check {
    let mut ret = Check::default();
    if let ModuleKind::Parallel = tree.graph.node_weight(tree.root).unwrap() {
        #[cfg(debug_assertions)]
        {
            ret.parallel = true;
        }
        ret.claw_free = true;
        ret.simplicial = true;
        for subgraph in
            tree.graph
                .neighbors_directed(tree.root, Direction::Outgoing)
                .map(|child| {
                    let nodes = tree.module_nodes(child, None);
                    unsafe { graph.subgraph(nodes.len(), nodes) }
                })
        {
            let subtree = subgraph.modular_decomposition();
            if !unsafe { subgraph.is_claw_free(&subtree) } {
                ret.claw_free = false;
                ret.simplicial = false;
                break;
            }
            if unsafe { subgraph.simplicial(&subtree).is_empty() } {
                ret.simplicial = false;
                break;
            }
        }
    } else {
        ret.claw_free = unsafe { graph.is_claw_free(tree) };
        if ret.claw_free && unsafe { !graph.simplicial(tree).is_empty() } {
            ret.simplicial = true;
        }
    }
    ret
}
