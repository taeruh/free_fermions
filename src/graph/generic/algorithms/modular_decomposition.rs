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

// tests covered in src/graph/algorithms/modular_decomposition.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::generic::Pet;

    #[test]
    fn this_test() {
        let list = [
            (1, 2),
            (1, 5),
            (2, 3),
            (2, 4),
            (2, 5),
            (3, 4),
            (4, 6),
            (5, 6),
        ];
        let graph: Graph<Pet> = Graph::from_edge_labels(list).unwrap();
        let tree = graph.modular_decomposition();
        println!("{:?}", tree);
        println!("{:?}", graph);
    }
}
