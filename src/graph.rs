// use std::collections::HashSet;
use hashbrown::HashSet;

use crate::fix_int::int;

// some of the following type aliases are not used, but they serve as documentation and
// orientation for variable names

pub type Node = usize;
pub type Edge = (Node, Node);
pub type Label = int; // i.e, the weight which is in our case just the label
pub type LabelEdge = (Label, Label); // i.e, the weight which is in our case just the label

// V for vector
pub(crate) type VNodes = Vec<Node>;
pub(crate) type VLabels = Vec<Label>;
#[allow(unused)]
pub(crate) type VNodeInfo = (Node, Vec<Node>);
#[allow(unused)]
pub(crate) type VLabelInfo = (Label, Vec<Label>);
// H for hash
pub(crate) type HNodes = HashSet<Node>;
#[allow(unused)]
pub(crate) type HNodeInfo = (Node, HashSet<Node>);
pub(crate) type HLabels = HashSet<Label>;
#[allow(unused)]
pub(crate) type HLabelInfo = (Label, HashSet<Label>);

/// Marker trait, that promises that the nodes in the graph go from 0 to n-1 without
/// skipping any values; and when a node is removed, its index place is reused by the last
/// node (i.e., swap_removed)
pub trait CompactNodes {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum InvalidGraph<T> {
    #[error("Self loop detected on node {0}")]
    SelfLoop(T),
    #[error("Incompatible neighbourhoods between the nodes {0} and {1}")]
    IncompatibleNeighbourhoods(T, T),
}

impl InvalidGraph<Node> {
    pub fn map(&self, map: impl Fn(Node) -> Label) -> InvalidGraph<Label> {
        match self {
            Self::SelfLoop(node) => InvalidGraph::SelfLoop(map(*node)),
            Self::IncompatibleNeighbourhoods(node, neighbour) => {
                InvalidGraph::IncompatibleNeighbourhoods(map(*node), map(*neighbour))
            },
        }
    }
}

/// A helper to keep track of swap-removals. Basically has to be used when some nodes to
/// remove are fixed and then we iterate over them and remove them one by one (cf. default
/// implemenation of retain nodes).
#[derive(Clone, Debug)]
pub struct SwapRemoveMap {
    map: Vec<Node>,
    position: Vec<Node>,
    len: usize,
}

impl SwapRemoveMap {
    #[inline]
    /// # Safety
    /// The `len` must be greater than 0.
    pub fn new_unchecked(len: usize) -> Self {
        debug_assert!(len > 0);
        let position: Vec<_> = (0..len).collect();
        Self {
            map: position.clone(),
            position,
            len,
        }
    }

    #[inline]
    pub fn new(len: usize) -> Self {
        assert!(len > 0);
        Self::new_unchecked(len)
    }

    /// # Safety
    /// The `node` must be in bounds, i.e., less than the `len` initialiser.
    #[inline(always)]
    pub unsafe fn map_unchecked(&self, node: Node) -> Node {
        debug_assert!(node < self.map.len(), "node: {}, map: {:?}", node, self.map);
        unsafe { *self.map.get_unchecked(node) }
    }

    #[inline]
    pub fn map(&self, node: Node) -> Node {
        assert!(node < self.map.len());
        unsafe { self.map_unchecked(node) }
    }

