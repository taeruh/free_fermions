use itertools::Itertools;

type Graph = petgraph::graph::Graph<(), (), petgraph::Undirected, u32>;

#[allow(dead_code)]
pub fn run() {
    let size = 6;

    let edge_pool = (0u32..size - 1).flat_map(|i| (i + 1..size).map(move |j| (i, j)));

    let mut counter = vec![0; (size * (size - 1) / 2 + 1) as usize];

    let mut graph = Graph::new_undirected();
    for _ in 0..size {
        graph.add_node(());
    }

    for edges in edge_pool.powerset() {
        let mut graph = graph.clone();
        graph.extend_with_edges(&edges);
        if naive_has_claw(&graph) {
            counter[edges.len()] += 1;
        }
    }
    for (i, count) in counter.iter().enumerate() {
        println!("{i:2} edges: {count:5} non-claw-free");
    }
}

fn naive_has_claw(graph: &Graph) -> bool {
    for node in graph.node_indices() {
        let neighbours = graph.neighbors(node).collect::<Vec<_>>();
        for (i, a) in neighbours.iter().enumerate() {
            let b_start = i + 1;
            for (j, b) in neighbours[b_start..].iter().enumerate() {
                let c_start = b_start + j + 1;
                for c in neighbours[c_start..].iter() {
                    if !graph.contains_edge(*a, *b)
                        && !graph.contains_edge(*a, *c)
                        && !graph.contains_edge(*b, *c)
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}
