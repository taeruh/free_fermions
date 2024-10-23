#[cfg(test)]
pub mod tests {

    use hashbrown::HashMap;
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    use crate::graph::{
        HLabels, Label,
        algorithms::{modular_decomposition::Tree, test_impls::RequiredMethods},
        test_utils::{RandomMap, collect},
    };

    fn check<G: RequiredMethods>(
        input: HashMap<Label, HLabels>,
        collapsed: impl IntoIterator<Item = HashMap<Label, HLabels>>,
        show_info: bool,
    ) {
        let mut graph = G::from_adj_list(input);
        let expected: Vec<G> =
            collapsed.into_iter().map(|adj| G::from_adj_list(adj)).collect();

        let mut tree = graph.modular_decomposition();
        if show_info {
            println!("BEFORE graph: {graph:?}");
            println!("BEFORE tree: {tree:?}\n");
        }
        graph.twin_collapse(&mut tree);
        if show_info {
            println!("AFTER graph: {graph:?}");
            println!("AFTER tree: {tree:?}\n");
        }

        let sanity_tree = graph.modular_decomposition();
        assert!(Tree::is_equivalent(
            &tree,
            &sanity_tree,
            graph.get_label_mapping(),
            graph.get_label_mapping()
        ));

        let mapped_graph = graph.map_to_labels();
        let equivalent_graph = expected
            .iter()
            .find(|graph| graph.map_to_labels() == mapped_graph)
            .expect("expected equivalent graph not found");
        // the following is redundant, I think
        let equivalent_tree = equivalent_graph.modular_decomposition();
        assert!(Tree::is_equivalent(
            &tree,
            &equivalent_tree,
            graph.get_label_mapping(),
            equivalent_graph.get_label_mapping()
        ));
    }

    pub fn some_test<G: RequiredMethods>() {
        let rng = &mut Pcg64::from_entropy();
        let map = RandomMap::with_rng(24, 42, rng);
        let input = collect!(
            hh, map;
            (0, [1]),
            (1, [0, 2]),
            (2, [1, 3, 4, 5]),
            (3, [2, 4]),
            (4, [2, 3]),
            (5, [2]),
        );
        let collapsed = [3, 4, 5].into_iter().map(|representative| {
            collect!(
                hh, map;
                (0, [1]),
                (1, [0, 2]),
                (2, [1, representative]),
                (representative, [2]),
            )
        });
        check::<G>(input, collapsed, false);
    }

    pub fn path4<G: RequiredMethods>() {
        let map = RandomMap::new(4, 10);
        let input = collect!(
            hh, map;
            (0, [1]),
            (1, [0, 2]),
            (2, [1, 3]),
            (3, [2]),
        );
        let collapsed = vec![input.clone()];
        check::<G>(input, collapsed, false);
    }

    pub fn complete<G: RequiredMethods>() {
        let map = RandomMap::new(5, 10);
        let input = collect!(
            hh, map;
            (0, [1, 2, 3, 4]),
            (1, [0, 2, 3, 4]),
            (2, [0, 1, 3, 4]),
            (3, [0, 1, 2, 4]),
            (4, [0, 1, 2, 3]),
        );
        let collapsed = [0, 1, 2, 3, 4].into_iter().map(|representative| {
            collect!(
                hh, map;
                (representative, []),
            )
        });
        check::<G>(input, collapsed, false);
    }

    pub fn cotree<G: RequiredMethods>() {
        let map = RandomMap::new(8, 16);
        let input = collect!(
            hh, map;
            (0, [2, 3]),
            (1, [2, 3]),
            (2, [0, 1]),
            (3, [0, 1]),
            (4, [6, 7]),
            (5, [6, 7]),
            (6, [4, 5]),
            (7, [4, 5]),
        );
        let collapsed = [0, 1, 2, 3, 4, 5, 6, 7].into_iter().map(|representative| {
            collect!(
                hh, map;
                (representative, []),
            )
        });
        check::<G>(input, collapsed, false);
    }

    pub fn independent<G: RequiredMethods>() {
        let map = RandomMap::new(3, 10);
        let input = collect!(
            hh, map;
            (0, []),
            (1, []),
            (2, []),
        );
        let collapsed = [0, 1, 2].into_iter().map(|representative| {
            collect!(
                hh, map;
                (representative, []),
            )
        });
        check::<G>(input, collapsed, false);
    }

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
        let collapsed = [5, 6].into_iter().map(|representative| {
            collect!(hh;
                (representative, [0, 1, 2, 3, 4]),
                (0, [representative, 3, 2]),
                (3, [representative, 0, 1]),
                (1, [representative, 3, 4]),
                (4, [representative, 1, 2]),
                (2, [representative, 4, 0]),
            )
        });
        check::<G>(data, collapsed, true);
    }

    macro_rules! test_it {
        ($module:ident, $typ:ty) => {
            mod $module {
                use super::*;
                crate::graph::algorithms::twin_collapse::tests::wrap!(
                    $typ,
                    some_test,
                    path4,
                    complete,
                    independent,
                    cotree,
                    create_simplicial_clique_via_sibling_collapse,
                    // TODO: more DEFINITELY NEEDED!!!!!!!!!!!!!
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
                    crate::graph::algorithms::twin_collapse::tests::$fun::<$typ>();
                }
            )*
        };
    }
    pub(crate) use wrap;
}
