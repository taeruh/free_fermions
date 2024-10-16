pub mod claw_free;
pub mod modular_decomposition;
pub mod obstinate;
pub mod twin_collapse;

#[cfg(test)]
pub mod test_impl {
    use std::fmt::Debug;

    use hashbrown::HashMap;

    use super::{modular_decomposition::Tree, obstinate::ObstinateMapped};
    use crate::graph::{HLabels, Label, Node};

    pub trait RequiredMethods: Debug {
        type ClawFree: Into<bool> + Debug;
        fn from_adj_list(adj_list: HashMap<Label, HLabels>) -> Self;
        fn modular_decomposition(&self) -> Tree;
        fn twin_collapse(&mut self, tree: &mut Tree);
        fn obstinate(&self) -> ObstinateMapped;
        fn is_claw_free(&self, tree: &Tree) -> Self::ClawFree;
        fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy;
        fn map_to_labels(&self) -> HashMap<Label, HLabels>;
    }
}
