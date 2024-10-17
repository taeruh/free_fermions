use std::collections::{HashMap, HashSet};

use modular_decomposition::ModuleKind;
use petgraph::{Direction, graph::DiGraph};

use crate::graph::{Label, Node};

pub type NodeIndex = petgraph::graph::NodeIndex<u32>;

pub type TreeGraph = DiGraph<ModuleKind<Node>, ()>;

#[derive(Debug, Clone, Default)]
pub struct Tree {
    pub graph: TreeGraph,
    pub root: NodeIndex,
}

fn mapped_eq(
    left_module: &ModuleKind<Node>,
    right_module: &ModuleKind<Node>,
    left_map: impl FnOnce(Node) -> Label,
    right_map: impl FnOnce(Node) -> Label,
) -> bool {
    match (left_module, right_module) {
        (ModuleKind::Prime, ModuleKind::Prime) => true,
        (ModuleKind::Series, ModuleKind::Series) => true,
        (ModuleKind::Parallel, ModuleKind::Parallel) => true,
        (ModuleKind::Node(left_node), ModuleKind::Node(right_node)) => {
            left_map(*left_node) == right_map(*right_node)
        },
        _ => false,
    }
}

// TODO: test these methods ...
impl Tree {
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

    /// `stack_size_hint` is how deep the tree can go from the module (which we usually
    /// know when calling this function, since we are then usually in the claw-free case)
    //
    // instead of doing a recursion, we keep a manual stack; I think this is more
    // performant, because we usually know the stack size and in the unsafe version (todo)
    // we can even put the our stack onto the stack; therefore we have barely any heap
    // (re)alloction overhead and save the context switching overhead of recursion
    //
    // we are stacking the iterators here, i.e, it is a depth-first traversal; instead we
    // could do a breadth-first traversal, where we would put the nodes into the stack; it
    // is not clear which one is better; the depth-first traversal requires a smaller
    // stack, however, one could imagine that stopping an iterator and then continuing it
    // later - instead of going through it in one go - is not that cache-friedly, but then
    // when roughly looking at the implementation of our Neighbors iterator here, it is
    // probably not cache-friendly anyway
    // TODO: implement an unsafe version, where we use a stack on the stack
    pub fn module_nodes(
        &self,
        module: NodeIndex,
        stack_size_hint: Option<usize>,
    ) -> Vec<Node> {
        if let ModuleKind::Node(idx) = self.graph.node_weight(module).unwrap() {
            return vec![*idx];
        }

        let mut ret = Vec::new();

        let mut stack = if let Some(stack_size) = stack_size_hint {
            Vec::with_capacity(stack_size)
        } else {
            Vec::new()
        };
        stack.push(self.graph.neighbors_directed(module, Direction::Outgoing));

        'outer: while let Some(iter) = stack.last_mut() {
            for child in iter {
                match self.graph.node_weight(child).unwrap() {
                    ModuleKind::Node(idx) => ret.push(*idx),
                    _ => {
                        stack.push(
                            self.graph.neighbors_directed(child, Direction::Outgoing),
                        );
                        // checking if the logic about claw-free graphs is correct
                        #[cfg(debug_assertions)]
                        if let Some(stack_size) = stack_size_hint {
                            assert!(stack.len() <= stack_size);
                        }
                        continue 'outer;
                    },
                }
            }
            stack.pop();
        }

