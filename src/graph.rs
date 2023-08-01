use super::hamiltonian::Operator;

pub type Node = Vec<usize>;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Graph {
    nodes: Vec<Node>,
}

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

impl<'l> From<Vec<&'l Operator>> for Graph {
    fn from(ops: Vec<&'l Operator>) -> Self {
        let iter = ops.iter();
        let mut nodes: Vec<Node> = Vec::with_capacity(ops.len());

        for (i, op) in iter.enumerate() {
            let mut node = Node::new();
            for (j, other_op) in ops[..i].iter().enumerate() {
                if op.commute(other_op) {
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
            Graph::from(sample)
        );
    }
}
