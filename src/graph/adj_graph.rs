use std::{
    collections::{HashMap, HashSet},
    mem,
};

use super::{Edge, HNeighbourhood, HNodes, ImplGraph, Node, NodeCollection};

pub type Nodes = HNodes;
pub type Neighbourhood = HNeighbourhood;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdjHashGraph {
    pub nodes: HashMap<Node, Neighbourhood>,
}

impl ImplGraph for AdjHashGraph {
    type NodeCollection = Nodes;

    fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self
    where
        Self: Sized,
    {
        let mut graph = AdjHashGraph { nodes: HashMap::new() };
        for (a, b) in edges {
            graph.nodes.entry(a).or_default().insert(b);
            graph.nodes.entry(b).or_default().insert(a);
        }
        graph
    }

    fn from_adjacencies<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Node, N)>,
        N: IntoIterator<Item = Node>,
    {
        AdjHashGraph {
            nodes: adj
                .into_iter()
                .map(|(node, neighbours)| (node, neighbours.into_iter().collect()))
                .collect(),
        }
    }

    fn from_adjacency_hash(adj: HashMap<Node, HashSet<Node>>) -> Self
    where
        Self: Sized,
    {
        AdjHashGraph { nodes: adj }
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn get(&self, node: Node) -> Option<&Self::NodeCollection> {
        self.nodes.get(&node)
    }

    fn get_mut(&mut self, node: Node) -> Option<&mut Self::NodeCollection> {
        self.nodes.get_mut(&node)
    }

    fn retain_nodes(&mut self, f: impl Fn(Node) -> bool) {
        let mut to_remove = Vec::new();
        for node in self.nodes.keys().copied() {
            if !f(node) {
                to_remove.push(node);
            }
        }
        for node in to_remove {
            // we can just unwrap here because we got it from the keys
            for neighbour in self.nodes.remove(&node).unwrap() {
                // we can just unwrap here because if the neighbour would not exist
                // anymore, we would have removed it, but removing it would have removed
                // the neighbour in the node's neighbourhood
                self.nodes.get_mut(&neighbour).unwrap().remove(&node);
            }
        }
    }

    fn subgraph(&self, nodes: impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        self.clone().into_subgraph(nodes)
    }

    fn complement(&mut self) {
        let nodes = self.nodes.keys().copied().collect::<Vec<_>>();
        for (&node, neighbourhood) in self.nodes.iter_mut() {
            let mut neighbourhood_to_complement = mem::take(neighbourhood);
            neighbourhood_to_complement.insert(node); // no self loops in the complement
            for other in nodes.iter() {
                if !neighbourhood_to_complement.contains(&other) {
                    neighbourhood.insert(other);
                }
            }
        }
    }

    fn iter_nodes(&self) -> impl Iterator<Item = Node> {
        self.nodes.keys().copied()
    }

    fn iter_node_info(&self) -> impl Iterator<Item = (Node, &Self::NodeCollection)> {
        self.nodes.iter().map(|(&node, neighbours)| (node, neighbours))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, thiserror::Error)]
pub enum UnvalidAdjGraph {
    #[error("Self loop detected on node {0}")]
    SelfLoop(Node),
    #[error("Incompatible neighbourhoods between the nodes {0} and {1}")]
    IncompatibleNeighbourhoods(Node, Node),
}

impl AdjHashGraph {
    /// Check whether the it is a valid graph description.
    pub fn check(&self) -> Result<(), UnvalidAdjGraph> {
        for (node, neighbours) in self.nodes.iter() {
            for neighbour in neighbours {
                if *node == *neighbour {
                    return Err(UnvalidAdjGraph::SelfLoop(*node));
                }
                if !self.nodes.get(neighbour).unwrap().contains(node) {
                    return Err(UnvalidAdjGraph::IncompatibleNeighbourhoods(
                        *node, *neighbour,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Correct (potentially) invalid graph description.
    pub fn correct(&mut self) {
        // PERF: safety bounds us here to first collect the keys, instead of doing things
        // in one loop
        let nodes = self.nodes.keys().copied().collect::<Vec<_>>();
        for node in nodes {
            let neighbours = self.nodes.get_mut(&node).unwrap();
            neighbours.remove(&node);
            // PERF: have to clone here
            for neighbour in neighbours.clone() {
                if !self.nodes.get(&neighbour).unwrap().contains(&node) {
                    self.nodes.get_mut(&neighbour).unwrap().insert(node);
                }
            }
        }
    }
}

impl From<HashMap<Node, HashSet<Node>>> for AdjHashGraph {
    fn from(adj: HashMap<Node, HashSet<Node>>) -> Self {
        AdjHashGraph { nodes: adj }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_utils::*;

    #[test]
    fn from_adj() {
        let list = collect!(adj, hash;
            (1, [2, 4]),
            (2, [1, 3]),
            (3, [2, 1]),
        );
        let expected = AdjHashGraph::from(list.clone());
        assert_eq!(AdjHashGraph::from_adjacency_hash(list), expected);
    }

    #[test]
    fn invalid_graphs() {
        let correct = AdjHashGraph::from(collect!(adj, hash; (1, [2]), (2, [1]),));

        let mut self_looped =
            AdjHashGraph::from(collect!(adj, hash; (1, [1, 2]), (2, [1]),));
        assert_eq!(self_looped.check(), Err(UnvalidAdjGraph::SelfLoop(1)));
        self_looped.correct();
        assert_eq!(self_looped, correct);

        let mut incompatible_neighbourhoods =
            AdjHashGraph::from(collect!(adj, hash; (1, [2]), (2, []),));
        assert_eq!(
            incompatible_neighbourhoods.check(),
            Err(UnvalidAdjGraph::IncompatibleNeighbourhoods(1, 2))
        );
        incompatible_neighbourhoods.correct();
        assert_eq!(incompatible_neighbourhoods, correct);

        let map = RandomMap::new(5, 42);
        let correct = AdjHashGraph::from(collect!(adj, hash, map;
                (0, [2, 4]),
                (1, [2, 3, 4]),
                (2, [0, 1, 4]),
                (3, [1]),
                (4, [0, 1, 2]),
        ));
        let mut wrong = AdjHashGraph::from(collect!(adj, hash, map;
                (0, [2, 4]),
                (1, [2, 1, 4]),
                (2, [0, 4]),
                (3, [1]),
                (4, [1]),
        ));
        wrong.correct();
        assert_eq!(wrong, correct);
        assert_eq!(wrong.check(), Ok(()));
    }

    #[test]
    fn from_edges() {
        let edges = collect!(edge, vec; (1, 2), (2, 3), (3, 4), (4, 1),);
        let expected = AdjHashGraph {
            nodes: collect!(adj, hash;
                (1, [2, 4]),
                (2, [1, 3]),
                (3, [2, 4]),
                (4, [1, 3]),
            ),
        };
        assert_eq!(AdjHashGraph::from_edges(edges), expected);
    }

    #[test]
    fn subgraph() {
        let graph = AdjHashGraph::from(collect!(adj, hash;
            (1, [2]),
            (2, [1, 3]),
            (3, [2]),
        ));
        let nodes = Nodes::from_iter(vec![1, 3]);
        let expected = AdjHashGraph::from(collect!(adj, hash;
            (1, []),
            (3, []),
        ));
        assert_eq!(graph.subgraph(nodes), expected);
    }
}
