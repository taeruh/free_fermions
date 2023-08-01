use free_fermions::{
    graph::Graph,
    hamiltonian::OperatorPool,
};
use rand::SeedableRng;
use rand_pcg::Pcg64;

fn main() {
    let n = 4;
    let p = 0.07;
    let seed = 7;

    let mut pool = OperatorPool::new_with(n, Pcg64::seed_from_u64(seed));
    let sample = pool
        .draw((pool.ops.len() as f64 * p).round() as usize)
        .collect::<Vec<_>>();
    let graph = Graph::from(sample.clone());

    println!("Hamiltonian:");
    for (i, op) in sample.iter().enumerate() {
        println!("{i}: {:?}; {:?}", op.index, op.data);
    }
    println!();
    println!("{:?}\n", graph);
}
