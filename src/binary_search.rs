use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
enum Tried {
    None,
    True,
    Both,
}

#[derive(Clone, Copy, Debug)]
enum Finished {
    True,
    False,
}

// go down the binary tree, along a path, updating the state and checking whether it is
// a valid state, i.e., a state that is not forbidden by the search problem
#[derive(Clone, Debug)]
struct TreePath<S> {
    index: usize,
    state: S,
}

impl<S> TreePath<S> {
    fn new(init_state: S) -> Self {
        Self {
            state: init_state,
            index: 0,
        }
    }

    // changes the state to the new state, according to the true branch, and returns
    // whether it is a valid state
    fn try_true<F: FnMut(&mut S, usize) -> bool>(&mut self, f: &mut F) -> bool {
        if f(&mut self.state, self.index) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    // changes the state to the new state, according to the false branch, and returns
    // whether it is a valid state
    fn try_false<F: FnMut(&mut S, usize) -> bool>(&mut self, f: &mut F) -> bool {
        if f(&mut self.state, self.index) {
            self.index += 1;
            true
        } else {
            false
        }
    }
}

// A (finite) tree path, and capture whether what the next step/try is
struct TreePathWithTry<S> {
    tree: TreePath<S>,
    tries: Tried,
}

impl<S: Clone> TreePathWithTry<S> {
    fn new(init_state: S) -> Self {
        Self {
            tree: TreePath::new(init_state),
            tries: Tried::None,
        }
    }

    fn clone_reset(&self) -> Self {
        Self {
            tree: self.tree.clone(),
            tries: Tried::None,
        }
    }
}

// go along all the possible paths, saving intermediate paths in a stack, and early
// stopping a path if an intermediate state is not valid
pub struct TreeStack<S, R, const N: usize> {
    stack: Vec<TreePathWithTry<S>>,
    top: usize,
    results: Vec<R>,
}

impl<S: Clone + Debug, R, const N: usize> TreeStack<S, R, N> {
    pub fn new(init_state: S) -> Self {
        let mut stack = Vec::with_capacity(N);
        stack.push(TreePathWithTry::new(init_state));
        Self {
            stack,
            top: 0,
            results: Vec::new(),
        }
    }

    fn step<Ft: FnMut(&mut S, usize) -> bool, Ff: FnMut(&mut S, usize) -> bool, Fr: Fn(S) -> R>(
        &mut self,
        f_true: &mut Ft,
        f_false: &mut Ff,
        f_result: Fr,
    ) -> bool {
        let current = &mut self.stack[self.top];
        if self.top + 1 == N {
            // leaf case
            // println!("{:?}", current.tree.state);
            // println!("at top");
            let mut leaf = current.clone_reset();
            if leaf.tree.try_true(f_true) {
                self.results.push(f_result(leaf.tree.state));
            }
            let mut leaf = current.clone_reset();
            if leaf.tree.try_false(f_false) {
                self.results.push(f_result(leaf.tree.state));
            }
            false
        } else {
            let mut next = current.clone_reset();
            next.tries = Tried::None;
            match current.tries {
                // everything down that path has been tried, so we can step back
                Tried::Both => false,
                Tried::None => {
                    current.tries = Tried::True;
                    if next.tree.try_true(f_true) {
                        self.top += 1;
                        self.stack.push(next);
                    }
                    // haven't tried the false branch yet, so we never step back
                    true
                }
                Tried::True => {
                    current.tries = Tried::Both;
                    if next.tree.try_false(f_false) {
                        self.top += 1;
                        self.stack.push(next);
                        true
                    } else {
                        // tried both paths, and the current (false) path is not valid, so
                        // step back
                        false
                    }
                }
            }
        }
    }

    fn step_back(&mut self) -> Finished {
        if self.top == 0 {
            // this makes sense even if we directly step back at the start of the search
            // as this means that nothing is valid
            Finished::True
        } else {
            self.top -= 1;
            self.stack.pop();
            Finished::False
        }
    }

    pub fn search<
        Ft: FnMut(&mut S, usize) -> bool,
        Ff: FnMut(&mut S, usize) -> bool,
        Fr: Fn(S) -> R,
    >(
        &mut self,
        f_true: &mut Ft,
        f_false: &mut Ff,
        f_result: &Fr,
    ) {
        let mut has_fininshed = Finished::False;
        while let Finished::False = has_fininshed {
            if self.step(f_true, f_false, f_result) {
                continue;
            }
            has_fininshed = self.step_back();
        }
    }

    pub fn into_results(self) -> Vec<R> {
        self.results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn never_three_next_to_each_other() {
        const N: usize = 4;
        let expect = [
            [1, 1, 0, 1],
            [1, 1, 0, 0],
            [1, 0, 1, 1],
            [1, 0, 1, 0],
            [1, 0, 0, 1],
            [1, 0, 0, 0],
            [0, 1, 1, 0],
            [0, 1, 0, 1],
            [0, 1, 0, 0],
            [0, 0, 1, 1],
            [0, 0, 1, 0],
            [0, 0, 0, 1],
            [0, 0, 0, 0],
        ];

        fn check(state: &[u8; N]) -> bool {
            for i in 0..N - 2 {
                if state[i] == 1 && state[i + 1] == 1 && state[i + 2] == 1 {
                    return false;
                }
            }
            true
        }
        fn f_true(state: &mut [u8; N], index: usize) -> bool {
            state[index] = 1;
            check(state)
        }
        fn f_false(state: &mut [u8; N], index: usize) -> bool {
            state[index] = 0;
            check(state)
        }

        let init_state = [0; N];
        let mut tree = TreeStack::<_, [u8; N], N>::new(init_state);
        tree.search(&mut f_true, &mut f_false, &|s| s);

        assert_eq!(tree.into_results().as_slice(), &expect);
    }
}
