#[cfg(test)]
pub mod tests {
    use crate::graph::{
        algorithms::{modular_decomposition::Tree, test_impls::RequiredMethods},
        generic::algorithms::is_line_graph::SageProcess,
        test_utils::collect,
    };

    fn check<G: RequiredMethods>(
        graph: &G,
        tree: &Tree,
        // I thought about passing in the full info, i.e., ClawFree as defined in the
        // generic algorithm, however, it is hard to compare the infos because the tree
        // indices are usually mixed up; instead, just write the expected ClawFree info
        // above the check call, call it with show_info=true once to see whether it fits,
        // up to tree indices, and then set show_info=false for the future
        expected: bool,
        show_info: bool,
    ) {
        let claw_free = graph.is_claw_free(tree);
        if show_info {
            println!("{tree:?}");
            println!("claw_free?: {claw_free:?}");
        }
        assert_eq!(claw_free.into(), expected);
    }

    pub fn claw<G: RequiredMethods>() {
        //    - 1
        //  /
        // 0 -- 2
        //  \
        //    - 3
        let data = collect!(hh;
                (0, [1, 2, 3]),
                (1, [0]),
                (2, [0]),
                (3, [0]),
        );
        let mut graph = G::from_adj_list(data);
        let mut tree = graph.modular_decomposition();
        // No(Structure(SeriesParallelCount(_, 3,)))
        check(&graph, &tree, false, false);
        graph.twin_collapse(&mut tree, &mut SageProcess::default());
        check(&graph, &tree, true, false);
    }

    pub fn long_claw<G: RequiredMethods>() {
        //    - 1 -- 4
        //  /
        // 0 -- 2 -- 5
        //  \
        //    - 3 -- 6
        let data = collect!(hh;
            (0, [1, 2, 3]),
            (1, [0, 4]),
            (2, [0, 5]),
            (3, [0, 6]),
            (4, [1]),
            (5, [2]),
            (6, [3]),
        );
        let mut graph = G::from_adj_list(data);
        let mut tree = graph.modular_decomposition();
        graph.twin_collapse(&mut tree, &mut SageProcess::default()); // nothing should happen
        check(
            &graph, &tree,
            // No(PrimeCase(Claw {
            //     center: 0,
            //     leaves: Triangles {
            //         indices: vec![1, 2, 3],
            //         counts: vec![2, 2, 2],
            //     },
            // })),
            false, false,
        );
    }

    pub fn high_claw_count<G: RequiredMethods>() {
        // 10 -- 7 -     - 1 -- 4
        //           \ /
        // 11 -- 8 -- 0 -- 2 -- 5
        //           / \
        // 13 -- 9 -     - 3 -- 6
        let data = collect!(hh;
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
        );
        let mut graph = G::from_adj_list(data);
        let mut tree = graph.modular_decomposition();
        graph.twin_collapse(&mut tree, &mut SageProcess::default()); // nothing should happen
        check(
            &graph, &tree,
            // No(PrimeCase(Claw {
            //     center: 0,
            //     leaves: Triangles {
            //         indices: vec![1, 2, 3, 7, 8, 9],
            //         counts: vec![20, 20, 20, 20, 20, 20], // #claws = binom(6, 3) = 20
            //     },
            // })),
            false, false,
        );
    }

    pub fn twins<G: RequiredMethods>() {
        // 1 and 2 form a module that will collapse
        //    - 1 -
        //  /       \
        // 0 -- 2 -- 4
        //  \
        //    - 3
        let data = collect!(hh;
                (0, [1, 2, 3]),
                (1, [0, 4]),
                (2, [0, 4]),
                (3, [0]),
                (4, [1, 2]),
        );
        let mut graph = G::from_adj_list(data);
        let mut tree = graph.modular_decomposition();
        // No(Structure(PrimeNonClique(1 and 2 module)))
        check(&graph, &tree, false, false);
        graph.twin_collapse(&mut tree, &mut SageProcess::default()); // nothing should happen
        check(&graph, &tree, true, false);
    }

    macro_rules! test_it {
        ($module:ident, $typ:ty) => {
            mod $module {
                use super::*;
                crate::graph::algorithms::claw_free::tests::wrap!(
                    $typ,
                    claw,
                    long_claw,
                    high_claw_count,
                    twins,
                    // TODO: more tests
                );
            }
        };
    }
    #[allow(unused_imports)]
    use rand::SeedableRng;
    #[allow(unused_imports)]
    use rand_pcg::Pcg64;
    pub(crate) use test_it;

    macro_rules! wrap {
        ($typ:ty, $($fun:ident,)*) => {
            $(
                #[test]
                fn $fun() {
                    crate::graph::algorithms::claw_free::tests::$fun::<$typ>();
                }
            )*
        };
    }
    pub(crate) use wrap;
}
