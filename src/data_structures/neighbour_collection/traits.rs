pub trait NeighbourCollectionNode<T: Clone> {
    fn val(&self) -> &T;
    fn idx(&self) -> usize;
}

pub trait NeighbourCollection<T: Clone> {
    type Node: NeighbourCollectionNode<T>;
    fn empty() -> Self;
    fn append(&mut self, val: T) -> &Self;

    fn first(&self) -> Option<Self::Node>;
    fn last(&self) -> Option<Self::Node>;
    fn next(&self, idx: usize) -> Option<Self::Node>;
    fn prev(&self, idx: usize) -> Option<Self::Node>;

    fn map_with_prev_next<F: Fn(&T, Option<&T>, Option<&T>) -> T>(&self, f: F) -> Self
    where
        Self: Sized,
    {
        let mut new_list = Self::empty();
        let mut node = self.first();
        while let Some(current) = node {
            let i = current.idx();
            new_list.append(f(
                current.val(),
                self.prev(i).map(|n| n.val()),
                self.next(i).map(|n| n.val()),
            ));
            node = self.next(i)
        }

        new_list
    }

    fn flatmap_with_prev_next<F: Fn(&T, Option<&T>, Option<&T>) -> Vec<T>>(&self, f: F) -> Self
    where
        Self: Sized,
    {
        let mut new_list = Self::empty();
        let mut node = self.first();
        while let Some(current) = node {
            let i = current.idx();
            let nodes = f(
                current.val(),
                self.prev(i).map(|n| n.val()),
                self.next(i).map(|n| n.val()),
            );
            for j in 0..nodes.len() {
                new_list.append(nodes[j].clone());
            }
            node = self.next(i)
        }

        new_list
    }
}
