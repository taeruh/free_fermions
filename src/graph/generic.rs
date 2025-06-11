use std::{
    fmt::Debug,
    iter::{self, Copied},
    ops::Range,
    ptr, slice,
};

use hashbrown::{HashMap, HashSet, hash_set};
use petgraph::{
    Undirected,
    visit::{
        GraphBase, GraphProp, IntoNeighbors, NodeCompactIndexable, NodeCount,
        NodeIndexable,
    },
};

use super::{
    CompactNodes, Edge, HLabels, HNodes, InvalidGraph, Label, LabelEdge, Node,
    SwapRemoveMap, VNodes,
};

/// Newtype around `impl `[ImplGraph] types that supports foreign traits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Graph<G = Adj>(pub G);

impl<G> Graph<G> {
    pub fn new(graph: G) -> Self {
        Self(graph)
    }
}

// TODO: maybe split this trait using some traits from petgraph (some of them I have to
// implement anyways, so it might be a good idea to use them directly)

/// A basic graph without any associated data
// I tried making it generic with associated types such that the nodes could have data, if
// wanted (or not), but this gets rather convoluted and there's no use case for it, since
// we don't need weights (if we need them later on, it is easier to adjust it accordingly
// then; remember that this is not a graph library, but instead this trait is only a
// convenience for us to use different graph representations)
//
// note that some graph representations use, for example, a vector of edges internally as
// representation; that's why we have additional from_* methods, so that they are are
// cheaper (instead of always using the iterator-based from_ methods)
// pub trait ImplGraph: CompactNodes + Clone {
pub trait ImplGraph: CompactNodes + Clone + Debug + Default {
    type Nodes: NodeCollectionMut + IntoIterator<Item = Node> + FromIterator<Node>;
    type Neighbours<'a>: NodeCollectionRef + NodeCollection<Collected = Self::Nodes>
    where
        Self: 'a;

    // for adding and removing, there are labelled and unlabelled versions; the unlabelled
    // versions directly work with the indices, while the labelled versions first do the
    // conversion from the label to the index; for the methods that create things, we
    // usually require the labelled versions, since we do not know how the conversion
    // works, but for the methods that remove things, we usually require the unlabelled
    // because they are more intuitive once the graph is created (the labelled version can
    // then be simply implement with the find_node method)

    fn add_labelled_edge(&mut self, edge: LabelEdge);

    fn add_labelled_node_symmetrically<N: IntoIterator<Item = Label>>(
        &mut self,
        node_adj: (Label, N),
    );

    fn from_edge_labels_unchecked(edges: impl IntoIterator<Item = LabelEdge>) -> Self
    where
        Self: Sized,
    {
        let mut ret = Self::default();
        for edge in edges {
            ret.add_labelled_edge(edge);
        }
        ret
    }

    fn from_adjacency_labels_unchecked<A, N>(adj: A) -> Self
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

