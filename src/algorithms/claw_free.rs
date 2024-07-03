use modular_decomposition::ModuleKind;
use petgraph::Direction;

use super::modular_decomposition::Tree;
use crate::{
    algorithms::modular_decomposition::TreeGraph,
    enumerate_offset::Enumerate,
    fix_int::int,
    graph::{Graph, ImplGraph, Node, NodeCollection, NodeIndex},
    mat_mul::Matrix,
};

// No might get some data in the future
#[derive(Debug, Clone)]
pub enum ClawFree {
    Yes,
    No,
}

// impl<G: ImplGraph> Graph<G> {
impl<G: ImplGraph + std::fmt::Debug> Graph<G> {
    pub fn is_claw_free(&self, tree: &Tree) -> ClawFree {
        if !self.has_right_structure(tree) {
            return ClawFree::No;
        }
        match tree.graph.node_weight(tree.root).unwrap() {
            ModuleKind::Prime => self.prime_claw_check(tree, tree.root),
            ModuleKind::Series => self.series_claw_check(tree, tree.root),
            ModuleKind::Parallel => self.parallel_claw_check(tree),
            ModuleKind::Node(_) => ClawFree::Yes,
        }
    }

    fn prime_claw_check(&self, tree: &Tree, node: NodeIndex) -> ClawFree {
        let reprs = tree.module_representatives(node);
        let representative_graph = self.subgraph(&reprs);
        representative_graph.is_claw_free_naive()
    }

    fn series_claw_check(&self, tree: &Tree, node: NodeIndex) -> ClawFree {
        for child in tree.graph.neighbors_directed(node, Direction::Outgoing) {
            if let ModuleKind::Prime = tree.graph.node_weight(child).unwrap() {
                let reprs = tree.module_representatives(child);
                let mut complement_representative_graph = self.subgraph(&reprs);
                complement_representative_graph.complement();
                let (indices, matrix) = to_matrix(&complement_representative_graph);
                let count = matrix.diag_cube();
                if count.iter().sum::<u32>() != 0 {
                    println!("{:?}", self.get_label(node.index() as int));
                    println!("{:?}", self.get_label(child.index() as int));
                    println!("{:?}", (indices, count));
                    return ClawFree::No;
                }
            }
        }
        ClawFree::Yes
    }

    fn parallel_claw_check(&self, tree: &Tree) -> ClawFree {
        for child in tree.graph.neighbors_directed(tree.root, Direction::Outgoing) {
            if let ClawFree::No = match tree.graph.node_weight(child).unwrap() {
                ModuleKind::Prime => self.prime_claw_check(tree, child),
                ModuleKind::Series => self.series_claw_check(tree, child),
                ModuleKind::Parallel => unreachable!("parallel node in parallel node"),
                ModuleKind::Node(_) => ClawFree::Yes,
            } {
                return ClawFree::No;
            }
        }
        ClawFree::Yes
    }

    fn has_right_structure(&self, tree: &Tree) -> bool {
        fn some_non_clique_children(tree: &TreeGraph, node: NodeIndex) -> bool {
            for child in tree.neighbors_directed(node, Direction::Outgoing) {
                match tree.node_weight(child).unwrap() {
                    ModuleKind::Prime => return true,
                    ModuleKind::Parallel => return true,
                    _ => {},
                }
            }
            false
        }

        #[inline]
        fn prime_check(tree: &TreeGraph, node: NodeIndex) -> bool {
            !some_non_clique_children(tree, node)
        }

        #[inline]
        fn series_check(tree: &TreeGraph, node: NodeIndex) -> bool {
            for child in tree.neighbors_directed(node, Direction::Outgoing) {
                match tree.node_weight(child).unwrap() {
                    ModuleKind::Prime => {
                        if some_non_clique_children(tree, child) {
                            return false;
                        }
                    },
                    ModuleKind::Series => {
                        unreachable!("series node in series node")
                    },
                    ModuleKind::Parallel => {
                        let mut count = 0;
                        for child in tree.neighbors_directed(child, Direction::Outgoing) {
                            count += 1;
                            if (count > 2) || some_non_clique_children(tree, child) {
                                return false;
                            }
                        }
                    },
                    ModuleKind::Node(_) => {},
                };
            }
            true
        }

        #[inline]
        fn parallel_check(tree: &TreeGraph, node: NodeIndex) -> bool {
            for child in tree.neighbors_directed(node, Direction::Outgoing) {
                match tree.node_weight(child).unwrap() {
                    ModuleKind::Prime => {
                        if some_non_clique_children(tree, child) {
                            return false;
                        }
                    },
                    ModuleKind::Series => {
                        if !series_check(tree, child) {
                            return false;
                        }
                    },
                    ModuleKind::Parallel => {
                        unreachable!("parallel node in parallel node")
                    },
                    ModuleKind::Node(_) => {},
                }
            }
            true
        }

        match tree.graph.node_weight(tree.root).unwrap() {
            ModuleKind::Prime => prime_check(&tree.graph, tree.root),
            ModuleKind::Series => series_check(&tree.graph, tree.root),
            ModuleKind::Parallel => parallel_check(&tree.graph, tree.root),
            ModuleKind::Node(_) => true,
        }
    }

    // the simplest algorithm without (trying to) doing any smart things
    pub fn is_claw_free_naive(&self) -> ClawFree {
        for (node, neighbourhood) in self.iter_with_neighbourhoods() {
            let mut graph = self.subgraph(neighbourhood);
            graph.complement();
            let (indices, matrix) = to_matrix(&graph);
            let count = matrix.diag_cube();
            println!(
                "node {}:\nidcs: {indices:?}\ncnt:  {count:?}\n",
                self.get_label(node).unwrap()
            );
        }
        ClawFree::Yes
    }
}

