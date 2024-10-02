use std::{iter::Map, mem};

use hashbrown::HashSet;
use petgraph::{Undirected, graph::Neighbors, operator};

use super::{CompactNodes, HNodes, ImplGraph, Node, NodeCollection, NodeCollectionRef};
use crate::graph::{Label, LabelEdge};

pub type NodeIndex = petgraph::graph::NodeIndex<Node>; // = int

pub type PetGraph = petgraph::Graph<Label, (), Undirected, Node>;

impl CompactNodes for PetGraph {}

impl ImplGraph for PetGraph {
    type Nodes = HNodes;

    type Neighbours<'a>
        = Neighbors<'a, (), Node>
    where
        Self: 'a;

    fn add_labelled_edge(&mut self, (a, b): LabelEdge) {
        let a_idx = insert_node(self, a);
        let b_idx = insert_node(self, b);
        self.update_edge(a_idx, b_idx, ());
        self.update_edge(b_idx, a_idx, ());
    }

    fn add_labelled_node_symmetrically<N: IntoIterator<Item = Label>>(
        &mut self,
        (node, adj): (Label, N),
    ) {
        let node_idx = insert_node(self, node);
        for n in adj {
            let n_idx = insert_node(self, n);
            self.update_edge(node_idx, n_idx, ());
            self.update_edge(n_idx, node_idx, ());
        }
    }

    fn len(&self) -> usize {
        self.node_count()
    }

    fn get_label(&self, node: Node) -> Option<Label> {
        self.node_weight(node.into()).copied()
    }

    fn get_label_mut(&mut self, node: Node) -> Option<&mut Label> {
        self.node_weight_mut(node.into())
    }

    fn get_neighbours(&self, node: Node) -> Option<Self::Neighbours<'_>> {
        Some(self.neighbors(node.into()))
    }

    fn remove_node(&mut self, node: Node) {
        self.remove_node(node.into());
    }

    fn complement(&mut self) {
        let mut complement = PetGraph::default();
        operator::complement(self, &mut complement, ());
        mem::swap(self, &mut complement);

        // PetGraph can store an edge multiple times and differentiates between the edge
        // (a, b) and (b, a). When calling the complement function above and we have an
        // edge (a, b) in the original graph, (a, b) will be removed and it won't insert
        // (b, a). However, if neither (a, b) nor (b, a) are in the original graph, then
        // (b, a) and (a, b) will be inserted; we don't want that, because it breaks some
        // algorithms if we have multi-edges.
        let mut single_edges = HashSet::new();
        for node in self.node_indices() {
            for neighbour in self.neighbors(node) {
                if !(single_edges.contains(&(node, neighbour))
                    || single_edges.contains(&(neighbour, node)))
                {
                    single_edges.insert((node, neighbour));
                }
            }
        }
        self.clear_edges();
        self.extend_with_edges(single_edges);
    }

    fn iter_labels_mut(&mut self) -> impl Iterator<Item = &mut Label> {
        self.node_weights_mut()
    }
}

fn insert_node(graph: &mut PetGraph, label: Label) -> NodeIndex {
    if let Some(idx) = graph
        .node_indices()
        .find(|idx| *graph.node_weight(*idx).unwrap() == label)
    {
        idx
    } else {
        graph.add_node(label)
    }
}

impl NodeCollection for Neighbors<'_, (), Node> {
    type Collected = HNodes;
    type Iter<'a>
        = Map<Self, fn(NodeIndex) -> Node>
    where
        Self: 'a;

    fn contains(&self, e: Node) -> bool {
        self.clone().any(|n| n.index() == e)
    }

    fn iter(&self) -> Self::Iter<'_> {
        self.clone().map((|e| e.index()) as fn(NodeIndex) -> Node)
    }
    fn collect(self) -> Self::Collected {
        self.iter().collect()
    }
}

impl NodeCollectionRef for Neighbors<'_, (), Node> {
    type Iter = Map<Self, fn(NodeIndex) -> Node>;

    fn iter_ref(self) -> Self::Iter {
        self.clone().map((|e| e.index()) as fn(NodeIndex) -> Node)
    }
}
