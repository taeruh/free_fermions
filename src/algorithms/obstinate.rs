use std::borrow::Cow;

use crate::{
    fix_int::enumerate,
    graph::{Graph, ImplGraph, Node, NodeCollection, VNodes},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Obstinate {
    True(ObstinateKind, (VNodes, VNodes)),
    False,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstinateKind {
    Itself,
    Complement,
}

impl<G: ImplGraph + Clone> Graph<G> {
    // note that, if a graph is obstinate, then there are two expected results, since we
    // can swap a with b and in each part we then reverse the order of the vertices; this
    // algorithm does not guarantee which of the two results will be returned, since we
    // use unstable sorting in some places
    pub fn obstinate(&self) -> Obstinate {
        // directly alias self(: &Self) with graph, because we have to do it later anyways
        // when I put into into a Cow (so that we don't confuse self/graph (for
        // consistencty))
        let graph = self;

        let len = graph.len();
        if len == 0 {
            return Obstinate::True(ObstinateKind::Itself, (vec![], vec![]));
        }
        if len % 2 != 0 {
            return Obstinate::False;
        }
        let len2 = len / 2;

        let mut degrees = enumerate!(graph.iter_neighbourhoods())
            .map(|(vertex, neighbours)| (vertex, neighbours.len()))
            .collect::<Vec<_>>();
        degrees.sort_unstable_by_key(|(_, degree)| *degree);

        // we want the sequence
        // start, start, start + 1, start + 1, ... end, end
        // where start = 1|len2 - 1 and end = len2 - 1|len - 2
        fn check_degree_sequence(
            start: usize,
            end_inclusive: usize,
            degrees: &[(Node, usize)],
        ) -> bool {
            let mut iter_degrees = degrees.iter().map(|(_, degree)| *degree);
            for i in start..=end_inclusive {
                if i != iter_degrees.next().unwrap() {
                    return false;
                }
                if i != iter_degrees.next().unwrap() {
                    return false;
                }
            }
            true
        }

        // we only need to clone when ObstinateKind::Complement
        let mut graph = Cow::Borrowed(graph);

        let kind = if check_degree_sequence(1, len2, &degrees) {
            ObstinateKind::Itself
        } else if check_degree_sequence(len2 - 1, len - 2, &degrees) {
            graph.to_mut().complement();
            // no need to update the `degrees`, since we can get the right (a_end,
            // b_start) nodes with some basic logic below
            ObstinateKind::Complement
        } else {
            return Obstinate::False;
        };

        let (a_end, b_start) = if let ObstinateKind::Itself = kind {
            (degrees[len - 2].0, degrees[len - 1].0)
        } else {
            // it is Complement
            // (degree_complement = len2) <=> (degree = len2 - 1)
            // since len2 = len - (len2 - 1) - 1   (last -1 is for the node itself)
            (degrees[0].0, degrees[1].0)
        };
        let a_part = graph[b_start].clone();
        let b_part = graph[a_end].clone();

        if (a_part.intersection(&b_part).count() != 0)
            || !graph.set_is_independent(&a_part)
            || !graph.set_is_independent(&b_part)
        {
            return Obstinate::False;
        }

        // now we want the sequence 1, 2, ..., len2
        fn check_part_degree_sequence(
            start: usize,
            end_inclusive: usize,
            degrees_part: &[(Node, usize)],
        ) -> bool {
            let mut iter_degrees = degrees_part.iter().map(|(_, degree)| *degree);
            for i in start..=end_inclusive {
                if i != iter_degrees.next().unwrap() {
                    return false;
                }
            }
            true
        }

        let mut a_degrees = a_part
            .iter()
            .map(|vertex| (vertex, graph[vertex].len()))
            .collect::<Vec<_>>();
        a_degrees.sort_unstable_by_key(|(_, degree)| *degree);
        if !check_part_degree_sequence(1, len2, &a_degrees) {
            return Obstinate::False;
        }

        let mut b_degrees = b_part
            .iter()
            .map(|vertex| (vertex, graph[vertex].len()))
            .collect::<Vec<_>>();
        b_degrees.sort_unstable_by(|(_, degree1), (_, degree2)| degree2.cmp(degree1));
        if !check_part_degree_sequence(len2, 1, &b_degrees) {
            return Obstinate::False;
        }

        // finally we check the edges between the two parts of the bi-partition
        //
        // one could get rid of the loop in the final return below, by adding the logic
        // here, i.e, pushing the vertices to a vector, however, it is more likely that we
        // early return here, so it would be inefficient to allocate the vector in all
        // cases
        for (mut i, (a, _)) in a_degrees.iter().enumerate() {
            i += 1;
            for b in b_degrees.iter().take(i) {
                if !graph[*a].contains(b.0) {
                    return Obstinate::False;
                }
            }
            for b in b_degrees.iter().skip(i) {
                if graph[*a].contains(b.0) {
                    return Obstinate::False;
                }
            }
        }

        Obstinate::True(
            kind,
            (
                a_degrees.into_iter().map(|(vertex, _)| vertex).collect(),
                b_degrees.into_iter().map(|(vertex, _)| vertex).collect(),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fix_int::int, graph::Graph};

    mod utils {
        use super::*;
        use crate::{
            fix_int::int,
            graph::{
                test_utils::{self, RandomMap},
                VNodeInfo, VNodes,
            },
        };

        pub fn create_graph(
            map_length: int,
            map_max: int,
            list: Vec<VNodeInfo>,
            rng: &mut impl Rng,
        ) -> (Graph, RandomMap) {
            let map = RandomMap::new(map_length, map_max, rng);
            let graph =
                Graph::from_adjacency_labels(test_utils::adj_hash_hash(&map, list))
                    .unwrap();
            (graph, map)
        }

        pub fn create_expected(
            kind: ObstinateKind,
            a: VNodes,
            b: VNodes,
            map: RandomMap,
        ) -> [Obstinate; 2] /* [obstinate, complement obstinate] */ {
            // we will need to adjust the expected results to the randomized vertex labels
            fn adjust_expected(expected: Obstinate, map: &RandomMap) -> Obstinate {
                match expected {
                    Obstinate::True(kind, (mut a, mut b)) => {
                        a.iter_mut().for_each(|node| *node = map.map(*node));
                        b.iter_mut().for_each(|node| *node = map.map(*node));
                        Obstinate::True(kind, (a, b))
                    },
                    Obstinate::False => Obstinate::False,
                }
            }

            [
                adjust_expected(Obstinate::True(kind, (a.clone(), b.clone())), &map),
                adjust_expected(
                    Obstinate::True(
                        kind,
                        (b.into_iter().rev().collect(), a.into_iter().rev().collect()),
                    ),
                    &map,
                ),
            ]
        }

        macro_rules! graph {
            (
                $max_list:expr, $max_rand:expr, $rng:expr;
                $(($vertex:expr, [$($neighbor:expr),*]),)*
            ) => {
                create_graph(
                    $max_list, $max_rand, vec![$(($vertex, vec![$($neighbor),*]),)*], $rng
                )
            };
            (
                $max_list:expr, $max_rand:expr;
                $($adj_elem:tt,)*
            ) => {
                graph!($max_list, $max_rand, &mut Pcg64::from_entropy(); $($adj_elem,)*)
            };
        }
        pub(super) use graph;

        macro_rules! expected {
            (False) => { Obstinate::False };
            ($kind:ident, [$($a:expr),*], [$($b:expr),*], $map:expr) => {
                create_expected(ObstinateKind::$kind, vec![$($a),*], vec![$($b),*], $map)
            };
        }
        pub(super) use expected;
        use rand::Rng;
    }
    use rand::{Rng, SeedableRng};
    use rand_pcg::Pcg64;
    pub use utils::*;

    #[test]
    // separate test case for the empty graph because:
    // a) I don't want to introduce special logic in the loops in the true_all test
    // b) I'm not sure yet, whether we want the empty graph to be obstinate or not
    fn true_empty() {
        let (graph, map) = graph!(0, 0, &mut Pcg64::from_entropy(););
        assert_eq!(graph.obstinate(), expected!(Itself, [], [], map)[0]);
    }

    #[test]
    // check all (co-)obstinate graphs (except the empty one) up to MAX vertices (up to
    // isomorphisms)
    fn true_all() {
        const MAX: int = 10;

        // the testing logic for two bi-partitions that are obstinate
        fn test(
            partition_size: int,
            kind: ObstinateKind,
            a_part_full: Vec<(Node, VNodes)>,
            b_part_full: Vec<(Node, VNodes)>,
            rng: &mut impl Rng,
        ) {
            let size = partition_size * 2 - 1;

            let mut list: Vec<(Node, VNodes)> = Vec::with_capacity(size as usize);
            let mut a_part: VNodes = Vec::with_capacity(partition_size as usize);
            let mut b_part: VNodes = Vec::with_capacity(partition_size as usize);
            for (a, b) in a_part_full.into_iter().zip(b_part_full.into_iter()) {
                a_part.push(a.0);
                b_part.push(b.0);
                list.push(a);
                list.push(b);
            }

            let (graph, map) = create_graph(size, size + 42, list, rng);
            let mut result = graph.obstinate();
            if let Obstinate::True(_, (a, b)) = &mut result {
                a.iter_mut().for_each(|node| *node = graph.get_label(*node).unwrap());
                b.iter_mut().for_each(|node| *node = graph.get_label(*node).unwrap());
            }
            let expected = create_expected(kind, a_part, b_part, map);
            if !expected.contains(&result) {
                panic!(
                    "expected:\n{:?} or\n{:?}\nbut got:\n{:?}",
                    expected[0], expected[1], result
                );
            }
        }

        let rng = &mut Pcg64::from_entropy();

        // create the obstinate bi-partitions; co_* is for the cases when the complement
        // of the graph is obstinate
        for part_size in 1..=MAX {
            let size = 2 * part_size;
            let mut a_part = Vec::with_capacity(part_size as usize);
            let mut b_part = Vec::with_capacity(part_size as usize);
            let mut co_a_part = Vec::with_capacity(part_size as usize);
            let mut co_b_part = Vec::with_capacity(part_size as usize);
            for i in 0..part_size {
                a_part.push((2 * i, (0..=i).map(|j| 2 * j + 1).collect()));
                b_part.push((2 * i + 1, (i..part_size).map(|j| 2 * j).collect()));
                let mut co_a_neighbourhood = Vec::with_capacity((size - 1 - i) as usize);
                let mut co_b_neighbourhood =
                    Vec::with_capacity((size - 1 - (part_size - i)) as usize);
                for j in 0..i {
                    co_a_neighbourhood.push(2 * j);
                    co_b_neighbourhood.push(2 * j + 1);
                    co_b_neighbourhood.push(2 * j);
                }
                for j in i + 1..part_size {
                    co_a_neighbourhood.push(2 * j);
                    co_b_neighbourhood.push(2 * j + 1);
                    co_a_neighbourhood.push(2 * j + 1);
                }
                co_a_part.push((2 * i, co_a_neighbourhood));
                co_b_part.push((2 * i + 1, co_b_neighbourhood));
            }

            test(part_size, ObstinateKind::Itself, a_part, b_part, rng);
            if part_size != 2 {
                test(part_size, ObstinateKind::Complement, co_a_part, co_b_part, rng);
            } else {
                // in that case, the graph itself is obstinate (as well as the
                // complement), but our algorithm goes down the Itself path, so we wont
                // get the result that the complement is obstinate; the pop order is
                // important here!
                let b_part = vec![co_b_part.pop().unwrap(), co_a_part.pop().unwrap()];
                let a_part = vec![co_b_part.pop().unwrap(), co_a_part.pop().unwrap()];
                test(part_size, ObstinateKind::Itself, a_part, b_part, rng);
            }
        }
    }

    #[test]
    // no need to do many tests for that, since this check is very simple and we just
    // ensure that it is there
    fn false_odd() {
        let (graph, _) = graph!(2, 2; (0, [1]), (1, [0]), (2, []),);
        assert_eq!(graph.obstinate(), Obstinate::False);
    }

    #[test]
    // only test graphs that have an even number of vertices
    fn false_other() {
        let rng = &mut Pcg64::from_entropy();

        // cycle
        let (graph, _) =
            graph!(3, 7, rng; (0, [3, 1]), (1, [0, 2]), (2, [1, 3]), (3, [2, 0]),);
        assert_eq!(graph.obstinate(), Obstinate::False);

        // same as above but with one additional edge
        let (graph, _) =
            graph!(3, 7, rng; (0, [3, 1, 2]), (1, [0, 2]), (2, [1, 3, 0]), (3, [2, 0]),);
        assert_eq!(graph.obstinate(), Obstinate::False);

        // all-to-all
        let (graph, _) = graph!(3, 7, rng; (0, [1, 2, 3]), (1, [0, 2, 3]), (2, [0, 1, 3]), (3, [0, 1, 2]),);
        assert_eq!(graph.obstinate(), Obstinate::False);

        // completely independent
        let (graph, _) = graph!(3, 7, rng; (0, []), (1, []), (2, []), (3, []),);
        assert_eq!(graph.obstinate(), Obstinate::False);

        // two disconnected paths
        let (graph, _) = graph!(3, 7, rng; (0, [1]), (1, [0]), (2, [3]), (3, [2]),);
        assert_eq!(graph.obstinate(), Obstinate::False);

        // TODO: more negative tests
    }
}
