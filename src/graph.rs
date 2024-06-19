use std::{
    collections::{hash_set, HashMap, HashSet},
    fmt::Debug,
    iter::{self, Copied},
    ops::{Index, IndexMut},
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

/// Newtype around `impl `[ImplGraph] types that supports foreign traits.
pub struct Graph<G>(G);

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

    fn from_adjacencies<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Node, N)>,
        N: IntoIterator<Item = Node>;

    fn from_adjacency_vec(adj: Vec<VNodeInfo>) -> Self
    where
        Self: Sized,
    {
        ImplGraph::from_adjacencies(adj)
    }

    fn from_adjacency_hash(adj: HashMap<Node, HashSet<Node>>) -> Self
    where
        Self: Sized,
    {
        ImplGraph::from_adjacencies(adj)
    }

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, node: Node) -> Option<&Self::NodeCollection>;

    fn get_mut(&mut self, node: Node) -> Option<&mut Self::NodeCollection>;

    fn retain_nodes(&mut self, f: impl Fn(Node) -> bool);

    /// Default implementation: Calls filter_nodes. You probably want to override this.
    fn remove_node(&mut self, node: Node) {
        self.retain_nodes(|n| n != node);
    }

    /// Default implementation: Calls filter_nodes. You may want to override this.
    fn into_subgraph(mut self, nodes: impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        self.retain_nodes(|n| nodes.contains(n));
        self
    }

    /// Note for implementation: One can easily implement this method by cloning and then
    /// calling into_subgraph.
    fn subgraph(&self, nodes: impl NodeCollection) -> Self
    where
        Self: Sized;

    fn complement(&mut self);

    fn iter_nodes(&self) -> impl Iterator<Item = Node>;

    fn iter_node_info(&self) -> impl Iterator<Item = (Node, &Self::NodeCollection)>;

    fn set_is_independent(&self, subset: &Self::NodeCollection) -> bool {
        let mut iter = subset.iter();
        while let Some(node) = iter.next() {
            let mut remaining = iter.clone();
            let neighbours = self.get(node).expect("invalid node");
            if remaining.any(|n| neighbours.contains(n)) {
                return false;
            }
        }
        true
    }
}

