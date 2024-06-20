use modular_decomposition::ModuleKind;
use petgraph::{graph::DiGraph, stable_graph::StableDiGraph};

use crate::graph::{Graph, ImplGraph, Node, NodeIndex};

// PERF: For now we use StableGraph instead of Graph, but if we can we use Graph by
// appropriately adjusting the twin_collapse algorithm, because the into() down below is
// O(|E| + |V|); at the moment I had the representation for our graphs as stable in mind,
// maybe we want to make it unstable?, but I'm not so sure
pub type TreeData = StableDiGraph<ModuleKind<Node>, ()>;

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
            data: md_tree.into_digraph().into(),
        }
    }
}
