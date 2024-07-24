use std::collections::{HashMap, HashSet};

use modular_decomposition::ModuleKind;
use petgraph::{graph::DiGraph, Direction};

use crate::graph::{
    generic::{Graph, ImplGraph},
    Label, Node,
};

pub type NodeIndex = petgraph::graph::NodeIndex<u32>;

pub type TreeGraph = DiGraph<ModuleKind<Node>, ()>;

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub graph: TreeGraph,
    pub root: NodeIndex,
}

impl<G: ImplGraph> Graph<G> {
    pub fn modular_decomposition(&self) -> Tree {
        let md_tree = modular_decomposition::modular_decomposition(&self).unwrap();
        Tree {
            root: NodeIndex::from(md_tree.root().index() as u32),
            graph: md_tree.into_digraph(),
        }
    }
}

// We often (mainly when testing) have the same graph and the same decomposition, in two
// different instances. The instances might not be the same under struct data structure
// equality since we do not use stable graphs (since we want them to be compact; unstable
// in the sense that our graphs are compact which requires differentiation between node
// indices and labels). However, the instances are equivalent in the sense that they
// describe the same graph and tree. We want to check for this equivalence. For graphs, it
// is simple with our `map_to_labels` method. For trees, it is a bit more complicated.
//
// How the equivalence is checked:
//
// Pre-note:

// Naively, one would start with the root node and then recursively check that
// the children are correct. However, this does not work just like that, because the same
// child node might have a different index; so one would need to consider all possible
// permutations of the children. This is not efficient.
// Instead, we start with the labels of the leaf nodes and then go up to the root node and
// do some checks. The checks trivially succeed if the trees are equivalent, and the
// algorithm is more about having enough checks to ensure that False is returned if the
// trees are not equivalent.
//
// Algorithm:

// First, we check whether the root node's weight is the same. Then, for all leaf nodes,
// we check that the direct path from the leaf to the root node is the same (i.e.,
// checking the weights). While doing this, we check that for each node in this path,
// except the initial leaf node, that the set of children which are leafs are the same
// (call this the leaf-sibling check). Finally, check that we covered all leaf nodes in
// both graphs after performing this loop.
//
// Proof of correctness:
//
// First, let's assume that we have to trees which are not equivalent. Then, there must be
// (at least) one node in the trees that differ or one tree has a node that the other tree
// does not have. Let's assume that the root node weight is not the same. Then, we
// directly return False, which is correct. Let's assume that is is not the case.
// W.l.o.g., we can assume that no leaf node is missing in one of the trees (because we
// also check that we cover all leaf nodes). Let's pick a node whose subtree differs at
// the highest possible level in the tree (i.e, closest to the root node); this node is
// then a child of the root node. If the node is a leaf (i.e., the sets of leaf children
// of the root node differs), then we correctly return False (take the parent path from
// this leaf node to the root and then when going to the first parent (i.e, the root
// node), the leaf-sibling check will fail). So let us assume it is not a leaf. Then, the
// corresponding module in the graph has more than one nodes, i.e., the tree node has more
// than one leaf. For all these leaves, we are doing the parent path check; if one fails,
// we correctly return False. Assume, the first of those parent path checks succeeds. Then
// we know that the node's weight is the same in both trees. But now we are in the same
// situation as before with respect to the subtree of this node (i.e., same root node
// weight), so we can repeat this argument until either one of the parent path checks
// fails - either because of different parent weights or failing leaf-sibling check - or
// we reach a node that has to be different but has only leaf nodes, but then the parent
// path check will also fail - because of the leaf-sibling check.
//
// Now let's assume that we have two trees which are equivalent. It is clear that we then
// correctly return True.
impl Tree {
    // this impl block is only for the is_equivalent method and its helper methods; make a
    // new impl block for other stuff (don't want to put the helper methods into the
    // is_equivalent method, so that the indentation does not get too deep; and to be able
    // to use self in these methods)

