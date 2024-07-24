use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use modular_decomposition::ModuleKind;
use petgraph::Direction;

use super::claw_free::ClawFree;
use crate::graph::{
    algorithms::{
        modular_decomposition::{NodeIndex, Tree},
        obstinate::{Obstinate, ObstinateKind},
    },
    generic::{Graph, ImplGraph, NodeCollection, NodeCollectionMut},
    Node, VLabels, VNodes,
};

impl<G: ImplGraph> Graph<G> {
    /// Return None if not claw-free (checked on claw_free if provided, otherwise
    /// calculated on the graph).
    pub fn simplicial(
        &self,
        tree: &Tree,
        claw_free: Option<&ClawFree>,
    ) -> Option<Vec<Vec<VLabels>>> {
        let check = if let Some(check) = claw_free {
            Cow::Borrowed(check)
        } else {
            Cow::Owned(self.is_claw_free(tree))
        };
        if !matches!(check.as_ref(), ClawFree::Yes) {
            return None;
        }

        match tree.graph.node_weight(tree.root).unwrap() {
            modular_decomposition::ModuleKind::Prime => {
                Some(vec![self.prime_simplicial(tree)])
            },
            modular_decomposition::ModuleKind::Series => {
                Some(vec![self.series_simplicial(tree)])
            },
            modular_decomposition::ModuleKind::Parallel => {
                // this is not very efficient here, but I don't really care, because in
                // the end we will throw away parallel graphs anyway, and this here is
                // only for correctness
                let mut ret = Vec::new();
                for child in tree.graph.neighbors(tree.root) {
                    let kind = tree.graph.node_weight(child).unwrap();
                    match kind {
                        modular_decomposition::ModuleKind::Node(node) => {
                            ret.push(self.map_simplicial_cliques(vec![vec![*node]]));
                        },
                        modular_decomposition::ModuleKind::Prime => {
                            // not really efficient here (e.g., instead of re-doing the
                            // modular partition for the subtree, we could just take the
                            // subtree and update the labels)
                            let graph = self.subgraph(&tree.module_nodes(child, None));
                            let tree = graph.modular_decomposition();
                            ret.push(graph.prime_simplicial(&tree));
                        },
                        modular_decomposition::ModuleKind::Series => {
                            let graph = self.subgraph(&tree.module_nodes(child, None));
                            let tree = graph.modular_decomposition();
                            ret.push(graph.series_simplicial(&tree));
                        },
                        modular_decomposition::ModuleKind::Parallel => {
                            unreachable!("parallel child of parallel node");
                        },
                    }
                }
                Some(ret)
            },
            modular_decomposition::ModuleKind::Node(a) => {
                Some(vec![self.map_simplicial_cliques(vec![vec![*a]])])
            },
        }
    }

    fn prime_simplicial(&self, tree: &Tree) -> Vec<VLabels> {
        // the flat_map onto the modules is unnecessary, because if we are claw-free,
        // these modules would have been cliques, so the twin_collapse would have removed
        // them; however, we keep it here for correctness

        let (mut modules, mut representatives) = (HashMap::new(), Vec::new());

        for child in tree.graph.neighbors_directed(tree.root, Direction::Outgoing) {
            let repr = tree.module_representative(child);
            representatives.push(repr);
            modules.insert(self.get_label(repr).unwrap(), child);
        }

        let graph = self.subgraph(&representatives);

        let cliques = graph.prime_recurse();

        let cliques = cliques.into_iter().map(|clique| {
            clique.into_iter().flat_map(|n| {
                // there might be more possibilities to flat_map (at least for one of
                // the nodes), but I only care about getting one simplicial clique at
                // the moment
                tree.module_nodes(modules[&graph.get_label(n).unwrap()], None)
            })
        });
        self.map_simplicial_cliques(cliques)
    }

    // TODO: first collect into a HashSet, because we obviously collect some cliques
    // multiple times (e.g., consider a simplicial clique of size 2, and for both nodes
    // G\{v} is prime, then we will collect the clique twice; assuming we are not directly
    // in the obstinate case; eplicitly example: 5 node hole)
    fn prime_recurse(&self) -> Vec<VNodes> {
        if let Some(cliques) = self.obstinate_case() {
            return cliques;
        }

        let mut cliques = Vec::new();

        for node in self.iter_nodes() {
            let mut graph = self.clone();
            graph.remove_node(node);
            let tree = graph.modular_decomposition();
            if !tree.graph_is_fully_prime() {
                continue;
            }
            if self.clique_is_simplicial(&[node]) {
                cliques.push(vec![node]);
            }
            let mut subcliques = graph.prime_recurse();
            // need to have them with the correct index sets in the parent graph (self)
            subcliques.iter_mut().for_each(|clique| {
                clique.iter_mut().for_each(|v| {
                    *v = self.find_node(graph.get_label(*v).unwrap()).unwrap()
                })
            });
            for mut clique in subcliques.into_iter() {
                if self.clique_is_simplicial(&clique) {
                    cliques.push(clique.clone());
                }
                clique.push(node);
                if self.set_is_clique(clique.iter()) && self.clique_is_simplicial(&clique)
                {
                    cliques.push(clique);
                }
            }
        }

        cliques
    }

