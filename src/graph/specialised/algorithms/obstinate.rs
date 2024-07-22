use std::borrow::Cow;

use crate::graph::{
    algorithms::obstinate::{Obstinate, ObstinateKind},
    specialised::{Graph, GraphData},
    Node,
};

impl<G: GraphData> Graph<G> {
    pub fn obstinate(&self) -> Obstinate {
        let len = self.len();
        if len % 2 != 0 {
            Obstinate::False
        } else if len == 0 {
            Obstinate::True(ObstinateKind::Itself, (vec![], vec![]))
        } else if len == 2 {
            Obstinate::True(ObstinateKind::Complement, (vec![0], vec![1]))
        } else {
            unsafe { self.obstinate_non_trivial() }
        }
    }

    /// # Safety
    /// The size of the graph must be odd and more than 2.
    pub unsafe fn obstinate_non_trivial(&self) -> Obstinate {
        let len = self.len();
        let len2 = len / 2;
        debug_assert_eq!(len % 2, 0);

        let mut degrees = self
            .enumerate_neighbours()
            .map(|(node, neighbours)| {
                let degree = neighbours.len();
                (node, degree)
            })
            .collect::<Vec<_>>();
        degrees.sort_unstable_by_key(|(_, degree)| *degree);
        debug_assert_eq!(degrees.len(), len);

        /// # Safety
        /// `degrees` is a valid pointer through the offsets from 0 to
        /// 2*(end_exclusive-start)-1.
        #[inline]
        unsafe fn check_degree_sequence(
            start: usize,
            end_exclusive: usize,
            degrees: *const (Node, usize),
        ) -> bool {
            for (i, deg) in (start..end_exclusive).enumerate() {
                // safety: this is the invariant of the function
                if unsafe {
                    (deg != (*degrees.add(2 * i)).1)
                        || (deg != (*degrees.add(2 * i + 1)).1)
                } {
                    return false;
                }
            }
            true
        }

        let mut graph = Cow::Borrowed(self);

        let deg_ptr = degrees.as_ptr();
        // safety: 2*(len2+1-1)+1 = len-1 = 2*(len-1-(len2-1))-1
        let kind = if unsafe { check_degree_sequence(1, len2 + 1, deg_ptr) } {
            ObstinateKind::Itself
        } else if unsafe { check_degree_sequence(len2 - 1, len - 1, deg_ptr) } {
            graph.to_mut().complement();
            ObstinateKind::Complement
        } else {
            return Obstinate::False;
        };

        // safety: we are clearly in bounds
        let (a_end, b_start) = match kind {
            ObstinateKind::Itself => unsafe {
                (degrees.get_unchecked(len - 2).0, degrees.get_unchecked(len - 1).0)
            },
            ObstinateKind::Complement => unsafe {
                (degrees.get_unchecked(0).0, degrees.get_unchecked(1).0)
            },
        };

        let a_part = unsafe { graph.get_neighbours_unchecked(b_start) };
        let b_part = unsafe { graph.get_neighbours_unchecked(a_end) };

        if (a_part.intersection(b_part).count() != 0)
            || !graph.set_is_independent(a_part.iter().copied())
            || !graph.set_is_independent(b_part.iter().copied())
        {
            return Obstinate::False;
        }

        let mut a_degrees = a_part
            .iter()
            .map(|&node| (node, unsafe { graph.get_neighbours_unchecked(node).len() }))
            .collect::<Vec<_>>();
        a_degrees.sort_unstable_by_key(|(_, degree)| *degree);
        let mut b_degrees = b_part
            .iter()
            .map(|&node| (node, unsafe { graph.get_neighbours_unchecked(node).len() }))
            .collect::<Vec<_>>();
        b_degrees.sort_unstable_by(|(_, degree1), (_, degree2)| degree2.cmp(degree1));
        debug_assert_eq!(a_degrees.len(), len2);
        debug_assert_eq!(b_degrees.len(), len2);

        for (i, deg) in a_degrees.iter().enumerate() {
            if deg.1 != i + 1 {
                return Obstinate::False;
            }
        }
        // we do not have to check b_degrees, because we know at this point that a_part
        // and b_part are independent, that together they have the degrees
        // 1,1,2,2,...,len2,len2 and that a_degrees has the degrees 1,2,...,len2; so
        // b_degrees must also have the degrees len2,...,2,1

        for (i, (a, _)) in a_degrees.iter().enumerate() {
            // safety: a comes from a node, so it is valid
            let a_neighbours = unsafe { graph.get_neighbours_unchecked(*a) };
            for j in 0..i {
                // safety b_degrees has the same length as a_degrees (len2)
                if !a_neighbours.contains(&unsafe { b_degrees.get_unchecked(j) }.0) {
                    return Obstinate::False;
                }
            }
            for j in (i + 1)..len2 {
                // safety b_degrees has the same length as a_degrees (len2)
                if a_neighbours.contains(&unsafe { b_degrees.get_unchecked(j) }.0) {
                    return Obstinate::False;
                }
            }
        }

        Obstinate::True(
            kind,
            (
                a_degrees.into_iter().map(|(node, _)| node).collect(),
                b_degrees.into_iter().map(|(node, _)| node).collect(),
            ),
        )
    }
}
