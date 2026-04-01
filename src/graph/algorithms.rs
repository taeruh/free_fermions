pub mod claw_free;
pub mod modular_decomposition;
pub mod obstinate;
pub mod simplicial;
pub mod twin_collapse;

#[cfg(test)]
pub mod test_impls {
    use std::fmt::Debug;

    use hashbrown::HashMap;

    use super::{modular_decomposition::Tree, obstinate::ObstinateMapped};
    use crate::graph::{
        generic::algorithms::is_line_graph::SageProcess, HLabels, Label, Node, VLabels,
    };

    /// Helper to call parts of generic tests only once (cf. the `check` in
    /// simplicial.rs).
    pub trait DoItOnce {
        /// Only one graph should return true; all the other types should return false. We
        /// choose `Custom` to be the one that returns true.
        fn once() -> bool;
    }

    pub trait RequiredMethods: Debug + DoItOnce {
        type ClawFree: Into<bool> + Debug;
        fn from_adj_list(adj_list: HashMap<Label, HLabels>) -> Self;
        fn modular_decomposition(&self) -> Tree;
        fn twin_collapse(&mut self, tree: &mut Tree, sage_process: &mut SageProcess);
        fn obstinate(&self) -> ObstinateMapped;
        fn is_claw_free(&self, tree: &Tree) -> Self::ClawFree;
        fn simplicial(&self, tree: &Tree) -> Vec<Vec<VLabels>>;
        fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy;
        fn map_to_labels(&self) -> HashMap<Label, HLabels>;
    }
}