    fn from_symmetric_adjacency_labels_unchecked<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        Self::from_adjacency_labels_unchecked(adj)
    }

    fn from_edge_labels(
        edges: impl IntoIterator<Item = LabelEdge>,
    ) -> Result<Self, (Self, InvalidGraph<Node>)>
    where
        Self: Sized,
    {
        let graph = Self::from_edge_labels_unchecked(edges);
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((graph, err)),
        }
    }

    fn from_adjacency_labels<A, N>(adj: A) -> Result<Self, (Self, InvalidGraph<Node>)>
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
        Self: Sized,
    {
        let graph = Self::from_adjacency_labels_unchecked(adj);
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((graph, err)),
        }
    }

    fn from_symmetric_adjacency_labels<A, N>(
        adj: A,
    ) -> Result<Self, (Self, InvalidGraph<Node>)>
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
        Self: Sized,
    {
        let graph = Self::from_symmetric_adjacency_labels_unchecked(adj);
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((graph, err)),
        }
    }

    // // adding edge based on the indices
    // fn add_edge(&mut self, (a, b): Edge) {
    //     self.get_neighbours_mut(a).unwrap().insert(b);
    //     self.get_neighbours_mut(b).unwrap().insert(a);
    // }

    // /// Probably want to override this for performance reasons
    // fn add_node(&mut self, (label, neighbours): (Label, Self::Nodes)) {
    //     assert!(self.get_label(label).is_none());
    //     self.add_labelled_node_symmetrically((label, []));
    //     // new_node is probably usually self.len() - 1, but we cannot be sure
    //     let new_node = self.find_node(label).unwrap();
    //     neighbours.iter().for_each(|n| {
    //         self.get_neighbours_mut(n).unwrap().insert(new_node);
    //     });
    //     debug_assert!(
    //         mem::replace(self.get_neighbours_mut(new_node).unwrap(), neighbours)
    //             .is_empty()
    //     );
    // }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get_label(&self, node: Node) -> Option<Label>;

    fn get_label_mut(&mut self, node: Node) -> Option<&mut Label>;

    fn get_neighbours(&self, node: Node) -> Option<Self::Neighbours<'_>>;

    // fn get_neighbours_mut(&mut self, node: int) -> Option<&mut Self::Nodes>;

    fn remove_node(&mut self, node: Node);

    fn remove_labelled_node(&mut self, label: Label) {
        self.remove_node(self.find_node(label).unwrap());
    }

    // fn remove_edge(&mut self, (a, b): Edge) {
    //     self.get_neighbours_mut(a).unwrap().remove(b);
    //     self.get_neighbours_mut(b).unwrap().remove(a);
    // }

    // fn remove_labelled_edge(&mut self, (a, b): Edge) {
    //     self.remove_edge((self.find_node(a).unwrap(), self.find_node(b).unwrap()));
    // }

    /// Default implementation uses [Self::remove_node]
    fn retain_nodes(&mut self, f: impl Fn(Node) -> bool) {
        let mut graph_map = SwapRemoveMap::new(self.len());
        for node in self.iter_nodes() {
            if !f(node) {
                self.remove_node(graph_map.swap_remove(node));
            }
        }
    }

    /// This can usually be implemented more efficiently (we have to filter here the
    /// neighbours that are already in the subgraph since we have only access to
    /// add_labelled_node_symmetrically
    fn subgraph_by_adding(&self, nodes: &impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        let mut ret = Self::default();
        for node in nodes.iter() {
            let neighbours: Vec<_> = self
                .get_neighbours(node)
                .unwrap()
                .iter_ref()
                .filter_map(|n| {
                    let label = self.get_label(n).unwrap();
                    if ret.find_node(label).is_some() {
                        Some(label)
                    } else {
                        None
                    }
                })
                .collect();
            ret.add_labelled_node_symmetrically((
                self.get_label(node).unwrap(),
                neighbours,
            ));
        }
        ret
    }

    fn subgraph_by_removing(mut self, nodes: &impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        self.retain_nodes(|n| nodes.contains(n));
        self
    }

    fn into_subgraph(self, nodes: &impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        // int=u32::MAX = 4294967295 < 1.7976931348623157e308 = f64::MAX
        if (nodes.len() as f64) < (0.6 * self.len() as f64) {
            self.subgraph_by_adding(nodes)
        } else {
            self.subgraph_by_removing(nodes)
        }
    }

    fn subgraph(&self, nodes: &impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        if (nodes.len() as f64) < (0.6 * self.len() as f64) {
            // if (nodes.len() as f64) < (0.0 * self.len() as f64) {
            self.subgraph_by_adding(nodes)
        } else {
            self.clone().subgraph_by_removing(nodes)
        }
    }

    fn complement(&mut self);

    fn set_is_independent(&self, mut subset: impl Iterator<Item = Node> + Clone) -> bool {
        while let Some(node) = subset.next() {
            let mut remaining = subset.clone();
            let neighbours = self.get_neighbours(node).expect("invalid node");
            if remaining.any(|n| neighbours.contains(n)) {
                return false;
            }
        }
        true
    }

    fn iter_nodes(&self) -> Range<Node> {
        0..self.len() as Node
    }

    fn iter_labels(&self) -> impl Iterator<Item = Label> + Clone {
        self.iter_nodes()
            .map(|node| self.get_label(node).expect("invalid node"))
    }

    fn iter_labels_mut(&mut self) -> impl Iterator<Item = &mut Label>;

    fn iter_with_labels(&self) -> impl Iterator<Item = (Node, Label)> + Clone {
        self.iter_nodes()
            .map(|node| (node, self.get_label(node).expect("invalid node")))
    }

    // fn iter_with_labels_mut(&mut self) -> impl Iterator<Item = (Node, &mut Node)>;

    fn iter_neighbourhoods(&self) -> impl Iterator<Item = Self::Neighbours<'_>> + Clone {
        self.iter_nodes()
            .map(|node| self.get_neighbours(node).expect("invalid node"))
    }

    // fn iter_neighbourhoods_mut(&mut self) -> impl Iterator<Item = &mut Self::Nodes>;

    fn iter_with_neighbourhoods(
        &self,
    ) -> impl Iterator<Item = (Node, Self::Neighbours<'_>)> + Clone {
        self.iter_nodes()
            .map(|node| (node, self.get_neighbours(node).expect("invalid node")))
    }

    // fn iter_with_neighbourhoods_mut(
    //     &mut self,
    // ) -> impl Iterator<Item = (Node, &mut Self::Nodes)> {
    //     enumerate!(self.iter_neighbourhoods_mut())
    // }

    /// Check whether it is a valid graph description.
    fn check(&self) -> Result<(), InvalidGraph<Node>> {
        for (node, neighbours) in self.iter_with_neighbourhoods() {
            for neighbour in neighbours.iter_ref() {
                if node == neighbour {
                    return Err(InvalidGraph::SelfLoop(node));
                }
                if !self.get_neighbours(neighbour).unwrap().contains(node) {
                    return Err(InvalidGraph::IncompatibleNeighbourhoods(
                        node, neighbour,
                    ));
                }
            }
        }
        Ok(())
    }

    fn map_to_labels(&self) -> HashMap<Label, HLabels> {
        self.iter_with_neighbourhoods()
            .map(|(node, neighbours)| {
                (
                    self.get_label(node).unwrap(),
                    neighbours.iter_ref().map(|n| self.get_label(n).unwrap()).collect(),
                )
            })
            .collect()
    }

    fn find_node(&self, label: Label) -> Option<Node> {
        self.iter_with_labels()
            .find_map(|(n, l)| if l == label { Some(n) } else { None })
    }

    fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy {
        |n| self.get_label(n).unwrap()
    }
}

