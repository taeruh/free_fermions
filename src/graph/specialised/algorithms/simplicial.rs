use hashbrown::{hash_set::Entry, HashMap, HashSet};
use modular_decomposition::ModuleKind;
use petgraph::Direction;

use crate::graph::{
    algorithms::{
        modular_decomposition::{NodeIndex, Tree},
        obstinate::{Obstinate, ObstinateKind},
    },
    specialised::{Graph, GraphData},
    Node, VNodes,
};

enum NonTrivialChild {
    Prime(NodeIndex),
    Parallel(NodeIndex),
}

impl<G: GraphData> Graph<G> {
    /// # Safety
    /// The graph `self` must be claw-free and connected. Furthermore the `tree` must be
    /// the graph's modular decomposition tree and we must have run [Self::twin_collapse].
    pub unsafe fn simplicial(&self, tree: &Tree) -> HashSet<VNodes> {
        match tree.graph.node_weight(tree.root).unwrap() {
            ModuleKind::Prime => self.prime_simplicial(tree),
            ModuleKind::Series => self.series_simplicial(tree),
            ModuleKind::Parallel => unsafe {
                // safety: invariant promises that the graph is connected
                debug_unreachable_unchecked!("graph is connected");
            },
            ModuleKind::Node(n) => HashSet::from_iter([vec![*n]]),
        }
    }

    fn prime_simplicial(&self, tree: &Tree) -> HashSet<VNodes> {
        // no need to get a representative graph, because we collapsed everything
        debug_assert!(tree.module_is_fully_prime(tree.root));
        // safety: we will only pass in labels from a subgraph (which has a subset of the
        // labels)
        self.prime_recurse(self)
    }

    #[inline]
    fn to_parent_map(&self, parent: &Self, mut clique: VNodes) -> VNodes {
        clique.iter_mut().for_each(|n| {
            *n = unsafe {
                // safety: the nodes come from the graph and then the labels are a subset
                // of the parent's labels
                parent.get_index_unchecked(self.get_label_unchecked(*n))
            }
        });
        clique.sort_unstable();
        clique
    }

    #[inline]
    fn clone_to_parent_map(&self, parent: &Self, clique: &[Node]) -> VNodes {
        let mut clique = Vec::from_iter((*clique).iter().map(|n| {
            unsafe {
                // safety: cf. `to_parent_map`
                parent.get_index_unchecked(self.get_label_unchecked(*n))
            }
        }));
        clique.sort_unstable();
        clique
    }

    // PERF: we could/should seperate the first call of the recursion and early return
    // from it as soon as we find one simplicial clique
    fn prime_recurse(&self, parent: &Self) -> HashSet<VNodes> {
        match self.obstinate() {
            Obstinate::True(ObstinateKind::Itself, (a, b)) => {
                let len = self.len();
                if len == 2 {
                    return HashSet::from_iter(
                        [vec![a[0]], vec![b[0]], vec![a[0], b[0]]]
                            .into_iter()
                            .map(|c| self.to_parent_map(parent, c)),
                    );
                } else if len == 4 {
                    return HashSet::from_iter(
                        [
                            vec![a[0]],
                            vec![a[0], b[0]],
                            vec![a[1], b[0]],
                            vec![a[1], b[1]],
                            vec![b[1]],
                        ]
                        .into_iter()
                        .map(|c| self.to_parent_map(parent, c)),
                    );
                } else {
                    unsafe {
                        // safety: assume the claw_free and obstinate algorithms are
                        // correct
                        debug_unreachable_unchecked!(
                            "claw-free and obstinate (itself), but the length is not 2 \
                             or 4"
                        );
                    }
                }
            },
            Obstinate::True(ObstinateKind::Complement, (a, b)) => {
                debug_assert_eq!(a.len(), self.len() / 2);
                let mut cliques = Vec::with_capacity(self.len());
                for i in 0..a.len() {
                    cliques.push(a[i..].to_vec());
                    cliques.push(b[..i + 1].to_vec());
                }
                return HashSet::from_iter(
                    cliques.into_iter().map(|c| self.to_parent_map(parent, c)),
                );
            },
            Obstinate::False => {},
        }

        let mut cliques: HashSet<VNodes> = HashSet::new();

        // we keep track of the cliques that we have already checked (duplicates happen;
        // see note in generic version); while this require 1 clone + 2 potential clones
        // (instead of 1 potential clone), we do not do any duplicate checks which are
        // something like O(n^3)
        let mut checked_cliques: HashSet<VNodes> = HashSet::new();

        for node in self.iter_nodes() {
            // In the paper "Growing without Cloning" it is not clear whether this step
            // should be done before are after checking whether the subgraph is (fully)
            // prime (rather after). However, this is wrong, as the example test
            // `simplicial_vertex_but_subgraph_not_prime` shows.
            if self.clique_is_simplicial(&[node]) {
                cliques.insert(vec![unsafe {
                    // safety: cf. `to_parent_map`
                    parent.get_index_unchecked(self.get_label_unchecked(node))
                }]);
            }

            let mut graph = self.clone();
            // safety: we are in bounds because node comes from the graph itself
            unsafe { graph.remove_node_unchecked(node) };
            let tree = graph.modular_decomposition();
            if !tree.module_is_fully_prime(tree.root) {
                continue;
            }

            let subcliques = graph.prime_recurse(self);
            for mut clique in subcliques.into_iter() {
                checked_cliques.get_or_insert_with(clique.as_slice(), |clique| {
                    if self.clique_is_simplicial(clique) {
                        cliques.insert(self.clone_to_parent_map(parent, clique));
                    }
                    clique.to_vec()
                });
                clique.push(node);
                clique.sort_unstable();
                match checked_cliques.entry(clique) {
                    Entry::Occupied(_) => {},
                    Entry::Vacant(e) => {
                        let clique = e.get();
                        if self.set_is_clique(clique.iter())
                            && self.clique_is_simplicial(clique)
                        {
                            cliques.insert(self.clone_to_parent_map(parent, clique));
                        }
                        e.insert();
                    },
                }
            }
        }

        cliques
    }

