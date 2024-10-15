use modular_decomposition::ModuleKind;
use petgraph::Direction;

use crate::graph::{
    algorithms::modular_decomposition::{NodeIndex, Tree, TreeGraph},
    generic::{Graph, ImplGraph, SwapRemoveMap},
};

impl<G: ImplGraph> Graph<G> {
    pub fn twin_collapse(&mut self, tree: &mut Tree) {
        let mut graph_map = SwapRemoveMap::new(self.len());
        let mut tree_map = SwapRemoveMap::new(tree.graph.node_count());
        self.recurse_collapse(&mut tree.graph, tree.root, &mut graph_map, &mut tree_map);
        for weight in tree.graph.node_weights_mut() {
            if let ModuleKind::Node(ref mut node) = weight {
                *node = graph_map.map(*node);
            }
        }
        tree.root = (tree_map.map(tree.root.index()) as u32).into();
    }

    fn recurse_collapse(
        &mut self,
        tree: &mut TreeGraph,
        root: NodeIndex,
        graph_map: &mut SwapRemoveMap,
        tree_map: &mut SwapRemoveMap,
    ) {
        let new_root = (tree_map.map(root.index()) as u32).into();
        if let ModuleKind::Node(_) = tree.node_weight(new_root).unwrap() {
            return;
        }

        let children: Vec<NodeIndex> =
            tree.neighbors_directed(new_root, Direction::Outgoing).collect();

        if *tree.node_weight(new_root).unwrap() == ModuleKind::Prime {
            for child in children {
                self.recurse_collapse(tree, child, graph_map, tree_map);
            }
            return;
        }

        let mut remaining_leaf = None;
        let mut num_children = children.len();

        let mut children = children.into_iter();
        for child in children.by_ref() {
            self.recurse_collapse(tree, child, graph_map, tree_map);
            if let ModuleKind::Node(node) =
                tree.node_weight((tree_map.map(child.index()) as u32).into()).unwrap()
            {
                remaining_leaf = Some(*node);
                tree.remove_node((tree_map.swap_remove(child.index()) as u32).into());
                num_children -= 1;
                break;
            }
        }
        // continue with the rest of the children
        for child in children {
            self.recurse_collapse(tree, child, graph_map, tree_map);
            if let ModuleKind::Node(node) =
                tree.node_weight((tree_map.map(child.index()) as u32).into()).unwrap()
            {
                self.remove_node(graph_map.swap_remove(*node));
                tree.remove_node((tree_map.swap_remove(child.index()) as u32).into());
                num_children -= 1;
            }
        }

        let new_root = (tree_map.map(root.index()) as u32).into();
        if num_children == 0 {
            *tree.node_weight_mut(new_root).unwrap() =
                ModuleKind::Node(remaining_leaf.unwrap());
        } else {
            self.remove_node(graph_map.swap_remove(remaining_leaf.unwrap()));
        }
    }
}

#[cfg(test)]
mod tests {
    use hashbrown::HashMap;

    use crate::graph::{
        HLabels, Label, Node,
        algorithms::{
            modular_decomposition::Tree, twin_collapse::tests::RequiredMethods,
        },
        generic::{self, ImplGraph, impl_petgraph::PetGraph},
    };

    type Graph = generic::Graph<PetGraph>;

    impl RequiredMethods for Graph {
        fn create(map: HashMap<Label, HLabels>) -> Graph {
            Graph::from_adjacency_labels(map).unwrap()
        }
        fn modular_decomposition(&self) -> Tree {
            self.modular_decomposition()
        }
        fn twin_collapse(&mut self, tree: &mut Tree) {
            self.twin_collapse(tree);
        }
        fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy {
            ImplGraph::get_label_mapping(self)
        }
        fn map_to_labels(&self) -> HashMap<Label, HLabels> {
            ImplGraph::map_to_labels(self)
        }
    }

    use crate::graph::algorithms::twin_collapse;

    #[test]
    fn foob() {
        twin_collapse::tests::test::<Graph>();
    }
}
