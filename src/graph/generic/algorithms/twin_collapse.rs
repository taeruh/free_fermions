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

        // PERF: collect because of borrow rules -> improve somehow
        // edit: actually, probably fine, since we use it twice ...
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
                tree.remove_node(
                    (tree_map.swap_remove_unchecked(child.index()) as u32).into(),
                );
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
                self.remove_node(graph_map.swap_remove_unchecked(*node));
                tree.remove_node(
                    (tree_map.swap_remove_unchecked(child.index()) as u32).into(),
                );
                num_children -= 1;
            }
        }

        let new_root = (tree_map.map(root.index()) as u32).into();
        if num_children == 0 {
            *tree.node_weight_mut(new_root).unwrap() =
                ModuleKind::Node(remaining_leaf.unwrap());
        } else {
            self.remove_node(graph_map.swap_remove_unchecked(remaining_leaf.unwrap()));
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{seq::SliceRandom, SeedableRng};
    use rand_pcg::Pcg64;

    use super::*;
    use crate::graph::{
        generic::adj::AdjGraph,
        test_utils::{collect, RandomMap},
        Label,
    };

    #[test]
    fn swap_remove() {
        const NUM_NODES: usize = 3000;
        const NUM_REMOVE: usize = NUM_NODES / 2;
        let mut rng = Pcg64::from_entropy();

        let mut pseudo_graph = (0..NUM_NODES).collect::<Vec<_>>();
        let to_remove = pseudo_graph
            .choose_multiple(&mut rng, NUM_REMOVE)
            .copied()
            .collect::<Vec<usize>>();
        let mut map = SwapRemoveMap::new(NUM_NODES);

        for node in to_remove.into_iter() {
            let removed = pseudo_graph.swap_remove(map.swap_remove_unchecked(node));
            assert_eq!(removed, node);
        }
    }

    fn check<A, N>(input: A, collapsed: impl IntoIterator<Item = A>)
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        let mut graph = Graph::<AdjGraph>::from_adjacency_labels(input).unwrap();
        let expected: Vec<Graph> = collapsed
            .into_iter()
            .map(|adj| Graph::from_adjacency_labels(adj).unwrap())
            .collect();

        let mut tree = graph.modular_decomposition();
        graph.twin_collapse(&mut tree);

        let sanity_tree = graph.modular_decomposition();
        assert!(Tree::is_equivalent(
            &tree,
            &sanity_tree,
            graph.get_label_mapping(),
            graph.get_label_mapping()
        ));

        let mapped_graph = graph.map_to_labels();
        let equivalent_graph: &Graph = expected
            .iter()
            .find(|graph| graph.map_to_labels() == mapped_graph)
            .unwrap();
        let equivalent_tree = equivalent_graph.modular_decomposition();
        assert!(Tree::is_equivalent(
            &tree,
            &equivalent_tree,
            graph.get_label_mapping(),
            equivalent_graph.get_label_mapping()
        ));
    }

    #[test]
    fn test() {
        let rng = &mut Pcg64::from_entropy();

        let map = RandomMap::new(24, 42, rng);
        let input = collect!(
            hh, map;
            (0, [1]),
            (1, [0, 2]),
            (2, [1, 3, 4, 5]),
            (3, [2, 4]),
            (4, [2, 3]),
            (5, [2]),
        );
        let collapsed = [3, 4, 5].into_iter().map(|representative| {
            collect!(
                hh, map;
                (0, [1]),
                (1, [0, 2]),
                (2, [1, representative]),
                (representative, [2]),
            )
        });
        check(input, collapsed);
    }
}
