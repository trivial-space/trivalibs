pub struct NeighbourMap<'a, T, I, F>
where
    I: Iterator,
{
    prev: Option<&'a T>,
    next: Option<&'a T>,
    curr: Option<&'a T>,
    iter: I,
    f: F,
}

impl<'a, T, I: Iterator, F> NeighbourMap<'a, T, I, F> {
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

impl<'a, T, I, F, B> Iterator for NeighbourMap<'a, T, I, F>
where
    I: Iterator<Item = &'a T>,
    F: FnMut(I::Item, Option<I::Item>, Option<I::Item>) -> B,
{
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        if self.prev.is_none() && self.curr.is_none() {
            self.curr = self.iter.next();
            self.next = self.iter.next();
        } else {
            self.prev = self.curr;
            self.curr = self.next;
            self.next = self.iter.next();
        }
        if let Some(r) = self.curr {
            let f = &mut self.f;
            Some(f(r, self.prev, self.next))
        } else {
            None
        }
    }
}

pub trait NeighbourMapTransform: Iterator + Sized {
    fn map_with_prev_next<'a, T, F, B>(self, f: F) -> NeighbourMap<'a, T, Self, F>
    where
        F: FnMut(Self::Item, Option<Self::Item>, Option<Self::Item>) -> B;
}

impl<I> NeighbourMapTransform for I
where
    I: Iterator,
{
    fn map_with_prev_next<'a, T, F, B>(self, f: F) -> NeighbourMap<'a, T, I, F>
    where
        F: FnMut(I::Item, Option<I::Item>, Option<I::Item>) -> B,
    {
        NeighbourMap::new(self, f)
    }
}

// TODO: Flatmap on iterator
pub trait NeighbourFlatMapTransform: Iterator {
    fn flatmap_with_prev_next<F, T, B, I: Iterator<Item = B>>(self, f: F) -> I
    where
        F: FnMut(&Self::Item, Option<&Self::Item>, Option<&Self::Item>) -> Vec<Self::Item>;
}

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
