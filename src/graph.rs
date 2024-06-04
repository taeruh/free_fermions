use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    mem,
};

use super::hamiltonian::Operator;
use crate::{enumerate_offset::Enumerate, matrix::MatrixTools};
type Matrix = ndarray::Array2<u32>;

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
                    },
                    None => continue,
                };
            }
        }

        ReducedGraph { nodes: representatives }
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
                if !op.commute(other_op) {
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
                if !op.borrow().commute(other_op.borrow()) {
                    node.push(j);
                    nodes[j].push(i)
                }
            }
            nodes.push(node);
        }

        Self { nodes }
    }
}

impl ReducedGraph {
    pub fn complement(&mut self) {
        let vertices = self.nodes.keys().copied().collect::<Vec<_>>();
        for node in self.nodes.iter_mut() {
            let mut neighbourhood = mem::take(node.1);
            neighbourhood.insert(*node.0);
            for vertex in vertices.iter() {
                if !neighbourhood.contains(vertex) {
                    node.1.insert(*vertex);
                }
            }
        }
    }

    pub fn remove_node(&mut self, node: usize) -> Option<HashSet<usize>> {
        let neighbors = self.nodes.remove(&node)?;
        for neighbor in neighbors.iter() {
            self.nodes.get_mut(neighbor).expect("graph incomplete").remove(&node);
        }
        Some(neighbors)
    }

    /// don't include the node itself, because it is not connected to anything
    pub fn ordered_complementary_neighborhood_subgraph(
        &self,
        neighbors: &HashSet<usize>,
    ) -> (Vec<usize>, Matrix) {
        let mut neighbor_idcs = Vec::with_capacity(neighbors.len());
        let neighbor_nodes = neighbors
            .iter()
            .map(|k| {
                neighbor_idcs.push(*k);
                (*k, self.nodes.get(k).unwrap())
            })
            .collect::<Vec<_>>();
        (neighbor_idcs, complementary_subgraph(&neighbor_nodes[..]))
    }

    pub fn complementary_neighborhood_subgraph(
        &self,
        neighbors: &HashSet<usize>,
    ) -> Matrix {
        complementary_subgraph(
            &neighbors
                .iter()
                .map(|k| (*k, self.nodes.get(k).unwrap()))
                .collect::<Vec<_>>()[..],
        )
    }

    pub fn ordered_raw_node_claw_count(
        &self,
        neighbors: &HashSet<usize>,
    ) -> (Vec<usize>, Vec<u32>) {
        if neighbors.len() < 3 {
            return (Vec::new(), Vec::new());
        }
        let (idcs, array) = self.ordered_complementary_neighborhood_subgraph(neighbors);
        // why is this the claw count for this node:
        // - cf. impl of diag_cube; diag_i = sum_k sum_j a_ik * a_kj * a_ji
        // - counting claws is the same as counting triangles in the complementary
        // graph;
        // - for each i, each single summand a_ik * a_kj * a_ji is only 1 if i,
        // k, j build a triangle
        // - when calculating the trace and if we have a triangle r, s, t, then there
        // are 3! = 6 terms in the sum that represent the triangle
        // - regarding the single diag entries:
        // -- diag_i, for i neq 0, counts the number of triangles that contain i, but
        // multiplied by a factor of 2, since the sums over k and j are symmetric and
        // count the same triangle twice; when summing over all diag_i, after dividing
        // by 2, the same triangle is counted 3 times, since it is counted by 3 diag_i
        // element, so in total we again have to divide by 6
        (idcs, array.diagonal_of_cubed())
    }

    pub fn raw_node_claw_count(&self, neighbors: &HashSet<usize>) -> Vec<u32> {
        if neighbors.len() < 3 {
            return Vec::new();
        }
        self.complementary_neighborhood_subgraph(neighbors)
            .diagonal_of_cubed()
    }

    pub fn check_all(&self) -> bool {
        for (node, neighbors) in self.nodes.iter() {
            println!("{:?}", (node, self.ordered_raw_node_claw_count(neighbors)));
        }
        false
    }

    pub fn has_claw(&self) -> bool {
        for (_, neighbors) in self.nodes.iter() {
            for count in self.raw_node_claw_count(neighbors) {
                if count > 0 {
                    return true;
                }
            }
        }
        false
    }
}

impl<I: IntoIterator<Item = usize>> FromIterator<(usize, I)> for ReducedGraph {
    fn from_iter<T: IntoIterator<Item = (usize, I)>>(iter: T) -> Self {
        Self {
            nodes: HashMap::from_iter(
                iter.into_iter().map(|(i, e)| (i, HashSet::from_iter(e))),
            ),
        }
    }
}

