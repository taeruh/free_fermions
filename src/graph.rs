use std::{
    collections::{hash_set, HashMap, HashSet},
    iter::{self, Copied},
    ops::{Deref, DerefMut, Index, IndexMut},
    slice,
};

use crate::fix_int::int;

// some of the following type aliases are not used, but they serve as documentation and
// orientation for variable names
pub type Node = int;
pub type Edge = (Node, Node);
// V for vec
pub type VNodes = Vec<Node>;
pub type VNeighbourhood = Vec<Node>;
pub type VNodeInfo = (Node, VNeighbourhood);
// H for hash
pub type HNodes = HashSet<Node>;
pub type HNeighbourhood = HashSet<Node>;
pub type HNodeInfo = (Node, HNeighbourhood);

// Intended misuse of the newtype pattern and Deref(Mut) traits: I want to implement
// foreign traits generically on G: ImplGraph (which is not directly possible), so I wrap
// into a newtype; it is not like we are extending G, but rather we want to say that
// Graph<G> is G, so I think the Deref(Mut) is justified
pub struct Graph<G>(G);
impl<G> Deref for Graph<G> {
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<G> DerefMut for Graph<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
pub trait ImplGraph {
    type NodeCollection: NodeCollection;

    fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self
    where
        Self: Sized;

    fn from_edge_vec(edges: Vec<Edge>) -> Self
    where
        Self: Sized,
    {
        ImplGraph::from_edges(edges)
    }

    fn from_edge_set(edges: HashSet<Edge>) -> Self
    where
        Self: Sized,
    {
        ImplGraph::from_edges(edges)
    }

    fn from_adjancencies<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Node, N)>,
        N: IntoIterator<Item = Node>;

    fn from_adjacency_vec(adj: Vec<Vec<Node>>) -> Self
    where
        Self: Sized,
    {
        ImplGraph::from_adjancencies((0u32..).zip(adj))
    }

    fn from_adjacency_map(adj: HashMap<Node, HashSet<Node>>) -> Self
    where
        Self: Sized,
    {
        ImplGraph::from_adjancencies(adj)
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, node: Node) -> Option<&Self::NodeCollection>;

    fn get_mut(&mut self, node: Node) -> Option<&mut Self::NodeCollection>;

    fn filter_nodes(&mut self, f: impl Fn(Node) -> bool);

    /// Default implementation: Calls filter_nodes.
    fn remove_node(&mut self, node: Node) {
        self.filter_nodes(|n| n != node);
    }

    fn into_subgraph(mut self, nodes: impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        self.filter_nodes(|n| nodes.contains(n));
        self
    }

    /// Note for implementation: One can easily implement this method by cloning and then
    /// calling into_subgraph.
    fn subgraph(&self, nodes: impl NodeCollection) -> Self
    where
        Self: Sized;

    fn complement(&mut self);
}

impl<G: ImplGraph> Index<Node> for Graph<G> {
    type Output = G::NodeCollection;

    fn index(&self, index: Node) -> &Self::Output {
        self.0.get(index).expect("node not found")
    }
}

impl<G: ImplGraph> IndexMut<Node> for Graph<G> {
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        self.0.get_mut(index).expect("node not found")
    }
}

// needed for modular-decomposition:
use petgraph::{
    visit::{
        GraphBase, GraphProp, IntoNeighbors, NodeCompactIndexable, NodeCount,
        NodeIndexable,
    },
    Undirected,
};

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
    fn node_bound(&self) -> usize {
        Node::MAX as usize
    }

    fn to_index(&self, a: Self::NodeId) -> usize {
        a as usize
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        i.try_into().expect("index out of bounds")
    }
}

impl<G: ImplGraph> NodeCompactIndexable for Graph<G> {}

impl<'a, G: ImplGraph> IntoNeighbors for &'a Graph<G> {
    type Neighbors = <<G as ImplGraph>::NodeCollection as NodeCollection>::Iter<'a>;
    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self[a].iter()
    }
}

impl<G: ImplGraph> GraphProp for Graph<G> {
    type EdgeType = Undirected;
}

pub trait NodeCollection: IntoIterator<Item = Node> {
    type Iter<'a>: Iterator<Item = Node>
    where
        Self: 'a;
    fn contains(&self, e: Node) -> bool;
    fn iter(&self) -> Self::Iter<'_>;
}

impl NodeCollection for VNodes {
    type Iter<'a> = iter::Copied<slice::Iter<'a, Node>>;
    fn contains(&self, e: Node) -> bool {
        <[Node]>::contains(self, &e)
    }
    fn iter(&self) -> Self::Iter<'_> {
        <[Node]>::iter(self).copied()
    }
}

impl NodeCollection for HNodes {
    type Iter<'a> = Copied<hash_set::Iter<'a, Node>>;
    fn contains(&self, e: Node) -> bool {
        HashSet::contains(self, &e)
    }
    fn iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }
}

pub mod hash_graph;
pub mod my_graph;
