use std::{
    fmt::{self, Debug},
    iter::Copied,
    mem,
    ops::Range,
};

use hashbrown::{HashMap, HashSet, hash_set};
use petgraph::{
    Undirected,
    visit::{
        GraphBase, GraphProp, IntoNeighbors, NodeCompactIndexable, NodeCount,
        NodeIndexable,
    },
};

use super::{CompactNodes, Edge, InvalidGraph, Label, LabelEdge, Node, SwapRemoveMap};

const DECIDER_SUBGRAPH_VIA_DELETION_IF_LESS: f64 = 0.5; // otherwise via creation

pub type Neighbours = HashSet<Node>;
pub type LabelNeighbours = HashSet<Label>;

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
/// The graph must not contain self-loops. Safe methods that create/change graphs
/// generally do not introduce self-loops or throw an error otherwise. However, there are
/// unsafe methods that allow for self-loops. Some methods rely on the invariant that
/// there are no self-loops and have UB otherwise.
#[derive(Clone, Default)]
pub struct Graph<T>(T);

#[derive(Debug)]
pub struct UnsafeGraph<T: GraphData>(Graph<T>);
impl<T: GraphData> UnsafeGraph<T> {
    /// # Safety
    /// The graph must not contain self-loops before used in any method that relies on the
    /// safety invariants on [Graph] (which are not always documented per method).
    pub unsafe fn get_graph(self) -> Graph<T> {
        self.0
    }
}

