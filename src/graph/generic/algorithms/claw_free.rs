use modular_decomposition::ModuleKind;
use petgraph::Direction;

use super::modular_decomposition::{NodeIndex, Tree};
use crate::{
    fix_int::int,
    graph::{
        generic::{
            algorithms::modular_decomposition::TreeGraph, Graph, ImplGraph,
            NodeCollection,
        },
        Node,
    },
    mat_mul::Matrix,
};

#[derive(Debug, Clone)]
pub enum ClawFree {
    Yes,
    No(FailKind),
}

#[derive(Debug, Clone)]
pub enum ClawFreeNaive {
    Yes,
    No(Claw),
}

#[derive(Debug, Clone)]
pub enum FailKind {
    Structure(StructureFail),
    PrimeCase(Claw),
    SeriesCase(Triangles),
}

#[derive(Debug, Clone)]
pub enum Structure {
    Yes,
    No(StructureFail),
}

#[derive(Debug, Clone)]
pub enum StructureFail {
    PrimeNonClique(NodeIndex),
    SeriesPrimeNonClique(NodeIndex, NodeIndex),
    SeriesParallelNonClique(NodeIndex, NodeIndex),
    SeriesParallelCount(NodeIndex, int),
    ParallelPrimeNonClique(NodeIndex, NodeIndex),
    ParallelSeriesPrimeNonClique(NodeIndex, NodeIndex, NodeIndex),
    ParallelSeriesParallelNonClique(NodeIndex, NodeIndex, NodeIndex),
    ParallelSeriesParallelCount(NodeIndex, NodeIndex, int),
}

#[derive(Debug, Clone)]
pub struct Triangles {
    pub indices: Vec<Node>,
    pub counts: Vec<int>,
}

#[derive(Debug, Clone)]
pub struct Claw {
    pub center: Node,
    pub leaves: Triangles,
}

// impl<G: ImplGraph> Graph<G> {
impl<G: ImplGraph> Graph<G> {
    pub fn is_claw_free(&self, tree: &Tree) -> ClawFree {
        match self.has_right_structure(tree) {
            Structure::No(fail) => return ClawFree::No(FailKind::Structure(fail)),
            Structure::Yes => {},
        }

        match tree.graph.node_weight(tree.root).unwrap() {
            ModuleKind::Prime => self.prime_claw_check(tree, tree.root),
            ModuleKind::Series => self.series_claw_check(tree, tree.root),
            ModuleKind::Parallel => self.parallel_claw_check(tree),
            ModuleKind::Node(_) => ClawFree::Yes,
        }
    }

    fn prime_claw_check(&self, tree: &Tree, module: NodeIndex) -> ClawFree {
        let reprs = tree.reduced_module(module);
        let representative_graph = self.subgraph(&reprs);
        // TODO: replace with a little bit more efficient matrix algorithm
        match representative_graph.is_claw_free_naive() {
            ClawFreeNaive::Yes => ClawFree::Yes,
            ClawFreeNaive::No(claw) => ClawFree::No(FailKind::PrimeCase(claw)),
        }
    }

    fn series_claw_check(&self, tree: &Tree, module: NodeIndex) -> ClawFree {
        for child in tree.graph.neighbors_directed(module, Direction::Outgoing) {
            if let ModuleKind::Prime = tree.graph.node_weight(child).unwrap() {
                let reprs = tree.reduced_module(child);
                let mut complement_representative_graph = self.subgraph(&reprs);
                complement_representative_graph.complement();
                let (indices, matrix) = to_matrix(&complement_representative_graph);
                let counts = matrix.diag_cube();
                for c in counts.iter() {
                    if c != 0 {
                        return ClawFree::No(FailKind::SeriesCase(Triangles {
                            indices,
                            counts,
                        }));
                    }
                }
            }
        }
        ClawFree::Yes
    }

    fn parallel_claw_check(&self, tree: &Tree) -> ClawFree {
        for child in tree.graph.neighbors_directed(tree.root, Direction::Outgoing) {
            if let ClawFree::No(fail) = match tree.graph.node_weight(child).unwrap() {
                ModuleKind::Prime => self.prime_claw_check(tree, child),
                ModuleKind::Series => self.series_claw_check(tree, child),
                ModuleKind::Parallel => unreachable!("parallel node in parallel node"),
                ModuleKind::Node(_) => ClawFree::Yes,
            } {
                return ClawFree::No(fail);
            }
        }
        ClawFree::Yes
    }