impl<G: CompactNodes> CompactNodes for Graph<G> {}

impl<G: ImplGraph> ImplGraph for Graph<G> {
    type Nodes = G::Nodes;
    type Neighbours<'a>
        = G::Neighbours<'a>
    where
        Self: 'a;
    #[inline]
    fn add_labelled_edge(&mut self, edge: LabelEdge) {
        self.0.add_labelled_edge(edge)
    }
    #[inline]
    fn add_labelled_node_symmetrically<N: IntoIterator<Item = Label>>(
        &mut self,
        node_adj: (Label, N),
    ) {
        self.0.add_labelled_node_symmetrically(node_adj)
    }
    #[inline]
    fn from_edge_labels_unchecked(edges: impl IntoIterator<Item = LabelEdge>) -> Self
    where
        Self: Sized,
    {
        Self(G::from_edge_labels_unchecked(edges))
    }
    #[inline]
    fn from_adjacency_labels_unchecked<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Label, N)>,
        N: IntoIterator<Item = Label>,
    {
        Self(G::from_adjacency_labels_unchecked(adj))
    }
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[inline]
    fn get_label(&self, node: Node) -> Option<Label> {
        self.0.get_label(node)
    }
    #[inline]
    fn get_label_mut(&mut self, node: Node) -> Option<&mut Label> {
        self.0.get_label_mut(node)
    }
    #[inline]
    fn get_neighbours(&self, node: Node) -> Option<Self::Neighbours<'_>> {
        self.0.get_neighbours(node)
    }
    #[inline]
    fn find_node(&self, label: Label) -> Option<Node> {
        self.0.find_node(label)
    }
    // #[inline]
    // fn get_neighbours_mut(&mut self, node: int) -> Option<&mut Self::Nodes> {
    //     self.0.get_neighbours_mut(node)
    // }
    #[inline]
    fn remove_node(&mut self, node: Node) {
        self.0.remove_node(node);
    }
    #[inline]
    fn retain_nodes(&mut self, f: impl Fn(Node) -> bool) {
        self.0.retain_nodes(f)
    }
    #[inline]
    fn into_subgraph(self, nodes: &impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        Self(self.0.into_subgraph(nodes))
    }
    #[inline]
    fn subgraph(&self, nodes: &impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        Self(self.0.subgraph(nodes))
    }
    #[inline]
    fn complement(&mut self) {
        self.0.complement()
    }
    #[inline]
    fn set_is_independent(&self, subset: impl Iterator<Item = Node> + Clone) -> bool {
        self.0.set_is_independent(subset)
    }
    #[inline]
    fn iter_nodes(&self) -> Range<Node> {
        self.0.iter_nodes()
    }
    #[inline]
    fn iter_labels(&self) -> impl Iterator<Item = Label> + Clone {
        self.0.iter_labels()
    }
    #[inline]
    fn iter_labels_mut(&mut self) -> impl Iterator<Item = &mut Label> {
        self.0.iter_labels_mut()
    }
    #[inline]
    fn iter_with_labels(&self) -> impl Iterator<Item = (Node, Label)> + Clone {
        self.0.iter_with_labels()
    }
    // #[inline]
    // fn iter_with_labels_mut(&mut self) -> impl Iterator<Item = (int, &mut int)> {
    //     self.0.iter_with_labels_mut()
    // }
    #[inline]
    fn iter_neighbourhoods(&self) -> impl Iterator<Item = Self::Neighbours<'_>> + Clone {
        self.0.iter_neighbourhoods()
    }
    // #[inline]
    // fn iter_neighbourhoods_mut(&mut self) -> impl Iterator<Item = &mut Self::Nodes> {
    //     self.0.iter_neighbourhoods_mut()
    // }
    #[inline]
    fn iter_with_neighbourhoods(
        &self,
    ) -> impl Iterator<Item = (Node, Self::Neighbours<'_>)> + Clone {
        self.0.iter_with_neighbourhoods()
    }
    // #[inline]
    // fn iter_with_neighbourhoods_mut(
    //     &mut self,
    // ) -> impl Iterator<Item = (int, &mut Self::Nodes)> {
    //     self.0.iter_with_neighbourhoods_mut()
    // }
    // #[inline]
    // fn check(&self) -> Result<(), InvalidGraph> {
    //     self.0.check()
    // }
    // #[inline]
    // fn correct(&mut self) {
    //     self.0.correct()
    // }
}