/// Inner graph data structure.
pub trait GraphData:
    CompactNodes + GraphDataSpecializerHelper + Debug + Clone + Default
{
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

    fn enumerate_neighbours(&self) -> impl Iterator<Item = (Node, &Neighbours)> + Clone;

    fn enumerate_neighbours_mut(
        &mut self,
    ) -> impl Iterator<Item = (Node, &mut Neighbours)>;

    fn enumerate_full(&self) -> impl Iterator<Item = (Node, Label, &Neighbours)>;

    fn iter_neighbours_mut(&mut self) -> impl Iterator<Item = &mut Neighbours>;
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
    /// # Safety
    /// The label must be valid.
    #[inline(always)]
    pub unsafe fn get_index_unchecked(&self, label: Label) -> Node {
        unsafe { self.0.get_index_unchecked(label) }
    }
    #[inline(always)]
    pub fn get_index(&self, label: Label) -> Option<Node> {
        self.0.get_index(label)
    }
    /// # Safety
    /// The node/index must be valid.
    #[inline(always)]
    pub unsafe fn get_label_unchecked(&self, node: Node) -> Label {
        unsafe { self.0.get_label_unchecked(node) }
    }
    #[inline(always)]
    pub fn get_label(&self, node: Node) -> Option<Label> {
        self.0.get_label(node)
    }
    #[inline(always)]
    /// # Safety
    /// The node/index must be valid.
    pub unsafe fn get_neighbours_unchecked(&self, node: Node) -> &Neighbours {
        unsafe { self.0.get_neighbours_unchecked(node) }
    }
    #[inline(always)]
    pub fn get_neighbours(&self, node: Node) -> Option<&Neighbours> {
        self.0.get_neighbours(node)
    }
    #[inline(always)]
    pub fn iter_neighbours(&self) -> impl Iterator<Item = &Neighbours> {
        self.0.iter_neighbours()
    }
    #[inline(always)]
    pub fn enumerate_neighbours(
        &self,
    ) -> impl Iterator<Item = (Node, &Neighbours)> + Clone {
        self.0.enumerate_neighbours()
    }
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// # Safety
    /// Must not introduce a self-loop.
    pub unsafe fn add_labelled_edge_unchecked(&mut self, (a, b): LabelEdge) {
        let a = self.0.get_index_or_insert(a);
        let b = self.0.get_index_or_insert(b);
        unsafe { self.0.get_neighbours_mut_unchecked(a) }.insert(b);
        unsafe { self.0.get_neighbours_mut_unchecked(b) }.insert(a);
    }

    /// # Safety
    /// Must not introduce a self-loop.
    pub unsafe fn add_labelled_node_symmetrically_unchecked(
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

    /// # Safety
    /// Must not introduce a self-loop.
    pub unsafe fn add_labelled_node_unchecked(
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

    /// # Safety
    /// Must not create a graph with self-loops.
    pub unsafe fn from_edge_labels_unchecked(
        edges: impl IntoIterator<Item = LabelEdge>,
    ) -> Self {
        let mut ret = Self::default();
        for edge in edges {
            unsafe { ret.add_labelled_edge_unchecked(edge) };
        }
        ret
    }

    pub fn from_edge_labels(
        edges: impl IntoIterator<Item = LabelEdge>,
    ) -> Result<Self, (UnsafeGraph<G>, InvalidGraph<Node>)> {
        // safety: checked below
        let graph = unsafe { Self::from_edge_labels_unchecked(edges) };
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((UnsafeGraph(graph), err)),
        }
    }

    /// # Safety
    /// Must not create a graph with self-loops.
    pub unsafe fn from_adjacency_labels_unchecked<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        let mut ret = Self::default();
        for node_adj in adj {
            unsafe { ret.add_labelled_node_symmetrically_unchecked(node_adj) };
        }
        ret
    }

    pub fn from_adjacency_labels<A, N>(
        adj: A,
    ) -> Result<Self, (UnsafeGraph<G>, InvalidGraph<Node>)>
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        // safety: checked below
        let graph = unsafe { Self::from_adjacency_labels_unchecked(adj) };
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((UnsafeGraph(graph), err)),
        }
    }

    /// # Safety
    /// Must not create a graph with self-loops.
    pub unsafe fn from_symmetric_adjancency_labels_unchecked<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        let mut ret = Self::default();
        for node_adj in adj {
            unsafe { ret.add_labelled_node_unchecked(node_adj) };
        }
        ret
    }

    pub fn from_symmetric_adjancency_labels<A, N>(
        adj: A,
    ) -> Result<Self, (UnsafeGraph<G>, InvalidGraph<Node>)>
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        // safety: checked below
        let graph = unsafe { Self::from_symmetric_adjancency_labels_unchecked(adj) };
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((UnsafeGraph(graph), err)),
        }
    }

    pub fn check(&self) -> Result<(), InvalidGraph<Node>> {
        for (node, neighbours) in self.0.enumerate_neighbours() {
            for &neighbour in neighbours.iter() {
                if node == neighbour {
                    return Err(InvalidGraph::SelfLoop(node));
                }
                if !self.0.get_neighbours(neighbour).unwrap().contains(&node) {
                    return Err(InvalidGraph::IncompatibleNeighbourhoods(
                        node, neighbour,
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn iter_nodes(&self) -> Range<Node> {
        0..self.0.len()
    }

    /// # Safety
    /// The `node` must be valid, i.e., between `0` and `self.len() - 1`. Furthermore, it
    /// must not contain itself as neighbour (not self-looped). The same has to hold for
    /// the node with the highest index.
    pub unsafe fn remove_node_unchecked(&mut self, node: Node) {
        debug_assert!(node < self.0.len());
        debug_assert!(!self.0.get_neighbours(node).unwrap().contains(&node),);
        // the API safety invariant ensures self.len > 0
        let last_node = self.0.len() - 1;
        debug_assert!(!self.0.get_neighbours(last_node).unwrap().contains(&last_node),);

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
        assert!(node < self.0.len(), "node out of bounds");
        assert!(
            !self.0.get_neighbours(node).unwrap().contains(&node),
            "node has self-loop"
        );
        let last_node = self.0.len() - 1;
        assert!(
            !self.0.get_neighbours(last_node).unwrap().contains(&last_node),
            "last node has self-loop"
        );
        unsafe { self.remove_node_unchecked(node) }
    }

    /// Remove some nodes from the graph. If `nodes` contains duplicates, it might remove
    /// unexpected nodes.
    ///
    /// # Safety
    /// The `nodes` must be valid, i.e., between `0` and `self.len() - 1`.
    pub unsafe fn delete_nodes(&mut self, nodes: impl IntoIterator<Item = Node>) {
        let mut swap_map = SwapRemoveMap::new(self.0.len());
        for node in nodes {
            unsafe {
                // safety: swap_remove_unchecked returns valid nodes, and per existence
                // invariant we have no self-loops
                self.remove_node_unchecked(
                    // safety: nodes are all valid
                    swap_map.swap_remove_unchecked(node),
                )
            };
        }
    }

    pub fn retain_nodes(&mut self, f: impl Fn(Node) -> bool) {
        let mut graph_map = SwapRemoveMap::new(self.0.len());
        for node in self.iter_nodes() {
            if !f(node) {
                unsafe {
                    // safety: we only filter the iter_nodes range, which contains valid
                    // nodes; no self-loops per existence invariant
                    self.remove_node_unchecked(graph_map.swap_remove_unchecked(node))
                };
            }
        }
    }

    /// # Safety
    /// The `nodes` must be valid, i.e., between `0` and `self.len() - 1`.
    #[inline]
    pub unsafe fn subgraph_via_deletion(
        mut self,
        nodes_to_delete: impl IntoIterator<Item = Node>,
    ) -> Self {
        unsafe { self.delete_nodes(nodes_to_delete) };
        self
    }

    /// # Safety
    /// The `nodes` must be valid, i.e., between `0` and `self.len() - 1`.
    pub unsafe fn subgraph_via_creation(
        &self,
        nodes_to_add: impl IntoIterator<Item = Node>,
    ) -> Self {
        let mut ret = Self::default();
        for node in nodes_to_add {
            let idx = ret.0.get_index_or_insert(self.0.get_label(node).unwrap());
            for &neighbour in unsafe { self.0.get_neighbours_unchecked(node) }.iter() {
                if let Some(neighbour_idx) =
                    ret.0.get_index(unsafe { self.0.get_label_unchecked(neighbour) })
                {
                    unsafe {
                        ret.0.get_neighbours_mut_unchecked(idx).insert(neighbour_idx);
                        ret.0.get_neighbours_mut_unchecked(neighbour_idx).insert(idx);
                    }
                }
            }
        }
        ret
    }

    /// # Safety
    /// The `nodes` must be valid, i.e., between `0` and `self.len() - 1`.
    pub unsafe fn subgraph(
        &self,
        subgraph_size: usize,
        nodes: impl IntoIterator<Item = Node>,
    ) -> Self {
        if subgraph_size as f64
            >= self.0.len() as f64 * DECIDER_SUBGRAPH_VIA_DELETION_IF_LESS
        {
            let nodes = HashSet::<_>::from_iter(nodes);
            let nodes_to_delete =
                self.iter_nodes().filter(|n| !nodes.contains(n)).collect::<Vec<_>>();
            unsafe { self.clone().subgraph_via_deletion(nodes_to_delete) }
        } else {
            unsafe { self.subgraph_via_creation(nodes) }
        }
    }

    /// Same as [subgraph], but potentially optimised for a HashSet as input (instead of a
    /// general IntoIterator).
    ///
    /// # Safety
    /// The `nodes` must be valid, i.e., between `0` and `self.len() - 1`.
    pub unsafe fn subgraph_from_set(
        &self,
        subgraph_size: usize,
        nodes: &HashSet<Node>,
    ) -> Self {
        if subgraph_size as f64
            >= self.0.len() as f64 * DECIDER_SUBGRAPH_VIA_DELETION_IF_LESS
        {
            let nodes_to_delete =
                self.iter_nodes().filter(|n| !nodes.contains(n)).collect::<Vec<_>>();
            unsafe { self.clone().subgraph_via_deletion(nodes_to_delete) }
        } else {
            unsafe { self.subgraph_via_creation(nodes.into_iter().copied()) }
        }
    }

    /// # Safety
    /// The `subset` must be valid, i.e., between `0` and `self.len() - 1`.
    fn set_is_independent(&self, mut subset: impl Iterator<Item = Node> + Clone) -> bool {
        while let Some(node) = subset.next() {
            let mut remaining = subset.clone();
            let neighbours = unsafe { self.0.get_neighbours_unchecked(node) };
            if remaining.any(|n| neighbours.contains(&n)) {
                return false;
            }
        }
        true
    }

    pub fn complement(&mut self) {
        #[cfg(debug_assertions)]
        {
            for (node, neighbours) in self.0.enumerate_neighbours() {
                if neighbours.contains(&node) {
                    panic!("node {node} has a self-loop in the complement");
                }
            }
        }
        let node_range = 0..self.0.len();
        for (node, neighbours) in self.0.enumerate_neighbours_mut() {
            for other in node_range.clone() {
                if !neighbours.contains(&other) && node != other {
                    neighbours.insert(other);
                } else {
                    neighbours.remove(&other);
                }
                // PERF: alternatively, we could mem::take(neighbours) and then only
                // insert into the replaced neighbours; not sure which is faster
            }
        }
        // let nodes = self.iter_nodes().collect::<Vec<_>>();
        // for (node, neighbours) in enumerate!(self.nodes.iter_mut()) {
        //     let mut neighbourhood_to_complement = mem::take(neighbours);
        //     neighbourhood_to_complement.insert(node); // no self loops in the complement
        //     for other in nodes.iter() {
        //         if !neighbourhood_to_complement.contains(&other) {
        //             neighbours.insert(other);
        //         }
        //     }
        // }
    }

    pub fn map_to_labels(&self) -> HashMap<Label, LabelNeighbours> {
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
                                    "node {node} has a neighbour {n} that is not in the \
                                     graph"
                                )
                            })
                        })
                        .collect(),
                )
            })
            .collect()
    }

    pub fn map_to_full(&self) -> Vec<GraphNode> {
        self.0
            .enumerate_full()
            .map(|(index, label, neighbours)| GraphNode { index, label, neighbours })
            .collect()
    }

    /// Convenience when performance does not matter. When it matters use the graph itself
    /// and the unchecked method.
    pub fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy + '_ {
        |n| self.0.get_label(n).unwrap()
    }

    /// Convenience when performance does not matter. When it matters use the graph itself
    /// and the unchecked method.
    pub fn get_index_mapping(&self) -> impl Fn(Label) -> Node + Copy + '_ {
        |l| self.0.get_index(l).unwrap()
    }
}