    // TODO: clean this stuff up somehow
    pub fn is_equivalent(
        left_tree: &Self,
        right_tree: &Self,
        left_graph: &impl ImplGraph,
        right_graph: &impl ImplGraph,
    ) -> bool {
        if left_tree.graph.node_weight(left_tree.root)
            != right_tree.graph.node_weight(right_tree.root)
        {
            return false;
        }

        let mut left_leaf_mapped = left_tree.get_leaves_with_inverted_map(left_graph);
        let mut right_leaf_mapped = right_tree.get_leaves_with_inverted_map(right_graph);
        let true_leafs = left_leaf_mapped.keys().copied().collect::<Vec<_>>();
        // below, after the loop we check that both (left|right)_leaf_mapped are empty
        // after loop, which ensures they were the same

        for ref true_leaf in true_leafs {
            let left_leaf = left_leaf_mapped.remove(true_leaf).unwrap();
            let right_leaf = right_leaf_mapped.remove(true_leaf).unwrap();
            if !Self::recurse_compare_parent_path(
                left_tree,
                right_tree,
                left_graph,
                right_graph,
                left_leaf,
                right_leaf,
            ) {
                return false;
            }
        }

        left_leaf_mapped.is_empty() && right_leaf_mapped.is_empty()
    }

    fn recurse_compare_parent_path(
        left_tree: &Self,
        right_tree: &Self,
        left_graph: &impl ImplGraph,
        right_graph: &impl ImplGraph,
        left_node: NodeIndex,
        right_node: NodeIndex,
    ) -> bool {
        let left_parent = left_tree.get_parent(left_node);
        let right_parent = right_tree.get_parent(right_node);

        let true_left_leaf_siblings: HashSet<_> =
            left_tree.get_leaf_children(left_graph, left_parent).collect();
        let true_right_leaf_siblings: HashSet<_> =
            right_tree.get_leaf_children(right_graph, right_parent).collect();

        if (true_left_leaf_siblings != true_right_leaf_siblings)
            || !compare_module_kind_mapped(
                &left_tree.graph[left_parent],
                &right_tree.graph[right_parent],
                left_graph,
                right_graph,
            )
        {
            return false;
        // we reached the root node (and since we checked at the start of the algorithm
        // the root node weights are the same, we can return true)
        } else if left_parent == left_tree.root && right_parent == right_tree.root {
            return true;
        }

        // don't put this into the conditional above (instead early return on the other
        // conditions there), to make it easier for the compiler to make tail
        // optimisations
        Self::recurse_compare_parent_path(
            left_tree,
            right_tree,
            left_graph,
            right_graph,
            left_parent,
            right_parent,
        )
    }

    fn get_parent(&self, node: NodeIndex) -> NodeIndex {
        self.graph
            .neighbors_directed(node, Direction::Incoming)
            .next()
            .unwrap()
    }

    fn get_leaf_children<'a>(
        &'a self,
        graph: &'a impl ImplGraph,
        node: NodeIndex,
    ) -> impl Iterator<Item = Label> + 'a {
        self.graph
            .neighbors_directed(node, Direction::Outgoing)
            .filter_map(|child| {
                if let ModuleKind::Node(weight) = self.graph[child] {
                    Some(graph.get_label(weight).unwrap())
                } else {
                    None
                }
            })
    }

    fn get_leaves_with_inverted_map(
        &self,
        graph: &impl ImplGraph,
    ) -> HashMap<Label, NodeIndex> {
        self.graph
            .node_indices()
            .filter_map(|node| {
                if let ModuleKind::Node(weight) = self.graph.node_weight(node).unwrap() {
                    Some((graph.get_label(*weight).unwrap(), node))
                } else {
                    None
                }
            })
            .collect()
    }
}

pub fn compare_module_kind(
    left_module: &ModuleKind<Node>,
    right_module: &ModuleKind<Node>,
) -> bool {
    match (left_module, right_module) {
        (ModuleKind::Prime, ModuleKind::Prime) => true,
        (ModuleKind::Series, ModuleKind::Series) => true,
        (ModuleKind::Parallel, ModuleKind::Parallel) => true,
        (ModuleKind::Node(left_node), ModuleKind::Node(right_node)) => {
            *left_node == *right_node
        },
        _ => false,
    }
}

