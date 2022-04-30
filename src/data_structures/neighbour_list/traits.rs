use std::iter::Flatten;

// With Neighbour iterator

pub struct WithNeighbours<I>
where
    I: Iterator,
{
    prev: Option<I::Item>,
    next: Option<I::Item>,
    curr: Option<I::Item>,
    iter: I,
}

impl<I: Iterator> WithNeighbours<I> {
    fn new(iter: I) -> Self {
        Self {
            prev: None,
            next: None,
            curr: None,
            iter,
        }
    }
}

impl<T, I> Iterator for WithNeighbours<I>
where
    T: Clone,
    I: Iterator<Item = T>,
{
    type Item = (Option<T>, T, Option<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.prev.is_none() && self.curr.is_none() {
            self.curr = self.iter.next();
            self.next = self.iter.next();
        } else {
            self.prev = self.curr.clone();
            self.curr = self.next.clone();
            self.next = self.iter.next();
        }
        if let Some(r) = self.curr.clone() {
            Some((self.prev.clone(), r, self.next.clone()))
        } else {
            None
        }
    }
}

pub trait WithNeighboursTransform: Iterator + Sized {
    fn with_neighbours(self) -> WithNeighbours<Self>;
}

impl<I> WithNeighboursTransform for I
where
    I: Iterator,
{
    fn with_neighbours(self) -> WithNeighbours<Self> {
        WithNeighbours::new(self)
    }
}

// Map iterator

pub struct NeighbourMap<I, F>
where
    I: Iterator,
{
    prev: Option<I::Item>,
    next: Option<I::Item>,
    curr: Option<I::Item>,
    iter: I,
    f: F,
}

impl<'a, I: Iterator, F> NeighbourMap<I, F> {
    fn new(iter: I, f: F) -> Self {
        Self {
            prev: None,
            next: None,
            curr: None,
            iter,
            f,
        }
    }
}

impl<T, I, F, B> Iterator for NeighbourMap<I, F>
where
    T: Clone,
    I: Iterator<Item = T>,
    F: Fn(I::Item, Option<I::Item>, Option<I::Item>) -> B,
{
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        if self.prev.is_none() && self.curr.is_none() {
            self.curr = self.iter.next();
            self.next = self.iter.next();
        } else {
            self.prev = self.curr.clone();
            self.curr = self.next.clone();
            self.next = self.iter.next();
        }
        if let Some(r) = self.curr.clone() {
            let f = &self.f;
            Some(f(r, self.prev.clone(), self.next.clone()))
        } else {
            None
        }
    }
}

pub trait NeighbourMapTransform: Iterator + Sized {
    fn map_with_prev_next<F, B>(self, f: F) -> NeighbourMap<Self, F>
    where
        F: Fn(Self::Item, Option<Self::Item>, Option<Self::Item>) -> B;
}

impl<I> NeighbourMapTransform for I
where
    I: Iterator,
{
    fn map_with_prev_next<F, B>(self, f: F) -> NeighbourMap<I, F>
    where
        F: Fn(I::Item, Option<I::Item>, Option<I::Item>) -> B,
    {
        NeighbourMap::new(self, f)
    }
}

// Flatmap on iterator

pub struct NeighbourFlatMap<T, I, U, F>
where
    T: Clone,
    U: IntoIterator,
    I: Iterator<Item = T>,
    F: Fn(I::Item, Option<I::Item>, Option<I::Item>) -> U,
{
    inner: Flatten<NeighbourMap<I, F>>,
}

impl<T, I, U, F> NeighbourFlatMap<T, I, U, F>
where
    T: Clone,
    I: Iterator<Item = T>,
    U: IntoIterator,
    F: Fn(I::Item, Option<I::Item>, Option<I::Item>) -> U,
{
    pub fn new(iter: I, f: F) -> NeighbourFlatMap<T, I, U, F> {
        NeighbourFlatMap {
            inner: NeighbourMap::new(iter, f).flatten(),
        }
    }
}