impl<G: GraphData> Debug for Graph<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Graph").field("nodes", &self.map_to_full()).finish()
    }
}

// needed for modular-decomposition: {{{
impl<G: GraphData> GraphBase for Graph<G> {
    type NodeId = Node;
    type EdgeId = Edge;
}

impl<G: GraphData> GraphProp for Graph<G> {
    type EdgeType = Undirected;
}

impl<G: GraphData> NodeCount for Graph<G> {
    fn node_count(&self) -> usize {
        self.len()
    }
}

impl<G: GraphData> NodeIndexable for Graph<G> {
    fn node_bound(&self) -> usize {
        self.len()
    }

    fn to_index(&self, i: Self::NodeId) -> usize {
        i
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        i
    }
}

impl<G: GraphData> NodeCompactIndexable for Graph<G> {}

impl<'a, G: GraphData> IntoNeighbors for &'a Graph<G> {
    type Neighbors = Copied<hash_set::Iter<'a, usize>>;
    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.0.get_neighbours(a).unwrap().iter().copied()
    }
}
// }}}

pub mod algorithms;
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
        fn _test<G: GraphData>(input: Vec<(Label, LabelNeighbours)>) {
            let mut graph = Graph::<G>::from_symmetric_adjancency_labels(input).unwrap();
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
