use std::{intrinsics::transmute, marker::PhantomData};

use self::traits::{NeighbourCollection, NeighbourCollectionNode};

mod traits;

pub trait AdjustToNextNeighbour {
    fn adjust_to_next(&mut self, next: &Self);
}

#[derive(Debug)]
pub struct NeighbourList<'a, T: AdjustToNextNeighbour + Clone> {
    nodes: Vec<NeighbourListNodeData<T>>,
    first: Option<usize>,
    last: Option<usize>,
    _phantom: &'a PhantomData<T>,
}

#[derive(Debug)]
struct NeighbourListNodeData<T: AdjustToNextNeighbour + Clone> {
    val: T,
    idx: usize,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct NeighbourListNode<'a, T: AdjustToNextNeighbour + Clone> {
    data: &'a NeighbourListNodeData<T>,
}

impl<'a, T: AdjustToNextNeighbour + Clone> NeighbourCollectionNode<T> for NeighbourListNode<'a, T> {
    #[inline]
    fn val(&self) -> &T {
        &self.data.val
    }

    #[inline]
    fn idx(&self) -> usize {
        self.data.idx
    }
}

pub struct NeighbourListIter<'a, T: AdjustToNextNeighbour + Clone> {
    list: &'a NeighbourList<'a, T>,
    next: Option<usize>,
    next_back: Option<usize>,
}

impl<'a, T: AdjustToNextNeighbour + Clone> NeighbourListIter<'a, T> {
    #[inline]
    pub fn new(list: &'a NeighbourList<T>) -> Self {
        Self {
            list,
            next: list.first,
            next_back: list.last,
        }
    }
}

impl<'a, T: AdjustToNextNeighbour + Clone> Iterator for NeighbourListIter<'a, T> {
    type Item = &'a NeighbourListNodeData<T>;

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

impl<'a, T: AdjustToNextNeighbour + Clone> DoubleEndedIterator for NeighbourListIter<'a, T> {
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

pub struct NeighbourListIterMut<'a, T: AdjustToNextNeighbour + Clone> {
    list: &'a mut NeighbourList<'a, T>,
    next: Option<usize>,
    next_back: Option<usize>,
}

impl<'a, T: AdjustToNextNeighbour + Clone> NeighbourListIterMut<'a, T> {
    #[inline]
    pub fn new(list: &'a mut NeighbourList<'a, T>) -> Self {
        let first = list.first;
        let last = list.last;
        Self {
            list,
            next: first,
            next_back: last,
        }
    }
}

impl<'a, T: AdjustToNextNeighbour + Clone> Iterator for NeighbourListIterMut<'a, T> {
    type Item = &'a mut NeighbourListNodeData<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.next {
            let node = &mut self.list.nodes[idx];
            self.next = node.next;
            unsafe {
                return Some(transmute(node));
            }
        }
        None
    }
}

impl<'a, T: AdjustToNextNeighbour + Clone> DoubleEndedIterator for NeighbourListIterMut<'a, T> {
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

impl<T: AdjustToNextNeighbour + Clone> PartialEq for NeighbourListNodeData<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl<'a, T: AdjustToNextNeighbour + Clone> NeighbourList<'a, T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            first: None,
            last: None,
            _phantom: &PhantomData,
        }
    }

    pub fn append_at(&mut self, curr_idx: usize, val: T) -> &Self {
        let node = &self.nodes[curr_idx];
        if let Some(next_idx) = node.next {
            let prev_idx = node.idx;
            let new_idx = self.nodes.len();
            self.nodes.push(NeighbourListNodeData {
                idx: new_idx,
                next: Some(next_idx),
                prev: Some(prev_idx),
                val,
            });

            self.nodes[next_idx].prev = Some(new_idx);
            self.nodes[prev_idx].next = Some(new_idx);
        } else {
            return self.append(val);
        }
        self
    }

    #[inline]
    pub fn iter(&self) -> NeighbourListIter<'_, T> {
        NeighbourListIter::new(self)
    }

    pub fn first_mut(&mut self) -> Option<&mut NeighbourListNodeData<T>> {
        self.first.map(|idx| &mut self.nodes[idx])
    }

    pub fn last_mut(&mut self) -> Option<&mut NeighbourListNodeData<T>> {
        self.last.map(|idx| &mut self.nodes[idx])
    }

    pub fn next_mut(&mut self, curr_idx: usize) -> Option<&mut NeighbourListNodeData<T>> {
        self.nodes[curr_idx].next.map(|idx| &mut self.nodes[idx])
    }

    pub fn prev_mut(&mut self, curr_idx: usize) -> Option<&mut NeighbourListNodeData<T>> {
        self.nodes[curr_idx].prev.map(|idx| &mut self.nodes[idx])
    }

    #[inline]
    pub fn iter_mut(&'a mut self) -> NeighbourListIterMut<'a, T> {
        NeighbourListIterMut::new(self)
    }

    pub fn adjust_all(self) -> Self {
        todo!()
    }
}

impl<'a, T: AdjustToNextNeighbour + Clone> NeighbourCollection<T> for NeighbourList<'a, T> {
    type Node = NeighbourListNode<'a, T>;

    #[inline]
    fn empty() -> Self {
        Self::new()
    }

    fn append(&mut self, val: T) -> &Self {
        let new_idx = self.nodes.len();
        if let Some(last_idx) = self.last {
            self.nodes.push(NeighbourListNodeData {
                prev: Some(last_idx),
                next: None,
                val,
                idx: new_idx,
            });
            self.nodes[last_idx].next = Some(new_idx);
        } else {
            self.first = Some(new_idx);
            self.nodes.push(NeighbourListNodeData {
                prev: None,
                next: None,
                val,
                idx: new_idx,
            });
        }
        self.last = Some(new_idx);
        self
    }

    fn first(&self) -> Option<Self::Node> {
        self.first.map(|idx| NeighbourListNode {
            data: unsafe { transmute(&self.nodes[idx]) },
        })
    }

    fn last(&self) -> Option<NeighbourListNode<'a, T>> {
        self.last.map(|idx| NeighbourListNode {
            data: unsafe { transmute(&self.nodes[idx]) },
        })
    }

    fn next(&self, curr_idx: usize) -> Option<NeighbourListNode<'a, T>> {
        self.nodes[curr_idx].next.map(|idx| NeighbourListNode {
            data: unsafe { transmute(&self.nodes[idx]) },
        })
    }

    fn prev(&self, curr_idx: usize) -> Option<NeighbourListNode<'a, T>> {
        self.nodes[curr_idx].prev.map(|idx| NeighbourListNode {
            data: unsafe { transmute(&self.nodes[idx]) },
        })
    }
}

#[cfg(test)]
mod tests;