    fn clique_is_simplicial(&self, clique: &[Node]) -> bool {
        for node in clique {
            let mut neighbours = self.get_neighbours(*node).unwrap().collect();
            for n in clique {
                neighbours.remove(*n);
            }
            if !self.set_is_clique(neighbours.iter()) {
                return false;
            }
        }
        true
    }

    fn set_is_clique<C, I>(&self, set: C) -> bool
    where
        C: IntoIterator<IntoIter = I>,
        I: Iterator<Item = Node> + Clone,
    {
        let mut set = set.into_iter();
        while let Some(node) = set.next() {
            let neighbours = self.get_neighbours(node).unwrap();
            for other in set.clone() {
                if !neighbours.contains(other) {
                    return false;
                }
            }
        }
        true
    }

    fn series_simplicial(&self, tree: &Tree) -> Vec<VLabels> {
        let mut complement = self.clone();
        complement.complement();
        if let Some((a, b)) = complement.try_bipartition() {
            return self.map_simplicial_cliques([a, b]);
        }

        let mut count = 0;
        let mut update_count = || {
            count += 1;
            if count == 2 {
                println!("more than one non-node child but no bipartition found");
                true
            } else {
                false
            }
        };
        let mut ret: Vec<VLabels> = Vec::new();
        let mut non_trivial_child: Option<NodeIndex> = None;

        for child in tree.graph.neighbors_directed(tree.root, Direction::Outgoing) {
            match tree.graph.node_weight(child).unwrap() {
                ModuleKind::Node(_) => continue,
                ModuleKind::Prime => {
                    if update_count() {
                        return ret;
                    } else {
                        non_trivial_child = Some(child);
                    }
                },
                ModuleKind::Series => unreachable!("series child of series node"),
                ModuleKind::Parallel => {
                    if update_count() {
                        return ret;
                    } else {
                        non_trivial_child = Some(child);
                    }
                },
            }
        }

        if let Some(child) = non_trivial_child {
            match tree.graph.node_weight(child).unwrap() {
                ModuleKind::Prime => {
                    let graph = self.subgraph(&tree.module_nodes(child, None));
                    let tree = graph.modular_decomposition();
                    // we could just early return here, but for debugging purposes, we
                    // continue the loop
                    ret = graph.prime_simplicial(&tree);
                },
                ModuleKind::Parallel => {
                    let mut cliques: Vec<VNodes> = Vec::with_capacity(2);
                    for gchild in
                        tree.graph.neighbors_directed(child, Direction::Outgoing)
                    {
                        let clique = tree.module_nodes(gchild, None);
                        assert!(self.set_is_clique(clique.iter()));
                        assert!(self.clique_is_simplicial(&clique));
                        cliques.push(clique);
                    }
                    assert_eq!(cliques.len(), 2);
                    // no early return ... cf. above
                    ret = self.map_simplicial_cliques(cliques);
                },
                _ => unreachable!(),
            }
        }

        ret
    }

    fn try_bipartition(&self) -> Option<(Vec<Node>, Vec<Node>)> {
        let (mut a, mut b) = (Vec::new(), Vec::new());
        let mut unvisited = self.iter_nodes().collect::<HashSet<_>>();
        let mut marked = HashMap::new(); // a: false, b: true
        let mut stack = Vec::new();

        // the outer loop is just for unconnected graphs, but it is actually not
        // necessary, because we will pass in connected graphs ...
        while let Some(&node) = unvisited.iter().next() {
            // println!("false: {:?}", node);
            a.push(node);
            marked.insert(node, false);
            assert!(unvisited.remove(&node));
            let neighbours = self.get_neighbours(node).unwrap();
            // println!("true: {:?}", neighbours);
            for neighbour in neighbours.iter() {
                stack.push((neighbour, true));
                marked.insert(neighbour, true);
                assert!(unvisited.remove(&neighbour));
            }

            while let Some((node, mark)) = stack.pop() {
                // println!("{mark}: {:?}", node);
                let neg_mark = !mark;
                let neighbours = self.get_neighbours(node).unwrap();
                // println!("{neg_mark}: {:?}", neighbours);
                for neighbour in neighbours.iter() {
                    if let Some(&neighbour_mark) = marked.get(&neighbour) {
                        if !(neighbour_mark ^ mark) {
                            return None;
                        }
                    } else {
                        stack.push((neighbour, neg_mark));
                        marked.insert(neighbour, neg_mark);
                        assert!(unvisited.remove(&neighbour));
                    }
                }
                if mark {
                    b.push(node);
                } else {
                    a.push(node);
                }
            }
        }

        Some((a, b))
    }

