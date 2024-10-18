#[cfg(test)]
pub mod tests {
    use hashbrown::HashMap;
    use modular_decomposition::ModuleKind;

    use crate::graph::{
        HLabels, Label, algorithms::test_impl::RequiredMethods, test_utils::collect,
    };

    fn check<G: RequiredMethods>(
        data: HashMap<Label, HLabels>,
        expected: Option<bool>, // cf. claw_free->check comment; None if not claw-free
        show_info: bool,
    ) {
        let mut graph = G::from_adj_list(data);
        let mut tree = graph.modular_decomposition();

        // the specialised version requires that if something can be collapsed, it has
        // been collapsed
        graph.twin_collapse(&mut tree);
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
            println!("{tree:?}");
            println!("cliques: {cliques:?}");
        }

        let result = cliques.into_iter().flatten().any(|c| !c.is_empty());
        assert_eq!(Some(result), expected);
    }

    pub fn claw_with_twins<G: RequiredMethods>() {
        // cf. claw_free->twins
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
        // it's a 4-path: edge nodes, and 2 cliques are simplicial cliques
        check::<G>(data, Some(true), false);
    }

    pub fn path_with_clique_end<G: RequiredMethods>() {
        // 0 -- 1 -- 2
        //  \
        //    - 3,4,5,6 clique
        let data = collect!(hh;
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

    // paper example; this is also one of the minimal claw-free, non-simplicial graphs,
    // but it has a twin
    pub fn create_simplicial_clique_via_sibling_collapse<G: RequiredMethods>() {
        let data = collect!(hh;
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
        let data = collect!(hh;
            (0, [1]),
            (1, [0]),
        );
        // just a single node after collapse
        check::<G>(data, Some(true), false);
    }

    pub fn path3<G: RequiredMethods>() {
        // 0 -- 1 -- 2
        let data = collect!(hh;
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
        let data = collect!(hh;
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
        let data = collect!(hh;
            (0, [1, 2]),
            (1, [0, 3]),
            (2, [0, 3]),
            (3, [1, 2]),
        );
        // just a single node after collapse
        check::<G>(data, Some(true), false);
    }

    pub fn single_node<G: RequiredMethods>() {
        let data = collect!(hh; (0, []),);
        check::<G>(data, Some(true), false);
    }

    pub fn circle5<G: RequiredMethods>() {
        // 0 -- 1 -- 2 -- 3 -- 4
        //  \_________________/
        let data = collect!(hh;
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
        let data = collect!(hh;
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
        let data = collect!(hh;
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
        let data = collect!(hh;
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
}