impl<G: ImplGraph> ImplGraph for Graph<G> {
    type NodeCollection = G::NodeCollection;
    #[inline]
    fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self
    where
        Self: Sized,
    {
        Self(G::from_edges(edges))
    }
    #[inline]
    fn from_edge_vec(edges: Vec<Edge>) -> Self
    where
        Self: Sized,
    {
        Self(G::from_edges(edges))
    }
    #[inline]
    fn from_edge_set(edges: HashSet<Edge>) -> Self
    where
        Self: Sized,
    {
        Self(G::from_edges(edges))
    }
    #[inline]
    fn from_adjacencies<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Node, N)>,
        N: IntoIterator<Item = Node>,
    {
        Self(G::from_adjacencies(adj))
    }
    #[inline]
    fn from_adjacency_vec(adj: Vec<VNodeInfo>) -> Self
    where
        Self: Sized,
    {
        Self(G::from_adjacencies(adj))
    }
    fn from_adjacency_hash(adj: HashMap<Node, HashSet<Node>>) -> Self
    where
        Self: Sized,
    {
        Self(G::from_adjacencies(adj))
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
    fn get(&self, node: Node) -> Option<&Self::NodeCollection> {
        self.0.get(node)
    }
    #[inline]
    fn get_mut(&mut self, node: Node) -> Option<&mut Self::NodeCollection> {
        self.0.get_mut(node)
    }
    #[inline]
    fn retain_nodes(&mut self, f: impl Fn(Node) -> bool) {
        self.0.retain_nodes(f)
    }
    #[inline]
    fn remove_node(&mut self, node: Node) {
        self.0.remove_node(node);
    }
    #[inline]
    fn into_subgraph(self, nodes: impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        Self(self.0.into_subgraph(nodes))
    }
    #[inline]
    fn subgraph(&self, nodes: impl NodeCollection) -> Self
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
    fn iter_nodes(&self) -> impl Iterator<Item = Node> {
        self.0.iter_nodes()
    }
    #[inline]
    fn iter_node_info(&self) -> impl Iterator<Item = (Node, &Self::NodeCollection)> {
        self.0.iter_node_info()
    }
    #[inline]
    fn set_is_independent(&self, subset: &Self::NodeCollection) -> bool {
        self.0.set_is_independent(subset)
    }
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

pub trait NodeCollection: Clone + Debug + IntoIterator<Item = Node> {
    type Iter<'a>: Iterator<Item = Node> + Clone
    where
        Self: 'a;

    fn contains(&self, e: Node) -> bool;

    fn iter(&self) -> Self::Iter<'_>;

    fn len(&self) -> usize {
        self.iter().count()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = Node> + 'a {
        self.iter().filter(|n| other.contains(*n))
    }

    fn pop(&mut self) -> Option<Node> {
        self.iter().next()
    }
}

impl NodeCollection for VNodes {
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
    fn pop(&mut self) -> Option<Node> {
        self.pop()
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
    fn len(&self) -> usize {
        HashSet::len(self)
    }
    fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = Node> + 'a {
        HashSet::intersection(self, other).copied()
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::collections::{HashMap, HashSet};

    use rand::{seq::IteratorRandom, SeedableRng};
    use rand_pcg::Pcg64;

    use super::{HNeighbourhood, Node, VNeighbourhood, VNodeInfo};
    use crate::fix_int::int;

    pub enum RandomMap {
        Random(Vec<int>),
        Identity,
    }

    impl RandomMap {
        pub fn new(map_length: int, map_max: int) -> Self {
            assert!(map_max >= map_length);
            let mut rng = Pcg64::from_entropy();
            Self::Random((0..=map_max).choose_multiple(&mut rng, map_length as usize + 1))
        }

        pub fn map(&self, node: Node) -> Node {
            match self {
                RandomMap::Random(v) => v[node as usize],
                RandomMap::Identity => node,
            }
        }
    }

    macro_rules! adj_map {
        ($map:expr, $list:expr) => {
            $list
                .into_iter()
                .map(|(node, neighbours)| {
                    (
                        $map.map(node),
                        neighbours
                            .into_iter()
                            .map(|neighbour| $map.map(neighbour))
                            .collect(),
                    )
                })
                .collect()
        };
    }

    pub fn adj_hash(
        map: &RandomMap,
        list: Vec<VNodeInfo>,
    ) -> HashMap<Node, HNeighbourhood> {
        adj_map!(map, list)
    }

    pub fn adj_vec(map: &RandomMap, list: Vec<VNodeInfo>) -> Vec<(Node, VNeighbourhood)> {
        adj_map!(map, list)
    }

    macro_rules! edge_map {
        ($map:expr, $list:expr) => {
            $list
                .into_iter()
                .map(|(node, neighbour)| ($map.map(node), $map.map(neighbour)))
                .collect()
        };
    }

    pub fn edge_hash(map: &RandomMap, list: Vec<(Node, Node)>) -> HashSet<(Node, Node)> {
        edge_map!(map, list)
    }

    pub fn edge_vec(map: &RandomMap, list: Vec<(Node, Node)>) -> Vec<(Node, Node)> {
        edge_map!(map, list)
    }

    macro_rules! collect_adj {
        ($(($node:expr, [$($neighbor:expr),*]),)*) => {
            vec![$(($node, vec![$($neighbor),*]),)*]
        };
    }
    pub(crate) use collect_adj;

    macro_rules! collect {
        (adj, hash, $map:expr; $($node_info:tt,)*) => {
            $crate::graph::test_utils::adj_hash(
                &$map, $crate::graph::test_utils::collect_adj!($($node_info,)*)
            )
        };
        (adj, hash; $($node_info:tt,)*) => {
            $crate::graph::test_utils::collect!(
                adj, hash, &$crate::graph::test_utils::RandomMap::Identity; $($node_info,)*
            )
        };
        (adj, vec, $map:expr; $($node_info:tt,)*) => {
            $crate::graph::test_utils::adj_vec(
                &$map, $crate::graph::test_utils::collect_adj!($($node_info,)*)
            )
        };
        (adj, vec; $($node_info:tt,)*) => {
            $crate::graph::test_utils::collect!(
                adj, vec, &$crate::graph::test_utils::RandomMap::Identity; $($node_info,)*
            )
        };
        (edge, hash, $map:expr; $($edge:tt,)*) => {
            $crate::graph::test_utils::edge_hash(&$map, vec![$($edge,)*])
        };
        (edge, hash; $($edge:tt,)*) => {
            $crate::graph::test_utils::collect!(
                edge, hash, &$crate::graph::test_utils::RandomMap::Identity; $($edge,)*
            )
        };
        (edge, vec, $map:expr; $($edge:tt,)*) => {
            $crate::graph::test_utils::edge_vec(&$map, vec![$($edge,)*])
        };
        (edge, vec; $($edge:tt,)*) => {
            $crate::graph::test_utils::collect!(
                edge, vec, &$crate::graph::test_utils::RandomMap::Identity; $($edge,)*
            )
        };
    }
    pub(crate) use collect;

    // just a naive test whether the utils compile, more or less
    #[test]
    fn macros() {
        assert_eq!(
            vec![(1, vec![2, 3]), (2, vec![1]), (3, vec![1])],
            collect!(adj, vec; (1, [2, 3]), (2, [1]), (3, [1]),)
        );
        let map = RandomMap::new(10, 20);
        assert_eq!(
            HashSet::from_iter(
                [(1, 2), (1, 3)].into_iter().map(|(a, b)| (map.map(a), map.map(b)))
            ),
            collect!(edge, hash, map; (1, 2), (1, 3),)
        );
    }
}

pub mod adj_graph;
pub mod my_graph;