    fn has_right_structure(&self, tree: &Tree) -> Structure {
        fn is_clique(tree: &TreeGraph, node: NodeIndex) -> bool {
            match tree.node_weight(node).unwrap() {
                ModuleKind::Prime => return false,
                ModuleKind::Series => {
                    for child in tree.neighbors_directed(node, Direction::Outgoing) {
                        match tree.node_weight(child).unwrap() {
                            ModuleKind::Node(_) => {},
                            _ => return false,
                        }
                    }
                },
                ModuleKind::Parallel => return false,
                _ => {},
            }
            true
        }

        fn some_non_clique_children(
            tree: &TreeGraph,
            node: NodeIndex,
        ) -> Option<NodeIndex> {
            tree.neighbors_directed(node, Direction::Outgoing)
                .find(|&child| !is_clique(tree, child))
        }

        #[inline]
        fn prime_check(tree: &TreeGraph, root: NodeIndex) -> Structure {
            match some_non_clique_children(tree, root) {
                Some(child) => Structure::No(StructureFail::PrimeNonClique(child)),
                None => Structure::Yes,
            }
        }

        #[inline]
        fn series_check(tree: &TreeGraph, node: NodeIndex) -> Structure {
            for child in tree.neighbors_directed(node, Direction::Outgoing) {
                match tree.node_weight(child).unwrap() {
                    ModuleKind::Prime => {
                        if let Some(grandchild) = some_non_clique_children(tree, child) {
                            return Structure::No(StructureFail::SeriesPrimeNonClique(
                                child, grandchild,
                            ));
                        }
                    },
                    ModuleKind::Series => {
                        unreachable!("series node in series node")
                    },
                    ModuleKind::Parallel => {
                        let mut count = 0;
                        for grandchild in
                            tree.neighbors_directed(child, Direction::Outgoing)
                        {
                            count += 1;
                            if count > 2 {
                                return Structure::No(
                                    StructureFail::SeriesParallelCount(child, count),
                                );
                            }
                            if !is_clique(tree, grandchild) {
                                return Structure::No(
                                    StructureFail::SeriesParallelNonClique(
                                        child, grandchild,
                                    ),
                                );
                            }
                        }
                    },
                    ModuleKind::Node(_) => {},
                };
            }
            Structure::Yes
        }

        #[inline]
        fn parallel_check(tree: &TreeGraph, root: NodeIndex) -> Structure {
            for child in tree.neighbors_directed(root, Direction::Outgoing) {
                match tree.node_weight(child).unwrap() {
                    ModuleKind::Prime => {
                        // if some_non_clique_children(tree, child) {
                        //     return false;
                        // }
                        match prime_check(tree, child) {
                            Structure::Yes => {},
                            Structure::No(StructureFail::PrimeNonClique(grandchild)) => {
                                return Structure::No(
                                    StructureFail::ParallelPrimeNonClique(
                                        child, grandchild,
                                    ),
                                );
                            },
                            _ => unreachable!(),
                        }
                    },
                    ModuleKind::Series => match series_check(tree, child) {
                        Structure::Yes => {},
                        Structure::No(StructureFail::SeriesPrimeNonClique(
                            grandchild,
                            ggchild,
                        )) => {
                            return Structure::No(
                                StructureFail::ParallelSeriesPrimeNonClique(
                                    child, grandchild, ggchild,
                                ),
                            );
                        },
                        Structure::No(StructureFail::SeriesParallelNonClique(
                            grandchild,
                            ggchild,
                        )) => {
                            return Structure::No(
                                StructureFail::ParallelSeriesParallelNonClique(
                                    child, grandchild, ggchild,
                                ),
                            );
                        },
                        Structure::No(StructureFail::SeriesParallelCount(
                            grandchild,
                            count,
                        )) => {
                            return Structure::No(
                                StructureFail::ParallelSeriesParallelCount(
                                    child, grandchild, count,
                                ),
                            );
                        },
                        _ => unreachable!(),
                    },
                    ModuleKind::Parallel => {
                        unreachable!("parallel node in parallel node")
                    },
                    ModuleKind::Node(_) => {},
                }
            }
            Structure::Yes
        }

        match tree.graph.node_weight(tree.root).unwrap() {
            ModuleKind::Prime => prime_check(&tree.graph, tree.root),
            ModuleKind::Series => series_check(&tree.graph, tree.root),
            ModuleKind::Parallel => parallel_check(&tree.graph, tree.root),
            ModuleKind::Node(_) => Structure::Yes,
        }
    }

