use super::modular_decomposition::Tree;
use crate::{
    enumerate_offset::Enumerate,
    graph::{Graph, ImplGraph, Node, NodeCollection},
    mat_mul::Matrix,
};

// No might get some data in the future
pub enum ClawFree {
    Yes,
    No,
}

// impl<G: ImplGraph> Graph<G> {
impl<G: ImplGraph + std::fmt::Debug> Graph<G> {
    pub fn is_claw_free(&self, tree: &Tree) -> ClawFree {
        todo!()
    }

    pub fn is_claw_free_naive(&self) -> ClawFree {
        for (node, neighbourhood) in self.iter_with_neighbourhoods() {
            let mut graph = self.subgraph(neighbourhood);
            graph.complement();
            let (indices, matrix) = to_matrix(&graph);
            let count = matrix.diag_cube();
            println!(
                "node {}:\nidcs: {indices:?}\ncnt:  {count:?}\n",
                self.get_label(node).unwrap()
            );
        }
        ClawFree::Yes
    }
}

fn to_matrix<G: ImplGraph>(graph: &Graph<G>) -> (Vec<Node>, Matrix) {
    let len = graph.len();
    let mut array = vec![0; len * len];
    let mut indices = Vec::with_capacity(len);
    let mut nodes = graph.iter_with_neighbourhoods().enumerate();
    while let Some((row, (node, _))) = nodes.next() {
        indices.push(graph.get_label(node).unwrap());
        let row_shift = row * len;
        for (col, (_, neighborhood)) in nodes.clone() {
            let has_edge = neighborhood.contains(node);
            array[row_shift + col] = has_edge.into();
            array[col * len + row] = has_edge.into();
        }
    }
    (indices, Matrix::from_vec_with_shape(array, (len, len)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::test_utils::collect;

    #[test]
    fn test() {
        //    - 1
        //  /
        // 0 -- 2
        //  \
        //    - 3
        let graph: Graph = Graph::from_adjacencies(collect!(vv;
                (0, [1, 2, 3]),
                (1, [0]),
                (2, [0]),
                (3, [0]),
        ))
        .unwrap();
        // println!("{:?}", graph);
        // graph.is_claw_free_naive();
        // // #claws = 1

        //    - 1 -- 4
        //  /
        // 0 -- 2 -- 5
        //  \
        //    - 3 -- 6
        let graph: Graph = Graph::from_adjacencies(collect!(vv;
            (0, [1, 2, 3]),
            (1, [0, 4]),
            (2, [0, 5]),
            (3, [0, 6]),
            (4, [1]),
            (5, [2]),
            (6, [3]),
        ))
        .unwrap();
        // println!("{:?}", graph);
        // graph.is_claw_free_naive();

        // 10 -- 7 -     - 1 -- 4
        //           \ /
        // 11 -- 8 -- 0 -- 2 -- 5
        //           / \
        // 13 -- 9 -     - 3 -- 6
        let graph: Graph = Graph::from_adjacencies(collect!(vv;
                (0, [1, 2, 3, 7, 8, 9]),
                (1, [0, 4]),
                (2, [0, 5]),
                (3, [0, 6]),
                (4, [1]),
                (5, [2]),
                (6, [3]),
                (7, [0, 10]),
                (8, [0, 11]),
                (9, [0, 12]),
                (10, [7]),
                (11, [8]),
                (12, [9]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();
        // #claws = binom(6, 3) = 20

        //    - 1 -
        //  /       \
        // 0 -- 2 -- 4
        //  \
        //    - 3
        let graph: Graph = Graph::from_adjacencies(collect!(vv;
                (0, [1, 2, 3]),
                (1, [0, 4]),
                (2, [0, 4]),
                (3, [0]),
                (4, [1, 2]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();

        //    - 1
        //  /
        // 0 -- 2
        //  \   |
        //    - 3
        let graph: Graph = Graph::from_adjacencies(collect!(vv;
                (0, [1, 2, 3]),
                (1, [0]),
                (2, [0, 3]),
                (3, [0, 2]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();

        // 7       - 1 -- 4
        // |     /
        // 8 -- 0 -- 2 -- 5
        // |     \
        // 9 --10  - 3 -- 6
        let mut graph: Graph = Graph::from_adjacencies(collect!(vv;
                (0, [1, 2, 3, 8]),
                (1, [0, 4]),
                (2, [0, 5]),
                (3, [0, 6]),
                (4, [1]),
                (5, [2]),
                (6, [3]),
                (7, [8]),
                (8, [0, 7, 9]),
                (9, [8, 10]),
                (10, [9]),
        ))
        .unwrap();
        // graph.is_claw_free_naive();
        // graph.remove_node(graph.find_node(0).unwrap());
        // graph.is_claw_free_naive();

        // 7 -     - 1 -- 4
        // |   \ /
        // 8 -- 0 -- 2 -- 5
        //       \
        //         - 3 -- 6
        let graph: Graph = Graph::from_adjacencies(collect!(vv;
                (0, [1, 2, 3, 7, 8]),
                (1, [0, 4]),
                (2, [0, 5]),
                (3, [0, 6]),
                (4, [1]),
                (5, [2]),
                (6, [3]),
                (7, [0, 8]),
                (8, [0, 7]),
        ))
        .unwrap();
        graph.is_claw_free_naive();
        // #claws = binom(6, 3) = 20
    }
}