    // _Statement_:
    /// Let a = (a_1, ldots, a_m) subset {1, ldots, n-1} be the set of pairwise different
    /// elements we want to swap_remove in some ordered list b = \[b_i, ldots, b_{n-1}\]
    /// (m leq n-1). Then the following holds: c_i = self_{i-1}.swap_remove_unchecked(a_i)
    /// is the right element to swap_remove. More specifically, self.map(j) returns the
    /// position of b_j in b for all i,j in {1, ldots, m}.
    // Furthermore, the elements of self.position[..n-i] mirrors the indices of the
    // elements in b.
    // _Proof_:
    // We prove it via induction for all i in {1, ldots, m}. The case i = 1 is clear: it
    // is c_i = a_i which is correct; we updated self.map, so that n-1 is mapped to c_i;
    // and in self.position[..n-1], c_i contains n-1. Now let the statement hold for i-1.
    // Then we know that c_i is the right element to swap_remove. We then get the index j
    // of b_j which is currently at the end of b (via self.position[self.len], which
    // holds via induction). b_j will be put in the position c_i, so we update
    // self.map[j] = c_i. Finally, we update self_position[c_i] = j, which mirrors the
    // actual swap_remove in b.
    ///
    /// # Safety
    /// The `node` must be "valid", that is, it must be `node < len` where `len` was the
    /// argument during initialisation of `self`, and previous calls to
    /// `swap_remove(_unchecked)` must not have had `node` as their argument.
    // The last requirement is actually stricter than necessary, but it enforces correct
    // use (we actually require self.len > 0)
    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, node: Node) -> Node {
        #[cfg(not(debug_assertions))]
        unsafe {
            // safety: every node can only be removed once, according to the safety
            // invariant, so self.len > 0
            self.len = self.len.unchecked_sub(1);
            let mapped = self.map_unchecked(node);
            // safety: position was initialised to have more then `len` elements and we
            // never remove elements from it
            let position_last = *self.position.get_unchecked(self.len);
            // safety: position_last is less then n and we never remove anything from
            // self.map
            *self.map.get_unchecked_mut(position_last) = mapped;
            // safety: mapped is less then n and we never remove anything from
            // self.position
            *self.position.get_unchecked_mut(mapped) = position_last;
            mapped
        }
        #[cfg(debug_assertions)]
        {
            self.len -= 1;
            let mapped = self.map[node];
            let position_last = self.position[self.len];
            self.map[position_last] = mapped;
            self.position[mapped] = position_last;
            mapped
        }
    }

    pub fn swap_remove(&mut self, node: Node) -> Node {
        assert!(node < self.map.len());
        assert!(self.len > 0);
        unsafe { self.swap_remove_unchecked(node) }
    }
}

pub mod algorithms;
pub mod generic;
pub mod specialised;

#[cfg(test)]
pub mod test_utils {

    use hashbrown::HashMap;
    use rand::{Rng, SeedableRng, distributions::Uniform, seq::IteratorRandom};
    use rand_pcg::Pcg64;

    use super::*;

    #[derive(Debug, Clone)]
    pub enum RandomMap {
        Random(Vec<Label>),
        Identity,
    }

    impl RandomMap {
        pub fn with_rng(map_length: int, map_max: int, rng: &mut impl Rng) -> Self {
            assert!(map_max >= map_length);
            // + 1 because often I accidentally use n-1
            Self::Random((0..=map_max).choose_multiple(rng, map_length as usize + 1))
        }

        pub fn new(map_length: int, map_max: int) -> Self {
            let mut rng = Pcg64::from_entropy();
            Self::with_rng(map_length, map_max, &mut rng)
        }

        pub fn map(&self, label: Label) -> Label {
            match self {
                RandomMap::Random(v) => v[label as usize],
                RandomMap::Identity => label,
            }
        }
    }

