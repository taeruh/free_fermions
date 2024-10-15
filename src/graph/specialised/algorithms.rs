pub mod claw_free;
pub mod modular_decomposition;
pub mod obstinate;
pub mod simplicial;
pub mod twin_collapse;

#[cfg(test)]
mod test_impl {
    use hashbrown::HashMap;

    use crate::graph::{
        HLabels, Label, Node,
        algorithms::{
            modular_decomposition::Tree, obstinate::ObstinateMapped,
            test_impl::RequiredMethods,
        },
        specialised::{Graph, GraphData},
    };

    impl<G: GraphData> RequiredMethods for Graph<G> {
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
        fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy {
            self.get_label_mapping()
        }
        fn map_to_labels(&self) -> HashMap<Label, HLabels> {
            self.map_to_labels()
        }
    }
}
