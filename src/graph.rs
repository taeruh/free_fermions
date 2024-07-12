use std::collections::HashSet;

use crate::fix_int::int;

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

pub mod generic;
pub mod specialised;

#[cfg(test)]
pub mod test_utils {

    use std::collections::HashMap;

    use rand::{seq::IteratorRandom, Rng, SeedableRng};
    use rand_pcg::Pcg64;

    use super::*;

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