// // we usually do not care about labels, until the very end, so we use indexing to get the
// // neighbourhoods for convenience
// impl<G: ImplGraph> Index<int> for Graph<G> {
//     type Output = G::Neighbours;

//     fn index(&self, index: int) -> &Self::Output {
//         self.0.get_neighbours(index).expect("invalid node")
//     }
// }

// impl<G: ImplGraph> IndexMut<int> for Graph<G> {
//     fn index_mut(&mut self, index: int) -> &mut Self::Output {
//         self.0.get_neighbours_mut(index).expect("invalid node")
//     }
// }

// needed for modular-decomposition: {{{
impl<G: ImplGraph> GraphBase for Graph<G> {
    type NodeId = Node;
    type EdgeId = Edge;
}

impl<G: ImplGraph> NodeCount for Graph<G> {
    fn node_count(&self) -> usize {
        self.len()
    }
}

impl<G: ImplGraph> NodeIndexable for Graph<G> {
    // this makes sense, because ImplGraph requires CompactNodes
    fn node_bound(&self) -> usize {
        self.len()
    }

    fn to_index(&self, a: Self::NodeId) -> usize {
        a
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        i
    }
}

// this makes sense, because ImplGraph requires CompactNodes
impl<G: ImplGraph> NodeCompactIndexable for Graph<G> {}