    fn series_simplicial(&self, tree: &Tree) -> HashSet<VNodes> {
        let mut complement = self.clone();
        complement.complement();
        if let Some(bipartition) = complement.try_bipartion() {
            return bipartition;
        }

        let mut count = 0;
        let mut non_trivial_child = None;

        for child in tree.graph.neighbors_directed(tree.root, Direction::Outgoing) {
            match tree.graph.node_weight(child).unwrap() {
                ModuleKind::Prime => {
                    count += 1;
                    // more than one non-node child but no bipartition found
                    if count == 2 {
                        return HashSet::new();
                    } else {
                        non_trivial_child = Some(NonTrivialChild::Prime(child));
                    }
                },
                ModuleKind::Series => unsafe {
                    // safety: assume modular decomposition is correct
                    debug_unreachable_unchecked!("series module has series children");
                },
                ModuleKind::Parallel => {
                    count += 1;
                    if count == 2 {
                        return HashSet::new();
                    } else {
                        non_trivial_child = Some(NonTrivialChild::Parallel(child));
                    }
                },
                ModuleKind::Node(_) => continue,
            }
        }

        if let Some(child) = non_trivial_child {
            match child {
                NonTrivialChild::Prime(child) => {
                    let module_nodes = tree.module_nodes(child, Some(2));
                    let graph =
                        // safety: nodes come from the graph itself
                        unsafe { self.subgraph(module_nodes.len(), module_nodes) };
                    // PERF: maybe we can construct the "subtree" frome `tree`
                    let tree = graph.modular_decomposition();
                    graph.prime_simplicial(&tree)
                },
                NonTrivialChild::Parallel(child) => {
                    #[cfg(debug_assertions)]
                    #[allow(clippy::needless_return)]
                    {
                        let mut cliques: HashSet<VNodes> = HashSet::with_capacity(2);
                        for gchild in
                            tree.graph.neighbors_directed(child, Direction::Outgoing)
                        {
                            let clique = tree.module_nodes(gchild, Some(1));
                            assert!(self.set_is_clique(clique.iter()));
                            assert!(self.clique_is_simplicial(&clique));
                            cliques.insert(clique);
                        }
                        assert_eq!(cliques.len(), 2);
                        return cliques;
                    }
                    #[cfg(any(not(debug_assertions), lsp_rust_analyzer))]
                    #[cfg_attr(debug_assertions, allow(unreachable_code))]
                    {
                        tree.graph
                            .neighbors_directed(child, Direction::Outgoing)
                            .map(|gchild| tree.module_nodes(gchild, Some(1)))
                            .collect()
                    }
                },
            }
        } else {
            HashSet::new()
        }
    }

