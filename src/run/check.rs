use modular_decomposition::ModuleKind;
use petgraph::Direction;

use super::{GenGraph, Graph};
use crate::graph::{
    algorithms::modular_decomposition::Tree,
    generic::{ImplGraph, algorithms::claw_free::ClawFree},
};

// Default: bool defaults to false
#[derive(Default, PartialEq, Debug)]
pub struct Check {
    pub claw_free: bool,
    pub simplicial: bool,
    #[cfg(debug_assertions)]
    pub parallel: bool,
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
