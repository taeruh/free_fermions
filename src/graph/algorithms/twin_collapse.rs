#[cfg(test)]
pub mod tests {

    use hashbrown::HashMap;

    use crate::graph::{
        algorithms::{modular_decomposition::Tree, test_impls::RequiredMethods}, generic::algorithms::is_line_graph::SageProcess, test_utils::{collect, RandomMap}, HLabels, Label
    };

    fn check<G: RequiredMethods>(
        input: HashMap<Label, HLabels>,
        collapsed: impl IntoIterator<Item = HashMap<Label, HLabels>>,
        show_info: bool,
    ) {
        let mut graph = G::from_adj_list(input);
        let expected: Vec<G> =
            collapsed.into_iter().map(|adj| G::from_adj_list(adj)).collect();

        let mut sage_process = SageProcess::default();

        let mut tree = graph.modular_decomposition();
        if show_info {
            println!("BEFORE graph: {graph:?}");
            println!("BEFORE tree: {tree:?}");
            println!("BEFORE labels: {:?}\n", graph.map_to_labels());
        }
        graph.twin_collapse(&mut tree, &mut sage_process);
        if show_info {
            println!("AFTER graph: {graph:?}");
            println!("AFTER tree: {tree:?}");
            println!("AFTER labels: {:?}", graph.map_to_labels());
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

    pub fn path4<G: RequiredMethods>() {
        let map = RandomMap::new(4, 10);
        let input = collect!(
            hh, map;
            (0, [1]),
            (1, [0, 2]),
            (2, [1, 3]),
            (3, [2]),
        );
        let collapsed = [0, 1, 2, 3].into_iter().map(|representative| {
            collect!(
                hh, map;
                (representative, []),
            )
        });
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
        check::<G>(data, collapsed, false);
    }

    pub fn some_test0<G: RequiredMethods>() {
        let map = RandomMap::new(24, 42);
        let input = collect!(hh, map;
            (0, [1]),
            (1, [0, 2]),
            (2, [1, 3, 4, 5]),
            (3, [2, 4]),
            (4, [2, 3]),
            (5, [2]),
        );
        let collapsed = [0, 1, 2, 3, 4, 5].into_iter().map(|representative| {
            collect!(
                hh, map;
                (representative, []),
            )
        });
        check::<G>(input, collapsed, false);
    }

    pub fn some_test1<G: RequiredMethods>() {
        let map = RandomMap::new(24, 42);
        let input = collect!(hh, map;
            (0, [1]),
            (1, [0, 2, 3, 4]),
            (2, [1, 3, 5]),
            (3, [1, 2, 5]),
            (4, [1, 5]),
            (5, [2, 3, 4]),
        );
        let collapsed = [0, 1, 2, 3, 4, 5].into_iter().map(|representative| {
            collect!(
                hh, map;
                (representative, []),
            )
        });
        check::<G>(input, collapsed, false);
    }

    pub fn some_test2<G: RequiredMethods>() {
        // let map = RandomMap::new(24, 42);
        let map = RandomMap::Identity;
        let input = collect!(hh, map;
            (0, [1, 6, 7, 8, 9]),
            (1, [0, 2, 3, 4, 5]),
            (2, [1, 6, 7, 8, 9, 10]),
            (3, [1, 4, 5]),
            (4, [1, 3]),
            (5, [1, 3]),
            (6, [0, 2]),
            (7, [0, 2, 8, 9]),
            (8, [0, 2, 7, 9]),
            (9, [0, 2, 7, 8]),
            (10, [2]),
        );
        let collapsed = [3, 4, 5]
            .into_iter()
            .flat_map(|co_a| [6, 7, 8, 9].into_iter().map(move |co_b| (co_a, co_b)))
            .map(|(co_a, co_b)| {
                collect!(
                    hh, map;
                    (0, [1, co_b]),
                    (1, [0, co_a, 2]),
                    (2, [1, co_b, 10]),
                    (co_a, [1]),
                    (co_b, [0, 2]),
                    (10, [2]),
                )
            });
        check::<G>(input, collapsed, false);
    }

    macro_rules! test_it {
        ($module:ident, $typ:ty) => {
            mod $module {
                use super::*;
                crate::graph::algorithms::twin_collapse::tests::wrap!(
                    $typ,
                    path4,
                    complete,
                    independent,
                    cotree,
                    create_simplicial_clique_via_sibling_collapse,
                    some_test0,
                    some_test1,
                    some_test2,
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
