use modular_decomposition::ModuleKind;
use petgraph::Direction;

use crate::{
    fix_int::int,
    graph::{
        Node,
        algorithms::modular_decomposition::{NodeIndex, Tree, TreeGraph},
        specialised::{Graph, GraphData, SwapRemoveMap},
    },
};

impl<G: GraphData> Graph<G> {
    // it is not clear whether implementing this here with a manual stack instead of a
    // recursion would be better, because we can not really efficiently bound the stack
    // size (-> would potentially cause reallocations; except by a large bound
    // graph.len()), and we have multiple enter/leave points of the recursion which adds
    // some overhead to the iteration (so the advantage to the context switch (in
    // recursion) is maybe not that great anymore); most importantly, for now, the
    // recursive version is easier to understand and implement (manual stack of the
    // iterators plays not well with the borrow checker; could stack the nodes instead,
    // but that would make the stack bigger, probably)
    /// # Safety
    /// The `tree` must be the decomposition tree of `self`.
    pub unsafe fn twin_collapse(&mut self, tree: &mut Tree) {
        // global safety: all the unsafe operations on (graph|tree)_map are okay, because
        // the inputs come from the tree's indices or weights, and we know the tree is the
        // decomposition tree of the graph

        let mut graph_map = SwapRemoveMap::new(self.len());
        let mut tree_map = SwapRemoveMap::new(tree.graph.node_count());
        self.recurse_collapse(&mut tree.graph, tree.root, &mut graph_map, &mut tree_map);
        // cf. comment in fn full_remove
        for node in tree.graph.node_weights_mut() {
            if let ModuleKind::Node(ref mut node) = node {
                *node = unsafe { graph_map.mapped_unchecked(*node) };
            }
        }
        tree.root =
            (unsafe { tree_map.mapped_unchecked(tree.root.index()) } as int).into();
    }

