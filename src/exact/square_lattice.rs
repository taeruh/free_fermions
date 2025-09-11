use std::fs;

use serde::{Deserialize, Serialize};

use crate::{
    binary_search::TreeStack,
    graph::generic::{self, algorithms::is_line_graph::SageProcess, ImplGraph, Pet},
    hamiltonian::{self, LocalOperator, Pauli, DOUBLES},
    run::check,
};

type GenGraph = generic::Graph<Pet>;

const N: usize = 9 * 3;

#[derive(Clone, Debug)]
struct State {
    ee_horizontal: Vec<(Pauli, Pauli)>,
    ee_vertical: Vec<(Pauli, Pauli)>,
    e_nuclei: Vec<(Pauli, Pauli)>,
    orig_valid: bool,
    coll_valid: bool,
    collapsed: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Result {
    orig_valid: bool,
    coll_valid: bool,
    collapsed: f64,
    is_2d: bool,
    is_empty: bool,
    num_ops: usize,
}

fn check(state: &mut State, sage_process: &mut SageProcess) -> bool {
    let mut operators = Vec::with_capacity(
        state.ee_horizontal.len() * 12 + state.ee_vertical.len() * 12 + state.e_nuclei.len() * 9,
    );

    for i in 0..3 {
        for j in 0..3 {
            for p in state.e_nuclei.iter() {
                operators.push(LocalOperator {
                    index: [1 + i * 3 + j, 10 + i * 3 + j],
                    operator_at_index: [p.0, p.1],
                });
            }
        }
    }
    for i in 0..3 {
        for j in 0..3 {
            let site = 1 + i * 3 + j;
            let horizontal = 1 + i * 3 + (j + 1) % 3;
            let vertical = 1 + (i + 1) % 3 * 3 + j;
            for p in state.ee_horizontal.iter() {
                operators.push(LocalOperator {
                    index: [site, horizontal],
                    operator_at_index: [p.0, p.1],
                });
            }
            for p in state.ee_vertical.iter() {
                operators.push(LocalOperator {
                    index: [site, vertical],
                    operator_at_index: [p.0, p.1],
                });
            }
        }
    }

    let edges = hamiltonian::get_edges(&operators);
    if edges.is_empty() {
        state.orig_valid = true;
        state.coll_valid = true;
        state.collapsed = 0.0;
        return true; // no edges means the graph is trivially valid
    }

    let mut graph = GenGraph::from_edge_labels(edges).unwrap();

    let orig_len = graph.len();
    let mut tree = graph.modular_decomposition();
    let check = check::do_gen_check(&graph, &tree);
    state.orig_valid = check.simplicial;
    graph.twin_collapse(&mut tree, sage_process);
    state.collapsed = (orig_len - graph.len()) as f64 / orig_len as f64;
    let check = check::do_gen_check(&graph, &tree);
    state.coll_valid = check.simplicial;
    state.coll_valid
}

pub fn run() {
    let init_state = State {
        ee_horizontal: vec![],
        ee_vertical: vec![],
        e_nuclei: vec![],
        orig_valid: false,
        coll_valid: false,
        collapsed: 0.0,
    };

    let mut sage_process = SageProcess::default();

    let mut f_true = |state: &mut State, index: usize| -> bool {
        let pindex = index / 3;
        match index % 3 {
            0 => {
                state.ee_horizontal.push(DOUBLES[pindex]);
            }
            1 => {
                state.ee_vertical.push(DOUBLES[pindex]);
            }
            2 => {
                state.e_nuclei.push(DOUBLES[pindex]);
            }
            _ => unreachable!(),
        }
        check(state, &mut sage_process)
    };
    fn f_false(_: &mut State, _: usize) -> bool {
        // the state stays the same and since we come from a valid state, it is valid
        true
    }
    fn f_result(state: State) -> Result {
        Result {
            orig_valid: state.orig_valid,
            coll_valid: state.coll_valid,
            collapsed: state.collapsed,
            is_2d: !state.ee_horizontal.is_empty() && !state.ee_vertical.is_empty(),
            is_empty: state.ee_horizontal.is_empty()
                && state.ee_vertical.is_empty()
                && state.e_nuclei.is_empty(),
            num_ops: state.ee_horizontal.len() + state.ee_vertical.len() + state.e_nuclei.len(),
        }
    }

    let mut tree = TreeStack::<State, Result, N>::new(init_state);
    tree.search(&mut f_true, &mut f_false, &f_result);
    let results = tree.into_results();

    fs::write(
        "output/exact_square_lattice.json",
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();
}

pub fn run_analyse() {
    let results: Vec<Result> =
        serde_json::from_str(&fs::read_to_string("output/exact_square_lattice.json").unwrap())
            .unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    struct Coefficients {
        scf: [f64; N],
        dscf: [f64; N], // Delta SCF
        collapsed: [f64; N],
    }
    let mut coefficients = Coefficients {
        scf: [0.0; N],
        dscf: [0.0; N],
        collapsed: [0.0; N],
    };

    for instance in results.into_iter() {
        if instance.is_empty || !instance.is_2d {
            continue;
        }
        let index = instance.num_ops;
        assert!(
            index > 1,
            "graph cannot be two-dimensional with only one operator"
        );
        assert!(
            instance.coll_valid,
            "all graphs should be scf after the collapse"
        );
        coefficients.scf[index] += 1.0;
        if !instance.orig_valid {
            coefficients.dscf[index] += 1.0;
        }
        coefficients.collapsed[index] += instance.collapsed;
    }

    // note that the coefficients are not normalised as we do not accept non-2d lattices;
    // we do the normalisation when plotting as it actually depends on the drawing
    // probability, i.e., we cannot just universally normalise the coefficients

    println!("{coefficients:?}");

    fs::write(
        "output/exact_square_lattice_coefficients.json",
        serde_json::to_string_pretty(&coefficients).unwrap(),
    )
    .unwrap();
}
