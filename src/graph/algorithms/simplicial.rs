#[cfg(test)]
pub mod tests {
    use hashbrown::{HashMap, HashSet};
    use modular_decomposition::ModuleKind;
    use rand::{Rng, SeedableRng};
    use rand_pcg::Pcg64;

    use crate::graph::{
        HLabels, Label, VLabels, VNodes,
        algorithms::test_impls::RequiredMethods,
        generic::{self, Adj, ImplGraph, Pet, algorithms::is_line_graph::SageProcess},
        specialised::{self, Custom, IndexMap},
        test_utils::{self, RandomMap, collect},
    };

    fn sort_cliques(cliques: Vec<Vec<VLabels>>) -> HashSet<VLabels> {
        cliques
            .into_iter()
            .flatten()
            .map(|mut clique| {
                clique.sort();
                clique
            })
            .collect()
    }

    fn get_cliques<G: RequiredMethods>(
        data: HashMap<Label, HLabels>,
    ) -> HashSet<VLabels> {
        let graph = G::from_adj_list(data);
        // println!("{:?}", graph);
        let tree = graph.modular_decomposition();
        sort_cliques(graph.simplicial(&tree))
    }

    // current implementation of the consistency check does not account for when we get
    // different bipartitions!
    fn check<G: RequiredMethods>(
        data: HashMap<Label, HLabels>,
        expected: Option<bool>, // cf. claw_free->check comment; None if not claw-free
        show_info: bool,
    ) {
        let mut graph = G::from_adj_list(data.clone());
        let mut tree = graph.modular_decomposition();

        let mut sage_process = SageProcess::default();

        // the specialised version requires that if something can be collapsed, it has
        // been collapsed
        graph.twin_collapse(&mut tree, &mut sage_process);
        // and it requires it to be claw-free, which we also assume for the generic
        // version in the RequiredMethods implementation (unwrapping there)
        if !graph.is_claw_free(&tree).into() {
            assert_eq!(expected, None);
            // while it is not safe to run the simplicial algorithm; one could still do it
            // and it still might return something that is likely to be a simplicial
            // cliques, e.g., for the first_prime_example_in_paper test it returns the
            // [0, 5] simplicial clique (although the graph is not claw-free)
            return;
        }
        // and it must be connected
        if let ModuleKind::Parallel = tree.graph.node_weight(tree.root).unwrap() {
            panic!("not connected");
        }

        let cliques = graph.simplicial(&tree);
        if show_info {
            // // println!("{:?}", graph.map_to_labels());
            // println!("{:?}", graph);
            println!("{tree:?}");
            // println!("cliques: {cliques:?}");
        }

        let result = cliques.into_iter().flatten().any(|c| !c.is_empty());
        assert_eq!(Some(result), expected);

        if expected.unwrap() && G::once() {
            // already use the collapsed graph, so that we can be sure to have the same
            // labels
            let data = graph.map_to_labels();
            let cliques = sort_cliques(graph.simplicial(&tree));
            let other = [
                get_cliques::<specialised::Graph<Custom>>(data.clone()),
                get_cliques::<generic::Graph<Pet>>(data.clone()),
                get_cliques::<generic::Graph<Adj>>(data),
            ];
            assert!(other.into_iter().all(|c| c == cliques));
        }
    }

    pub fn claw_with_twins<G: RequiredMethods>() {
        // cf. claw_free->twins
        //    - 1 -
        //  /       \
        // 0 -- 2 -- 4
        //  \
        //    - 3
        let data = collect!(hh, RandomMap::new(42, 42);
                (0, [1, 2, 3]),
                (1, [0, 4]),
                (2, [0, 4]),
                (3, [0]),
                (4, [1, 2]),
        );
        // it's a 4-path: edge nodes, and 2 cliques are simplicial cliques
        check::<G>(data, Some(true), false);
    }

    pub fn path_with_clique_end<G: RequiredMethods>() {
        // 0 -- 1 -- 2
        //  \
        //    - 3,4,5,6 clique
        let data = collect!(hh, RandomMap::new(42, 42);
                (0, [1, 3, 4, 5, 6]),
                (1, [0, 2]),
                (2, [1]),
                (3, [0, 4, 5, 6]),
                (4, [0, 3, 5, 6]),
                (5, [0, 3, 4, 6]),
                (6, [0, 3, 4, 5]),
        );
        check::<G>(data, Some(true), false);
    }

    // paper example; this is also one of the (two) minimal claw-free, non-simplicial
    // graphs, but this one has a twin
    pub fn create_simplicial_clique_via_sibling_collapse<G: RequiredMethods>() {
        let data = collect!(hh, RandomMap::new(42, 42);
            (5, [0, 1, 2, 3, 4]),
            (6, [0, 1, 2, 3, 4]),
            (0, [5, 6, 3, 2]),
            (3, [5, 6, 0, 1]),
            (1, [5, 6, 3, 4]),
            (4, [5, 6, 1, 2]),
            (2, [5, 6, 4, 0]),
        );
        check::<G>(data, Some(true), false);
    }