    fn clique_is_simplicial(&self, clique: &[Node]) -> bool {
        for node in clique {
            // safety: we only ever pass in nodes from the graph itself
            let mut neighbours = unsafe { self.get_neighbours_unchecked(*node) }.clone();
            for n in clique {
                neighbours.remove(n);
            }
            if !self.set_is_clique(neighbours.iter()) {
                return false;
            }
        }
        true
    }

    fn set_is_clique<'s, I: Iterator<Item = &'s Node> + Clone>(
        &'s self,
        mut set: I,
    ) -> bool {
        while let Some(node) = set.next() {
            // safety: we only ever pass in nodes from the graph itself
            let neighbours = unsafe { self.get_neighbours_unchecked(*node) };
            for other in set.clone() {
                if !neighbours.contains(other) {
                    return false;
                }
            }
        }
        true
    }

    fn try_bipartion(&self) -> Option<HashSet<VNodes>> {
        let (mut fcolor, mut tcolor) = (Vec::new(), Vec::new());
        let mut unvisited = self.iter_nodes().collect::<HashSet<_>>();
        let mut marked = HashMap::new(); // fcolor: false, tcolor: true
        let mut stack = Vec::new();

        // outer loop, because we pass in non-connected graphs (complements of series
        // graphs)
        while let Some(&node) = unvisited.iter().next() {
            fcolor.push(node);
            marked.insert(node, false);
            #[cfg(debug_assertions)]
            debug_assert!(unvisited.remove(&node));
            #[cfg(any(not(debug_assertions), lsp_rust_analyzer))]
            unvisited.remove(&node);
            // safety: unvisited->node was created from the graph itself
            for neighbour in unsafe { self.get_neighbours_unchecked(node) } {
                stack.push((node, true));
                marked.insert(*neighbour, true);
                #[cfg(debug_assertions)]
                assert!(unvisited.remove(neighbour));
                #[cfg(any(not(debug_assertions), lsp_rust_analyzer))]
                unvisited.remove(neighbour);
            }

            while let Some((node, mark)) = stack.pop() {
                // safety: node comes from unvisited originally, which was created from he
                // graph itself
                for neighbour in unsafe { self.get_neighbours_unchecked(node) } {
                    if let Some(&neighbour_mark) = marked.get(neighbour) {
                        if neighbour_mark == mark {
                            return None;
                        }
                    } else {
                        let neg_mark = !mark;
                        stack.push((*neighbour, neg_mark));
                        marked.insert(*neighbour, neg_mark);
                        #[cfg(debug_assertions)]
                        assert!(unvisited.remove(neighbour));
                        #[cfg(any(not(debug_assertions), lsp_rust_analyzer))]
                        unvisited.remove(neighbour);
                    }
                }
                if mark {
                    tcolor.push(node);
                } else {
                    fcolor.push(node);
                }
            }
        }

        Some(HashSet::from_iter([tcolor, fcolor]))
    }
}

#[cfg(test)]
mod tests {
    use modular_decomposition::ModuleKind;
    use petgraph::Direction::Outgoing;

    use crate::graph::{
        specialised::{data::Custom, Graph},
        test_utils::collect,
        Label,
    };

    #[test]
    fn simplicial_vertex_but_subgraph_not_prime() {
        const VERTEX: Label = 2; // the simplicial vertex we remove
        const MODULE: [Label; 2] = [3, 4]; // the module we then get
        //         ------
        //        /      \
        // 0 -- 1 -- 2 -- 3 -- 5
        //       \     \     /
        //         ------ 4 -
        let mut graph = Graph::<Custom>::from_edge_labels(collect!(v;
            (0, 1),
            (1, 2),
            (1, 3),
            (1, 4),
            (2, 4),
            (3, 5),
            (4, 5),
        ))
        .unwrap();
        let tree = graph.modular_decomposition();
        assert!(tree.module_is_fully_prime(tree.root));
        let node = graph.get_index(VERTEX).unwrap();
        assert!(graph.set_is_clique([node].iter()));
        assert!(graph.clique_is_simplicial(&[node]));
        graph.remove_node(node);
        let tree = graph.modular_decomposition();
        assert!(!tree.module_is_fully_prime(tree.root));
        let module_node = tree
            .graph
            .neighbors_directed(tree.root, Outgoing)
            .find(|&node| {
                matches!(tree.graph.node_weight(node).unwrap(), ModuleKind::Parallel)
            })
            .unwrap();
        let mut module: Vec<_> = tree
            .module_nodes(module_node, None)
            .into_iter()
            .map(|n| graph.get_label(n).unwrap())
            .collect();
        module.sort_unstable();
        assert_eq!(module, MODULE);
    }
}
