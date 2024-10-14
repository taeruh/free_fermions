use crate::graph::{
    algorithms::modular_decomposition::{NodeIndex, Tree},
    specialised::{Graph, GraphData},
};

impl<G: GraphData> Graph<G> {
    pub fn modular_decomposition(&self) -> Tree {
        let md_tree = modular_decomposition::modular_decomposition(&self).unwrap();
        Tree {
            root: NodeIndex::from(md_tree.root().index() as u32),
            graph: md_tree.into_digraph(),
        }
    }
}

// tests covered in src/graph/algorithms/modular_decomposition.rs
