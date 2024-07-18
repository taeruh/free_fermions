use hashbrown::{HashMap, HashSet};

use crate::graph::specialised::{
    GraphData, GraphDataSpecializerHelper, Label, Neighbours, Node,
};

#[derive(Debug, Clone, Default)]
pub struct Custom {
    pub nodes: Vec<Neighbours>,
    pub labels: Vec<Label>,
    pub invert_labels: HashMap<Label, Node>,
}

impl GraphData for Custom {
    unsafe fn get_index_unchecked(&self, label: Label) -> Node {
        *self.invert_labels.get(&label).unwrap()
    }

    unsafe fn get_label_unchecked(&self, node: Node) -> Label {
        *unsafe { self.labels.get_unchecked(node) }
    }

    unsafe fn get_neighbours_unchecked(&self, node: Node) -> &Neighbours {
        unsafe { self.nodes.get_unchecked(node) }
    }

    unsafe fn get_neighbours_mut_unchecked(&mut self, node: Node) -> &mut Neighbours {
        unsafe { self.nodes.get_unchecked_mut(node) }
    }

    fn get_index(&self, label: Label) -> Option<Node> {
        self.invert_labels.get(&label).map(|&idx| idx as Node)
    }

    fn get_label(&self, node: Node) -> Option<Label> {
        self.labels.get(node).copied()
    }

    fn get_neighbours(&self, node: Node) -> Option<&Neighbours> {
        self.nodes.get(node)
    }

    fn get_neighbours_mut(&mut self, node: Node) -> Option<&mut Neighbours> {
        self.nodes.get_mut(node)
    }

    fn get_index_or_insert(&mut self, label: Label) -> Node {
        *self.invert_labels.entry(label).or_insert_with(|| {
            self.nodes.push(HashSet::new());
            self.labels.push(label);
            self.nodes.len() - 1
        })
    }

    fn get_neighbours_mut_or_insert(&mut self, label: Label) -> &mut Neighbours {
        let idx = self.get_index_or_insert(label);
        self.nodes.get_mut(idx).unwrap()
    }

    fn get_index_and_neighbours_mut_or_insert(
        &mut self,
        label: Label,
    ) -> (Node, &mut HashSet<Node>) {
        let idx = self.get_index_or_insert(label);
        (idx, self.nodes.get_mut(idx).unwrap())
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn iter_neighbours(&self) -> impl Iterator<Item = &Neighbours> {
        self.nodes.iter()
    }

    fn enumerate_neighbours(&self) -> impl Iterator<Item = (Node, &Neighbours)> {
        self.nodes.iter().enumerate()
    }

    fn enumerate_full(&self) -> impl Iterator<Item = (Node, Label, &Neighbours)> {
        self.labels
            .iter()
            .enumerate()
            .zip(self.nodes.iter())
            .map(|((node, &label), neighbours)| (node, label, neighbours))
    }

    fn pop(&mut self) -> Option<Neighbours> {
        let label = self.labels.pop().unwrap();
        self.invert_labels.remove(&label);
        self.nodes.pop()
    }

    unsafe fn swap_remove_unchecked(&mut self, node: Node) -> Neighbours {
        // self.len > 0 because of safety invariant
        if node == self.len() - 1 {
            self.pop().unwrap()
        } else {
            let label = self.labels.swap_remove(node);
            // safety: again because of safety invariant we know self.len > 1 and
            // furthermore it is node < self.len - 1, so labels[node] is valid.
            let new_label = unsafe { self.labels.get_unchecked(node) };
            self.invert_labels.remove(&label);
            *self.invert_labels.get_mut(new_label).unwrap() = node;
            self.nodes.swap_remove(node)
        }
    }

    fn swap_remove(&mut self, node: Node) -> Option<Neighbours> {
        if node >= self.len() {
            return None;
        }
        // safety: node is in bounds
        Some(unsafe { self.swap_remove_unchecked(node) })
    }
}

impl GraphDataSpecializerHelper for Custom {
    unsafe fn raw_node_swap_remove(&mut self, node: Node) {
        // for a safer version, see what we how we do it for IndexMap

        let ptr = self.nodes.as_mut_ptr();
        // safety: API safety invariant promises that node is valid
        let neighbours = unsafe { self.nodes.get_unchecked_mut(node) };
        for &neighbour in neighbours.iter() {
            // safety: API safety invariant promises that neighbour is valid. Furthermore
            // we are not aliasing here, again because of the API safety invariant.
            unsafe { (*ptr.add(neighbour)).remove(&node) };
        }
        // safety: API safety invariant promises that node is valid
        unsafe { self.swap_remove_unchecked(node) };
    }

    unsafe fn raw_node_neighbours_update(&mut self, node: Node, before: &Node) {
        // for a safer version, see what we how we do it for IndexMap

        let ptr = self.nodes.as_mut_ptr();
        // safety: API safety invariant promises that node is valid
        let neighbours = unsafe { self.nodes.get_unchecked_mut(node) };
        for &neighbour in neighbours.iter() {
            // safety: API safety invariant promises that neighbour is valid. Furthermore
            // we are not aliasing here, again because of the API safety invariant.
            let neighbour_neighbours = unsafe { &mut *ptr.add(neighbour) };
            neighbour_neighbours.remove(before);
            neighbour_neighbours.insert(node);
        }
    }
}
