use std::thread;

use rand::SeedableRng;
use rand_pcg::Pcg64;
use serde::Serialize;

use super::GenGraph;
use crate::{graph::generic::ImplGraph, hamiltonian::Density, run::check};

#[derive(Debug, Serialize)]
pub struct Results {
    densities: Vec<f64>,
    sizes: Vec<usize>,
    seed: u64,
    num_samples: usize,
    // first index is the size, second is the density; averaged over the samples
    before_claw_free: Vec<Vec<f64>>,
    after_claw_free: Vec<Vec<f64>>,
    before_simplicial: Vec<Vec<f64>>,
    after_simplicial: Vec<Vec<f64>>,
    collapsed: Vec<Vec<f64>>,
}

pub struct CountResults {
    before_claw_free: Vec<Vec<usize>>,
    after_claw_free: Vec<Vec<usize>>,
    before_simplicial: Vec<Vec<usize>>,
    after_simplicial: Vec<Vec<usize>>,
    collapsed: Vec<Vec<f64>>,
}

impl CountResults {
    pub fn init(num_density_steps: usize, num_sizes: usize) -> Self {
        Self {
            before_claw_free: vec![vec![0; num_density_steps]; num_sizes],
            after_claw_free: vec![vec![0; num_density_steps]; num_sizes],
            before_simplicial: vec![vec![0; num_density_steps]; num_sizes],
            after_simplicial: vec![vec![0; num_density_steps]; num_sizes],
            collapsed: vec![vec![0.; num_density_steps]; num_sizes],
        }
    }

    pub fn merge(results: Vec<Self>) -> Self {
        let ex_vec = &results.first().unwrap().before_claw_free;
        let num_density_steps = ex_vec[0].len();
        let num_sizes = ex_vec.len();
        let mut ret = Self::init(num_density_steps, num_sizes);
        for result in results {
            for i in 0..num_sizes {
                let ret_before_claw_free = ret.before_claw_free.get_mut(i).unwrap();
                let before_claw_free = result.before_claw_free.get(i).unwrap();
                let ret_after_claw_free = ret.after_claw_free.get_mut(i).unwrap();
                let after_claw_free = result.after_claw_free.get(i).unwrap();
                let ret_before_simp = ret.before_simplicial.get_mut(i).unwrap();
                let before_simp = result.before_simplicial.get(i).unwrap();
                let ret_after_simp = ret.after_simplicial.get_mut(i).unwrap();
                let after_simp = result.after_simplicial.get(i).unwrap();
                let ret_collapsed = ret.collapsed.get_mut(i).unwrap();
                let collapsed = result.collapsed.get(i).unwrap();
                for j in 0..num_density_steps {
                    ret_before_claw_free[j] += before_claw_free[j];
                    ret_after_claw_free[j] += after_claw_free[j];
                    ret_before_simp[j] += before_simp[j];
                    ret_after_simp[j] += after_simp[j];
                    ret_collapsed[j] += collapsed[j];
                }
            }
        }
        ret
    }
}

impl Results {
    // pub fn init(num_density_steps: usize, num_sizes: usize, num_samples: usize) -> Self {
    pub fn init(
        densities: Vec<f64>,
        sizes: Vec<usize>,
        seed: u64,
        num_samples: usize,
    ) -> Self {
        let num_density_steps = densities.len();
        let num_sizes = sizes.len();
        Self {
            densities,
            sizes,
            seed,
            num_samples,
            before_claw_free: vec![vec![0.0; num_density_steps]; num_sizes],
            after_claw_free: vec![vec![0.0; num_density_steps]; num_sizes],
            before_simplicial: vec![vec![0.0; num_density_steps]; num_sizes],
            after_simplicial: vec![vec![0.0; num_density_steps]; num_sizes],
            collapsed: vec![vec![0.0; num_density_steps]; num_sizes],
        }
    }

