use modular_decomposition::ModuleKind;
use petgraph::Direction;

use super::modular_decomposition::{Tree, TreeData};
use crate::{
    fix_int::int,
    graph::{Graph, ImplGraph, NodeIndex},
};

impl<G: ImplGraph> Graph<G> {
    pub fn twin_collapse(&mut self, tree: &mut Tree) {
        let mut graph_map = SwapRemoveMap::new(self.len());
        let mut tree_map = SwapRemoveMap::new(tree.data.node_count());
        self.recurse_collapse(&mut tree.data, tree.root, &mut graph_map, &mut tree_map);
        for weight in tree.data.node_weights_mut() {
            if let ModuleKind::Node(node) = weight {
                *node = graph_map.map(*node);
            }
        }
        tree.root = tree_map.map(tree.root.index() as u32).into();
    }

    fn recurse_collapse(
        &mut self,
        data: &mut TreeData,
        root: NodeIndex,
        graph_map: &mut SwapRemoveMap,
        tree_map: &mut SwapRemoveMap,
    ) {
        let new_root = tree_map.map(root.index() as u32).into();
        if let ModuleKind::Node(_) = data.node_weight(new_root).unwrap() {
            return;
        }

        // PERF: collect because of borrow rules -> improve somehow
        // edit: actually, probably fine, since we use it twice ...
        let children: Vec<NodeIndex> =
            data.neighbors_directed(new_root, Direction::Outgoing).collect();

        if *data.node_weight(new_root).unwrap() == ModuleKind::Prime {
            for child in children {
                self.recurse_collapse(data, child, graph_map, tree_map);
            }
            return;
        }

        let mut remaining_leaf = None;
        let mut num_children = children.len();

        let mut children = children.into_iter();
        for child in children.by_ref() {
            self.recurse_collapse(data, child, graph_map, tree_map);
            if let ModuleKind::Node(node) = data.node_weight(child).unwrap() {
                remaining_leaf = Some(*node);
                data.remove_node(tree_map.swap_remove(child.index() as u32).into());
                num_children -= 1;
                break;
            }
        }
        for child in children {
            self.recurse_collapse(data, child, graph_map, tree_map);
            if let ModuleKind::Node(node) = data.node_weight(child).unwrap() {
                self.remove_node(graph_map.swap_remove(*node));
                data.remove_node(tree_map.swap_remove(child.index() as u32).into());
                num_children -= 1;
            }
        }

        let new_root = tree_map.map(root.index() as u32).into();
        if num_children == 0 {
            *data.node_weight_mut(new_root).unwrap() =
                ModuleKind::Node(remaining_leaf.unwrap());
        } else {
            self.remove_node(graph_map.swap_remove(remaining_leaf.unwrap()));
        }
    }
}

#[derive(Debug)]
struct SwapRemoveMap {
    map: Vec<int>,
    position: Vec<int>,
    end: usize,
}

impl SwapRemoveMap {
    fn new(len: usize) -> Self {
        assert!(len > 0);
        Self {
            map: (0..len as int).collect(),
            position: (0..len as int).collect(),
            end: len - 1, // assert above len > 0
        }
    }

    fn map(&self, node: int) -> int {
        self.map[node as usize]
    }

    fn swap_remove(&mut self, node: int) -> int {
        let mapped = self.map[node as usize];
        self.map[self.position[self.end] as usize] = mapped;
        self.position.swap(mapped as usize, self.end);
        self.end -= 1;
        mapped
    }
}

#[cfg(test)]
mod tests {
    use rand::{seq::SliceRandom, SeedableRng};
    use rand_pcg::Pcg64;

    use super::*;
    use crate::graph::{
        adj::AdjGraph,
        test_utils::{collect, RandomMap},
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
            let removed = pseudo_graph.swap_remove(map.swap_remove(node as u32) as usize);
            assert_eq!(removed, node);
        }
    }

    fn check<A, N>(input: A, collapsed: impl IntoIterator<Item = A>)
    where
        A: IntoIterator<Item = (int, N)>,
        N: IntoIterator<Item = int>,
    {
        let mut graph = Graph::<AdjGraph>::from_adjacencies(input).unwrap();
        let expected = collapsed
            .into_iter()
            .map(|adj| Graph::<AdjGraph>::from_adjacencies(adj).unwrap())
            .collect::<Vec<_>>();

        let mut tree = graph.modular_decomposition();
        graph.twin_collapse(&mut tree);

        let sanity_tree = graph.modular_decomposition();
        assert!(Tree::is_equivalent(&tree, &sanity_tree, &graph, &graph));

        let mapped_graph = graph.map_to_labels();
        let equivalent_graph = expected
            .iter()
            .find(|graph| graph.map_to_labels() == mapped_graph)
            .unwrap();
        let equivalent_tree = equivalent_graph.modular_decomposition();
        assert!(Tree::is_equivalent(&tree, &equivalent_tree, &graph, equivalent_graph));
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
