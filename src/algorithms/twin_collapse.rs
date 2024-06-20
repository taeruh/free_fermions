use modular_decomposition::ModuleKind;
use petgraph::Direction;

use super::modular_decomposition::{Tree, TreeData};
use crate::graph::{Graph, ImplGraph, NodeIndex};

impl<G: ImplGraph> Graph<G> {
    pub fn twin_collapse(&mut self, tree: &mut Tree) {
        self.recurse_collapse(&mut tree.data, tree.root);
    }

    fn recurse_collapse(&mut self, data: &mut TreeData, root: NodeIndex) {
        if let ModuleKind::Node(_) = data.node_weight(root).unwrap() {
            return;
        }

        // PERF: collect because of borrow rules -> improve somehow
        // edit: actually, probably fine, since we use twice ...
        let children: Vec<NodeIndex> =
            data.neighbors_directed(root, Direction::Outgoing).collect();

        if *data.node_weight(root).unwrap() == ModuleKind::Prime {
            for child in children {
                self.recurse_collapse(data, child)
            }
            return;
        }

        let mut remaining_leaf = None;
        let mut num_children = children.len();

        let mut children = children.into_iter();
        for child in children.by_ref() {
            self.recurse_collapse(data, child);
            if let ModuleKind::Node(node) = data.node_weight(child).unwrap() {
                remaining_leaf = Some(*node);
                data.remove_node(child);
                num_children -= 1;
                break;
            }
        }
        for child in children {
            self.recurse_collapse(data, child);
            if let ModuleKind::Node(node) = data.node_weight(child).unwrap() {
                self.remove_node(*node);
                data.remove_node(child);
                num_children -= 1;
            }
        }

        if num_children == 0 {
            *data.node_weight_mut(root).unwrap() =
                ModuleKind::Node(remaining_leaf.unwrap());
        } else {
            self.remove_node(remaining_leaf.unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{test_utils::collect, Graph};

    #[test]
    fn test() {
        let mut graph = Graph::from(collect!(
            adj,
            hash;
            (0, [1]),
            (1, [0, 2]),
            (2, [1, 3, 4, 5]),
            (3, [2, 4]),
            (4, [2, 3]),
            (5, [2]),
        ));
        let mut tree = graph.modular_decomposition();
        println!("{:?}", graph);
        println!("{:#?}", tree);
        graph.twin_collapse(&mut tree);
        println!("{:?}", graph);
        println!("{:#?}", tree);
    }
}