    pub fn normalise_merged_count_results(
        results: CountResults,
        densities: Vec<f64>,
        sizes: Vec<usize>,
        seed: u64,
        num_samples: usize,
    ) -> Self {
        let ex_vec = &results.before_claw_free;
        let num_density_steps = ex_vec[0].len();
        let num_sizes = ex_vec.len();
        let mut ret = Self::init(densities, sizes, seed, num_samples);

        for i in 0..num_sizes {
            let ret_before_claw_free = ret.before_claw_free.get_mut(i).unwrap();
            let before_claw_free = results.before_claw_free.get(i).unwrap();
            let ret_after_claw_free = ret.after_claw_free.get_mut(i).unwrap();
            let after_claw_free = results.after_claw_free.get(i).unwrap();
            let ret_before_simp = ret.before_simplicial.get_mut(i).unwrap();
            let before_simp = results.before_simplicial.get(i).unwrap();
            let ret_after_simp = ret.after_simplicial.get_mut(i).unwrap();
            let after_simp = results.after_simplicial.get(i).unwrap();
            let ret_collapsed = ret.collapsed.get_mut(i).unwrap();
            let collapsed = results.collapsed.get(i).unwrap();
            for j in 0..num_density_steps {
                ret_before_claw_free[j] = before_claw_free[j] as f64 / num_samples as f64;
                ret_after_claw_free[j] = after_claw_free[j] as f64 / num_samples as f64;
                ret_before_simp[j] = before_simp[j] as f64 / num_samples as f64;
                ret_after_simp[j] = after_simp[j] as f64 / num_samples as f64;
                ret_collapsed[j] = collapsed[j] / num_samples as f64;
            }
        }
        ret
    }
}

pub fn sweep(
    seed: u64,
    seeds: Vec<u64>,
    densities: Vec<f64>,
    sizes: Vec<usize>,
    num_thread_samples: usize,
    get_graph: impl Fn(Density, usize, &mut Pcg64) -> GenGraph + Sync,
) -> Results {
    let num_threads = seeds.len();

    let job = |id: usize| {
        let rng = &mut Pcg64::seed_from_u64(seeds[id]);
        let mut ret = CountResults::init(densities.len(), sizes.len());

        for (size_idx, size) in sizes.iter().enumerate() {
            println!("{:?}", size);
            let ret_before_claw_free = ret.before_claw_free.get_mut(size_idx).unwrap();
            let ret_after_claw_free = ret.after_claw_free.get_mut(size_idx).unwrap();
            let ret_before_simp = ret.before_simplicial.get_mut(size_idx).unwrap();
            let ret_after_simp = ret.after_simplicial.get_mut(size_idx).unwrap();
            let ret_collapsed = ret.collapsed.get_mut(size_idx).unwrap();

            for (density_idx, density) in densities.iter().copied().enumerate() {
                let d = Density::new(density);
                let mut before_claw_free = 0;
                let mut after_claw_free = 0;
                let mut before_simplicial = 0;
                let mut after_simplicial = 0;
                let mut collapsed = 0.;

                let mut i = 0;
                while i < num_thread_samples {
                    // println!("{:?}", i);
                    let mut graph = get_graph(d, *size, rng);

                    if graph.is_empty() {
                        i += 1;
                        before_claw_free += 1;
                        after_claw_free += 1;
                        before_simplicial += 1;
                        after_simplicial += 1;
                        continue;
                    }

                    let orig_len = graph.len();
                    let mut tree = graph.modular_decomposition();

                    let check = check::do_gen_check(&graph, &tree);
                    if check.claw_free {
                        before_claw_free += 1;
                    }
                    if check.simplicial {
                        before_simplicial += 1;
                    }

                    graph.twin_collapse(&mut tree);
                    collapsed += (orig_len - graph.len()) as f64 / orig_len as f64;

                    let check = check::do_gen_check(&graph, &tree);
                    if check.claw_free {
                        after_claw_free += 1;
                    }
                    if check.simplicial {
                        after_simplicial += 1;
                    }

                    i += 1;
                }

                ret_before_claw_free[density_idx] = before_claw_free;
                ret_after_claw_free[density_idx] = after_claw_free;
                ret_before_simp[density_idx] = before_simplicial;
                ret_after_simp[density_idx] = after_simplicial;
                ret_collapsed[density_idx] = collapsed;
            }
        }
        println!("thread {id} finished");
        ret
    };

    let results: Vec<_> = thread::scope(|scope| {
        let handles: Vec<_> =
            (0..num_threads).map(|i| scope.spawn(move || job(i))).collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    Results::normalise_merged_count_results(
        CountResults::merge(results),
        densities,
        sizes,
        seed,
        num_thread_samples * num_threads,
    )
}
