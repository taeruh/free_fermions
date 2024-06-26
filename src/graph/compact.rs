use super::{HNeighbourhood, HNodeInfo, ImplGraph};

type Neighbourhood = HNeighbourhood;
type NodeInfo = HNodeInfo;

pub struct CompactGraph {
    nodes: Vec<NodeInfo>,
}

impl ImplGraph for CompactGraph {
    type NodeCollection;

    fn from_edges(edges: impl IntoIterator<Item = super::Edge>) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn from_adjacencies<A, N>(adj: A) -> Self
    where
        A: IntoIterator<Item = (super::Node, N)>,
        N: IntoIterator<Item = super::Node>,
    {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn get(&self, node: super::Node) -> Option<&Self::NodeCollection> {
        todo!()
    }

    fn get_mut(&mut self, node: super::Node) -> Option<&mut Self::NodeCollection> {
        todo!()
    }

    fn retain_nodes(&mut self, f: impl Fn(super::Node) -> bool) {
        todo!()
    }

    fn subgraph(&self, nodes: impl super::NodeCollection) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn complement(&mut self) {
        todo!()
    }

    fn iter_nodes(&self) -> impl Iterator<Item = super::Node> {
        todo!()
    }

    fn iter_node_info(
        &self,
    ) -> impl Iterator<Item = (super::Node, &Self::NodeCollection)> {
        todo!()
    }
}
