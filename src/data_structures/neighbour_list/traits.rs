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

impl<T: Clone, I, F, B> Iterator for NeighbourMap<I, F>
where
    I: Iterator<Item = T>,
    F: FnMut(I::Item, Option<I::Item>, Option<I::Item>) -> B,
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
            let f = &mut self.f;
            Some(f(r, self.prev.clone(), self.next.clone()))
        } else {
            None
        }
    }
}

pub trait NeighbourMapTransform: Iterator + Sized {
    fn map_with_prev_next<F, B>(self, f: F) -> NeighbourMap<Self, F>
    where
        F: FnMut(Self::Item, Option<Self::Item>, Option<Self::Item>) -> B;
}

impl<I> NeighbourMapTransform for I
where
    I: Iterator,
{
    fn map_with_prev_next<F, B>(self, f: F) -> NeighbourMap<I, F>
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

#[cfg(test)]
mod tests {
    use super::NeighbourMapTransform;

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

    // impl Clone for SomeBox {
    //     fn clone(&self) -> Self {
    //         Self {
    //             val: self.val.clone(),
    //             count: self.count.clone() + 1,
    //         }
    //     }
    // }

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
}