    pub fn path2<G: RequiredMethods>() {
        // 0 -- 1
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [1]),
            (1, [0]),
        );
        // just a single node after collapse
        check::<G>(data, Some(true), false);
    }

    pub fn path3<G: RequiredMethods>() {
        // 0 -- 1 -- 2
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [1]),
            (1, [0, 2]),
            (2, [1]),
        );
        // just a single node after collapse
        check::<G>(data, Some(true), false);
    }

    pub fn triangle<G: RequiredMethods>() {
        // 0 -- 1
        //  \ /
        //    2
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [1, 2]),
            (1, [0, 2]),
            (2, [0, 1]),
        );
        // just a single node after collapse
        check::<G>(data, Some(true), false);
    }

    pub fn square<G: RequiredMethods>() {
        // 0 -- 1
        // |    |
        // 2 -- 3
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [1, 2]),
            (1, [0, 3]),
            (2, [0, 3]),
            (3, [1, 2]),
        );
        // just a single node after collapse
        check::<G>(data, Some(true), false);
    }

    pub fn single_node<G: RequiredMethods>() {
        let data = collect!(hh, RandomMap::new(42, 42); (0, []),);
        check::<G>(data, Some(true), false);
    }

    pub fn circle5<G: RequiredMethods>() {
        // 0 -- 1 -- 2 -- 3 -- 4
        //  \_________________/
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [1, 4]),
            (1, [0, 2]),
            (2, [1, 3]),
            (3, [2, 4]),
            (4, [0, 3]),
        );
        // all the cliques are the edges and they are all simplicial
        check::<G>(data, Some(true), false);
    }

    pub fn first_prime_example_in_paper<G: RequiredMethods>() {
        //         - 4 -
        //       /       \
        // 0 -- 1 -- 2 -- 3
        //  \       /
        //    - 5 -
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [1, 5]),
            (1, [0, 2, 4]),
            (2, [1, 3, 5]),
            (3, [2, 4]),
            (4, [1, 3]),
            (5, [0, 2]),
        );
        let tree = G::from_adj_list(data.clone()).modular_decomposition();
        assert!(tree.module_is_fully_prime(tree.root));
        // e.g., claw 1-0,2,4
        check::<G>(data, None, true);
    }

    pub fn antihole7<G: RequiredMethods>() {
        // see example_graphs/antihole7.png
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [6, 1, 5, 2]),
            (1, [0, 2, 6, 3]),
            (2, [1, 3, 0, 4]),
            (3, [2, 4, 1, 5]),
            (4, [3, 5, 2, 6]),
            (5, [4, 6, 3, 0]),
            (6, [5, 0, 4, 1]),
        );
        // minimal example of sibling-free, claw-free, non-simplicial
        check::<G>(data, Some(false), false);
    }

    pub fn five_non_simplicial<G: RequiredMethods>() {
        // see example_graphs/five_non_simplicial.png
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [3, 4]),
            (1, [3, 4]),
            (2, [3, 4]),
            (3, [0, 1, 2]),
            (4, [0, 1, 2]),
        );
        // this is just a square with one additional node connected to two non-adjacent
        // nodes -> full collapse
        check::<G>(data, Some(true), false);
    }

    pub fn six0<G: RequiredMethods>() {
        // see example_graphs/six0.png
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [3, 4, 5]),
            (1, [3, 4, 5]),
            (2, [4, 5]),
            (3, [0, 1, 5]),
            (4, [0, 1, 2]),
            (5, [0, 1, 2, 3]),
        );
        // 5-0,1,2 is a claw, but 0,1 are twins; 3, for example, is after the collapse a
        // simp clique
        check::<G>(data, Some(true), false);
    }

    pub fn six1<G: RequiredMethods>() {
        // see example_graphs/six1.png
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [3, 4, 5]),
            (1, [3, 4, 5]),
            (2, [3, 4, 5]),
            (3, [0, 1, 2, 5]),
            (4, [0, 1, 2]),
            (5, [0, 1, 2, 3]),
        );
        // 0,1,2 collapse to 0; 3,5 collapse to 3; now 3-0-4 are are path that collapses
        check::<G>(data, Some(true), false);
    }

    pub fn eight0<G: RequiredMethods>() {
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [2, 4, 5, 6]),
            (1, [3, 4, 6, 7]),
            (2, [0, 4, 5, 7]),
            (3, [1, 5, 6, 7]),
            (4, [0, 1, 2, 6]),
            (5, [0, 2, 3, 7]),
            (6, [0, 1, 3, 4]),
            (7, [1, 2, 3, 5]),
        );
        // this example was generated as claw-free, non-simplicial; and I cannot see any
        // siblings
        check::<G>(data, Some(false), false);
    }

    pub fn eight1<G: RequiredMethods>() {
        let data = collect!(hh, RandomMap::new(42, 42);
            (0, [2, 4, 5, 6]),
            (1, [3, 4, 6, 7]),
            (2, [0, 4, 5, 7]),
            (3, [1, 5, 6, 7]),
            (4, [0, 1, 2, 6, 7]),
            (5, [0, 2, 3, 6, 7]),
            (6, [0, 1, 3, 4, 5]),
            (7, [1, 2, 3, 4, 5]),
        );
        // this example was generated as claw-free, non-simplicial; and I cannot see any
        // siblings
        check::<G>(data, Some(false), false);
    }

    pub fn eight2<G: RequiredMethods>() {
        // let data = collect!(hh, RandomMap::new(24, 42);
        let data = collect!(hh, RandomMap::Identity;
            (0, [2, 3, 5, 6, 7]),
            (1, [3, 4, 5, 6, 7]),
            (2, [0, 4, 5, 6, 7]),
            (3, [0, 1, 5, 6, 7]),
            (4, [1, 2, 5, 6, 7]),
            (5, [0, 1, 2, 3, 4, 7]),
            (6, [0, 1, 2, 3, 4, 7]),
            (7, [0, 1, 2, 3, 4, 5, 6]),
        );
        // 5,6,7 are a module and they are a path that collapses; this results in a 5-hole
        // with one vertex in the middle connected to al the other vertices; every edge of
        // the hole is a simplicial clique
        check::<G>(data, Some(true), false);
    }

    macro_rules! test_it {
        ($module:ident, $typ:ty) => {
            mod $module {
                use super::*;
                crate::graph::algorithms::simplicial::tests::wrap!(
                    $typ,
                    claw_with_twins,
                    path_with_clique_end,
                    create_simplicial_clique_via_sibling_collapse,
                    path2,
                    path3,
                    triangle,
                    square,
                    single_node,
                    circle5,
                    first_prime_example_in_paper,
                    antihole7,
                    five_non_simplicial,
                    six0,
                    six1,
                    eight0,
                    eight1,
                    eight2,
                );
            }
        };
    }
    pub(crate) use test_it;

    macro_rules! wrap {
        ($typ:ty, $($fun:ident,)*) => {
            $(
                #[test]
                fn $fun() {
                    crate::graph::algorithms::simplicial::tests::$fun::<$typ>();
                }
            )*
        };
    }
    pub(crate) use wrap;

    #[test]
    fn consistency() {
        let rng = &mut Pcg64::from_entropy();
        let mut sage_process = SageProcess::default();
        'outer: for _ in 0..20 {
            let mut counter = 0;
            let (graph, tree) = loop {
                // most go through with that
                if counter == 10 {
                    continue 'outer;
                } else {
                    counter += 1;
                }

                let num_nodes = rng.gen_range(1..50);
                let num_edges = rng.gen_range(0..100);
                let data = test_utils::random_data(rng, num_nodes, num_edges);

                let mut graph = generic::Graph::<Pet>::from_adj_list(data);
                let mut tree = graph.modular_decomposition();
                graph.twin_collapse(&mut tree, &mut sage_process);

                if !bool::from(graph.is_claw_free(&tree)) {
                    continue;
                } else if let ModuleKind::Parallel =
                    tree.graph.node_weight(tree.root).unwrap()
                {
                    continue;
                } else {
                    break (graph, tree);
                }
            };

            let data = RequiredMethods::map_to_labels(&graph);
            let cliques = [
                get_cliques::<specialised::Graph<Custom>>(data.clone()),
                get_cliques::<specialised::Graph<IndexMap>>(data.clone()),
                get_cliques::<generic::Graph<Adj>>(data.clone()),
                sort_cliques(RequiredMethods::simplicial(&graph, &tree)),
            ];

            let this = &cliques[3];
            if !cliques.iter().all(|c| *c == *this) {
                // in that case, all of them should be bipartitions of the the complement
                // graph, and we just got different bipartitions; let's check whether that
                // is correct
                for c in cliques.iter() {
                    assert_eq!(c.len(), 2);

                    let get_nodes = |set: &VLabels| {
                        set.iter()
                            .map(|l| graph.find_node(*l).unwrap())
                            .collect::<VNodes>()
                    };
                    let bipartition = c.iter().map(get_nodes).collect::<Vec<_>>();

                    let mut complement = graph.clone();
                    complement.complement();
                    let check_independence =
                        |set: &VNodes| complement.set_is_independent(set.iter().copied());

                    bipartition.iter().for_each(|set| {
                        assert!(
                            check_independence(set)
                                && graph.set_is_clique(set.iter().copied())
                                && graph.clique_is_simplicial(set)
                        )
                    });
                }
            }
        }
    }
}
