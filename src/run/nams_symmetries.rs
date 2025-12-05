use std::fs;

use hashbrown::HashSet;
use serde::Deserialize;

use super::GenGraph;
use crate::{
    fix_int::int,
    graph::generic::{ImplGraph, algorithms::is_line_graph::SageProcess},
    hamiltonian::{self, PauliString},
};

#[derive(Deserialize)]
struct Data {
    operators: Vec<String>,
    weights: Vec<f64>,
}

#[derive(Debug)]
struct Hamiltonian {
    operators: Vec<PauliString>,
    weights: Vec<f64>,
}

impl Hamiltonian {
    fn from_text_data(data: &str) -> Self {
        let data: Data = serde_json::from_str(data).unwrap();
        Self {
            operators: data.operators.iter().map(|s| PauliString::from_str(s)).collect(),
            weights: data.weights,
        }
    }

    fn get_edges_and_vertex_map(&self) -> (Vec<(int, int)>, Vec<usize>) {
        // the edges function below labels the edges according to the order of the
        // operators in self
        (
            hamiltonian::get_edges(&self.operators),
            (0..self.operators.len()).collect(),
        )
    }
}

pub fn run() {
    let mut sage_process = SageProcess::default();

    // let file = "data/nam1.json";
    let file = "data/nam2.json";
    let ham = Hamiltonian::from_text_data(&fs::read_to_string(file).unwrap());

    let (edges, map) = ham.get_edges_and_vertex_map();

    let mut graph = GenGraph::from_edge_labels(edges).unwrap();

    // let label = graph.get_label(487).unwrap();
    // println!("label of vertex 487: {:?}", label);
    // panic!("stop here");

    let mut tree = graph.modular_decomposition();
    let cgraph = graph.clone();

    let sibling_sets = graph.trace_twin_collapse(&mut tree, &mut sage_process);

    for siblings in &sibling_sets {
        println!("{:?}, {}", siblings.vertices, siblings.typ);
        println!(
            "{:?}",
            siblings
                .vertices
                .iter()
                .map(|v| ham.operators[map[*v as usize]].draw_as_pauli_string())
                .collect::<Vec<_>>()
        );
        // this is specifically to nam1.json, as for that, one we apparently only have
        // false twins; let's check whether they are actually false twins:
        let get_neighbours = |v| {
            cgraph
                .get_neighbours(cgraph.find_node(v).unwrap())
                .unwrap()
                .map(|n| n.index())
                .collect::<HashSet<usize>>()
        };
        let neighbourhood = get_neighbours(siblings.vertices[0]);
        for &v in &siblings.vertices[1..] {
            assert_eq!(&neighbourhood, &get_neighbours(v));
        }
        println!();
    }

    println!("{:?}", sibling_sets.len());

    // println!();
    // cgraph.get_neighbours(544).iter().for_each(|n| {
    //     println!("neighbor: {:?}\n", n.clone().collect::<Vec<_>>());
    // });
    // cgraph.get_neighbours(439).iter().for_each(|n| {
    //     println!("neighbor: {:?}", n.clone().collect::<Vec<_>>());
    // });
}
