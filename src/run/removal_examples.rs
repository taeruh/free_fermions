use core::fmt;

#[derive(Clone, Copy)]
struct Pauli {
    z: bool,
    x: bool,
}

impl fmt::Display for Pauli {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.z, self.x) {
            (false, false) => write!(f, "I"),
            (false, true) => write!(f, "X"),
            (true, false) => write!(f, "Z"),
            (true, true) => write!(f, "Y"),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct PauliString<const N: usize> {
    z: [bool; N],
    x: [bool; N],
}

impl<const N: usize> Default for PauliString<N> {
    fn default() -> Self {
        PauliString {
            z: [false; N],
            x: [false; N],
        }
    }
}

impl<const N: usize> fmt::Display for PauliString<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for s in 0..N {
            write!(
                f,
                "{}",
                Pauli {
                    z: self.z[s],
                    x: self.x[s]
                }
            )?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
impl<const N: usize> PauliString<N> {
    fn set(&mut self, pauli: Pauli, idx: usize) {
        self.z[idx] = pauli.z;
        self.x[idx] = pauli.x;
    }

    /// false=1, true=-1
    fn commutator(&self, other: &Self) -> bool {
        let mut res = false;
        for s in 0..N {
            res ^= (self.z[s] & other.x[s]) ^ (self.x[s] & other.z[s]);
        }
        res
    }

    fn product(&self, other: &Self) -> Self {
        let mut res = Self::default();
        for s in 0..N {
            res.z[s] = self.z[s] ^ other.z[s];
            res.x[s] = self.x[s] ^ other.x[s];
        }
        res
    }
}

const SINGLES: [Pauli; 4] = [
    Pauli { z: false, x: false },
    Pauli { z: false, x: true },
    Pauli { z: true, x: false },
    Pauli { z: true, x: true },
];

fn four_pauli_iterator() -> impl Iterator<Item = (Pauli, Pauli, Pauli, Pauli)> {
    SINGLES.into_iter().flat_map(move |a| {
        SINGLES.into_iter().flat_map(move |b| {
            SINGLES
                .into_iter()
                .flat_map(move |c| SINGLES.into_iter().map(move |d| (a, b, c, d)))
        })
    })
}

#[allow(dead_code)]
pub fn run() {
    const N: usize = 4;

    let mut a = PauliString::<N>::default();
    let mut b = PauliString::<N>::default();
    let mut c = PauliString::<N>::default();
    let mut d = PauliString::<N>::default();

    fn condition(
        a: &PauliString<N>,
        b: &PauliString<N>,
        c: &PauliString<N>,
        d: &PauliString<N>,
    ) -> bool {
        let different = a != b && a != c && a != d && b != c && b != d && c != d;
        if !different {
            return false;
        }

        let same = a.product(b) == c.product(d);
        if !same {
            return false;
        }

        let frust_graph = a.commutator(b)
            && !a.commutator(c)
            && !a.commutator(d)
            && !b.commutator(c)
            && !b.commutator(d)
            && c.commutator(d);
        if !frust_graph {
            return false;
        }

        true
    }

    'outer: for first in four_pauli_iterator() {
        a.set(first.0, 0);
        b.set(first.1, 0);
        c.set(first.2, 0);
        d.set(first.3, 0);
        for second in four_pauli_iterator() {
            a.set(second.0, 1);
            b.set(second.1, 1);
            c.set(second.2, 1);
            d.set(second.3, 1);
            for third in four_pauli_iterator() {
                a.set(third.0, 2);
                b.set(third.1, 2);
                c.set(third.2, 2);
                d.set(third.3, 2);
                for fourth in four_pauli_iterator() {
                    a.set(fourth.0, 3);
                    b.set(fourth.1, 3);
                    c.set(fourth.2, 3);
                    d.set(fourth.3, 3);

                    if condition(&a, &b, &c, &d) {
                        println!("a={a}\nb={b}\nc={c}\nd={d}");
                        break 'outer;
                    }
                }
            }
        }
    }
}
