use std::cmp;

use hashbrown::HashSet;
use itertools::Itertools;
use modular_decomposition::ModuleKind;
use petgraph::{Direction, Undirected, graph::NodeIndex};

use super::{GenGraph, Graph};
use crate::{
    fix_int::int,
    graph::{
        Node,
        algorithms::modular_decomposition::Tree,
        generic::{
            ImplGraph, NodeCollection,
            algorithms::claw_free::{ClawFree, ClawFreeNaive},
        },
    },
};

// Default: bool defaults to false
#[derive(Default, PartialEq, Debug)]
pub struct Check {
    pub claw_free: bool,
    pub simplicial: bool,
    pub sc_size: usize,
    #[cfg(debug_assertions)]
    pub parallel: bool,
}

fn first_order_simplicial(graph: &GenGraph) -> Option<usize> {
    for a in graph.0.node_indices().map(|n| n.index()) {
        if graph.clique_is_simplicial(&[a]) {
            return Some(1);
        }
    }
    for (a, b) in graph
        .0
        .raw_edges()
        .iter()
        .map(|e| (e.source().index(), e.target().index()))
    {
        if graph.clique_is_simplicial(&[a, b]) {
            return Some(2);
        }
    }
    None
}

fn full_simplicial(graph: &GenGraph, tree: &Tree) -> Option<usize> {
    graph
        .simplicial(tree, Some(&ClawFree::Yes))
        .unwrap()
        .into_iter()
        .flatten()
        .find(|clique| !clique.is_empty())
        .map(|clique| clique.len())
}

pub fn do_gen_check(graph: &GenGraph, tree: &Tree) -> Check {
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
            if let Some(size) = first_order_simplicial(&subgraph) {
                ret.sc_size = cmp::max(ret.sc_size, size);
                continue;
            }
            let subtree = subgraph.modular_decomposition();
            // we already checked that the whole graph is claw-free, but we required that
            // each subgraph is simplicial (each one has to be solved independently)
            if let Some(size) = full_simplicial(&subgraph, &subtree) {
                ret.sc_size = cmp::max(ret.sc_size, size);
            } else {
                ret.simplicial = false;
                break;
            }
        }
    } else if ret.claw_free {
        if let Some(size) = first_order_simplicial(graph) {
            ret.sc_size = size;
            ret.simplicial = true;
        } else if let Some(size) = full_simplicial(graph, tree) {
            ret.sc_size = size;
            ret.simplicial = true;
        }
    }
    ret
}

pub fn do_check(graph: &Graph, tree: &Tree) -> Check {
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

pub fn contains_claw(graph: &GenGraph, a: Node, b: Node, c: Node, d: Node) -> bool {
    let a_neighbours = graph.get_neighbours(a).unwrap();
    let b_neighbours = graph.get_neighbours(b).unwrap();
    let c_neighbours = graph.get_neighbours(c).unwrap();
    let d_neighbours = graph.get_neighbours(d).unwrap();
    (a_neighbours.contains(b)
        && a_neighbours.contains(c)
        && a_neighbours.contains(d)
        && !b_neighbours.contains(c)
        && !b_neighbours.contains(d)
        && !c_neighbours.contains(d))
        || (b_neighbours.contains(a)
            && b_neighbours.contains(c)
            && b_neighbours.contains(d)
            && !a_neighbours.contains(c)
            && !a_neighbours.contains(d)
            && !c_neighbours.contains(d))
        || (c_neighbours.contains(a)
            && c_neighbours.contains(b)
            && c_neighbours.contains(d)
            && !a_neighbours.contains(b)
            && !a_neighbours.contains(d)
            && !b_neighbours.contains(d))
        || (d_neighbours.contains(a)
            && d_neighbours.contains(b)
            && d_neighbours.contains(c)
            && !a_neighbours.contains(b)
            && !a_neighbours.contains(c)
            && !b_neighbours.contains(c))
}

pub fn pet_contains_claw(
    graph: &petgraph::Graph<(), (), Undirected, int>,
    a: NodeIndex,
    b: NodeIndex,
    c: NodeIndex,
    d: NodeIndex,
) -> bool {
    let a_neighbours = graph.neighbors(a).collect_vec();
    let b_neighbours = graph.neighbors(b).collect_vec();
    let c_neighbours = graph.neighbors(c).collect_vec();
    let d_neighbours = graph.neighbors(d).collect_vec();
    (a_neighbours.contains(&b)
        && a_neighbours.contains(&c)
        && a_neighbours.contains(&d)
        && !b_neighbours.contains(&c)
        && !b_neighbours.contains(&d)
        && !c_neighbours.contains(&d))
        || (b_neighbours.contains(&a)
            && b_neighbours.contains(&c)
            && b_neighbours.contains(&d)
            && !a_neighbours.contains(&c)
            && !a_neighbours.contains(&d)
            && !c_neighbours.contains(&d))
        || (c_neighbours.contains(&a)
            && c_neighbours.contains(&b)
            && c_neighbours.contains(&d)
            && !a_neighbours.contains(&b)
            && !a_neighbours.contains(&d)
            && !b_neighbours.contains(&d))
        || (d_neighbours.contains(&a)
            && d_neighbours.contains(&b)
            && d_neighbours.contains(&c)
            && !a_neighbours.contains(&b)
            && !a_neighbours.contains(&c)
            && !b_neighbours.contains(&c))
}
