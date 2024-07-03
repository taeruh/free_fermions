use std::{
    collections::{HashMap, HashSet},
    mem,
};

use super::{CompactNodes, Edge, HNodes, ImplGraph, Label, Node, NodeCollection};
use crate::fix_int::{enumerate, int};

pub type Neighbourhood = HashSet<Node>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdjGraph {
    // separate labels and neighbourhoods, because the labels are usually in the way,
    // except when removing a node (which will require just one more swap_remove) or when
    // reading the graph
    pub nodes: Vec<Neighbourhood>,
    pub labels: Vec<Label>,
    pub invert_labels: HashMap<Label, usize>,
}

impl CompactNodes for AdjGraph {}

impl ImplGraph for AdjGraph {
    // TODO: implement a bunch of default methods more efficiently (if possible)

    type Nodes = HNodes;

    fn add_labelled_edge(&mut self, (a, b): Edge) {
        let idxa = self.insert(a);
        let idxb = self.insert(b);
        self.nodes[idxa].insert(idxb as int);
        self.nodes[idxb].insert(idxa as int);
    }

    fn add_labelled_node_symmetrically<N: IntoIterator<Item = int>>(
        &mut self,
        (node, adj): (int, N),
    ) {
        let idx = self.insert(node);
        for neighbour in adj {
            let idx_neighbour = self.insert(neighbour);
            self.nodes[idx].insert(idx_neighbour as int);
            self.nodes[idx_neighbour].insert(idx as int);
        }
    }

