use std::mem;

use hashbrown::HashSet;
use indexmap::map::Entry;

use super::super::GraphDataSpecializerHelper;
use crate::graph::specialised::{GraphData, Label, Neighbours, Node};

pub type IndexMap = indexmap::IndexMap<Label, Neighbours>;

impl GraphData for IndexMap {
    unsafe fn get_index_unchecked(&self, label: Label) -> Node {
        // TODO: actually do something faster unsafe here
        self.get_full(&label).unwrap().0
    }

    unsafe fn get_label_unchecked(&self, node: Node) -> Label {
        // cf. get_index_unchecked
        *self.get_index(node).unwrap().0
    }

    unsafe fn get_neighbours_unchecked(&self, node: Node) -> &Neighbours {
        // cf. get_index_unchecked
        self.get_index(node).unwrap().1
    }

    unsafe fn get_neighbours_mut_unchecked(&mut self, node: Node) -> &mut Neighbours {
        // cf. get_index_unchecked
        self.get_index_mut(node).unwrap().1
    }

    fn get_index(&self, label: Label) -> Option<Node> {
        self.get_full(&label).map(|(idx, _, _)| idx)
    }

    fn get_label(&self, node: Node) -> Option<Label> {
        self.get_index(node).map(|(label, _)| *label)
    }

    fn get_neighbours(&self, node: Node) -> Option<&Neighbours> {
        self.get_index(node).map(|(_, neighbours)| neighbours)
    }

    fn get_neighbours_mut(&mut self, node: Node) -> Option<&mut Neighbours> {
        self.get_index_mut(node).map(|(_, neighbours)| neighbours)
    }

    fn get_index_or_insert(&mut self, label: Label) -> Node {
        match self.entry(label) {
            Entry::Occupied(e) => e.index(),
            Entry::Vacant(e) => {
                let idx = e.index();
                e.insert(HashSet::new());
                idx
            },
        }
    }

    fn get_neighbours_mut_or_insert(&mut self, label: Label) -> &mut Neighbours {
        self.entry(label).or_default()
    }

    fn get_index_and_neighbours_mut_or_insert(
        &mut self,
        label: Label,
    ) -> (Node, &mut HashSet<Node>) {
        match self.entry(label) {
            Entry::Occupied(e) => (e.index(), e.into_mut()),
            Entry::Vacant(e) => (e.index(), e.insert(HashSet::new())),
        }
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn iter_neighbours(&self) -> impl Iterator<Item = &Neighbours> {
        self.values()
    }

    fn iter_neighbours_mut(&mut self) -> impl Iterator<Item = &mut Neighbours> {
        self.values_mut()
    }

    fn enumerate_neighbours(&self) -> impl Iterator<Item = (Node, &Neighbours)> + Clone {
        // enumerate() makes sense here; cf. doc (or code) of values()
        self.values().enumerate()
    }

    fn enumerate_full(&self) -> impl Iterator<Item = (Node, Label, &Neighbours)> {
        // enumerate() makes sense here; cf. doc (or code) of values()
        self.iter()
            .enumerate()
            .map(|(idx, (label, neighbours))| (idx, *label, neighbours))
    }

    fn pop(&mut self) -> Option<Neighbours> {
        self.pop().map(|(_, neighbours)| neighbours)
    }

    unsafe fn swap_remove_unchecked(&mut self, node: Node) -> Neighbours {
        // TODO: actually do something faster unsafe here
        self.swap_remove_index(node).unwrap().1
    }

    fn swap_remove(&mut self, node: Node) -> Option<Neighbours> {
        self.swap_remove_index(node).map(|(_, neighbours)| neighbours)
    }
}

impl GraphDataSpecializerHelper for IndexMap {
    // we could also do something similar as for the custom::Foo by getting mut refs to
    // the entries and then transmute_copy them or turn them into raw pointers; while the
    // aliasing rules would probably hold, it relies on that the getter methods of
    // IndexMap don't do something unexpected (which would probably be catch by miri, but
    // I don't want to rely on that); I couldn't really find a way around the getter
    // methods of IndexMap, because it does not provide any public raw access methods to
    // the underlining vector of entries
    unsafe fn raw_node_swap_remove(&mut self, node: Node) {
        let neighbours = mem::take(
            // safety: API safety invariant promises that node is valid
            unsafe { self.get_neighbours_mut_unchecked(node) },
        );
        for neighbour in neighbours {
            // safety: API safety invariant promises that neighbour is valid
            unsafe { self.get_neighbours_mut_unchecked(neighbour) }.remove(&node);
        }
        // safety: API safety invariant promises that node is valid
        unsafe { self.swap_remove_unchecked(node) };
    }

    unsafe fn raw_node_neighbours_update(&mut self, node: Node, before: &Node) {
        let neighbours = mem::take(
            // safety: API safety invariant promises that node is valid
            unsafe { self.get_neighbours_mut_unchecked(node) },
        );
        for &neighbour in neighbours.iter() {
            let neighbour_neighbours =
                // safety: API safety invariant promises that neighbour is valid
                unsafe { self.get_neighbours_mut_unchecked(neighbour) };
            neighbour_neighbours.remove(before);
            neighbour_neighbours.insert(node);
        }
        let _ = mem::replace(
            // safety: still valid
            unsafe { self.get_neighbours_mut_unchecked(node) },
            neighbours,
        );
    }
}
