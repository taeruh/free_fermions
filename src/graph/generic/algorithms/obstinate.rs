use std::borrow::Cow;

use crate::graph::{
    Node,
    algorithms::obstinate::{Obstinate, ObstinateKind},
    generic::{Graph, ImplGraph, NodeCollection, NodeCollectionRef},
};

impl<G: ImplGraph + Clone> Graph<G> {
    // note that, if a graph is obstinate, then there are two expected results, since we
    // can swap a with b and in each part we then reverse the order of the vertices; this
    // algorithm does not guarantee which of the two results will be returned, since we
    // use unstable sorting in some places
    pub fn obstinate(&self) -> Obstinate {
        // directly alias self(: &Self) with graph, because we have to do it later anyways
        // when we put into into a Cow (so that we don't confuse self/graph (for
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

        let mut degrees = graph
            .iter_neighbourhoods()
            .enumerate()
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
        let a_part = graph.get_neighbours(b_start).unwrap().clone();
        let b_part = graph.get_neighbours(a_end).unwrap().clone();

        if (a_part.intersection(&b_part).count() != 0)
            || !graph.set_is_independent(a_part.iter())
            || !graph.set_is_independent(b_part.iter())
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
            .iter_ref()
            .map(|vertex| (vertex, graph.get_neighbours(vertex).unwrap().len()))
            .collect::<Vec<_>>();
        a_degrees.sort_unstable_by_key(|(_, degree)| *degree);
        if !check_part_degree_sequence(1, len2, &a_degrees) {
            return Obstinate::False;
        }

        let mut b_degrees = b_part
            .iter_ref()
            .map(|vertex| (vertex, graph.get_neighbours(vertex).unwrap().len()))
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
                if !graph.get_neighbours(*a).unwrap().contains(b.0) {
                    return Obstinate::False;
                }
            }
            for b in b_degrees.iter().skip(i) {
                if graph.get_neighbours(*a).unwrap().contains(b.0) {
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
    use crate::graph::{
        HLabels, Label,
        algorithms::obstinate::{self, ObstinateMapped},
        generic::impl_petgraph::PetGraph,
    };

    type Graph = super::Graph<PetGraph>;

    use hashbrown::HashMap;

    fn create(map: HashMap<Label, HLabels>) -> Graph {
        Graph::from_adjacency_labels(map).unwrap()
    }
    fn obstianate_algorithm(graph: &Graph) -> ObstinateMapped {
        graph.obstinate().map(|n| graph.get_label(n).unwrap())
    }

    obstinate::tests::test_it!(create, obstianate_algorithm);
}