impl<'a, G: ImplGraph> IntoNeighbors for &'a Graph<G> {
    type Neighbors = <<G as ImplGraph>::Neighbours<'a> as NodeCollectionRef>::Iter;
    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self.get_neighbours(a).unwrap().iter_ref()
    }
}

impl<G: ImplGraph> GraphProp for Graph<G> {
    type EdgeType = Undirected;
}
// }}}

pub trait NodeCollection: Clone + Debug {
    type Collected: NodeCollectionMut;

    type Iter<'a>: Iterator<Item = Node> + Clone
    where
        Self: 'a;

    fn contains(&self, e: Node) -> bool;

    fn iter(&self) -> Self::Iter<'_>;

    fn len(&self) -> usize {
        self.iter_ref().count()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = Node> + 'a {
        self.iter_ref().filter(|n| other.contains(*n))
    }

    fn collect(self) -> Self::Collected;
}

pub trait NodeCollectionRef {
    type Iter: Iterator<Item = Node> + Clone;

    fn iter_ref(self) -> Self::Iter;
}

impl<'a, T: NodeCollection> NodeCollectionRef for &'a T {
    type Iter = T::Iter<'a>;

    fn iter_ref(self) -> Self::Iter {
        self.iter()
    }
}

impl<'a, T: NodeCollection> NodeCollection for &'a T {
    type Collected = T::Collected;
    type Iter<'b>
        = T::Iter<'b>
    where
        Self: 'b;
    #[inline]
    fn contains(&self, e: Node) -> bool {
        (*self).contains(e)
    }
    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        (*self).iter()
    }
    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline]
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
    #[inline]
    fn intersection<'b>(&'b self, other: &'b Self) -> impl Iterator<Item = Node> + 'b {
        (*self).intersection(other)
    }
    #[inline]
    fn collect(self) -> Self::Collected {
        self.clone().collect()
    }
}

// split mut stuff (and also do not require IntoIterator as a supertrait, but instead do
// this in the associated type of Graph), so that I could implement it for (mut)
// references if needed
pub trait NodeCollectionMut: NodeCollection {
    fn remove(&mut self, e: Node) -> Option<Node>;

    fn insert(&mut self, e: Node);
}

impl NodeCollection for VNodes {
    type Collected = VNodes;
    type Iter<'a> = iter::Copied<slice::Iter<'a, Node>>;
    fn contains(&self, e: Node) -> bool {
        <[Node]>::contains(self, &e)
    }
    fn iter(&self) -> Self::Iter<'_> {
        <[Node]>::iter(self).copied()
    }
    fn len(&self) -> usize {
        <[Node]>::len(self)
    }
    fn collect(self) -> Self::Collected {
        self
    }
}

impl NodeCollectionMut for VNodes {
    fn remove(&mut self, e: Node) -> Option<Node> {
        let len = self.len();
        if e >= len as Node {
            return None;
        }
        // copy-paste code from Vec::swap_remove
        unsafe {
            let value = ptr::read(self.as_ptr().add(e));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(len - 1), base_ptr.add(e), 1);
            self.set_len(len - 1);
            Some(value)
        }
    }
    fn insert(&mut self, e: Node) {
        self.push(e);
    }
}

impl NodeCollection for HNodes {
    type Collected = HNodes;
    type Iter<'a> = Copied<hash_set::Iter<'a, Node>>;
    fn contains(&self, e: Node) -> bool {
        HashSet::contains(self, &e)
    }
    fn iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }
    fn len(&self) -> usize {
        HashSet::len(self)
    }
    fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = Node> + 'a {
        HashSet::intersection(self, other).copied()
    }
    fn collect(self) -> Self::Collected {
        self
    }
}
impl NodeCollectionMut for HNodes {
    fn remove(&mut self, e: Node) -> Option<Node> {
        HashSet::take(self, &e)
    }
    fn insert(&mut self, e: Node) {
        HashSet::insert(self, e);
    }
}

pub mod algorithms;

mod impl_graphs;
pub use impl_graphs::{adj::Adj, impl_petgraph::Pet};