    // the simplest algorithm without (trying to) doing any smart things
    pub fn is_claw_free_naive(&self) -> ClawFreeNaive {
        for (node, neighbourhood) in self.iter_with_neighbourhoods() {
            let mut graph = self.subgraph(&neighbourhood);
            graph.complement();
            let (indices, matrix) = to_matrix(&graph);
            let counts = matrix.diag_cube();
            for c in counts.iter() {
                if c != 0 {
                    return ClawFreeNaive::No(Claw {
                        center: node,
                        leaves: Triangles { indices, counts },
                    });
                }
            }
        }
        ClawFreeNaive::Yes
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
    use crate::graph::{
        generic::{adj, impl_petgraph},
        test_utils::collect,
    };

    type AdjGraph = Graph<adj::AdjGraph>;
    type PetGraph = Graph<impl_petgraph::PetGraph>;

    fn check(a: ClawFree, b: ClawFree) {
        match (a, b) {
            (ClawFree::Yes, ClawFree::Yes) => {},
            (ClawFree::No(a), ClawFree::No(b)) => match (a, b) {
                (FailKind::Structure(_), FailKind::Structure(_)) => {},
                (FailKind::PrimeCase(_), FailKind::PrimeCase(_)) => {},
                (FailKind::SeriesCase(_), FailKind::SeriesCase(_)) => {},
                _ => panic!("not equal"),
            },
            _ => panic!("not equal"),
        }
    }

    #[test]
    fn test() {
        //    - 1
        //  /
        // 0 -- 2
        //  \
        //    - 3
        let data = collect!(vv;
                (0, [1, 2, 3]),
                (1, [0]),
                (2, [0]),
                (3, [0]),
        );
        let mut graph = AdjGraph::from_adjacency_labels(data.clone()).unwrap();
        let mut tree = graph.modular_decomposition();
        let mut pgraph = PetGraph::from_adjacency_labels(data).unwrap();
        let mut ptree = pgraph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);
        // // #claws = 1
        graph.twin_collapse(&mut tree);
        pgraph.twin_collapse(&mut ptree);
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);

        //    - 1 -- 4
        //  /
        // 0 -- 2 -- 5
        //  \
        //    - 3 -- 6
        let data = collect!(vv;
            (0, [1, 2, 3]),
            (1, [0, 4]),
            (2, [0, 5]),
            (3, [0, 6]),
            (4, [1]),
            (5, [2]),
            (6, [3]),
        );
        let mut graph = AdjGraph::from_adjacency_labels(data.clone()).unwrap();
        let mut tree = graph.modular_decomposition();
        let mut pgraph = PetGraph::from_adjacency_labels(data).unwrap();
        let mut ptree = pgraph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);
        graph.twin_collapse(&mut tree);
        assert_eq!(pgraph.map_to_labels(), graph.map_to_labels());

        // 10 -- 7 -     - 1 -- 4
        //           \ /
        // 11 -- 8 -- 0 -- 2 -- 5
        //           / \
        // 13 -- 9 -     - 3 -- 6
        let data = collect!(vv;
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
        );
        let graph = AdjGraph::from_adjacency_labels(data.clone()).unwrap();
        let tree = graph.modular_decomposition();
        let pgraph = PetGraph::from_adjacency_labels(data.clone()).unwrap();
        let ptree = pgraph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);
        // #claws = binom(6, 3) = 20

        //    - 1 -
        //  /       \
        // 0 -- 2 -- 4
        //  \
        //    - 3
        let data = collect!(vv;
                (0, [1, 2, 3]),
                (1, [0, 4]),
                (2, [0, 4]),
                (3, [0]),
                (4, [1, 2]),
        );
        let mut graph = AdjGraph::from_adjacency_labels(data.clone()).unwrap();
        let mut tree = graph.modular_decomposition();
        let mut pgraph = PetGraph::from_adjacency_labels(data.clone()).unwrap();
        let mut ptree = pgraph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);
        graph.twin_collapse(&mut tree);
        pgraph.twin_collapse(&mut ptree);
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);

        // 7       - 1 -- 4
        // |     /
        // 8 -- 0 -- 2 -- 5
        // |     \
        // 9 --10  - 3 -- 6
        // note that we collect here with hh
        let data = collect!(hh;
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
        );
        let graph = AdjGraph::from_adjacency_labels(data.clone()).unwrap();
        let tree = graph.modular_decomposition();
        let pgraph = PetGraph::from_adjacency_labels(data).unwrap();
        let ptree = pgraph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);

        //    - 1
        //  /
        // 0 -- 2
        //  \   |
        //    - 3
        let data = collect!(vv;
                (0, [1, 2, 3]),
                (1, [0]),
                (2, [0, 3]),
                (3, [0, 2]),
        );
        let graph = AdjGraph::from_adjacency_labels(data.clone()).unwrap();
        let tree = graph.modular_decomposition();
        let pgraph = PetGraph::from_adjacency_labels(data.clone()).unwrap();
        let ptree = graph.modular_decomposition();
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);

        // 7 -     - 1 -- 4
        // |   \ /
        // 8 -- 0 -- 2 -- 5
        //       \
        //         - 3 -- 6
        // note that we collect here with hh
        let data = collect!(hh;
                (0, [1, 2, 3, 7, 8]),
                (1, [0, 4]),
                (2, [0, 5]),
                (3, [0, 6]),
                (4, [1]),
                (5, [2]),
                (6, [3]),
                (7, [0, 8]),
                (8, [0, 7]),
        );
        let graph = AdjGraph::from_adjacency_labels(data.clone()).unwrap();
        let tree = graph.modular_decomposition();
        let pgraph = PetGraph::from_adjacency_labels(data).unwrap();
        let ptree = pgraph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:?}", tree);
        println!("naive: {:?}", graph.is_claw_free_naive());
        let ret = graph.is_claw_free(&tree);
        let pret = pgraph.is_claw_free(&ptree);
        println!("real: {:?}\n", ret);
        check(ret, pret);
    }
}
