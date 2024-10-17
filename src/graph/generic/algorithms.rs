pub mod claw_free;
pub mod modular_decomposition;
pub mod obstinate;
pub mod simplicial;
pub mod twin_collapse;

#[cfg(test)]
mod test_impl {
    use hashbrown::HashMap;

    use super::claw_free::ClawFree;
    use crate::graph::{
        HLabels, Label, Node, VLabels,
        algorithms::{
            modular_decomposition::Tree, obstinate::ObstinateMapped,
            test_impl::RequiredMethods,
        },
        generic::{Graph, ImplGraph},
    };

    impl<G: ImplGraph> RequiredMethods for Graph<G> {
        type ClawFree = ClawFree;
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
            self.twin_collapse(tree)
        }
        fn is_claw_free(&self, tree: &Tree) -> Self::ClawFree {
            self.is_claw_free(tree)
        }
        fn simplicial(&self, tree: &Tree) -> Vec<Vec<VLabels>> {
            self.simplicial(tree, None).unwrap()
        }
        fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy {
            ImplGraph::get_label_mapping(self)
        }
        fn map_to_labels(&self) -> HashMap<Label, HLabels> {
            ImplGraph::map_to_labels(self)
        }
    }

    impl From<ClawFree> for bool {
        fn from(claw_free: ClawFree) -> Self {
            match claw_free {
                ClawFree::Yes => true,
                ClawFree::No(_) => false,
            }
        }
    }
}
