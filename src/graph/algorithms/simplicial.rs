#[cfg(test)]
pub mod tests {
    use crate::graph::{
        HLabels, Label,
        algorithms::{modular_decomposition::Tree, test_impl::RequiredMethods},
        test_utils::collect,
    };

    fn check<G: RequiredMethods>(
        data: HashMap<Label, HLabels>,
        expected: Option<bool>, // cf. claw_free->check comment
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

    macro_rules! test_it {
        ($module:ident, $typ:ty) => {
            mod $module {
                use super::*;
                crate::graph::algorithms::simplicial::tests::wrap!(
                    $typ,
                    claw_with_twins,
                    path_with_clique_end,
                );
            }
        };
    }
    use hashbrown::HashMap;
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