    fn recurse_collapse(
        &mut self,
        tree: &mut TreeGraph,
        module: NodeIndex,
        graph_map: &mut SwapRemoveMap,
        tree_map: &mut SwapRemoveMap,
    ) {
        // we might already have removed some earlier children, so the module index might
        // be wrong
        let updated_module =
            (unsafe { tree_map.mapped_unchecked(module.index()) } as int).into();

        // in the leaf or prime case, we know that the module wont be collapsed
        match tree.node_weight(updated_module).unwrap() {
            ModuleKind::Node(_) => return,
            ModuleKind::Prime => {
                // need to collect, because the tree will be changed which screws up the
                // iterator (but we track it with tree_map)
                let children: Vec<_> = tree
                    .neighbors_directed(updated_module, Direction::Outgoing)
                    .collect();
                for child in children.iter() {
                    if matches!(tree.node_weight(*child).unwrap(), ModuleKind::Node(_)) {
                        continue;
                    }
                    self.recurse_collapse(tree, *child, graph_map, tree_map);
                }

                // TODO: this is just copied from the generic version to make the tests
                // pass; should implement that more efficiently here ... {{
                if children.len() <= 4 // (should be) equivalent to "== 4"
                && children.iter().all(|child| {
                    matches!(
                        tree.node_weight((tree_map.mapped(child.index()) as u32).into())
                            .unwrap(),
                        ModuleKind::Node(_)
                    )
                }) {
                    for child in children[1..].iter() {
                        let node = if let ModuleKind::Node(node) = tree
                            .node_weight((tree_map.mapped(child.index()) as u32).into())
                            .unwrap()
                        {
                            node
                        } else {
                            unreachable!(
                                "already checked above that all children are nodes"
                            )
                        };
                        self.remove_node(graph_map.swap_remove(*node));
                        tree.remove_node(
                            (tree_map.swap_remove(child.index()) as u32).into(),
                        );
                    }
                    let updated_module = (tree_map.mapped(module.index()) as u32).into();
                    *tree.node_weight_mut(updated_module).unwrap() = ModuleKind::Node(
                        if let ModuleKind::Node(node) = tree
                            .node_weight(
                                (tree_map.mapped(children[0].index()) as u32).into(),
                            )
                            .unwrap()
                        {
                            *node
                        } else {
                            unreachable!(
                                "already checked above that all children are nodes"
                            )
                        },
                    );
                    tree.remove_node(
                        (tree_map.swap_remove(children[0].index()) as u32).into(),
                    );
                    // }}
                }
                return;
            },
            _ => {},
        }

        #[inline(always)]
        fn get_weight<'t>(
            tree: &'t TreeGraph,
            node: NodeIndex,
            tree_map: &SwapRemoveMap,
        ) -> &'t ModuleKind<Node> {
            tree.node_weight((tree_map.mapped(node.index()) as int).into())
                .unwrap()
        }

        #[inline(always)]
        fn tree_remove(
            tree: &mut TreeGraph,
            tree_map: &mut SwapRemoveMap,
            child: NodeIndex,
        ) {
            tree.remove_node(
                (unsafe { tree_map.swap_remove_unchecked(child.index()) } as int).into(),
            );
        }

        #[inline(always)]
        fn full_remove(
            graph: &mut Graph<impl GraphData>,
            graph_map: &mut SwapRemoveMap,
            tree: &mut TreeGraph,
            tree_map: &mut SwapRemoveMap,
            child: NodeIndex,
            node: Node,
        ) {
            // note that we do not update the according tree weight according to
            // graph_map, instead we just use graph_map to get the correct node in the
            // graph; we'll update the tree weights later after the recursion
            graph.remove_node(unsafe { graph_map.swap_remove_unchecked(node) });
            tree.remove_node(
                (unsafe { tree_map.swap_remove_unchecked(child.index()) } as int).into(),
            );
        }

        let mut children = tree
            .neighbors_directed(updated_module, Direction::Outgoing)
            .collect::<Vec<_>>()
            .into_iter();

        // we cannot just directly remove every leaf, because if all children are leaves,
        // then the module is suddenly empty, which would be wrong; instead, in that case,
        // we want to change the module to one of those leaves; we do this by potentially
        // storing one leaf and either remove it or change the module to it, in the end
        let mut remaining_leaf = None;
        let mut module_may_become_remaining_leaf = true;

        // break the loop into two loops to so that we only have the `to_remaining_leaf`
        // logic once

        for child in children.by_ref() {
            if let ModuleKind::Node(node) = get_weight(tree, child, tree_map) {
                remaining_leaf = Some((*node, child));
                break;
            } else {
                self.recurse_collapse(tree, child, graph_map, tree_map);
                if let ModuleKind::Node(node) = get_weight(tree, child, tree_map) {
                    remaining_leaf = Some((*node, child));
                    break;
                } else {
                    module_may_become_remaining_leaf = false;
                }
            }
        }
        for child in children {
            if let ModuleKind::Node(node) = get_weight(tree, child, tree_map) {
                full_remove(self, graph_map, tree, tree_map, child, *node);
            } else {
                self.recurse_collapse(tree, child, graph_map, tree_map);
                if let ModuleKind::Node(node) = get_weight(tree, child, tree_map) {
                    full_remove(self, graph_map, tree, tree_map, child, *node);
                } else {
                    module_may_become_remaining_leaf = false;
                }
            }
        }

        let new_module_root =
            (unsafe { tree_map.mapped_unchecked(module.index()) } as int).into();
        if let Some(new_leaf) = remaining_leaf {
            if module_may_become_remaining_leaf {
                *tree.node_weight_mut(new_module_root).unwrap() =
                    ModuleKind::Node(new_leaf.0);
                tree_remove(tree, tree_map, new_leaf.1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::{
        algorithms::twin_collapse,
        specialised::{Custom, Graph, IndexMap},
        test_utils::collect,
    };

    twin_collapse::tests::test_it!(custom, Graph<Custom>);
    twin_collapse::tests::test_it!(indexmap, Graph<IndexMap>);
}