pub fn compare_module_kind_mapped(
    left_module: &ModuleKind<Node>,
    right_module: &ModuleKind<Node>,
    left_graph: &impl ImplGraph,
    right_graph: &impl ImplGraph,
) -> bool {
    match (left_module, right_module) {
        (ModuleKind::Prime, ModuleKind::Prime) => true,
        (ModuleKind::Series, ModuleKind::Series) => true,
        (ModuleKind::Parallel, ModuleKind::Parallel) => true,
        (ModuleKind::Node(left_node), ModuleKind::Node(right_node)) => {
            left_graph.get_label(*left_node) == right_graph.get_label(*right_node)
        },
        _ => false,
    }
}

impl Tree {
    pub fn reduced_module(&self, module: NodeIndex) -> Vec<Node> {
        if let ModuleKind::Node(idx) = self.graph.node_weight(module).unwrap() {
            return vec![*idx];
        }

        let mut ret = Vec::new();
        for child in self.graph.neighbors_directed(module, Direction::Outgoing) {
            ret.push(self.module_representative(child));
        }
        ret
    }

    pub fn module_representative(&self, mut module: NodeIndex) -> Node {
        loop {
            module = if let Some(m) =
                self.graph.neighbors_directed(module, Direction::Outgoing).next()
            {
                m
            } else {
                break; // child is a leaf
            };
        }

        if let ModuleKind::Node(idx) = self.graph.node_weight(module).unwrap() {
            *idx
        } else {
            unreachable!()
        }
    }

    pub fn module_nodes(&self, module: NodeIndex) -> Vec<Node> {
        if let ModuleKind::Node(idx) = self.graph.node_weight(module).unwrap() {
            return vec![*idx];
        }

        let mut ret = Vec::new();

        // PERF: tail recursion ...
        fn recurse(tree: &TreeGraph, module: NodeIndex, ret: &mut Vec<Node>) {
            for child in tree.neighbors_directed(module, Direction::Outgoing) {
                match tree.node_weight(child).unwrap() {
                    ModuleKind::Node(idx) => ret.push(*idx),
                    _ => recurse(tree, child, ret),
                }
            }
        }

        recurse(&self.graph, module, &mut ret);
        ret
    }

    pub fn graph_is_really_prime(&self) -> bool {
        if !matches!(self.graph.node_weight(self.root).unwrap(), ModuleKind::Prime) {
            return false;
        }
        for child in self.graph.neighbors_directed(self.root, Direction::Outgoing) {
            if !matches!(self.graph.node_weight(child).unwrap(), ModuleKind::Node(_)) {
                return false;
            }
        }
        true
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
    };

    #[test]
    fn equivalences() {
        let rng = &mut Pcg64::from_entropy();

        let map = RandomMap::new(1000, 2000, rng);
        let mut edges = collect!(v, map;
            (0, 1), (1, 2), (2, 3), (3, 4), (3, 5), (3, 6), (3, 7), (4, 5), (6, 7),);

        edges.shuffle(rng);
        let graph1 = Graph::<AdjGraph>::from_edge_labels(edges.clone()).unwrap();
        let tree1 = graph1.modular_decomposition();
        edges.shuffle(rng);
        let graph2 = Graph::<AdjGraph>::from_edge_labels(edges).unwrap();
        let tree2 = graph2.modular_decomposition();
        assert!(Tree::is_equivalent(&tree1, &tree2, &graph1, &graph2));

        // nearly same as above, but connecting 5 and 6
        let other_edges = collect!(v, map;
            (0, 1), (1, 2), (2, 3), (3, 4), (3, 5), (3, 6), (3, 7), (4, 5), (5, 6), (6, 7),
        );
        let graph3 = Graph::<AdjGraph>::from_edge_labels(other_edges).unwrap();
        let tree3 = graph3.modular_decomposition();
        assert!(!Tree::is_equivalent(&tree1, &tree3, &graph1, &graph3));

        let graph = graph1;
        let tree = tree1;
        let reprs = tree.reduced_module(tree.root);
        let repr_graph = graph.subgraph(&reprs);
        println!("{:?}", graph);
        println!("{:?}", repr_graph);
    }
}
