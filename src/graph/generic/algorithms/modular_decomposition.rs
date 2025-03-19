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
            (0, 1),
            (0, 2),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
            (1, 6),
            (1, 7),
            (1, 8),
            (2, 3),
            (2, 4),
            (2, 5),
            (2, 6),
            (2, 7),
            (2, 8),
            (3, 4),
            (4, 5),
            (4, 6),
            (4, 7),
            (4, 8),
            (5, 6),
            (6, 7),
            (6, 8),
            (7, 8),
        ];
        let graph: Graph<Pet> = Graph::from_edge_labels(list).unwrap();
        let tree = graph.modular_decomposition();
        println!("{:?}", tree);
    }
}
