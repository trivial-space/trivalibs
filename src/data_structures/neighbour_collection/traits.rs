pub trait NeighbourMapTransform: Iterator {
    fn map_with_prev_next<F>(self, f: F) -> Self
    where
        F: Fn(&Self::Item, Option<&Self::Item>, Option<&Self::Item>) -> Self::Item;
}

pub trait NeighbourFlatMapTransform: Iterator {
    fn flatmap_with_prev_next<F>(self, f: F) -> Self
    where
        F: Fn(&Self::Item, Option<&Self::Item>, Option<&Self::Item>) -> Self::Item;
}

struct NeighbourMap<'a, T, I>
where
    T: 'a,
    I: Iterator<Item = &'a T>,
{
    prev: Option<&'a T>,
    next: Option<&'a T>,
    iter: I,
}

impl<I> NeighbourMapTransform for I
where
    I: Iterator,
{
    fn map_with_prev_next<F>(self, f: F) -> Self
    where
        F: Fn(&I::Item, Option<&I::Item>, Option<&I::Item>) -> I::Item,
    {
        let mut prev: Option<&I::Item> = None;
        let mut next: Option<&I::Item> = None;
        let mut current = self.next();
        // let mut new_list = Self::empty();
        // let mut node = self.first();
        // while let Some(current) = node {
        //     let i = current.idx();
        //     new_list.append(f(
        //         current.val(),
        //         self.prev(i).map(|n| n.val()),
        //         self.next(i).map(|n| n.val()),
        //     ));
        //     node = self.next(i)
        // }

        // new_list
    }
}
