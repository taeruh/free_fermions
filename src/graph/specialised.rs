use std::{
    fmt::{self, Debug},
    mem,
};

use hashbrown::{HashMap, HashSet};

use super::InvalidGraph;
use crate::fix_int::int;

pub type Node = usize;
pub type Label = usize;
pub type Edge = (usize, usize);
pub type Neighbours = HashSet<Node>;

#[derive(Clone)]
pub struct GraphNode<'a> {
    index: Node,
    label: Label,
    neighbours: &'a Neighbours,
}

impl Debug for GraphNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("index", &self.index)
            .field("label", &self.label)
            .field("neighbours", &self.neighbours)
            .finish()
    }
}

/// Must not contain self-loops (can be checked with `check`).
///
/// # Safety
/// The non-unsafe methods are generally safe, however, the some of the unsafe methods
/// rely on the fact that the graph does not contain self-loops (but this is also stated
/// in the according methods again).
#[derive(Clone, Default)]
pub struct Graph<T>(T);

// // TODO: remove these derefs, and instead provide inline methods for Graph(T) for all
// // the GraphData methods
// impl<T> std::ops::Deref for Graph<T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl<T> std::ops::DerefMut for Graph<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

pub trait GraphData: GraphDataSpecializerHelper + Debug + Clone + Default {
    /// # Safety
    /// The label must be valid.
    unsafe fn get_index_unchecked(&self, label: Label) -> Node;

    /// # Safety
    /// The node/index must be valid.
    unsafe fn get_label_unchecked(&self, node: Node) -> Label;

    /// # Safety
    /// The node/index must be valid.
    unsafe fn get_neighbours_unchecked(&self, node: Node) -> &Neighbours;

    /// # Safety
    /// The node/index must be valid.
    unsafe fn get_neighbours_mut_unchecked(&mut self, node: Node) -> &mut Neighbours;

    fn get_index(&self, label: Label) -> Option<Node>;

    fn get_label(&self, node: Node) -> Option<Label>;

    fn get_neighbours(&self, node: Node) -> Option<&Neighbours>;

    fn get_neighbours_mut(&mut self, node: Node) -> Option<&mut Neighbours>;

    fn get_index_or_insert(&mut self, label: Label) -> Node;

    fn get_neighbours_mut_or_insert(&mut self, label: Label) -> &mut Neighbours;

    fn get_index_and_neighbours_mut_or_insert(
        &mut self,
        label: Label,
    ) -> (Node, &mut HashSet<Node>);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn pop(&mut self) -> Option<Neighbours>;

    /// # Safety
    /// The `node` has to be valid.
    unsafe fn swap_remove_unchecked(&mut self, node: Node) -> Neighbours;

    fn swap_remove(&mut self, node: Node) -> Option<Neighbours>;

    fn iter_neighbours(&self) -> impl Iterator<Item = &Neighbours>;

    fn enumerate_neighbours(&self) -> impl Iterator<Item = (Node, &Neighbours)>;

    fn enumerate_full(&self) -> impl Iterator<Item = (Node, Label, &Neighbours)>;
}

/// Rather specific stuff that is not really needed for the general use case. We basically
/// use it to specialise parts of the Graph<G> methods.
pub trait GraphDataSpecializerHelper {
    /// Remove `node` in the neighbours of all its neighbours. Then swap_remove `node`.
    ///
    /// # Safety
    /// The `node` has to be valid. Furthermore all its neighbours have to be valid and do
    /// not contain `node`.
    unsafe fn raw_node_swap_remove(&mut self, node: Node);

    /// Update the neighbours of all neighbours of `node` to replace `before` with `node`.
    ///
    /// # Safety
    /// The `node` has to be valid. Furthermore all its neighbours have to be valid and do
    /// not contain `node`.
    unsafe fn raw_node_neighbours_update(&mut self, node: Node, before: &Node);
}

impl<G: GraphData> Graph<G> {
    #[inline]
    pub fn get_index(&self, label: Label) -> Option<Label> {
        self.0.get_index(label)
    }
    #[inline]
    pub fn enumerate_neighbours(&self) -> impl Iterator<Item = (Label, &Neighbours)> {
        self.0.enumerate_neighbours()
    }
    #[inline]
    pub fn get_label(&self, node: Node) -> Option<Label> {
        self.0.get_label(node)
    }

    pub fn add_labelled_edge(&mut self, (a, b): Edge) {
        let a = self.0.get_index_or_insert(a);
        let b = self.0.get_index_or_insert(b);
        unsafe { self.0.get_neighbours_mut_unchecked(a) }.insert(b);
        unsafe { self.0.get_neighbours_mut_unchecked(b) }.insert(a);
    }

    pub fn add_labelled_node_symmetrically(
        &mut self,
        (label, labelled_neighbours): (Label, impl IntoIterator<Item = Label>),
    ) {
        let idx = self.0.get_index_or_insert(label);
        let neighbours = labelled_neighbours
            .into_iter()
            .map(|neighbour| {
                let (neighbour_idx, neighbour_neighbours) =
                    self.0.get_index_and_neighbours_mut_or_insert(neighbour);
                neighbour_neighbours.insert(idx);
                neighbour_idx
            })
            .collect();
        // neighbours.insert(neighbour_idx);
        let _ = mem::replace(
            // safety: we only potentially inserted new nodes, which does not change the
            // index of `neighbours` (and especially does not remove the node)
            unsafe { self.0.get_neighbours_mut_unchecked(idx) },
            neighbours,
        );
    }

