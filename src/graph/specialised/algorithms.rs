pub mod claw_free;
pub mod modular_decomposition;
pub mod obstinate;
pub mod simplicial;
pub mod twin_collapse;

#[cfg(test)]
mod test_impl {
    use hashbrown::HashMap;

    use crate::graph::{
        HLabels, Label, Node, VLabels,
        algorithms::{
            modular_decomposition::Tree,
            obstinate::ObstinateMapped,
            test_impls::{DoItOnce, RequiredMethods},
        },
        specialised::{Custom, Graph, GraphData, IndexMap},
    };

    impl<G: GraphData> RequiredMethods for Graph<G>
    where
        Graph<G>: DoItOnce,
    {
        type ClawFree = bool;
        fn from_adj_list(adj_list: HashMap<Label, HLabels>) -> Self {
            Graph::from_adjacency_labels(adj_list).unwrap()
        }
        fn obstinate(&self) -> ObstinateMapped {
            self.obstinate().map(|n| self.get_label(n).unwrap())
        }
        fn modular_decomposition(&self) -> Tree {
            self.modular_decomposition()
        }
        fn twin_collapse(&mut self, tree: &mut Tree) {
            unsafe { self.twin_collapse(tree) }
        }
        fn is_claw_free(&self, tree: &Tree) -> Self::ClawFree {
            unsafe { self.is_claw_free(tree) }
        }
        fn simplicial(&self, tree: &Tree) -> Vec<Vec<VLabels>> {
            let cliques = unsafe { self.simplicial(tree) };
            vec![Vec::from_iter(
                cliques
                    .into_iter()
                    .map(|c| c.into_iter().map(|v| self.get_label(v).unwrap()).collect()),
            )]
        }
        fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy {
            self.get_label_mapping()
        }
        fn map_to_labels(&self) -> HashMap<Label, HLabels> {
            self.map_to_labels()
        }
    }

    impl DoItOnce for Graph<Custom> {
        fn once() -> bool {
            true
        }
    }
    impl DoItOnce for Graph<IndexMap> {
        fn once() -> bool {
            false
        }
    }
}
