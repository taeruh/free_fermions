use modular_decomposition::ModuleKind;
use petgraph::Direction;

use crate::{
    graph::{
        algorithms::modular_decomposition::{NodeIndex, Tree},
        specialised::{Graph, GraphData},
    },
    mat_mul::Matrix,
};

impl<G: GraphData> Graph<G> {
    pub fn is_claw_free_naive(&self) -> bool {
        for neighbourhood in self.iter_neighbours() {
            // safety: neighbourhood nodes have to be valid nodes
            let mut graph = unsafe { self.subgraph_from_set(neighbourhood.len(), neighbourhood) };
            graph.complement();
            if graph.has_triangle() {
                return false;
            }
        }
        true
    }

    /// # Safety
    /// The graph must be connected
    pub unsafe fn is_claw_free(&self, tree: &Tree) -> bool {
        if !Self::has_right_tree_structure(tree) {
            return false;
        }
        match tree.graph.node_weight(tree.root).unwrap() {
            ModuleKind::Prime => self.prime_claw_check(tree),
            ModuleKind::Series => self.series_claw_check(tree),
            ModuleKind::Parallel => unsafe {
                // safety: invariant promises that the graph is connected
                debug_unreachable_unchecked!("graph is connected");
            },
            ModuleKind::Node(_) => true,
        }
    }

    fn has_right_tree_structure(tree: &Tree) -> bool {
        #[inline]
        fn prime_check(tree: &Tree, module: NodeIndex) -> bool {
            tree.graph
                .neighbors_directed(module, Direction::Outgoing)
                .all(|child| tree.module_is_clique(child))
        }

        #[inline]
        fn series_check(tree: &Tree) -> bool {
            for child in tree
                .graph
                .neighbors_directed(tree.root, Direction::Outgoing)
            {
                match tree.graph.node_weight(child).unwrap() {
                    ModuleKind::Prime => {
                        if !prime_check(tree, child) {
                            return false;
                        }
                    }
                    ModuleKind::Series => unsafe {
                        // safety: assume modular decomposition is correct
                        debug_unreachable_unchecked!("series module has series children");
                    },
                    ModuleKind::Parallel => {
                        let mut count = 0;
                        for gchild in tree.graph.neighbors_directed(child, Direction::Outgoing) {
                            count += 1;
                            if (count > 2) || !tree.module_is_clique(gchild) {
                                return false;
                            }
                        }
                    }
                    ModuleKind::Node(_) => {}
                }
            }
            true
        }

        match tree.graph.node_weight(tree.root).unwrap() {
            ModuleKind::Prime => prime_check(tree, tree.root),
            ModuleKind::Series => series_check(tree),
            ModuleKind::Parallel => unsafe {
                // safety: invariant promises that the graph is connected
                debug_unreachable_unchecked!("graph is connected");
            },
            ModuleKind::Node(_) => true,
        }
    }

    fn prime_claw_check(&self, tree: &Tree) -> bool {
        let representatives = tree.reduced_module(tree.root);
        // safety: representatives are collected from graph nodes
        unsafe { self.subgraph(representatives.len(), representatives) }.is_claw_free_naive()
    }

    fn series_claw_check(&self, tree: &Tree) -> bool {
        for child in tree
            .graph
            .neighbors_directed(tree.root, Direction::Outgoing)
        {
            match tree.graph.node_weight(child).unwrap() {
                ModuleKind::Prime => {
                    // any independent set of size 3 will induce a claw, because they have
                    // a shared neighbour in some other module (cf. paper)
                    let representatives = tree.reduced_module(child);
                    let mut complement_representative_graph =
                        unsafe { self.subgraph(representatives.len(), representatives) };
                    complement_representative_graph.complement();
                    if complement_representative_graph.has_triangle() {
                        return false;
                    }
                }
                ModuleKind::Series => unsafe {
                    // safety: assume modular decomposition is correct
                    debug_unreachable_unchecked!("series module has series children");
                },

                ModuleKind::Parallel => {
                    // nothing to do here: we know that we have the right structure, so
                    // this module does not contain any independent set of size 3 (cf.
                    // paper)
                }
                ModuleKind::Node(_) => {}
            }
        }
        true
    }

    // TODO: check the complexity here again (vs naive triangle search), and maybe use a
    // more efficient matrix multiplication algorithm
    fn has_triangle(&self) -> bool {
        let len = self.len();
        let mut array = vec![0; len * len];
        let mut nodes = self.enumerate_neighbours();
        while let Some((row, _)) = nodes.next() {
            let row_shift = row * len;
            for (col, neighborhood) in nodes.clone() {
                let has_edge = neighborhood.contains(&row);
                array[row_shift + col] = has_edge.into();
                array[col * len + row] = has_edge.into();
            }
        }

        let matrix = Matrix::from_vec_with_shape(array, (len, len));
        for &c in matrix.diag_cube().iter() {
            if c != 0 {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{
        algorithms::claw_free,
        specialised::{Custom, Graph, IndexMap},
    };

    claw_free::tests::test_it!(custom, Graph<Custom>);
    claw_free::tests::test_it!(indexmap, Graph<IndexMap>);
}
