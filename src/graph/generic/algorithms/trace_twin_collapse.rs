use std::fmt::{self, Display};

use modular_decomposition::ModuleKind;
use petgraph::Direction;

use super::is_line_graph::SageProcess;
use crate::graph::{
    algorithms::modular_decomposition::{NodeIndex, Tree, TreeGraph},
    generic::{Graph, ImplGraph, SwapRemoveMap},
};

#[derive(Debug)]
pub enum SiblingType {
    True,
    False,
}
#[derive(Debug)]
pub struct Siblings {
    pub vertices: Vec<u32>,
    pub remaining: u32,
    pub typ: SiblingType,
}

impl Display for SiblingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SiblingType::True => write!(f, "True"),
            SiblingType::False => write!(f, "False"),
        }
    }
}

impl<G: ImplGraph> Graph<G> {
    pub fn trace_twin_collapse(
        &mut self,
        tree: &mut Tree,
        sage_process: &mut SageProcess,
    ) -> Vec<Siblings> {
        let mut graph_map = SwapRemoveMap::new(self.len());
        let mut tree_map = SwapRemoveMap::new(tree.graph.node_count());

        println!("{:?}", tree.module_is_fully_prime(tree.root));

        let mut sibling_sets = Vec::new();

        self.trace_recurse_collapse(
            &mut tree.graph,
            tree.root,
            &mut graph_map,
            &mut tree_map,
            &mut sibling_sets,
            sage_process,
        );
        // cf. below in `recurse_collapse` about wrong node weights for leaves
        for weight in tree.graph.node_weights_mut() {
            if let ModuleKind::Node(ref mut node) = weight {
                *node = graph_map.mapped(*node);
            }
        }
        tree.root = (tree_map.mapped(tree.root.index()) as u32).into();
        sibling_sets
    }

