#[cfg(test)]
pub mod tests {
    use std::fmt::Debug;

    use hashbrown::{HashMap, HashSet};
    use rand::SeedableRng;
    use rand_pcg::Pcg64;

    use crate::{
        fix_int::int,
        graph::{
            HLabels, Label, Node,
            algorithms::modular_decomposition::Tree,
            test_utils::{RandomMap, collect},
        },
    };

    pub trait RequiredMethods: Debug {
        fn create(adj_list: HashMap<Label, HLabels>) -> Self;
        fn modular_decomposition(&self) -> Tree;
        fn twin_collapse(&mut self, tree: &mut Tree);
        fn get_label_mapping(&self) -> impl Fn(Node) -> Label + Copy;
        fn map_to_labels(&self) -> HashMap<Label, HLabels>;
    }

    fn check<G: RequiredMethods>(
        input: HashMap<int, HashSet<int>>,
        collapsed: impl IntoIterator<Item = HashMap<int, HashSet<int>>>,
        show_info: bool,
    ) {
        let mut graph = G::create(input);
        let expected: Vec<G> = collapsed.into_iter().map(|adj| G::create(adj)).collect();

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
        check::<G>(input, collapsed, true);
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
                    // TODO: more
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
