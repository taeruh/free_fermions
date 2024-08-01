use crate::graph::{
    algorithms::modular_decomposition::{NodeIndex, Tree},
    generic::{Graph, ImplGraph},
};

impl<G: ImplGraph> Graph<G> {
    pub fn modular_decomposition(&self) -> Tree {
        let md_tree = modular_decomposition::modular_decomposition(&self).unwrap();
        Tree {
            root: NodeIndex::from(md_tree.root().index() as u32),
            graph: md_tree.into_digraph(),
        }
    }
}
