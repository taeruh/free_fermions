use modular_decomposition::ModuleKind;
use petgraph::{graph::DiGraph, stable_graph::StableDiGraph};

use crate::{
    fix_int::int,
    graph::{Graph, ImplGraph, Node, NodeIndex},
};

pub type TreeData = DiGraph<ModuleKind<Node>, ()>;

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub data: TreeData,
    pub root: NodeIndex,
}

impl<G: ImplGraph> Graph<G> {
    pub fn modular_decomposition(&self) -> Tree {
        let md_tree = modular_decomposition::modular_decomposition(&self).unwrap();
        Tree {
            root: NodeIndex::from(md_tree.root().index() as Node),
            data: md_tree.into_digraph(),
        }
    }
}