    fn obstinate_case(&self) -> Option<Vec<VNodes>> {
        match self.obstinate() {
            Obstinate::True(ObstinateKind::Itself, (a, b)) => {
                let len = self.len();
                if len == 2 {
                    Some(vec![vec![a[0]], vec![b[0]], vec![a[0], b[0]]])
                } else if len == 4 {
                    return Some(vec![
                        vec![a[0]],
                        vec![a[0], b[0]],
                        vec![a[1], b[0]],
                        vec![a[1], b[1]],
                        vec![b[1]],
                    ]);
                } else {
                    panic!(
                        "claw-free and obstinate (itself), but the length is not 2 or 4"
                    );
                }
            },
            Obstinate::True(ObstinateKind::Complement, (a, b)) => {
                debug_assert_eq!(a.len(), self.len() / 2);
                let mut ret = Vec::with_capacity(self.len());
                for i in 0..a.len() {
                    ret.push(a[i..].to_vec());
                    ret.push(b[..i + 1].to_vec());
                }
                Some(ret)
            },
            Obstinate::False => None,
        }
    }

    fn map_simplicial_cliques(
        &self,
        cliques: impl IntoIterator<Item = impl IntoIterator<Item = Node>>,
    ) -> Vec<VLabels> {
        cliques
            .into_iter()
            .map(|clique| {
                clique.into_iter().map(|v| self.get_label(v).unwrap()).collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{generic::adj::AdjGraph, test_utils::collect};

    #[test]
    fn test() {
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
        let mut graph: Graph<AdjGraph> =
            Graph::from_adjacency_labels(data.clone()).unwrap();
        let mut tree = graph.modular_decomposition();
        graph.twin_collapse(&mut tree);
        let cliques = graph.simplicial(&tree, None);
        println!("{:?}", cliques);

        // 0 -- 1 -- 2
        //  \
        //    - 3,4,5,6 clique
        let data = collect!(vv;
                (0, [1, 3]),
                (1, [0, 2]),
                (2, [1]),
                (3, [0, 4, 5, 6]),
                (4, [0, 3, 5, 6]),
                (5, [0, 3, 4, 6]),
                (6, [0, 3, 4, 5]),
        );
        let graph: Graph<AdjGraph> = Graph::from_adjacency_labels(data.clone()).unwrap();
        let tree = graph.modular_decomposition();
        let cliques = graph.simplicial(&tree, None);
        println!("{:?}", cliques);

        let data = collect!(vv;
            (5, [0, 1, 2, 3, 4]),
            (6, [0, 1, 2, 3, 4]),
            (0, [5, 6, 4, 1]),
            (1, [5, 6, 2, 0]),
            (2, [5, 6, 3, 1]),
            (3, [5, 6, 4, 2]),
            (4, [5, 6, 0, 3]),
        );
        let mut graph: Graph<AdjGraph> =
            Graph::from_adjacency_labels(data.clone()).unwrap();
        let mut tree = graph.modular_decomposition();
        println!("{:?}", tree);
        println!("{:?}", graph.simplicial(&tree, None));
        graph.twin_collapse(&mut tree);
        let cliques = graph.simplicial(&tree, None).unwrap().pop().unwrap();
        println!("{:?}", cliques.len());
        let cliques_set = cliques.into_iter().collect::<HashSet<_>>();
        println!("{:?}", cliques_set.len());
        println!("{:?}", cliques_set);
    }

    #[test]
    fn bipartition() {
        let data = collect!(v;
            (0, 1),
            (2, 3),
            (4, 5),
            (1, 4),
            (0, 2),
            // (2, 5),
        );
        let graph: Graph<AdjGraph> = Graph::from_edge_labels(data).unwrap();
        println!("{:?}", graph);
        println!("{:?}", graph.try_bipartition());
    }
}
