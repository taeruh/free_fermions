use modular_decomposition::ModuleKind;
use ndarray::Array2;
use petgraph::Direction;

use crate::{
    graph::{
        Label,
        algorithms::modular_decomposition::{NodeIndex, Tree},
        generic::{Graph, ImplGraph, NodeCollection},
        int,
    },
    matrix::MatrixTools,
};

type Matrix = Array2<int>;

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
pub struct Claw {
    pub center: Label,
    pub leaves: Triangles,
}

#[derive(Debug, Clone)]
pub struct Triangles {
    pub indices: Vec<Label>,
    pub counts: Vec<int>,
}

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
                    if *c != 0 {
                        return ClawFree::No(FailKind::SeriesCase(Triangles {
                            indices,
                            counts: counts.into_iter().map(|c| c as int).collect(),
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
        fn some_non_clique_children(tree: &Tree, node: NodeIndex) -> Option<NodeIndex> {
            tree.graph
                .neighbors_directed(node, Direction::Outgoing)
                .find(|&child| !tree.module_is_clique(child))
        }

        #[inline]
        fn prime_check(tree: &Tree, root: NodeIndex) -> Structure {
            match some_non_clique_children(tree, root) {
                Some(child) => Structure::No(StructureFail::PrimeNonClique(child)),
                None => Structure::Yes,
            }
        }

        #[inline]
        fn series_check(tree: &Tree, node: NodeIndex) -> Structure {
            for child in tree.graph.neighbors_directed(node, Direction::Outgoing) {
                match tree.graph.node_weight(child).unwrap() {
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
                            tree.graph.neighbors_directed(child, Direction::Outgoing)
                        {
                            count += 1;
                            if count > 2 {
                                return Structure::No(
                                    StructureFail::SeriesParallelCount(child, count),
                                );
                            }
                            if !tree.module_is_clique(grandchild) {
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
        fn parallel_check(tree: &Tree, root: NodeIndex) -> Structure {
            for child in tree.graph.neighbors_directed(root, Direction::Outgoing) {
                match tree.graph.node_weight(child).unwrap() {
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
            ModuleKind::Prime => prime_check(tree, tree.root),
            ModuleKind::Series => series_check(tree, tree.root),
            ModuleKind::Parallel => parallel_check(tree, tree.root),
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
                if *c != 0 {
                    return ClawFreeNaive::No(Claw {
                        center: self.get_label(node).expect("fo"),
                        leaves: Triangles {
                            indices,
                            counts: counts.into_iter().map(|c| c as int).collect(),
                        },
                    });
                }
            }
        }
        ClawFreeNaive::Yes
    }
}

fn to_matrix<G: ImplGraph>(graph: &Graph<G>) -> (Vec<Label>, Matrix) {
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
    (indices, Matrix::from_vec_with_shape(array, (len, len)).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{
        algorithms::claw_free,
        generic::{Adj, Pet},
    };

    claw_free::tests::test_it!(adjgraph, Graph<Adj>);
    claw_free::tests::test_it!(petgraph, Graph<Pet>);
}
