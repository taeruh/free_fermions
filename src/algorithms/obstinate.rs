use std::collections::HashSet;

use crate::graph::ReducedGraph;

#[derive(Debug, Clone)]
pub enum Obstinate {
    True(ObstinateKind, (Vec<usize>, Vec<usize>)),
    False,
}

#[derive(Debug, Clone)]
pub enum ObstinateKind {
    Itself,
    Complement,
}

pub fn check(graph: ReducedGraph) -> Obstinate {
    let len = graph.nodes.len();
    let len2 = if len % 2 != 0 {
        return Obstinate::False;
    } else {
        len / 2
    };

    let mut degrees = graph
        .nodes
        .iter()
        .map(|(node, neighbors)| (*node, neighbors.len()))
        .collect::<Vec<_>>();

    degrees.sort_unstable_by_key(|(_, degree)| *degree);

    // println!("{:?}", degrees);

    fn check_sequence(
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

    let kind = if check_sequence(1, len2, &degrees) {
        ObstinateKind::Itself
    } else if check_sequence(len2 - 1, len - 2, &degrees) {
        ObstinateKind::Complement
    } else {
        return Obstinate::False;
    };

    // println!("{:?}", kind);

    let ak = degrees[len - 2].0;
    let b1 = degrees[len - 1].0;
    let A = graph.nodes[&b1].clone();
    let B = graph.nodes[&ak].clone();

    // println!("{:?}", ak);
    // println!("{:?}", b1);
    // println!("{:?}", A);
    // println!("{:?}", B);

    fn is_independent(graph: &ReducedGraph, subset: &HashSet<usize>) -> bool {
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

    if (A.intersection(&B).count() != 0)
        || !is_independent(&graph, &A)
        || !is_independent(&graph, &B)
    {
        return Obstinate::False;
    }

    // println!("{:?}", kind);

    fn check_single_sequence(
        start: usize,
        end_inclusive: usize,
        degrees: &[(usize, usize)],
    ) -> bool {
        let mut iter_degrees = degrees.iter().map(|(_, degree)| *degree);
        for i in start..=end_inclusive {
            if i != iter_degrees.next().unwrap() {
                return false;
            }
        }
        true
    }

    let mut degreesA = A
        .iter()
        .map(|node| (*node, graph.nodes[node].len()))
        .collect::<Vec<_>>();
    degreesA.sort_unstable_by_key(|(_, degree)| *degree);
    if !check_single_sequence(1, len2, &degreesA) {
        return Obstinate::False;
    }

    // println!("{:?}", degreesA);

    let mut degreesB = B
        .iter()
        .map(|node| (*node, graph.nodes[node].len()))
        .collect::<Vec<_>>();
    degreesB.sort_unstable_by(|(_, degree1), (_, degree2)| degree2.cmp(degree1));
    if !check_single_sequence(len2, 1, &degreesB) {
        return Obstinate::False;
    }

    // println!("{:?}", degreesB);

    for (mut i, (a, _)) in degreesA.iter().enumerate() {
        i += 1;
        for b in degreesB.iter().take(i) {
            if !graph.nodes[a].contains(&b.0) {
                println!("!edge: {:?}", (a, b.0));
                return Obstinate::False;
            }
        }
        for b in degreesB.iter().skip(i) {
            if graph.nodes[a].contains(&b.0) {
                println!("edge: {:?}", (a, b.0));
                return Obstinate::False;
            }
        }
    }

    Obstinate::True(
        kind,
        (
            degreesA.into_iter().map(|(node, _)| node).collect(),
            degreesB.into_iter().map(|(node, _)| node).collect(),
        ),
    )
}

#[cfg(test)]
mod tests {
    use rand::{
        seq::{IteratorRandom, SliceRandom},
        SeedableRng,
    };
    use rand_pcg::Pcg64;

    use super::*;

    fn randomize_labels(
        mut list: Vec<(usize, Vec<usize>)>,
        max_list: usize,
        max_rand: usize,
    ) -> Vec<(usize, Vec<usize>)> {
        let mut rng = Pcg64::from_entropy();
        let map = (0..=max_rand).choose_multiple(&mut rng, max_list + 1);

        for (node, neighbors) in list.iter_mut() {
            *node = map[*node];
            for neighbor in neighbors.iter_mut() {
                *neighbor = map[*neighbor];
            }
        }

        list
    }

    #[test]
    fn obstinate() {
        let graph = ReducedGraph::from_iter(randomize_labels(
            vec![(0, vec![1]), (1, vec![0, 2]), (2, vec![1, 3]), (3, vec![2])],
            3,
            42,
        ));

        let obs = check(graph);

        println!("{:?}", obs);
        // assert!(matches!(obs, Obstinate::True(ObstinateKind::Itself, _)));
    }
}
