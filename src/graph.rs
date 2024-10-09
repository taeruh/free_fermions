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

pub mod algorithms;
pub mod generic;
pub mod specialised;

#[cfg(test)]
pub mod test_utils {

    use hashbrown::HashMap;
    use rand::{Rng, SeedableRng, seq::IteratorRandom};
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
