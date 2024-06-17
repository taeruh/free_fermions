use std::collections::{HashMap, HashSet};

use petgraph::{
    visit::{GraphProp, IntoNeighbors, NodeCompactIndexable},
    Undirected,
};

use super::{Edge, HNeighbourhood, HNodes, ImplGraph, Node, NodeCollection};

pub type Nodes = HNodes;
pub type Neighbourhood = HNeighbourhood;

pub struct HashGraph {
    pub nodes: HashMap<Node, Neighbourhood>,
}

impl ImplGraph for HashGraph {
    type NodeCollection = Nodes;

    fn from_edges(edges: impl IntoIterator<Item = Edge>) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn from_adjancencies<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (Node, N)>,
        N: IntoIterator<Item = Node>,
    {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn get(&self, node: Node) -> Option<&Self::NodeCollection> {
        todo!()
    }

    fn get_mut(&mut self, node: Node) -> Option<&mut Self::NodeCollection> {
        todo!()
    }

    fn filter_nodes(&mut self, f: impl Fn(Node) -> bool) {
        todo!()
    }

    fn subgraph(&self, nodes: impl NodeCollection) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn complement(&mut self) {
        todo!()
    }
}
