pub trait AdjustToNextNeighbour {
    fn adjust_to_next(&mut self, next: &Self);
}

#[derive(Debug)]
pub struct NeighbourList<T: AdjustToNextNeighbour> {
    nodes: Vec<NeighbourListNode<T>>,
    first: Option<usize>,
    last: Option<usize>,
}

#[derive(Debug)]
pub struct NeighbourListNode<T: AdjustToNextNeighbour> {
    pub val: T,
    index: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct NeighbourListIter<'a, T: AdjustToNextNeighbour> {
    list: &'a NeighbourList<T>,
    next: Option<usize>,
    next_back: Option<usize>,
}

impl<'a, T: AdjustToNextNeighbour> NeighbourListIter<'a, T> {
    #[inline]
    pub fn new(list: &'a NeighbourList<T>) -> Self {
        Self {
            list,
            next: list.first,
            next_back: list.last,
        }
    }
}

impl<'a, T: AdjustToNextNeighbour> Iterator for NeighbourListIter<'a, T> {
    type Item = &'a NeighbourListNode<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next {
            let node = &self.list.nodes[idx];
            self.next = node.next;
            return Some(node);
        }
        None
    }
}

impl<'a, T: AdjustToNextNeighbour> DoubleEndedIterator for NeighbourListIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next_back {
            let node = &self.list.nodes[idx];
            self.next_back = node.prev;
            return Some(node);
        }
        None
    }
}

pub struct NeighbourListIterMut<'a, T: AdjustToNextNeighbour> {
    list: &'a mut NeighbourList<T>,
    next: Option<usize>,
    next_back: Option<usize>,
}

impl<'a, T: AdjustToNextNeighbour> NeighbourListIterMut<'a, T> {
    #[inline]
    pub fn new(list: &'a mut NeighbourList<T>) -> Self {
        let first = list.first;
        let last = list.last;
        Self {
            list,
            next: first,
            next_back: last,
        }
    }
}

impl<'a, T: AdjustToNextNeighbour> Iterator for NeighbourListIterMut<'a, T> {
    type Item = &'a mut NeighbourListNode<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next {
            let node = &mut self.list.nodes[idx];
            self.next = node.next;
            unsafe {
                return Some(std::mem::transmute(node));
            }
        }
        None
    }
}

impl<'a, T: AdjustToNextNeighbour> DoubleEndedIterator for NeighbourListIterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next_back {
            let node = &mut self.list.nodes[idx];
            self.next_back = node.prev;
            unsafe {
                return Some(std::mem::transmute(node));
            }
        }
        None
    }
}

impl<T: AdjustToNextNeighbour> PartialEq for NeighbourListNode<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T: AdjustToNextNeighbour> NeighbourList<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            first: None,
            last: None,
        }
    }

    pub fn append(&mut self, val: T) -> &Self {
        let new_idx = self.nodes.len();
        if let Some(last_idx) = self.last {
            let last_node = &mut self.nodes[last_idx];
            last_node.next = Some(new_idx);
            self.nodes.push(NeighbourListNode {
                prev: Some(last_idx),
                next: None,
                val,
                index: new_idx,
            });
        } else {
            self.first = Some(new_idx);
            self.nodes.push(NeighbourListNode {
                prev: None,
                next: None,
                val,
                index: new_idx,
            });
        }
        self.last = Some(new_idx);
        self
    }

    #[inline]
    pub fn next(&self, node: &NeighbourListNode<T>) -> Option<&NeighbourListNode<T>> {
        node.next.map(|idx| &self.nodes[idx])
    }
    #[inline]
    pub fn prev(&self, node: &NeighbourListNode<T>) -> Option<&NeighbourListNode<T>> {
        node.prev.map(|idx| &self.nodes[idx])
    }
    #[inline]
    pub fn first(&self) -> Option<&NeighbourListNode<T>> {
        self.first.map(|idx| &self.nodes[idx])
    }
    #[inline]
    pub fn last(&self) -> Option<&NeighbourListNode<T>> {
        self.last.map(|idx| &self.nodes[idx])
    }

    #[inline]
    pub fn iter(&self) -> NeighbourListIter<'_, T> {
        NeighbourListIter::new(self)
    }
    #[inline]
    pub fn next_mut(&mut self, node: &NeighbourListNode<T>) -> Option<&mut NeighbourListNode<T>> {
        node.next.map(|idx| &mut self.nodes[idx])
    }
    #[inline]
    pub fn prev_mut(&mut self, node: &NeighbourListNode<T>) -> Option<&mut NeighbourListNode<T>> {
        node.prev.map(|idx| &mut self.nodes[idx])
    }
    #[inline]
    pub fn first_mut(&mut self) -> Option<&mut NeighbourListNode<T>> {
        self.first.map(|idx| &mut self.nodes[idx])
    }
    #[inline]
    pub fn last_mut(&mut self) -> Option<&mut NeighbourListNode<T>> {
        self.last.map(|idx| &mut self.nodes[idx])
    }
    #[inline]
    pub fn iter_mut(&mut self) -> NeighbourListIterMut<'_, T> {
        NeighbourListIterMut::new(self)
    }

    pub fn adjust_all(self) -> Self {
        todo!()
    }
}

#[cfg(test)]
mod tests;