fn to_matrix<G: ImplGraph>(graph: &Graph<G>) -> (Vec<Node>, Matrix) {
    let len = graph.len();
    let mut array = vec![0; len * len];
    let mut indices = Vec::with_capacity(len);
    let mut nodes = graph.iter_with_neighbourhoods().enumerate();
    while let Some((row, (node, _))) = nodes.next() {
        indices.push(graph.get_label(node).unwrap());
        let row_shift = row * len;
        for (col, (_, neighborhood)) in nodes.clone() {
            let has_edge = neighborhood.contains(node);
            array[row_shift + col] = has_edge.into();
            array[col * len + row] = has_edge.into();
        }
    }
    (indices, Matrix::from_vec_with_shape(array, (len, len)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_utils::collect;

    #[test]
    fn test() {
        //    - 1
        //  /
        // 0 -- 2
        //  \
        //    - 3
        let mut graph: Graph = Graph::from_adjacency_labels(collect!(vv;
                (0, [1, 2, 3]),
                (1, [0]),
                (2, [0]),
                (3, [0]),
        ))
        .unwrap();
        // let mut tree = graph.modular_decomposition();
        // println!("{:?}", graph);
        // println!("{:?}", tree);
        // graph.is_claw_free_naive();
        // println!("real: {:?}", graph.is_claw_free(&tree));
        // // // #claws = 1
        // println!("structure: {:?}", graph.has_right_structure(&tree));
        // graph.twin_collapse(&mut tree);
        // println!("{:?}", graph);
        // println!("{:?}", tree);
        // graph.is_claw_free_naive();
        // println!("structure: {:?}", graph.has_right_structure(&tree));
        // println!("real: {:?}", graph.is_claw_free(&tree));

        //    - 1 -- 4
        //  /
        // 0 -- 2 -- 5
        //  \
        //    - 3 -- 6
        let mut graph: Graph = Graph::from_adjacency_labels(collect!(vv;
            (0, [1, 2, 3]),
            (1, [0, 4]),
            (2, [0, 5]),
            (3, [0, 6]),
            (4, [1]),
            (5, [2]),
            (6, [3]),
        ))
        .unwrap();
        // let mut tree = graph.modular_decomposition();
        // println!("{:?}", graph);
        // println!("{:?}", tree);
        // graph.is_claw_free_naive();
        // println!("structure: {:?}", graph.has_right_structure(&tree));
        // let clone = graph.clone();
        // graph.twin_collapse(&mut tree);
        // assert_eq!(clone.map_to_labels(), graph.map_to_labels());

        // 10 -- 7 -     - 1 -- 4
        //           \ /
        // 11 -- 8 -- 0 -- 2 -- 5
        //           / \
        // 13 -- 9 -     - 3 -- 6
        let graph: Graph = Graph::from_adjacency_labels(collect!(vv;
                (0, [1, 2, 3, 7, 8, 9]),
                (1, [0, 4]),
                (2, [0, 5]),
                (3, [0, 6]),
                (4, [1]),
                (5, [2]),
                (6, [3]),
                (7, [0, 10]),
                (8, [0, 11]),
                (9, [0, 12]),
                (10, [7]),
                (11, [8]),
                (12, [9]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();
        // #claws = binom(6, 3) = 20

        //    - 1 -
        //  /       \
        // 0 -- 2 -- 4
        //  \
        //    - 3
        let mut graph: Graph = Graph::from_adjacency_labels(collect!(vv;
                (0, [1, 2, 3]),
                (1, [0, 4]),
                (2, [0, 4]),
                (3, [0]),
                (4, [1, 2]),
        ))
        .unwrap();
        let mut tree = graph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:?}", tree);
        graph.is_claw_free_naive();
        println!("real: {:?}", graph.is_claw_free(&tree));
        graph.twin_collapse(&mut tree);
        println!("{:?}", graph);
        println!("{:?}", tree);
        graph.is_claw_free_naive();
        println!("real: {:?}", graph.is_claw_free(&tree));

        //    - 1
        //  /
        // 0 -- 2
        //  \   |
        //    - 3
        let graph: Graph = Graph::from_adjacency_labels(collect!(vv;
                (0, [1, 2, 3]),
                (1, [0]),
                (2, [0, 3]),
                (3, [0, 2]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();

        // 7       - 1 -- 4
        // |     /
        // 8 -- 0 -- 2 -- 5
        // |     \
        // 9 --10  - 3 -- 6
        let mut graph: Graph = Graph::from_adjacency_labels(collect!(vv;
                (0, [1, 2, 3, 8]),
                (1, [0, 4]),
                (2, [0, 5]),
                (3, [0, 6]),
                (4, [1]),
                (5, [2]),
                (6, [3]),
                (7, [8]),
                (8, [0, 7, 9]),
                (9, [8, 10]),
                (10, [9]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();
        // graph.remove_node(graph.find_node(0).unwrap());
        // graph.is_claw_free_naive();

        // 7 -     - 1 -- 4
        // |   \ /
        // 8 -- 0 -- 2 -- 5
        //       \
        //         - 3 -- 6
        let graph: Graph = Graph::from_adjacency_labels(collect!(vv;
                (0, [1, 2, 3, 7, 8]),
                (1, [0, 4]),
                (2, [0, 5]),
                (3, [0, 6]),
                (4, [1]),
                (5, [2]),
                (6, [3]),
                (7, [0, 8]),
                (8, [0, 7]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();
        // #claws = binom(6, 3) = 20
    }
}
