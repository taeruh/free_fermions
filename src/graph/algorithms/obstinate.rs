use crate::graph::{Label, Node, VLabels, VNodes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Obstinate {
    True(ObstinateKind, (VNodes, VNodes)),
    False,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObstinateMapped {
    True(ObstinateKind, (VLabels, VLabels)),
    False,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstinateKind {
    Itself,
    Complement,
}

impl Obstinate {
    pub fn map(&self, map: impl Fn(Node) -> Label) -> ObstinateMapped {
        match self {
            Obstinate::True(kind, (a, b)) => ObstinateMapped::True(
                *kind,
                (
                    a.iter().map(|&n| map(n)).collect(),
                    b.iter().map(|&n| map(n)).collect(),
                ),
            ),
            Obstinate::False => ObstinateMapped::False,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::fmt::Debug;

    use hashbrown::HashMap;
    use rand::SeedableRng;
    use rand_pcg::Pcg64;
    use utils::{ObstinateGraphInfo, ObstinateLists};

    use super::*;
    use crate::{
        fix_int::int,
        graph::{
            algorithms::{obstinate::ObstinateKind, test_impls::RequiredMethods},
            test_utils::collect,
        },
    };

    mod utils {
        use rand::Rng;

        use super::*;
        use crate::{
            fix_int::int,
            graph::{
                test_utils::{self, RandomMap},
                VLabelInfo, VLabels,
            },
        };

        pub struct ObstinateGraphInfo {
            pub info: ObstinateInfo,
            pub co_info: ObstinateInfo,
        }

        pub struct ObstinateInfo {
            pub a: Vec<VLabelInfo>,
            pub b: Vec<VLabelInfo>,
        }

        #[derive(Debug)]
        pub struct ObstinateLists {
            pub a_partition: VLabels,
            pub b_partition: VLabels,
            pub graph_list: Vec<VLabelInfo>,
        }

        impl ObstinateGraphInfo {
            pub fn new(partition_size: u32) -> Self {
                let size = 2 * partition_size;
                let mut a = Vec::with_capacity(partition_size as usize);
                let mut b = Vec::with_capacity(partition_size as usize);
                let mut co_a = Vec::with_capacity(partition_size as usize);
                let mut co_b = Vec::with_capacity(partition_size as usize);
                for i in 0..partition_size {
                    a.push((2 * i, (0..=i).map(|j| 2 * j + 1).collect()));
                    b.push((2 * i + 1, (i..partition_size).map(|j| 2 * j).collect()));
                    let mut co_a_neighbourhood = Vec::with_capacity((size - 1 - i) as usize);
                    let mut co_b_neighbourhood =
                        Vec::with_capacity((size - 1 - (partition_size - i)) as usize);
                    for j in 0..i {
                        co_a_neighbourhood.push(2 * j);
                        co_b_neighbourhood.push(2 * j + 1);
                        co_b_neighbourhood.push(2 * j);
                    }
                    for j in i + 1..partition_size {
                        co_a_neighbourhood.push(2 * j);
                        co_b_neighbourhood.push(2 * j + 1);
                        co_a_neighbourhood.push(2 * j + 1);
                    }
                    co_a.push((2 * i, co_a_neighbourhood));
                    co_b.push((2 * i + 1, co_b_neighbourhood));
                }
                Self {
                    info: ObstinateInfo { a, b },
                    co_info: ObstinateInfo { a: co_a, b: co_b },
                }
            }
        }

        impl ObstinateInfo {
            pub fn into_lists(self) -> ObstinateLists {
                let partition_size = self.a.len();
                let mut a_partition = Vec::with_capacity(partition_size);
                let mut b_partition = Vec::with_capacity(partition_size);
                let mut graph_list = Vec::with_capacity(partition_size * 2);
                for (a, b) in self.a.into_iter().zip(self.b.into_iter()) {
                    a_partition.push(a.0);
                    b_partition.push(b.0);
                    graph_list.push(a);
                    graph_list.push(b);
                }
                ObstinateLists {
                    a_partition,
                    b_partition,
                    graph_list,
                }
            }
        }

        impl ObstinateLists {
            pub fn into_graph_and_expected<G: RequiredMethods>(
                self,
                map_length: int,
                map_max: int,
                rng: &mut impl Rng,
                kind: ObstinateKind,
            ) -> (G, [ObstinateMapped; 2]) {
                let map = RandomMap::with_rng(map_length, map_max, rng);

                fn create_expected(
                    kind: ObstinateKind,
                    a: VLabels,
                    b: VLabels,
                    map: RandomMap,
                ) -> [ObstinateMapped; 2] /* [obstinate, complement obstinate] */ {
                    // we will need to adjust the expected results to the randomized vertex
                    // labels
                    fn adjust_expected(
                        expected: ObstinateMapped,
                        map: &RandomMap,
                    ) -> ObstinateMapped {
                        match expected {
                            ObstinateMapped::True(kind, (mut a, mut b)) => {
                                a.iter_mut().for_each(|node| *node = map.map(*node));
                                b.iter_mut().for_each(|node| *node = map.map(*node));
                                ObstinateMapped::True(kind, (a, b))
                            }
                            ObstinateMapped::False => ObstinateMapped::False,
                        }
                    }
                    [
                        adjust_expected(ObstinateMapped::True(kind, (a.clone(), b.clone())), &map),
                        adjust_expected(
                            ObstinateMapped::True(
                                kind,
                                (b.into_iter().rev().collect(), a.into_iter().rev().collect()),
                            ),
                            &map,
                        ),
                    ]
                }

                (
                    G::from_adj_list(test_utils::adj_hash_hash(&map, self.graph_list)),
                    create_expected(kind, self.a_partition, self.b_partition, map),
                )
            }
        }
    }

    // separate test case for the empty graph because:
    // a) I don't want to introduce special logic in the loops in the test_positive test
    // b) I'm not sure yet, whether we want the empty graph to be obstinate or not
    pub fn empty<G: RequiredMethods>() {
        let graph = G::from_adj_list(HashMap::new());
        assert_eq!(
            graph.obstinate(),
            ObstinateMapped::True(ObstinateKind::Itself, (vec![], vec![]))
        );
    }

    // check all (co-)obstinate graphs (except the empty one) up to MAX vertices (up to
    // isomorphisms)
    pub fn test_positive<G: RequiredMethods>() {
        const MAX: int = 10;
        let rng = &mut Pcg64::from_entropy();
        let mut test = |lists: ObstinateLists, size, kind| {
            let (graph, expected): (G, _) =
                lists.into_graph_and_expected(2 * size + 1, 2 * size + 42, rng, kind);
            let result = graph.obstinate();
            if !expected.contains(&result) {
                panic!(
                    "expected:\n{:?} or\n{:?}\nbut got:\n{:?}",
                    expected[0], expected[1], result
                );
            }
        };
        for size in 1..=MAX {
            let info = ObstinateGraphInfo::new(size);
            test(info.info.into_lists(), size, ObstinateKind::Itself);
            if size != 2 {
                test(info.co_info.into_lists(), size, ObstinateKind::Complement);
            } else {
                // in that case, the graph itself is obstinate (as well as the
                // complement), but our algorithm goes down the Itself path, so we wont
                // get the result that the complement is obstinate; the pop order is
                // important here!
                let mut lists = info.co_info.into_lists();
                let b = vec![
                    lists.b_partition.pop().unwrap(),
                    lists.a_partition.pop().unwrap(),
                ];
                let a = vec![
                    lists.b_partition.pop().unwrap(),
                    lists.a_partition.pop().unwrap(),
                ];
                lists.a_partition = a;
                lists.b_partition = b;
                test(lists, size, ObstinateKind::Itself);
            }
        }
    }

    // there should be an early fail in the algorithm for that case
    pub fn false_odd<G: RequiredMethods>() {
        let graph = G::from_adj_list(collect!(hh; (0, [1]), (1, [0]), (2, []),));
        assert_eq!(graph.obstinate(), ObstinateMapped::False,);
    }

    pub fn false_cycle<G: RequiredMethods>() {
        let graph = G::from_adj_list(
            collect!(hh, 4, 7; (0, [3, 1]), (1, [0, 2]), (2, [1, 3]), (3, [2, 0]),),
        );
        assert_eq!(graph.obstinate(), ObstinateMapped::False,);
    }

    pub fn false_cycle_extra_edge<G: RequiredMethods>() {
        let graph = G::from_adj_list(
            collect!(hh, 4, 72; (0, [3, 1, 2]), (1, [0, 2]), (2, [1, 3, 0]), (3, [2, 0]),),
        );
        assert_eq!(graph.obstinate(), ObstinateMapped::False,);
    }

    pub fn false_all_to_all<G: RequiredMethods>() {
        let graph = G::from_adj_list(collect!(hh, 4, 15;
                (0, [1, 2, 3]), (1, [0, 2, 3]), (2, [0, 1, 3]), (3, [0, 1, 2]),));
        assert_eq!(graph.obstinate(), ObstinateMapped::False,);
    }

    pub fn false_completely_independent<G: RequiredMethods>() {
        let graph = G::from_adj_list(collect!(hh, 4, 57; (0, []), (1, []), (2, []), (3, []),));
        assert_eq!(graph.obstinate(), ObstinateMapped::False,);
    }

    pub fn false_disconnect_paths<G: RequiredMethods>() {
        let graph = G::from_adj_list(collect!(hh, 4, 55; (0, [1]), (1, [0]), (2, [3]), (3, [2]),));
        assert_eq!(graph.obstinate(), ObstinateMapped::False,);
    }

    // TODO: more negative tests

    macro_rules! test_it {
        ($module:ident, $typ:ty) => {
            mod $module {
                use super::*;
                crate::graph::algorithms::obstinate::tests::wrap!(
                    $typ,
                    empty,
                    test_positive,
                    false_odd,
                    false_cycle,
                    false_cycle_extra_edge,
                    false_all_to_all,
                    false_completely_independent,
                    false_disconnect_paths,
                );
            }
        };
    }
    pub(crate) use test_it;

    macro_rules! wrap {
        ($typ:ty, $($fun:ident,)*) => {
            $(
                #[test]
                fn $fun() {
                    crate::graph::algorithms::obstinate::tests::$fun::<$typ>();
                }
            )*
        };
    }
    pub(crate) use wrap;
}