    fn from_symmetric_adjacency_labels_unchecked<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (int, N)>,
        N: IntoIterator<Item = int>,
    {
        let mut ret = Self::default();
        for (node, neighbourhood) in adj {
            // just like add_labelled_node_symmetrically, but without
            // ret.nodes[idx_neighbour].insert(idx as int);
            let idx = ret.insert(node);
            for neighbour in neighbourhood {
                let idx_neighbour = ret.insert(neighbour);
                ret.nodes[idx].insert(idx_neighbour as int);
            }
        }
        ret
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn get_label(&self, node: int) -> Option<int> {
        self.labels.get(node as usize).copied()
    }

    fn get_label_mut(&mut self, node: int) -> Option<&mut int> {
        self.labels.get_mut(node as usize)
    }

    fn get_neighbours(&self, node: int) -> Option<&Neighbourhood> {
        self.nodes.get(node as usize)
    }

    fn get_neighbours_mut(&mut self, node: int) -> Option<&mut Neighbourhood> {
        self.nodes.get_mut(node as usize)
    }

    fn remove_node(&mut self, node: int) {
        let len = self.nodes.len() as int;
        assert!(len > 0, "cannot remove from empty graph");
        let last_node = len as int - 1;

        self.invert_labels.remove(&self.labels[node as usize]).unwrap();

        // if last node, we can simply pop it
        if node == last_node {
            let neighbours = self.nodes.pop().unwrap();
            self.labels.pop().unwrap();
            for neighbour in neighbours {
                self.nodes[neighbour as usize].remove(&node);
            }
            return;
        }

        // if not last node, we have to swap_remove to keep the graph compact
        //
        // do not swap_remove yet, because the node might have a neighbour in the last
        // node
        let node_neighbours = mem::take(self.nodes.get_mut(node as usize).unwrap());
        for neighbour in node_neighbours {
            self.nodes[neighbour as usize].remove(&node);
        }
        let last_node_neighbours = self.nodes[last_node as usize].clone();
        for neighbour in last_node_neighbours.iter() {
            let neighbours = &mut self.nodes[*neighbour as usize];
            neighbours.insert(node);
            neighbours.remove(&last_node);
        }
        self.nodes.swap_remove(node as usize);
        *self.invert_labels.get_mut(&self.labels[last_node as usize]).unwrap() =
            node as usize;
        self.labels.swap_remove(node as usize);
    }

    fn complement(&mut self) {
        let nodes = self.iter_nodes().collect::<Vec<_>>();
        for (node, neighbours) in enumerate!(self.nodes.iter_mut()) {
            let mut neighbourhood_to_complement = mem::take(neighbours);
            neighbourhood_to_complement.insert(node); // no self loops in the complement
            for other in nodes.iter() {
                if !neighbourhood_to_complement.contains(&other) {
                    neighbours.insert(other);
                }
            }
        }
    }

    fn iter_labels_mut(&mut self) -> impl Iterator<Item = &mut Label> {
        self.labels.iter_mut()
    }

    fn iter_neighbourhoods_mut(&mut self) -> impl Iterator<Item = &mut Neighbourhood> {
        self.nodes.iter_mut()
    }
}

impl AdjGraph {
    fn insert(
        &mut self,
        // nodes: &mut Vec<Neighbourhood>,
        // labels: &mut Vec<Label>,
        // invert_labels: &mut HashMap<Label, usize>,
        node: Node,
    ) -> usize {
        *self.invert_labels.entry(node).or_insert_with(|| {
            self.nodes.push(HashSet::new());
            self.labels.push(node);
            self.nodes.len() - 1
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    use super::*;
    use crate::graph::{test_utils::*, InvalidGraph};

    #[test]
    fn from_adj() {
        let list = collect!(vv; (1, [2, 3]), (2, [1, 3]), (3, [1, 2]),);
        let expected_nodes = collect!(vh; [1, 2], [0, 2], [0, 1],);
        let graph = AdjGraph::from_adjacency_labels(list).unwrap();
        assert_eq!(
            graph.iter_neighbourhoods().cloned().collect::<Vec<_>>(),
            expected_nodes
        );
        assert_eq!(graph.iter_labels().collect::<Vec<_>>(), vec![1, 2, 3]);

        let list = collect!(vv; (2, [3]), (1, [3]), (3, [1, 2]),);
        let expected_graph = HashMap::from_iter(
            list.clone().into_iter().map(|(a, b)| (a, HashSet::from_iter(b))),
        );
        let expected_nodes = vec![
            HashSet::from_iter(vec![1]),    // label 2
            HashSet::from_iter(vec![2, 0]), // label 3
            HashSet::from_iter(vec![1]),    // label 1
        ];

        let graph = AdjGraph::from_adjacency_labels(list).unwrap();
        assert_eq!(
            graph.iter_neighbourhoods().cloned().collect::<Vec<_>>(),
            expected_nodes
        );
        assert_eq!(graph.map_to_labels(), expected_graph);
    }

    #[test]
    fn invalid_graphs() {
        let correct =
            AdjGraph::from_adjacency_labels_unchecked(collect!(vh; (1, [2]), (2, [1]),));

        let (mut self_looped, self_looped_err) =
            AdjGraph::from_adjacency_labels(collect!(vh; (1, [1, 2]), (2, [1]),))
                .unwrap_err();
        assert_eq!(
            self_looped_err.map_to_labels(&self_looped),
            InvalidGraph::SelfLoop(1)
        );
        self_looped.correct();
        assert_eq!(self_looped, correct);

        let mut incompatible_neighbourhoods =
            AdjGraph::from_symmetric_adjacency_labels_unchecked(
                collect!(vh; (1, [2]), (2, []),),
            );
        assert_eq!(
            incompatible_neighbourhoods
                .check()
                .map_err(|e| e.map_to_labels(&incompatible_neighbourhoods)),
            Err(InvalidGraph::IncompatibleNeighbourhoods(1, 2))
        );
        incompatible_neighbourhoods.correct();
        assert_eq!(incompatible_neighbourhoods, correct);

        // we use collect with vv here because we want to do a naive check without calling
        // map_to_labels, but instead creating the expected graph with the graph
        // constructor (therefore we cannot allow any randomness on the order of
        // insertion)
        let map = RandomMap::new(5, 42, &mut Pcg64::from_entropy());
        let correct = AdjGraph::from_adjacency_labels(collect!(vv, map;
                (0, [2, 4]),
                (1, [2, 3, 4]),
                (2, [0, 1, 4]),
                (3, [1]),
                (4, [0, 1, 2]),
        ))
        .unwrap(); // insert order 0, 2, 4, 1, 3
        let (mut wrong, _) = AdjGraph::from_adjacency_labels(collect!(vv, map;
                // importantly, the same insert order
                (0, [2, 4]),
                (4, [1]),
                (3, [1]),
                (1, [2, 1, 4]),
                (2, [0, 4]),
        ))
        .unwrap_err();
        wrong.correct();
        assert_eq!(wrong, correct);
        assert_eq!(wrong.check(), Ok(()));
    }

    #[test]
    fn from_edges() {
        let graph = AdjGraph::from_edge_labels(collect!(v; (1, 2), (2, 3), (3, 4), (4,
                    1),))
        .unwrap();
        let labelled_graph = graph.map_to_labels();

        let expected = collect!(hh;
            (1, [2, 4]),
            (2, [1, 3]),
            (3, [2, 4]),
            (4, [1, 3]),
        );
        assert_eq!(labelled_graph, expected);
    }

    #[test]
    fn subgraph() {
        let graph = AdjGraph::from_adjacency_labels(collect!(hh;
            (1, [2]),
            (2, [1, 3]),
            (3, [2]),
        ))
        .unwrap();
        let nodes =
            HNodes::from_iter([1, 3].into_iter().map(|e| graph.find_node(e).unwrap()));
        let expected = AdjGraph::from_adjacency_labels_unchecked(collect!(hh;
            (1, []),
            (3, []),
        ))
        .map_to_labels();
        let subgraph = graph.subgraph(&nodes);
        assert_eq!(subgraph.map_to_labels(), expected);
        assert_eq!(expected, collect!(hh; (1, []), (3, []),));

        let graph = AdjGraph::from_adjacency_labels(collect!(hh;
            (1, [2]),
            (2, [1, 3]),
            (3, [2]),
        ))
        .unwrap();
        let nodes =
            HNodes::from_iter([1].into_iter().map(|e| graph.find_node(e).unwrap()));
        let expected = AdjGraph::from_adjacency_labels_unchecked(collect!(hh;
            (1, []),
        ))
        .map_to_labels();
        let subgraph = graph.subgraph(&nodes);
        assert_eq!(subgraph.map_to_labels(), expected);
        assert_eq!(expected, collect!(hh; (1, []),));
    }

    // #[test]
    // fn foo() {
    //     let mut graph = AdjGraph::default();
    //     println!("{:?}", graph);
    //     graph.add_labelled_node((1, [2, 3]));
    //     println!("{:?}", graph);
    //     graph.remove_node(1);
    //     println!("{:?}", graph);
    //     graph.add_node((2, [1].into()));
    //     println!("{:?}", graph);
    //     graph.add_labelled_edge((1, 2));
    //     println!("{:?}", graph);
    //     graph.remove_edge((0, 1));
    //     println!("{:?}", graph);
    //     graph.remove_labelled_edge((1, 2));
    //     println!("{:?}", graph);
    // }
}