        ret
    }

    pub fn module_is_fully_prime(&self, module: NodeIndex) -> bool {
        if !matches!(self.graph.node_weight(module).unwrap(), ModuleKind::Prime) {
            return false;
        }
        for child in self.graph.neighbors_directed(self.root, Direction::Outgoing) {
            if !matches!(self.graph.node_weight(child).unwrap(), ModuleKind::Node(_)) {
                return false;
            }
        }
        true
    }

    pub fn module_is_clique(&self, module: NodeIndex) -> bool {
        match self.graph.node_weight(module).unwrap() {
            ModuleKind::Prime => return false,
            ModuleKind::Series => {
                for child in self.graph.neighbors_directed(module, Direction::Outgoing) {
                    match self.graph.node_weight(child).unwrap() {
                        ModuleKind::Node(_) => {},
                        _ => return false,
                    }
                }
            },
            ModuleKind::Parallel => return false,
            ModuleKind::Node(_) => {},
        }
        true
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
        left_map: impl FnOnce(Node) -> Label + Copy,
        right_map: impl FnOnce(Node) -> Label + Copy,
    ) -> bool {
        if left_tree.graph.node_count() != right_tree.graph.node_count() {
            return false;
        } else if left_tree.graph.node_count() == 1 {
            let get_node = |tree: &Self| {
                if let ModuleKind::Node(node) = tree.graph.node_weight(tree.root).unwrap()
                {
                    *node
                } else {
                    unreachable!("tree has only one node, so it must be a leaf node")
                }
            };
            return left_map(get_node(left_tree)) == right_map(get_node(right_tree));
        }
        if left_tree.graph.node_weight(left_tree.root)
            != right_tree.graph.node_weight(right_tree.root)
        {
            return false;
        }

        let mut left_leaf_mapped = left_tree.get_leaves_with_inverted_map(left_map);
        let mut right_leaf_mapped = right_tree.get_leaves_with_inverted_map(right_map);
        let true_leafs = left_leaf_mapped.keys().copied().collect::<Vec<_>>();
        // below, after the loop we check that both (left|right)_leaf_mapped are empty
        // after loop, which ensures they were the same

        for ref true_leaf in true_leafs {
            let left_leaf = left_leaf_mapped.remove(true_leaf).unwrap();
            let right_leaf = match right_leaf_mapped.remove(true_leaf) {
                Some(l) => l,
                // in that case, the labelled leaf nodes were already different
                None => return false,
            };
            if !Self::recurse_compare_parent_path(
                left_tree, right_tree, left_map, right_map, left_leaf, right_leaf,
            ) {
                return false;
            }
        }

        left_leaf_mapped.is_empty() && right_leaf_mapped.is_empty()
    }

    fn recurse_compare_parent_path(
        left_tree: &Self,
        right_tree: &Self,
        left_map: impl FnOnce(Node) -> Label + Copy,
        right_map: impl FnOnce(Node) -> Label + Copy,
        left_node: NodeIndex,
        right_node: NodeIndex,
    ) -> bool {
        let left_parent = left_tree.get_parent(left_node);
        let right_parent = right_tree.get_parent(right_node);

        let true_left_leaf_siblings: HashSet<_> =
            left_tree.get_leaf_children(left_map, left_parent).collect();
        let true_right_leaf_siblings: HashSet<_> =
            right_tree.get_leaf_children(right_map, right_parent).collect();

        if (true_left_leaf_siblings != true_right_leaf_siblings)
            || !mapped_eq(
                &left_tree.graph[left_parent],
                &right_tree.graph[right_parent],
                left_map,
                left_map,
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
            left_map,
            right_map,
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
        map: impl FnOnce(Node) -> Label + Copy + 'a,
        node: NodeIndex,
    ) -> impl Iterator<Item = Label> + 'a {
        self.graph.neighbors_directed(node, Direction::Outgoing).filter_map(
            move |child| {
                if let ModuleKind::Node(weight) = self.graph[child] {
                    Some(map(weight))
                } else {
                    None
                }
            },
        )
    }

    fn get_leaves_with_inverted_map(
        &self,
        map: impl FnOnce(Node) -> Label + Copy,
    ) -> HashMap<Label, NodeIndex> {
        self.graph
            .node_indices()
            .filter_map(|node| {
                if let ModuleKind::Node(weight) = self.graph.node_weight(node).unwrap() {
                    Some((map(*weight), node))
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
pub mod tests {
    use rand::{Rng, SeedableRng, seq::SliceRandom};
    use rand_pcg::Pcg64;

    use super::*;
    use crate::{
        fix_int::int,
        graph::{
            generic::{self, Adj, ImplGraph, Pet},
            specialised::{self, IndexMap},
            test_utils::RandomMap,
        },
    };

    fn random_edges(
        rng: &mut impl Rng,
        num_nodes: int,
        num_edges: int,
    ) -> Vec<(Label, Label)> {
        assert!(num_nodes > 1); // otherwise, the loop below will never terminate
        let map = RandomMap::with_rng(num_nodes, num_nodes * 2, rng);
        let dist = rand::distributions::Uniform::new(0, num_nodes);
        let mut edges = Vec::with_capacity(num_edges as usize);
        for _ in 0..num_edges {
            loop {
                let (a, b) = (rng.sample(dist), rng.sample(dist));
                if a != b {
                    edges.push((map.map(a), map.map(b)));
                    break;
                }
            }
        }
        edges
    }

    #[test]
    fn positive_equivalences() {
        let rng = &mut Pcg64::from_entropy();
        // modular decomposition fails if there are no nodes or no edges
        let num_nodes = rng.gen_range(2..50);
        let num_edges = rng.gen_range(1..100);
        let mut edges = random_edges(rng, num_nodes, num_edges);

        let gen_adj = generic::Graph::<Adj>::from_edge_labels(edges.clone()).unwrap();
        let tree_gen_adj = gen_adj.modular_decomposition();
        edges.shuffle(rng);
        let gen_pet = generic::Graph::<Pet>::from_edge_labels(edges.clone()).unwrap();
        let tree_gen_pet = gen_pet.modular_decomposition();
        edges.shuffle(rng);
        let spec_index =
            specialised::Graph::<IndexMap>::from_edge_labels(edges.clone()).unwrap();
        let tree_spec_index = spec_index.modular_decomposition();
        edges.shuffle(rng);
        let spec_cus = specialised::Graph::<IndexMap>::from_edge_labels(edges).unwrap();
        let tree_spec_cus = spec_cus.modular_decomposition();

        assert!(Tree::is_equivalent(
            &tree_gen_adj,
            &tree_gen_pet,
            gen_adj.get_label_mapping(),
            gen_pet.get_label_mapping()
        ));
        assert!(Tree::is_equivalent(
            &tree_gen_pet,
            &tree_spec_index,
            gen_pet.get_label_mapping(),
            spec_index.get_label_mapping()
        ));
        assert!(Tree::is_equivalent(
            &tree_spec_index,
            &tree_spec_cus,
            spec_index.get_label_mapping(),
            spec_cus.get_label_mapping()
        ));

        // // the following does not make sense, since modules can be collapsed onto
        // // different representatives; however, I it succeeds quite often (and it did
        // // indeed collapse some stuff in these cases), and also in the cases where it
        // // failed, the smaller graphs (checked manually) where same up to labelling, which
        // // is a good positive indication:
        // // while we are at it, let's compare them after twin_collapsing
        // gen_adj.twin_collapse(&mut tree_gen_adj);
        // gen_pet.twin_collapse(&mut tree_gen_pet);
        // unsafe { spec_index.twin_collapse(&mut tree_spec_index) };
        // unsafe { spec_cus.twin_collapse(&mut tree_spec_cus) };
        // assert!(Tree::is_equivalent(
        //     &tree_gen_adj,
        //     &tree_gen_pet,
        //     gen_adj.get_label_mapping(),
        //     gen_pet.get_label_mapping()
        // ));
        // assert!(Tree::is_equivalent(
        //     &tree_gen_pet,
        //     &tree_spec_index,
        //     gen_pet.get_label_mapping(),
        //     spec_index.get_label_mapping()
        // ));
        // assert!(Tree::is_equivalent(
        //     &tree_spec_index,
        //     &tree_spec_cus,
        //     |n| spec_index.get_label(n).unwrap(),
        //     |n| spec_cus.get_label(n).unwrap()
        // ));
    }

    #[test]
    fn negative_equivalences() {
        let rng = &mut Pcg64::from_entropy();
        let num_nodes_a = rng.gen_range(2..50);
        let num_nodes_b = rng.gen_range(2..50);
        let num_edges_a = rng.gen_range(1..100);
        let num_edges_b = rng.gen_range(1..100);
        let (edges_a, edges_b) = loop {
            let edges_a = random_edges(rng, num_nodes_a, num_edges_a);
            let edges_b = random_edges(rng, num_nodes_b, num_edges_b);
            if HashSet::<_>::from_iter(edges_a.iter().cloned())
                != HashSet::<_>::from_iter(edges_b.iter().cloned())
            {
                break (edges_a, edges_b);
            }
        };
        let graph_a = specialised::Graph::<IndexMap>::from_edge_labels(edges_a).unwrap();
        let tree_a = graph_a.modular_decomposition();
        let graph_b = specialised::Graph::<IndexMap>::from_edge_labels(edges_b).unwrap();
        let tree_b = graph_b.modular_decomposition();
        assert!(!Tree::is_equivalent(
            &tree_a,
            &tree_b,
            |n| graph_a.get_label(n).unwrap(),
            |n| graph_b.get_label(n).unwrap()
        ));
    }
}