impl<T, I, U, F> Iterator for NeighbourFlatMap<T, I, U, F>
where
    T: Clone,
    U: IntoIterator,
    I: Iterator<Item = T>,
    F: Fn(I::Item, Option<I::Item>, Option<I::Item>) -> U,
{
    type Item = U::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub trait NeighbourFlatMapTransform: Iterator + Sized {
    fn flat_map_with_prev_next<F, U>(self, f: F) -> NeighbourFlatMap<Self::Item, Self, U, F>
    where
        Self::Item: Clone,
        U: IntoIterator,
        F: Fn(Self::Item, Option<Self::Item>, Option<Self::Item>) -> U;
}

impl<T, I> NeighbourFlatMapTransform for I
where
    T: Clone,
    I: Iterator<Item = T>,
{
    fn flat_map_with_prev_next<F, U>(self, f: F) -> NeighbourFlatMap<I::Item, I, U, F>
    where
        U: IntoIterator,
        F: Fn(I::Item, Option<I::Item>, Option<I::Item>) -> U,
    {
        NeighbourFlatMap::new(self, f)
    }
}

// Tests

#[cfg(test)]
mod tests {
    use super::{NeighbourFlatMapTransform, NeighbourMapTransform, WithNeighboursTransform};

    #[test]
    fn test_map_with_prev_next() {
        let v = vec![1, 2, 3];

        let res = v
            .iter()
            .map_with_prev_next(|curr, prev, next| curr + prev.unwrap_or(&0) + next.unwrap_or(&0))
            .collect::<Vec<_>>();

        assert_eq!(res, [3, 6, 5]);

        let res = v
            .iter()
            .map_with_prev_next(|curr, prev, next| (prev.map(|x| *x), *curr, next.map(|x| *x)))
            .collect::<Vec<_>>();

        assert_eq!(
            res,
            [
                (None, 1, Some(2)),
                (Some(1), 2, Some(3)),
                (Some(2), 3, None)
            ]
        );
    }

    #[derive(Default, PartialEq, Debug)]
    struct SomeBox {
        val: usize,
        count: usize,
    }

    impl Clone for SomeBox {
        fn clone(&self) -> Self {
            Self {
                val: self.val.clone(),
                count: self.count.clone() + 1,
            }
        }
    }

    fn make_box(val: usize) -> SomeBox {
        SomeBox { val, count: 0 }
    }

    #[test]
    fn test_does_not_clone_ref_values() {
        let v = vec![make_box(1), make_box(2), make_box(3)];

        let res = v
            .iter()
            .map_with_prev_next(|curr, prev, next| SomeBox {
                val: curr.val
                    + prev.unwrap_or(&Default::default()).val
                    + next.unwrap_or(&Default::default()).val,
                count: curr.count,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            res,
            [
                SomeBox { val: 3, count: 0 },
                SomeBox { val: 6, count: 0 },
                SomeBox { val: 5, count: 0 }
            ]
        )
    }

    #[test]
    fn test_flat_map() {
        let v = vec![1, 2, 3];

        let res = v
            .iter()
            .flat_map_with_prev_next(|curr, prev, next| {
                let mut vs = vec![];
                if let Some(i) = prev {
                    vs.push(*i);
                }
                vs.push(*curr);
                if let Some(i) = next {
                    vs.push(*i);
                }
                vs
            })
            .collect::<Vec<_>>();

        assert_eq!(res, [1, 2, 1, 2, 3, 2, 3])
    }

    #[test]
    fn with_neighbours() {
        let v = vec![1, 2, 3, 4];

        let res = v.iter().with_neighbours().collect::<Vec<_>>();

        assert_eq!(
            res,
            [
                (None, &1, Some(&2)),
                (Some(&1), &2, Some(&3)),
                (Some(&2), &3, Some(&4)),
                (Some(&3), &4, None)
            ]
        );
    }
}
