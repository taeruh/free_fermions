use std::fs;

use serde::{Deserialize, Serialize};

use crate::{
    binary_search::TreeStack,
    graph::generic::{self, algorithms::is_line_graph::SageProcess, ImplGraph, Pet},
    hamiltonian::{self, LocalOperator, Pauli, DOUBLES},
    run::check,
};

type GenGraph = generic::Graph<Pet>;

const N: usize = 9 * 5;

#[derive(Clone, Debug)]
struct State {
    e1: Vec<(Pauli, Pauli)>,
    e2: Vec<(Pauli, Pauli)>,
    e3: Vec<(Pauli, Pauli)>,
    e4: Vec<(Pauli, Pauli)>,
    e5: Vec<(Pauli, Pauli)>,
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
    let mut operators = Vec::new();

    for i in 0..2 {
        let row = i * 8;
        for j in 0..2 {
            let col = j * 4;
            for p in state.e5.iter() {
                operators.push(LocalOperator {
                    index: [row + col, ((row + 8) % 16) + ((col + 7) % 8)],
                    pauli: [p.0, p.1],
                });
            }
            for p in state.e1.iter() {
                operators.push(LocalOperator {
                    index: [row + col, row + col + 1],
                    pauli: [p.0, p.1],
                });
            }
            for p in state.e2.iter() {
                operators.push(LocalOperator {
                    index: [row + col + 1, row + col + 2],
                    pauli: [p.0, p.1],
                });
            }
            for p in state.e3.iter() {
                operators.push(LocalOperator {
                    index: [row + col + 2, row + col + 3],
                    pauli: [p.0, p.1],
                });
            }
            for p in state.e4.iter() {
                operators.push(LocalOperator {
                    index: [row + col + 3, row + (col + 4) % 8],
                    pauli: [p.0, p.1],
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
        e1: vec![],
        e2: vec![],
        e3: vec![],
        e4: vec![],
        e5: vec![],
        orig_valid: false,
        coll_valid: false,
        collapsed: 0.0,
    };

    let mut sage_process = SageProcess::default();

    // let bin_state = RefCell::new([true; N]);
    let mut min_index = N;

    let mut f_true = |state: &mut State, index: usize| -> bool {
        // bin_state.borrow_mut()[index] = true;
        let pindex = index / 5;
        match index % 5 {
            0 => {
                state.e1.push(DOUBLES[pindex]);
            }
            1 => {
                state.e2.push(DOUBLES[pindex]);
            }
            2 => {
                state.e3.push(DOUBLES[pindex]);
            }
            3 => {
                state.e4.push(DOUBLES[pindex]);
            }
            4 => {
                state.e5.push(DOUBLES[pindex]);
            }
            _ => unreachable!(),
        }
        check(state, &mut sage_process)
    };
    let mut f_false = |_: &mut State, index: usize| -> bool {
        if index < min_index {
            min_index = index;
            println!("{index}");
        }
        // the state stays the same and since we come from a valid state, it is valid
        // bin_state.borrow_mut()[index] = false;
        true
    };
    let f_result = |state: State| -> Result {
        // println!("{:?}", bin_state.borrow());
        Result {
            orig_valid: state.orig_valid,
            coll_valid: state.coll_valid,
            collapsed: state.collapsed,
            is_2d: !state.e1.is_empty()
                && !state.e2.is_empty()
                && !state.e3.is_empty()
                && !state.e4.is_empty()
                && !state.e5.is_empty(),
            is_empty: state.e1.is_empty()
                && state.e2.is_empty()
                && state.e3.is_empty()
                && state.e4.is_empty()
                && state.e5.is_empty(),
            num_ops: state.e1.len()
                + state.e2.len()
                + state.e3.len()
                + state.e4.len()
                + state.e5.len(),
        }
    };

    let mut tree = TreeStack::<State, Result, N>::new(init_state);
    tree.search(&mut f_true, &mut f_false, &f_result);
    let results = tree.into_results();

    fs::write(
        "output/exact_bricks.msgpack",
        rmp_serde::to_vec(&results).unwrap(),
    )
    .unwrap();
}

pub fn run_analyse() {
    let results: Vec<Result> =
        rmp_serde::from_slice(&fs::read("output/exact_bricks.msgpack").unwrap()).unwrap();

    #[derive(Debug, Serialize, Deserialize)]
    struct Coefficients {
        scf: Vec<f64>,
        dscf: Vec<f64>, // Delta SCF
        collapsed: Vec<f64>,
    }
    let mut coefficients = Coefficients {
        scf: [0.0; N].to_vec(),
        dscf: [0.0; N].to_vec(),
        collapsed: [0.0; N].to_vec(),
    };

    for instance in results.into_iter() {
        if instance.is_empty || !instance.is_2d {
            continue;
        }
        let index = instance.num_ops;
        assert!(
            index > 4,
            "graph cannot be two-dimensional with less than 5 operator"
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
        "output/exact_bricks_coefficients.json",
        serde_json::to_string_pretty(&coefficients).unwrap(),
    )
    .unwrap();
}