pub fn complementary_subgraph(nodes: &[(usize, &ReducedNode)]) -> Matrix {
    let len = nodes.len();
    // directly calculate the complement, instead of calculating the real subgraph
    // and then complementing it
    let mut array = vec![0; len * len];
    for (row, (node, _)) in nodes.iter().enumerate() {
        let row_shift = row * len;
        for (col, (_, neighborhood)) in Enumerate::new(nodes[row + 1..].iter(), row + 1) {
            let has_edge = !neighborhood.contains(node); // complement!
            array[row_shift + col] = has_edge.into();
            array[col * len + row] = has_edge.into();
        }
    }
    Matrix::from_vec_with_shape(array, (len, len)).unwrap()

    // about the second loop above, one should test which one is the best
    //
    // let mut iter = neighbor_nodes.iter().enumerate();
    // iter.nth(row);
    // for (col, (_, neighbor)) in iter {
    //
    // for (col, (_, neighbor)) in neighbor_nodes[row + 1..].iter().enumerate() {
    //     let col = col + row + 1;
    //
    // for (col, (_, neighbor)) in
    //     Enumerate::new(neighbor_nodes[row + 1..].iter(), row + 1)
    // {
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
            (vec![vec![2], vec![2], vec![1, 0]], vec![(1, vec![2]), (2, vec![1])]),
            (vec![vec![1], vec![0, 2], vec![1]], vec![(2, vec![1]), (1, vec![2])]),
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

    #[test]
    fn claws() {
        //

        // //    - 1
        // //  /
        // // 0 -- 2
        // //  \
        // //    - 3
        // let graph = Graph {
        //     nodes: vec![
        //         vec![1, 2, 3],
        //         vec![0],
        //         vec![0],
        //         vec![0],
        //         // ... preventing line concatenation on format
        //     ],
        // };
        // graph.reduce().check_all();
        // // #claws = 1

        // //    - 1 -- 4
        // //  /
        // // 0 -- 2 -- 5
        // //  \
        // //    - 3 -- 6
        // let graph = Graph {
        //     nodes: vec![
        //         vec![1, 2, 3],
        //         vec![0, 4],
        //         vec![0, 5],
        //         vec![0, 6],
        //         vec![1],
        //         vec![2],
        //         vec![3],
        //     ],
        // }
        // .reduce();
        // graph.check_all();
        // println!("has claw: {:?}", graph.has_claw());

        // // 10 -- 7 -     - 1 -- 4
        // //           \ /
        // // 11 -- 8 -- 0 -- 2 -- 5
        // //           / \
        // // 13 -- 9 -     - 3 -- 6
        // let graph = Graph {
        //     nodes: vec![
        //         vec![1, 2, 3, 7, 8, 9],
        //         vec![0, 4],
        //         vec![0, 5],
        //         vec![0, 6],
        //         vec![1],
        //         vec![2],
        //         vec![3],
        //         vec![0, 10],
        //         vec![0, 11],
        //         vec![0, 12],
        //         vec![7],
        //         vec![8],
        //         vec![9],
        //     ],
        // }
        // .reduce();
        // graph.check_all();
        // // #claws = binom(6, 3) = 20
        // println!("has claw: {:?}", graph.has_claw());

        // //    - 1 -
        // //  /       \
        // // 0 -- 2 -- 4
        // //  \
        // //    - 3
        // let graph = Graph {
        //     nodes: vec![
        //         vec![1, 2, 3],
        //         vec![0, 4],
        //         vec![0, 4],
        //         vec![0],
        //         vec![1, 2],
        //         // ... preventing line concatenation on format
        //     ],
        // }
        // .reduce();
        // graph.check_all();
        // println!("has claw: {:?}", graph.has_claw());

        // //    - 1
        // //  /
        // // 0 -- 2
        // //  \   |
        // //    - 3
        // let graph = Graph {
        //     nodes: vec![
        //         vec![1, 2, 3],
        //         vec![0],
        //         vec![0, 3],
        //         vec![0, 2],
        //         // ... preventing line concatenation on format
        //     ],
        // };
        // graph.reduce().check_all();

        // // 7       - 1 -- 4
        // // |     /
        // // 8 -- 0 -- 2 -- 5
        // // |     \
        // // 9 --10  - 3 -- 6
        // let graph = Graph {
        //     nodes: vec![
        //         vec![1, 2, 3, 8],
        //         vec![0, 4],
        //         vec![0, 5],
        //         vec![0, 6],
        //         vec![1],
        //         vec![2],
        //         vec![3],
        //         vec![8],
        //         vec![0, 7, 9],
        //         vec![8, 10],
        //         vec![9],
        //     ],
        // };
        // let mut reduced = graph.reduce();
        // reduced.check_all();
        // reduced.remove_node(0);
        // let (node, neighbors) = reduced.nodes.get_key_value(&8).unwrap();
        // println!("\n{:?}", (node, reduced.raw_claw_count(neighbors)));
        // // reduced.check_all();

        // // 7 -     - 1 -- 4
        // // |   \ /
        // // 8 -- 0 -- 2 -- 5
        // //       \
        // //         - 3 -- 6
        // let graph = Graph {
        //     nodes: vec![
        //         vec![1, 2, 3, 7, 8],
        //         vec![0, 4],
        //         vec![0, 5],
        //         vec![0, 6],
        //         vec![1],
        //         vec![2],
        //         vec![3],
        //         vec![0, 8],
        //         vec![0, 7],
        //     ],
        // };
        // graph.reduce().check_all();
        // // #claws = binom(6, 3) = 20

        //
    }
}
