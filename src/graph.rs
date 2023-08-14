use std::{
    borrow::Borrow,
    collections::{
        HashMap,
        HashSet,
    },
};

use super::hamiltonian::Operator;

pub type Node = Vec<usize>;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

pub type ReducedNode = HashSet<usize>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReducedGraph {
    pub nodes: HashMap<usize, ReducedNode>,
}

impl Graph {
    #[inline(always)]
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Alias for the corresponding [FromIterator::from_iter] method (which sorts
    /// automatically).
    ///
    /// The neighbors are sorted from lowest to highest, for each node.
    #[inline(always)]
    pub fn sorted_from_iter<'l, T: IntoIterator<Item = &'l Operator>>(iter: T) -> Self {
        FromIterator::from_iter(iter)
    }

    /// Alias for corresponding [From::from] method (which sorts automatically).
    ///
    /// The neighbors are sorted from lowest to highest, for each node.
    #[inline(always)]
    pub fn sorted_from<O: Borrow<Operator>>(ops: &[O]) -> Self {
        From::from(ops)
    }

    /// Remove *false twins* and singular nodes.
    ///
    /// It is assumed that the neighbors of the nodes are sorted; which kind of sorting
    /// doesn't matter, it only has to be consistent
    ///
    /// Note that elements are `swap_remove`d, so the order of the nodes, and their
    /// neighbours, is not preserved.
    pub fn reduce(self) -> ReducedGraph {
        let mut representatives = HashMap::new();
        let mut twins_set = Vec::new();

        let mut rest = self.nodes;
        let mut idcs = (0..rest.len()).collect::<Vec<_>>();
        while let (Some(node), Some(idx)) = (rest.pop(), idcs.pop()) {
            if node.is_empty() {
                continue;
            }

            let mut twins_pos = Vec::new();
            let mut twins_idx = Vec::new();
            'outer: for (tp, (other, &ti)) in rest.iter().zip(idcs.iter()).enumerate() {
                if node.len() != other.len() {
                    continue;
                }
                for (s, o) in node.iter().zip(other.iter()) {
                    if s != o {
                        continue 'outer;
                    }
                }
                twins_idx.push(ti);
                twins_pos.push(tp);
            }

            if !twins_idx.is_empty() {
                let neighbors = {
                    let tp = *twins_pos.last().expect("checked len > 0");
                    idcs.swap_remove(tp);
                    rest.swap_remove(tp)
                };
                for &tp in twins_pos[..twins_pos.len() - 1].iter().rev() {
                    idcs.swap_remove(tp);
                    rest.swap_remove(tp);
                }
                twins_set.push((neighbors, twins_idx));
            }

            representatives.insert(idx, HashSet::from_iter(node));
        }

        debug_assert!(rest.is_empty());

        for (neighbors, twins) in twins_set {
            for neighbor in neighbors {
                match representatives.get_mut(&neighbor) {
                    Some(node) => {
                        for twin in twins.iter() {
                            node.remove(twin);
                        }
                    }
                    None => continue,
                };
            }
        }

        ReducedGraph { nodes: representatives }
    }

    pub fn claw_check(&self) -> bool {
        todo!()
    }
}

/// The neighbors are sorted.
impl<'l> FromIterator<&'l Operator> for Graph {
    fn from_iter<T: IntoIterator<Item = &'l Operator>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let size = iter.size_hint().0;
        let mut ops: Vec<&Operator> = Vec::with_capacity(size);
        let mut nodes: Vec<Node> = Vec::with_capacity(size);

        for (i, op) in iter.enumerate() {
            let mut node = Node::new();
            for (j, other_op) in ops.iter().enumerate() {
                if op.commute(other_op) {
                    node.push(j);
                    nodes[j].push(i)
                }
            }
            nodes.push(node);
            ops.push(op);
        }

        Self { nodes }
    }
}

/// The neighbors are sorted.
impl<O: Borrow<Operator>> From<&[O]> for Graph {
    fn from(ops: &[O]) -> Self {
        let iter = ops.iter();
        let mut nodes: Vec<Node> = Vec::with_capacity(ops.len());

        for (i, op) in iter.enumerate() {
            let mut node = Node::new();
            for (j, other_op) in ops[..i].iter().enumerate() {
                if op.borrow().commute(other_op.borrow()) {
                    node.push(j);
                    nodes[j].push(i)
                }
            }
            nodes.push(node);
        }

        Self { nodes }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::hamiltonian::OperatorPool;

    #[test]
    fn from_sanity() {
        let mut pool = OperatorPool::new(5);
        let sample = pool.draw(10).collect::<Vec<_>>();

        assert_eq!(
            Graph::from_iter(sample.iter().map(Deref::deref)),
            Graph::from(sample.as_slice())
        );
    }

    #[test]
    fn reduction() {
        let hamiltonians = [
            // the nodes and neighbors have to be sorted for the left tuple entry!
            (vec![], vec![]),
            (vec![vec![]], vec![]),
            (vec![vec![0]], vec![(0, vec![0])]),
            (vec![vec![0], vec![]], vec![(0, vec![0])]),
            (
                vec![vec![2], vec![2], vec![1, 0]],
                vec![(1, vec![2]), (2, vec![1])],
            ),
            (
                vec![vec![1], vec![0, 2], vec![1]],
                vec![(2, vec![1]), (1, vec![2])],
            ),
            (vec![vec![0, 1], vec![0]], vec![(0, vec![0, 1]), (1, vec![0])]),
            (
                vec![vec![2, 3], vec![2, 3], vec![0, 1, 3], vec![0, 1, 2], vec![]],
                vec![(1, vec![2, 3]), (2, vec![1, 3]), (3, vec![1, 2])],
            ),
        ];

        for (origin, expected) in hamiltonians.into_iter() {
            let graph = Graph { nodes: origin };
            let reduced_graph = graph.clone().reduce();
            assert_eq!(
                reduced_graph.nodes,
                HashMap::from_iter(
                    expected.into_iter().map(|(i, e)| (i, HashSet::from_iter(e)))
                )
            )
        }
    }
}
