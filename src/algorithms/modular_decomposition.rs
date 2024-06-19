use crate::{
    fix_int::int,
    graph::{Graph, ImplGraph},
};

pub type MDTree = modular_decomposition::MDTree<int>;

impl<G: ImplGraph> Graph<G> {
    pub fn modular_decomposition(&self) -> MDTree {
        modular_decomposition::modular_decomposition(&self).unwrap()
    }
}
