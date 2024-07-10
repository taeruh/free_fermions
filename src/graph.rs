use std::{
    collections::{hash_set, HashMap, HashSet},
    fmt::Debug,
    iter::{self, Copied},
    mem,
    ops::{Index, IndexMut, Range},
    ptr, slice,
};

use adj::AdjGraph;
use petgraph::{
    visit::{
        GraphBase, GraphProp, IntoNeighbors, NodeCompactIndexable, NodeCount,
        NodeIndexable,
    },
    Undirected,
};

use crate::fix_int::{self, enumerate, int};

// some of the following type aliases are not used, but they serve as documentation and
// orientation for variable names
pub type Node = int;
pub type Edge = (Node, Node);
pub type Label = int; // i.e, the weight which is in our case just the label

// V for vector
pub(crate) type VNodes = Vec<Node>;
#[allow(unused)]
pub(crate) type VNodeInfo = (int, Vec<Node>);
// H for hash
pub(crate) type HNodes = HashSet<Node>;
#[allow(unused)]
pub(crate) type HNodeInfo = (Node, HashSet<Node>);

// // petgraph::petgraph::NodeIndex is per default already
// petgraph::petgraph::NodeIndex<DefaultIx> // where DefaultIx is int; this is just to
// makes things clear
pub type NodeIndex = petgraph::graph::NodeIndex<int>; // = int

/// Newtype around `impl `[ImplGraph] types that supports foreign traits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Graph<G = AdjGraph>(G);

impl<G> Graph<G> {
    pub fn new(graph: G) -> Self {
        Self(graph)
    }
}

/// Marker trait, that promises that the nodes in the graph go from 0 to n-1 without
/// skipping any values; and when a node is removed, its index place is reused by the last
/// node (i.e., swap_removed)
pub trait CompactNodes {}

/// A helper to keep track of swap-removals. Basically has to be used when some nodes to
/// remove are fixed and then we iterate over them and remove them one by one (cf. default
/// implemenation of retain nodes).
#[derive(Clone, Debug)]
pub struct SwapRemoveMap {
    map: Vec<int>,
    position: Vec<int>,
    len: usize,
}

impl SwapRemoveMap {
    pub fn new(len: usize) -> Self {
        assert!(len > 0);
        Self {
            map: (0..len as int).collect(),
            position: (0..len as int).collect(),
            len,
        }
    }

    pub fn map(&self, node: int) -> int {
        self.map[node as usize]
    }

    // assert above len > 0
    /// No bounds checking (usually automatically done since this type only accompanies
    /// some other indexing type which defines the indices which should be valid).
    pub fn swap_remove_unchecked(&mut self, node: int) -> int {
        self.len -= 1;
        let mapped = self.map[node as usize];
        self.map[self.position[self.len] as usize] = mapped;
        self.position.swap(mapped as usize, self.len);
        mapped
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
    type Nodes: NodeCollectionMut + IntoIterator<Item = int> + FromIterator<int>;
    type Neighbours<'a>: NodeCollectionRef + NodeCollection<Collected = Self::Nodes>
    where
        Self: 'a;

    // for adding and removing there are labelled and unlabelled versions; the unlabelled
    // versions directly work with the indices, while the labelled versions first do the
    // conversion from the label to the index; for the methods that create things, we
    // usually require the labelled versions, since we do not know how the conversion
    // works, but for the methods that remove things, we usually require the unlabelled
    // because they are more intuitive once the graph is created (the labelled version can
    // then be simply implement with the find_node method)

    fn add_labelled_edge(&mut self, edge: Edge);

    fn add_labelled_node_symmetrically<N: IntoIterator<Item = int>>(
        &mut self,
        node_adj: (int, N),
    );

    fn from_edge_labels_unchecked(edges: impl IntoIterator<Item = Edge>) -> Self
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
        A: IntoIterator<Item = (int, N)>,
        N: IntoIterator<Item = int>,
    {
        let mut ret = Self::default();
        for node_adj in adj {
            ret.add_labelled_node_symmetrically(node_adj);
        }
        ret
    }

    fn from_symmetric_adjacency_labels_unchecked<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (int, N)>,
        N: IntoIterator<Item = int>,
    {
        Self::from_adjacency_labels_unchecked(adj)
    }

    fn from_edge_labels(
        edges: impl IntoIterator<Item = Edge>,
    ) -> Result<Self, (Self, InvalidGraph)>
    where
        Self: Sized,
    {
        let graph = Self::from_edge_labels_unchecked(edges);
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((graph, err)),
        }
    }