    pub fn random_data(
        rng: &mut impl Rng,
        num_nodes: int,
        num_edges: int, // not really, since duplicates are dropped
    ) -> HashMap<Label, HLabels> {
        assert!(num_nodes >= 1);
        if num_nodes == 1 {
            return HashMap::from_iter([(0, HLabels::new())]);
        }
        let map = RandomMap::with_rng(num_nodes, num_nodes * 5, rng);
        let dist = Uniform::new(0, num_nodes);
        let mut ret =
            HashMap::from_iter((0..num_nodes).map(|i| (map.map(i), HLabels::new())));
        for _ in 0..num_edges {
            loop {
                let (a, b) = (rng.sample(dist), rng.sample(dist));
                if a != b {
                    ret.get_mut(&map.map(a)).unwrap().insert(map.map(b));
                    ret.get_mut(&map.map(b)).unwrap().insert(map.map(a));
                    break;
                }
            }
        }
        ret
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

    pub fn adj_hash_hash(
        map: &RandomMap,
        list: Vec<VLabelInfo>,
    ) -> HashMap<Label, HLabels> {
        adj_map!(map, list)
    }

    pub fn adj_hash_vec(
        map: &RandomMap,
        list: Vec<VLabelInfo>,
    ) -> HashMap<Label, VLabels> {
        adj_map!(map, list)
    }

    pub fn adj_vec_hash(map: &RandomMap, list: Vec<VLabelInfo>) -> Vec<(Label, HLabels)> {
        adj_map!(map, list)
    }

    pub fn adj_vec_vec(map: &RandomMap, list: Vec<VLabelInfo>) -> Vec<(Label, VLabels)> {
        adj_map!(map, list)
    }

    pub fn col_hash_vec(map: &RandomMap, list: Vec<VLabels>) -> HashSet<VLabels> {
        col_map!(map, list)
    }

    pub fn col_vec_hash(map: &RandomMap, list: Vec<VLabels>) -> Vec<HLabels> {
        col_map!(map, list)
    }

    pub fn col_vec_vec(map: &RandomMap, list: Vec<VLabels>) -> Vec<VLabels> {
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

    pub fn edge_hash(map: &RandomMap, list: Vec<LabelEdge>) -> HashSet<LabelEdge> {
        edge_map!(map, list)
    }

    pub fn edge_vec(map: &RandomMap, list: Vec<LabelEdge>) -> Vec<LabelEdge> {
        edge_map!(map, list)
    }

    macro_rules! collect_adj {
        ($(($node:expr, [$($neighbor:expr),*]),)*) => {
            vec![$(($node, vec![$($neighbor),*]),)*]
        };
    }
    pub(crate) use collect_adj;

    #[allow(unused)]
    macro_rules! collect_col {
        ($([$($neighbor:expr),*],)*) => {
            vec![$(vec![$($neighbor),*],)*]
        };
    }
    #[allow(unused)]
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
        ($repr:tt, $map_length:expr, $map_max:expr, $rng:expr; $($input:tt,)*) => {
            $crate::graph::test_utils::collect!(
                $repr, &$crate::graph::test_utils::RandomMap::with_rng(
                    $map_length, $map_max, &mut $rng
                ); $($input,)*
            )
        };
        ($repr:tt, $map_length:expr, $map_max:expr; $($input:tt,)*) => {
            $crate::graph::test_utils::collect!(
                $repr, &$crate::graph::test_utils::RandomMap::new($map_length, $map_max);
                $($input,)*
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
        let map = RandomMap::with_rng(10, 20, &mut Pcg64::from_entropy());
        assert_eq!(
            HashSet::from_iter(
                [(1, 2), (1, 3)].into_iter().map(|(a, b)| (map.map(a), map.map(b)))
            ),
            collect!(h, map; (1, 2), (1, 3),)
        );
    }
}

#[cfg(test)]
mod tests {
    use rand::{SeedableRng, seq::SliceRandom};
    use rand_pcg::Pcg64;

    use super::*;

    #[test]
    fn swap_remove() {
        const NUM_NODES: usize = 3000;
        const NUM_REMOVE: usize = NUM_NODES / 2;
        let mut rng = Pcg64::from_entropy();

        let mut pseudo_graph = (0..NUM_NODES).collect::<Vec<_>>();
        let to_remove = pseudo_graph
            .choose_multiple(&mut rng, NUM_REMOVE)
            .copied()
            .collect::<Vec<usize>>();
        let mut map = SwapRemoveMap::new(NUM_NODES);

        for node in to_remove.into_iter() {
            let removed = pseudo_graph.swap_remove(map.swap_remove(node));
            assert_eq!(removed, node);
        }
    }
}