    pub fn add_labelled_node(
        &mut self,
        (label, labelled_neighbours): (Label, impl IntoIterator<Item = Label>),
    ) {
        let neighbours = labelled_neighbours
            .into_iter()
            .map(|n| self.0.get_index_or_insert(n))
            .collect();
        let to_replace_neighbours = self.0.get_neighbours_mut_or_insert(label);
        let _ = mem::replace(to_replace_neighbours, neighbours);
    }

    pub fn from_edge_labels(edges: impl IntoIterator<Item = Edge>) -> Self {
        let mut ret = Self::default();
        for edge in edges {
            ret.add_labelled_edge(edge);
        }
        ret
    }

    pub fn from_edge_labels_checked(
        edges: impl IntoIterator<Item = Edge>,
    ) -> Result<Self, (Self, InvalidGraph)> {
        let graph = Self::from_edge_labels(edges);
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((graph, err)),
        }
    }

    pub fn from_adjacency_labels<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        let mut ret = Self::default();
        for node_adj in adj {
            ret.add_labelled_node_symmetrically(node_adj);
        }
        ret
    }

    pub fn from_symmetric_adjancency_labels<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        let mut ret = Self::default();
        for node_adj in adj {
            ret.add_labelled_node(node_adj);
        }
        ret
    }

    /// # Safety
    /// The `node` must be valid, i.e., between `0` and `self.len() - 1`. Furthermore, it
    /// must not contain itself as neighbour. The same has to hold for the node with the
    /// highest index.
    pub unsafe fn remove_node_unchecked(&mut self, node: Node) {
        // the API safety invariant ensures self.len > 0
        let last_node = self.0.len() - 1;

        if node == last_node {
            let neighbours = self.0.pop().unwrap();
            for neighbour in neighbours {
                // safety: neighbours are only ever inserted by first getting the index of
                // the label, so they have to exist; furthermore, they can only be removed
                // through this method here, but then - according to this implementation -
                // no other node could have it as a neighbour
                unsafe { self.0.get_neighbours_mut_unchecked(neighbour) }.remove(&node);
            }
            return;
        }
        // safety: API safety invariant exactly that
        unsafe { self.0.raw_node_swap_remove(node) };
        // safety: we swapped in the last_node into the node's position, so `node` is
        // valid, furthermore, since last_node had no self-loop, its neighbours are valid
        // and its neighbours do not contain `node` since we removed it above from
        // last_node's neighbours
        unsafe { self.0.raw_node_neighbours_update(node, &last_node) };
    }

    pub fn remove_node(&mut self, node: Node) {
        assert!(node < self.0.len(), "Node out of bounds");
        assert!(
            !self.0.get_neighbours(node).unwrap().contains(&node),
            "Node has self-loop"
        );
        let last_node = self.0.len() - 1;
        assert!(
            !self.0.get_neighbours(last_node).unwrap().contains(&last_node),
            "Last node has self-loop"
        );
        unsafe { self.remove_node_unchecked(node) }
    }

    pub fn map_to_labels(&self) -> HashMap<Node, Neighbours> {
        self.0
            .enumerate_neighbours()
            .map(|(node, neighbours)| {
                (
                    self.0.get_label(node).unwrap(),
                    neighbours
                        .iter()
                        .map(|n| {
                            // could just do unchecked here, but this is only used in
                            // tests, so we do the safe thing
                            self.0.get_label(*n).unwrap_or_else(|| {
                                panic!(
                                    "Node {node} has a neighbour {n} that is not in the \
                                     graph"
                                )
                            })
                        })
                        .collect(),
                )
            })
            .collect()
    }

    pub fn check(&self) -> Result<(), InvalidGraph> {
        for (node, neighbours) in self.0.enumerate_neighbours() {
            for &neighbour in neighbours.iter() {
                if node == neighbour {
                    return Err(InvalidGraph::SelfLoop(node as int));
                }
                if !self.0.get_neighbours(neighbour).unwrap().contains(&node) {
                    return Err(InvalidGraph::IncompatibleNeighbourhoods(
                        node as int,
                        neighbour as int,
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn map_to_full(&self) -> Vec<GraphNode> {
        self.0
            .enumerate_full()
            .map(|(index, label, neighbours)| GraphNode { index, label, neighbours })
            .collect()
    }
}

impl<G: GraphData> Debug for Graph<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graph").field("nodes", &self.map_to_full()).finish()
    }
}

pub mod data;

#[cfg(test)]
mod tests {
    use super::{
        super::test_utils::collect,
        data::{Custom, IndexMap},
        *,
    };

    #[test]
    fn test() {
        let input = collect!(vh;
            (0, [1, 2]),
            (1, [0, 2]),
            (2, [0, 1]),
        );
        fn _test<G: GraphData>(input: Vec<(Label, Neighbours)>) {
            let mut graph = Graph::<G>::from_symmetric_adjancency_labels(input);
            println!("{:?}", graph);
            let idx = graph.get_index(1).unwrap();
            println!("{:?}", idx);
            graph.remove_node(idx);
            println!("{:?}", graph);
        }
        _test::<Custom>(input.clone());
        _test::<IndexMap>(input);
    }
}