    fn from_adjacency_labels<A, N>(adj: A) -> Result<Self, (Self, InvalidGraph)>
    where
        A: IntoIterator<Item = (int, N)>,
        N: IntoIterator<Item = int>,
        Self: Sized,
    {
        let graph = Self::from_adjacency_labels_unchecked(adj);
        match graph.check() {
            Ok(()) => Ok(graph),
            Err(err) => Err((graph, err)),
        }
    }

    fn from_symmetric_adjacency_labels<A, N>(adj: A) -> Result<Self, (Self, InvalidGraph)>
    where
        A: IntoIterator<Item = (int, N)>,
        N: IntoIterator<Item = int>,
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

    fn get_label(&self, node: int) -> Option<int>;

    fn get_label_mut(&mut self, node: int) -> Option<&mut int>;

    fn get_neighbours(&self, node: int) -> Option<Self::Neighbours<'_>>;

    // fn get_neighbours_mut(&mut self, node: int) -> Option<&mut Self::Nodes>;

    fn remove_node(&mut self, node: int);

    fn remove_labelled_node(&mut self, label: int) {
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
    fn retain_nodes(&mut self, f: impl Fn(int) -> bool) {
        let mut graph_map = SwapRemoveMap::new(self.len());
        for node in self.iter_nodes() {
            if !f(node) {
                self.remove_node(graph_map.swap_remove_unchecked(node));
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
        if fix_int::to_float(nodes.len() as int)
            < 0.6 * fix_int::to_float(self.len() as int)
        {
            self.subgraph_by_adding(nodes)
        } else {
            self.subgraph_by_removing(nodes)
        }
    }

    fn subgraph(&self, nodes: &impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        if fix_int::to_float(nodes.len() as int)
            < 0.6 * fix_int::to_float(self.len() as int)
        {
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

    fn iter_labels(&self) -> impl Iterator<Item = Node> + Clone {
        self.iter_nodes()
            .map(|node| self.get_label(node).expect("invalid node"))
    }

    fn iter_labels_mut(&mut self) -> impl Iterator<Item = &mut Node>;

    fn iter_with_labels(&self) -> impl Iterator<Item = (Node, Node)> + Clone {
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
    fn check(&self) -> Result<(), InvalidGraph> {
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

    fn map_to_labels(&self) -> HashMap<Node, HNodes> {
        self.iter_with_neighbourhoods()
            .map(|(node, neighbours)| {
                (
                    self.get_label(node).unwrap(),
                    neighbours.iter_ref().map(|n| self.get_label(n).unwrap()).collect(),
                )
            })
            .collect()
    }

    fn find_node(&self, label: int) -> Option<Node> {
        self.iter_with_labels()
            .find_map(|(n, l)| if l == label { Some(n) } else { None })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum InvalidGraph {
    #[error("Self loop detected on node {0}")]
    SelfLoop(Node),
    #[error("Incompatible neighbourhoods between the nodes {0} and {1}")]
    IncompatibleNeighbourhoods(Node, Node),
}

impl InvalidGraph {
    pub fn map_to_labels(&self, graph: &impl ImplGraph) -> Self {
        match self {
            Self::SelfLoop(node) => Self::SelfLoop(graph.get_label(*node).unwrap()),
            Self::IncompatibleNeighbourhoods(node, neighbour) => {
                Self::IncompatibleNeighbourhoods(
                    graph.get_label(*node).unwrap(),
                    graph.get_label(*neighbour).unwrap(),
                )
            },
        }
    }
}

impl<G: CompactNodes> CompactNodes for Graph<G> {}

impl<G: ImplGraph> ImplGraph for Graph<G> {
    type Nodes = G::Nodes;
    type Neighbours<'a> = G::Neighbours<'a> where Self: 'a;
    #[inline]
    fn add_labelled_edge(&mut self, edge: Edge) {
        self.0.add_labelled_edge(edge)
    }
    #[inline]
    fn add_labelled_node_symmetrically<N: IntoIterator<Item = int>>(
        &mut self,
        node_adj: (int, N),
    ) {
        self.0.add_labelled_node_symmetrically(node_adj)
    }
    #[inline]
    fn from_edge_labels_unchecked(edges: impl IntoIterator<Item = Edge>) -> Self
    where
        Self: Sized,
    {
        Self(G::from_edge_labels_unchecked(edges))
    }
    #[inline]
    fn from_adjacency_labels_unchecked<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (int, N)>,
        N: IntoIterator<Item = int>,
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
    fn get_label(&self, node: int) -> Option<int> {
        self.0.get_label(node)
    }
    #[inline]
    fn get_label_mut(&mut self, node: int) -> Option<&mut int> {
        self.0.get_label_mut(node)
    }
    #[inline]
    fn get_neighbours(&self, node: int) -> Option<Self::Neighbours<'_>> {
        self.0.get_neighbours(node)
    }
    // #[inline]
    // fn get_neighbours_mut(&mut self, node: int) -> Option<&mut Self::Nodes> {
    //     self.0.get_neighbours_mut(node)
    // }
    #[inline]
    fn remove_node(&mut self, node: int) {
        self.0.remove_node(node);
    }
    #[inline]
    fn retain_nodes(&mut self, f: impl Fn(int) -> bool) {
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
    fn iter_nodes(&self) -> Range<int> {
        self.0.iter_nodes()
    }
    #[inline]
    fn iter_labels(&self) -> impl Iterator<Item = int> + Clone {
        self.0.iter_labels()
    }
    #[inline]
    fn iter_labels_mut(&mut self) -> impl Iterator<Item = &mut int> {
        self.0.iter_labels_mut()
    }
    #[inline]
    fn iter_with_labels(&self) -> impl Iterator<Item = (int, int)> + Clone {
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
    ) -> impl Iterator<Item = (int, Self::Neighbours<'_>)> + Clone {
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
    type NodeId = int;
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
        a as usize
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        i.try_into().expect("index out of bounds")
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

    type Iter<'a>: Iterator<Item = int> + Clone
    where
        Self: 'a;

    fn contains(&self, e: int) -> bool;

    fn iter(&self) -> Self::Iter<'_>;

    fn len(&self) -> usize {
        self.iter_ref().count()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = int> + 'a {
        self.iter_ref().filter(|n| other.contains(*n))
    }

    fn collect(self) -> Self::Collected;
}

pub trait NodeCollectionRef {
    type Iter: Iterator<Item = int> + Clone;

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
    type Iter<'b> = T::Iter<'b> where Self: 'b;
    #[inline]
    fn contains(&self, e: int) -> bool {
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
    fn intersection<'b>(&'b self, other: &'b Self) -> impl Iterator<Item = int> + 'b {
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
    fn remove(&mut self, e: int) -> Option<int>;

    fn insert(&mut self, e: int);
}

impl NodeCollection for VNodes {
    type Collected = VNodes;
    type Iter<'a> = iter::Copied<slice::Iter<'a, int>>;
    fn contains(&self, e: int) -> bool {
        <[int]>::contains(self, &e)
    }
    fn iter(&self) -> Self::Iter<'_> {
        <[int]>::iter(self).copied()
    }
    fn len(&self) -> usize {
        <[int]>::len(self)
    }
    fn collect(self) -> Self::Collected {
        self
    }
}

impl NodeCollectionMut for VNodes {
    fn remove(&mut self, e: int) -> Option<int> {
        let len = self.len();
        if e >= len as int {
            return None;
        }
        // copy-paste code from Vec::swap_remove
        unsafe {
            let value = ptr::read(self.as_ptr().add(e as usize));
            let base_ptr = self.as_mut_ptr();
            ptr::copy(base_ptr.add(len - 1), base_ptr.add(e as usize), 1);
            self.set_len(len - 1);
            Some(value)
        }
    }
    fn insert(&mut self, e: int) {
        self.push(e);
    }
}

impl NodeCollection for HNodes {
    type Collected = HNodes;
    type Iter<'a> = Copied<hash_set::Iter<'a, int>>;
    fn contains(&self, e: int) -> bool {
        HashSet::contains(self, &e)
    }
    fn iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }
    fn len(&self) -> usize {
        HashSet::len(self)
    }
    fn intersection<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = int> + 'a {
        HashSet::intersection(self, other).copied()
    }
    fn collect(self) -> Self::Collected {
        self
    }
}
impl NodeCollectionMut for HNodes {
    fn remove(&mut self, e: int) -> Option<int> {
        HashSet::take(self, &e)
    }
    fn insert(&mut self, e: int) {
        HashSet::insert(self, e);
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::collections::{HashMap, HashSet};

    use rand::{seq::IteratorRandom, Rng, SeedableRng};
    use rand_pcg::Pcg64;

    use super::{int, HNodes, VNodeInfo, VNodes};

    #[derive(Debug, Clone)]
    pub enum RandomMap {
        Random(Vec<int>),
        Identity,
    }

    impl RandomMap {
        pub fn new(map_length: int, map_max: int, rng: &mut impl Rng) -> Self {
            assert!(map_max >= map_length);
            Self::Random((0..=map_max).choose_multiple(rng, map_length as usize + 1))
        }

        pub fn map(&self, node: int) -> int {
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

    pub fn adj_hash_hash(map: &RandomMap, list: Vec<VNodeInfo>) -> HashMap<int, HNodes> {
        adj_map!(map, list)
    }

    pub fn adj_hash_vec(map: &RandomMap, list: Vec<VNodeInfo>) -> HashMap<int, VNodes> {
        adj_map!(map, list)
    }

    pub fn adj_vec_hash(map: &RandomMap, list: Vec<VNodeInfo>) -> Vec<(int, HNodes)> {
        adj_map!(map, list)
    }

    pub fn adj_vec_vec(map: &RandomMap, list: Vec<VNodeInfo>) -> Vec<(int, VNodes)> {
        adj_map!(map, list)
    }

    // pub fn col_hash_hash(map: &RandomMap, list: Vec<VNodes>) -> HashSet<HNodes> {
    //     col_map!(map, list)
    // }

    pub fn col_hash_vec(map: &RandomMap, list: Vec<VNodes>) -> HashSet<VNodes> {
        col_map!(map, list)
    }

    pub fn col_vec_hash(map: &RandomMap, list: Vec<VNodes>) -> Vec<HNodes> {
        col_map!(map, list)
    }

    pub fn col_vec_vec(map: &RandomMap, list: Vec<VNodes>) -> Vec<VNodes> {
        col_map!(map, list)
    }

    macro_rules! col_map {
        ($map:expr, $list:expr) => {
            $list
                .into_iter()
                .map(|collection| {
                    collection.into_iter().map(|element| $map.map(element)).collect()
                })
                .collect()
        };
    }
    pub(crate) use col_map;

    macro_rules! edge_map {
        ($map:expr, $list:expr) => {
            $list
                .into_iter()
                .map(|(node, neighbour)| ($map.map(node), $map.map(neighbour)))
                .collect()
        };
    }

    pub fn edge_hash(map: &RandomMap, list: Vec<(int, int)>) -> HashSet<(int, int)> {
        edge_map!(map, list)
    }

    pub fn edge_vec(map: &RandomMap, list: Vec<(int, int)>) -> Vec<(int, int)> {
        edge_map!(map, list)
    }

    macro_rules! collect_adj {
        ($(($node:expr, [$($neighbor:expr),*]),)*) => {
            vec![$(($node, vec![$($neighbor),*]),)*]
        };
    }
    pub(crate) use collect_adj;

    macro_rules! collect_col {
        ($([$($neighbor:expr),*],)*) => {
            vec![$(vec![$($neighbor),*],)*]
        };
    }
    pub(crate) use collect_col;

    macro_rules! collect {
        (hh, $map:expr; $(($node:expr, $neighbours:tt),)*) => {
            $crate::graph::test_utils::adj_hash_hash(
                &$map, $crate::graph::test_utils::collect_adj!($(($node, $neighbours),)*)
            )
        };
        (hv, $map:expr; $(($node:expr, $neighbours:tt),)*) => {
            $crate::graph::test_utils::adj_hash_vec(
                &$map, $crate::graph::test_utils::collect_adj!($(($node, $neighbours),)*)
            )
        };
        (vh, $map:expr; $(($node:expr, $neighbours:tt),)*) => {
            $crate::graph::test_utils::adj_vec_hash(
                &$map, $crate::graph::test_utils::collect_adj!($(($node, $neighbours),)*)
            )
        };
        (vv, $map:expr; $(($node:expr, $neighbours:tt),)*) => {
            $crate::graph::test_utils::adj_vec_vec(
                &$map, $crate::graph::test_utils::collect_adj!($(($node, $neighbours),)*)
            )
        };
        (hh, $map:expr; $($collection:tt,)*) => {
            $crate::graph::test_utils::adj_hash_hash(
                &$map, $crate::graph::test_utils::collect_col!($($neighbours,)*)
            )
        };
        (hv, $map:expr; $($collection:tt,)*) => {
            $crate::graph::test_utils::col_hash_vec(
                &$map, $crate::graph::test_utils::collect_col!($($collection,)*)
            )
        };
        (vh, $map:expr; $($collection:tt,)*) => {
            $crate::graph::test_utils::col_vec_hash(
                &$map, $crate::graph::test_utils::collect_col!($($collection,)*)
            )
        };
        (vv, $map:expr; $($collection:tt,)*) => {
            $crate::graph::test_utils::col_vec_vec(
                &$map, $crate::graph::test_utils::collect_col!($($collection,)*)
            )
        };
        (h, $map:expr; $($edge:tt,)*) => {
            $crate::graph::test_utils::edge_hash(&$map, vec![$($edge,)*])
        };
        (v, $map:expr; $($edge:tt,)*) => {
            $crate::graph::test_utils::edge_vec(&$map, vec![$($edge,)*])
        };
        ($repr:tt; $($input:tt,)*) => {
            $crate::graph::test_utils::collect!(
                $repr, &$crate::graph::test_utils::RandomMap::Identity; $($input,)*
            )
        };
    }
    pub(crate) use collect;

    // just a naive test whether the utils compile, more or less
    #[test]
    fn macros() {
        assert_eq!(
            vec![(1, vec![2, 3]), (2, vec![1]), (3, vec![1])],
            collect!(vv; (1, [2, 3]), (2, [1]), (3, [1]),)
        );
        let map = RandomMap::new(10, 20, &mut Pcg64::from_entropy());
        assert_eq!(
            HashSet::from_iter(
                [(1, 2), (1, 3)].into_iter().map(|(a, b)| (map.map(a), map.map(b)))
            ),
            collect!(h, map; (1, 2), (1, 3),)
        );
    }
}

pub mod adj;
pub mod impl_petgraph;
// pub mod my_graph;