    // here, we don't really have to care about wrong node weights for leaves since we
    // only get all the information from the tree structure; we correct
    // them at the end; see above in parent function
    fn trace_recurse_collapse(
        &mut self,
        tree: &mut TreeGraph,
        root: NodeIndex,
        graph_map: &mut SwapRemoveMap,
        tree_map: &mut SwapRemoveMap,
        sibling_sets: &mut Vec<Siblings>,
        sage_process: &mut SageProcess,
    ) {
        let new_root = (tree_map.mapped(root.index()) as u32).into();

        if let ModuleKind::Node(_) = tree.node_weight(new_root).unwrap() {
            return;
        }

        println!("{:?}", tree.node_weight(new_root).unwrap());

        let children: Vec<NodeIndex> =
            tree.neighbors_directed(new_root, Direction::Outgoing).collect();

        if *tree.node_weight(new_root).unwrap() == ModuleKind::Prime {
            for child in children.iter() {
                self.trace_recurse_collapse(
                    tree,
                    *child,
                    graph_map,
                    tree_map,
                    sibling_sets,
                    sage_process,
                );
            }

            let mut nodes = Vec::new(); // for potential line graph check
            let mut only_leaf_children = true;
            for child in children.iter() {
                if let ModuleKind::Node(node) = tree
                    .node_weight((tree_map.mapped(child.index()) as u32).into())
                    .unwrap()
                {
                    // okay, here we actually need the right node weights, because we
                    // might use them later to get a subgraph
                    nodes.push(graph_map.mapped(*node));
                } else {
                    only_leaf_children = false;
                    break;
                }
            }

            if !only_leaf_children {
                // don't know any methods to handle that case
                return;
            }

            let full_collapse = if children.len() <= 4
            // (should be) equivalent to "len== 4"
            {
                true
            } else {
                // line graph check
                let module_graph = self.subgraph(&nodes);
                // alternatively, we could do the following
                // let new_root = (tree_map.mapped(root.index()) as u32).into();
                // let current_tree = Tree {
                //     graph: tree.clone(),
                //     root: new_root,
                // };
                // let nodes = Vec::from_iter(
                //     current_tree
                //         .module_nodes(new_root, Some(2))
                //         // or that here
                //         // .reduced_module(new_root)
                //         .into_iter()
                //         .map(|n| graph_map.mapped(n)),
                // );
                // let module_graph = self.subgraph(&nodes);
                module_graph.is_line_graph(sage_process)
            };
            // actually let's just ignore non-twin-stuff for now:
            println!("full collapse: {full_collapse}");
            let full_collapse = false;
            // WARN: the following has to be adjusted if we do non-twin-stuff (if we want
            // to correctly record what is being removed
            if full_collapse {
                for child in children[1..].iter() {
                    let node = if let ModuleKind::Node(node) = tree
                        .node_weight((tree_map.mapped(child.index()) as u32).into())
                        .unwrap()
                    {
                        node
                    } else {
                        unreachable!("already checked above that all children are nodes")
                    };
                    self.remove_node(graph_map.swap_remove(*node));
                    tree.remove_node((tree_map.swap_remove(child.index()) as u32).into());
                }
                let new_root = (tree_map.mapped(root.index()) as u32).into();
                *tree.node_weight_mut(new_root).unwrap() = ModuleKind::Node(
                    if let ModuleKind::Node(node) = tree
                        .node_weight((tree_map.mapped(children[0].index()) as u32).into())
                        .unwrap()
                    {
                        *node
                    } else {
                        unreachable!("already checked above that all children are nodes")
                    },
                );
                tree.remove_node(
                    (tree_map.swap_remove(children[0].index()) as u32).into(),
                );
            }
            return;
        }

        let sibling_type = match *tree.node_weight(new_root).unwrap() {
            ModuleKind::Prime => unreachable!("handled above"),
            ModuleKind::Series => SiblingType::True,
            ModuleKind::Parallel => SiblingType::False,
            ModuleKind::Node(_) => unreachable!("handled above"),
        };

        let mut remaining_leaf = None;
        let mut num_children = children.len();
        let mut siblings = Vec::new();

        let mut children = children.into_iter();
        for child in children.by_ref() {
            self.trace_recurse_collapse(
                tree,
                child,
                graph_map,
                tree_map,
                sibling_sets,
                sage_process,
            );
            if let ModuleKind::Node(node) = tree
                .node_weight((tree_map.mapped(child.index()) as u32).into())
                .unwrap()
            {
                remaining_leaf = Some((*node, child.index()));
                num_children -= 1;
                siblings.push(self.get_label(graph_map.mapped(*node)).unwrap());
                break;
            }
        }
        // continue with the rest of the children
        for child in children {
            self.trace_recurse_collapse(
                tree,
                child,
                graph_map,
                tree_map,
                sibling_sets,
                sage_process,
            );
            if let ModuleKind::Node(node) = tree
                .node_weight((tree_map.mapped(child.index()) as u32).into())
                .unwrap()
            {
                let to_remove = graph_map.swap_remove(*node);
                siblings.push(self.get_label(to_remove).unwrap());
                self.remove_node(to_remove);
                tree.remove_node((tree_map.swap_remove(child.index()) as u32).into());
                num_children -= 1;
            }
        }

        if !siblings.is_empty() {
            sibling_sets.push(Siblings {
                remaining: siblings[0],
                vertices: siblings,
                typ: sibling_type,
            });
        }

        let new_root = (tree_map.mapped(root.index()) as u32).into();
        if num_children == 0 {
            // otherwise we would have never reached `num_children -= 1`
            let remaining_leaf = remaining_leaf.unwrap();
            *tree.node_weight_mut(new_root).unwrap() = ModuleKind::Node(remaining_leaf.0);
            tree.remove_node((tree_map.swap_remove(remaining_leaf.1) as u32).into());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{
        algorithms::twin_collapse,
        generic::{Adj, Graph, Pet},
    };

    twin_collapse::tests::test_it!(petgraph, Graph<Pet>);
    twin_collapse::tests::test_it!(adjgraph, Graph<Adj>);
}
