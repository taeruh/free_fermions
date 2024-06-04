use std::collections::HashSet;

use crate::graph::{SGraph, SVertices};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Obstinate {
    True(ObstinateKind, (Vec<usize>, Vec<usize>)),
    False,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstinateKind {
    Itself,
    Complement,
}

// note that, if a graph is obstinate, then there are two expected results, since we can
// swap a with b and in each part we then reverse the order of the vertices; this
// algorithm does not guarantee which of the two results will be returned, since we use
// unstable sorting in some places
pub fn obstinate(mut graph: SGraph) -> Obstinate {
    let len = graph.nodes.len();
    if len == 0 {
        return Obstinate::True(ObstinateKind::Itself, (vec![], vec![]));
    }
    if len % 2 != 0 {
        return Obstinate::False;
    }
    let len2 = len / 2;

    let mut degrees = graph
        .nodes
        .iter()
        .map(|(vertex, neighbours)| (*vertex, neighbours.len()))
        .collect::<Vec<_>>();
    degrees.sort_unstable_by_key(|(_, degree)| *degree);

    // we want the sequence
    // start, start, start + 1, start + 1, ... end, end
    // where start = 1|len2 - 1 and end = len2 - 1|len - 2
    fn check_degree_sequence(
        start: usize,
        end_inclusive: usize,
        degrees: &[(usize, usize)],
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

    let kind = if check_degree_sequence(1, len2, &degrees) {
        ObstinateKind::Itself
    } else if check_degree_sequence(len2 - 1, len - 2, &degrees) {
        graph.complement();
        // no need to update the `degrees`, since we can get the right (a_end, b_start)
        // nodes with some basic logic below
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
    let a_part = graph.nodes[&b_start].clone();
    let b_part = graph.nodes[&a_end].clone();

    fn is_independent(graph: &SGraph, subset: &SVertices) -> bool {
        let subset = subset.iter().collect::<Vec<_>>();
        // TODO: use my Enumerate here
        for i in 0..subset.len() {
            for j in i + 1..subset.len() {
                if graph.nodes[subset[i]].contains(subset[j]) {
                    return false;
                }
            }
        }
        true
    }

    if (a_part.intersection(&b_part).count() != 0)
        || !is_independent(&graph, &a_part)
        || !is_independent(&graph, &b_part)
    {
        return Obstinate::False;
    }

    // now we want the sequence 1, 2, ..., len2
    fn check_part_degree_sequence(
        start: usize,
        end_inclusive: usize,
        degrees_part: &[(usize, usize)],
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
        .map(|vertex| (*vertex, graph.nodes[vertex].len()))
        .collect::<Vec<_>>();
    a_degrees.sort_unstable_by_key(|(_, degree)| *degree);
    if !check_part_degree_sequence(1, len2, &a_degrees) {
        return Obstinate::False;
    }

    let mut b_degrees = b_part
        .iter()
        .map(|vertex| (*vertex, graph.nodes[vertex].len()))
        .collect::<Vec<_>>();
    b_degrees.sort_unstable_by(|(_, degree1), (_, degree2)| degree2.cmp(degree1));
    if !check_part_degree_sequence(len2, 1, &b_degrees) {
        return Obstinate::False;
    }

    // finally we check the edges between the two parts of the bi-partition
    //
    // one could get rid of the loop in the final return below, by adding the logic here,
    // i.e, pushing the vertices to a vector, however, it is more likely that we early
    // return here, so it would be inefficient to allocate the vector in all cases
    for (mut i, (a, _)) in a_degrees.iter().enumerate() {
        i += 1;
        for b in b_degrees.iter().take(i) {
            if !graph.nodes[a].contains(&b.0) {
                return Obstinate::False;
            }
        }
        for b in b_degrees.iter().skip(i) {
            if graph.nodes[a].contains(&b.0) {
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

#[cfg(test)]
mod tests {
    use rand::{seq::IteratorRandom, SeedableRng};
    use rand_pcg::Pcg64;

    use super::*;
    use crate::graph::{Nodes, Vertices};

    // we will randomize vertex labels, so that we can always use simple vertex labels,
    // i.e., 0, 1, 2, ..., when creating the examples, but we still check that the
    // algorithm does not depend on structured vertex labels accidentally
    fn randomize_labels(
        max_list: usize,
        max_rand: usize,
        mut list: Vec<(usize, Vec<usize>)>,
    ) -> (Vec<(usize, Vec<usize>)>, Vec<usize>) {
        let mut rng = Pcg64::from_entropy();
        // let mut rng = Pcg64::seed_from_u64(42);
        let map = (0..=max_rand).choose_multiple(&mut rng, max_list + 1);
        // let map = (0..=max_rand).collect::<Vec<_>>();

        for (vertex, neighbours) in list.iter_mut() {
            *vertex = map[*vertex];
            for neighbor in neighbours.iter_mut() {
                *neighbor = map[*neighbor];
            }
        }

        (list, map)
    }

    // we will need to adjust the expected results to the randomized vertex labels
    fn adjust_expected(expected: Obstinate, map: &[usize]) -> Obstinate {
        match expected {
            Obstinate::True(kind, (mut a, mut b)) => {
                a.iter_mut().for_each(|vertex| *vertex = map[*vertex]);
                b.iter_mut().for_each(|vertex| *vertex = map[*vertex]);
                Obstinate::True(kind, (a, b))
            },
            Obstinate::False => Obstinate::False,
        }
    }

    fn create_graph(
        max_list: usize,
        max_rand: usize,
        list: Vec<(usize, Vec<usize>)>,
    ) -> (SGraph, Vec<usize>) {
        let (graph, map) = randomize_labels(max_list, max_rand, list);
        (SGraph::from_iter(graph), map)
    }

    fn create_expected(
        kind: ObstinateKind,
        a: Vec<usize>,
        b: Vec<usize>,
        map: Vec<usize>,
    ) -> [Obstinate; 2] {
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

    macro_rules! create_graph {
        (
            $max_list:expr, $max_rand:expr,
            $(($vertex:expr, [$($neighbor:expr),*]),)*
        ) => {
            create_graph($max_list, $max_rand, vec![$(($vertex, vec![$($neighbor),*]),)*])
        };
    }

    macro_rules! create_expected {
        (False) => { Obstinate::False };
        ($kind:ident, [$($a:expr),*], [$($b:expr),*], $map:expr) => {
            create_expected(ObstinateKind::$kind, vec![$($a),*], vec![$($b),*], $map)
        };
    }

    #[test]
    // separate test case for the empty graph because:
    // a) I don't want to introduce special logic in the loops in the true_all test
    // b) I'm not sure yet, whether we want the empty graph to be obstinate or not
    fn true_empty() {
        let (graph, map) = create_graph!(0, 0,);
        assert_eq!(obstinate(graph), create_expected!(Itself, [], [], map)[0]);
    }

    #[test]
    // check all (co-)obstinate graphs (except the empty one) up to MAX vertices (up to
    // isomorphisms)
    fn true_all() {
        const MAX: usize = 10;

        // the testing logic, when two bi-partitions are given that are obstinate
        fn test(
            partition_size: usize,
            kind: ObstinateKind,
            a_part_full: Nodes,
            b_part_full: Nodes,
        ) {
            let size = partition_size * 2 - 1;

            let mut list: Nodes = Vec::with_capacity(size);
            let mut a_part: Vertices = Vec::with_capacity(partition_size);
            let mut b_part: Vertices = Vec::with_capacity(partition_size);
            for (a, b) in a_part_full.into_iter().zip(b_part_full.into_iter()) {
                a_part.push(a.0);
                b_part.push(b.0);
                list.push(a);
                list.push(b);
            }

            let (graph, map) = create_graph(size, size + 42, list);
            let result = obstinate(graph);
            let expected = create_expected(kind, a_part, b_part, map);
            if !expected.contains(&result) {
                panic!(
                    "expected:\n{:?} or\n{:?}\nbut got:\n{:?}",
                    expected[0], expected[1], result
                );
            }
        }

        // create the obstinate bi-partitions; co_* is for the cases when the complement
        // of the graph is obstinate
        for part_size in 1..=MAX {
            let size = 2 * part_size;
            let mut a_part: Nodes = Vec::with_capacity(part_size);
            let mut b_part: Nodes = Vec::with_capacity(part_size);
            let mut co_a_part: Nodes = Vec::with_capacity(part_size);
            let mut co_b_part: Nodes = Vec::with_capacity(part_size);
            for i in 0..part_size {
                a_part.push((2 * i, (0..=i).map(|j| 2 * j + 1).collect()));
                b_part.push((2 * i + 1, (i..part_size).map(|j| 2 * j).collect()));
                let mut co_a_neighbourhood = Vec::with_capacity(size - 1 - i);
                let mut co_b_neighbourhood =
                    Vec::with_capacity(size - 1 - (part_size - i));
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
            test(part_size, ObstinateKind::Itself, a_part, b_part);
            if part_size != 2 {
                test(part_size, ObstinateKind::Complement, co_a_part, co_b_part);
            } else {
                // in that case, the graph itself is obstinate (as well as the
                // complement), but our algorithm goes down the Itself path, so we wont
                // get the result that the complement is obstinate; the pop order is
                // important here!
                let b_part = vec![co_b_part.pop().unwrap(), co_a_part.pop().unwrap()];
                let a_part = vec![co_b_part.pop().unwrap(), co_a_part.pop().unwrap()];
                test(part_size, ObstinateKind::Itself, a_part, b_part);
            }
        }
    }

    #[test]
    // no need to do many tests for that, since this check is very simple and we just
    // ensure that it is there
    fn fail_odd() {
        let (graph, _) = create_graph!(3, 3, (0, [1]), (1, [2]), (2, [0]),);
        assert_eq!(obstinate(graph), Obstinate::False);
    }
}
