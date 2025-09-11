#[allow(unused_imports)]
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use crate::{
    graph::generic::{algorithms::is_line_graph::SageProcess, ImplGraph},
    hamiltonian::{oned_chain::OpenChain, Density},
    run::{check, GenGraph},
};

#[allow(dead_code)]
pub fn run() {
    let rng = &mut Pcg64::from_entropy();
    // let rng = &mut Pcg64::seed_from_u64(25);

    let mut sage_process = SageProcess::default();

    let density = Density::new(0.2);
    let chain = OpenChain::draw(density, rng);
    let mut graph = GenGraph::from_edge_labels(chain.get_graph()).unwrap();
    let mut tree = graph.modular_decomposition();

    let num_nodes = graph.iter_labels().count();
    println!("{:?}", num_nodes);
    println!("{:?}", chain);
    let _check = check::do_gen_check(&graph, &tree);
    // println!("{:?}", check);

    // println!("{:?}", chain);
    // println!("{:?}", chain.get_graph());
    // println!("{:?}", graph);

    graph.twin_collapse(&mut tree, &mut sage_process);

    let num_nodes = graph.iter_labels().count();
    println!("{:?}", num_nodes);
    let check = check::do_gen_check(&graph, &tree);
    println!("{:?}", check);
}
