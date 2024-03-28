// I really think something like enumerate_with_offset should be in the standard
// library; there are of course other ways to accomplish it, but I would guess that this
// would be the most efficient way to do it.
//
// So the below is a partial copy from the enumerate module in the standard library,
// however, it might be less optimized than the standard library version, because we
// are not using nightly stuff.

pub struct Enumerate<I> {
    iter: I,
    count: usize,
}
impl<I> Enumerate<I> {
    pub fn new(iter: I, offset: usize) -> Enumerate<I> {
        Enumerate { iter, count: offset }
    }
}

impl<I> Iterator for Enumerate<I>
where
    I: Iterator,
{
    type Item = (usize, <I as Iterator>::Item);

    #[inline]
    fn next(&mut self) -> Option<(usize, <I as Iterator>::Item)> {
        let a = self.iter.next()?;
        let i = self.count;
        self.count += 1;
        Some((i, a))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<(usize, I::Item)> {
        let a = self.iter.nth(n)?;
        let i = self.count + n;
        self.count = i + 1;
        Some((i, a))
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, fold: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        #[inline]
        fn enumerate<T, Acc>(
            mut count: usize,
            mut fold: impl FnMut(Acc, (usize, T)) -> Acc,
        ) -> impl FnMut(Acc, T) -> Acc {
            move |acc, item| {
                let acc = fold(acc, (count, item));
                count += 1;
                acc
            }
        }

        self.iter.fold(init, enumerate(self.count, fold))
    }
}
