pub trait AdjustToNextNeighbour {
    fn adjust_to_next(&mut self, next: &Self);
}

pub struct NeighbourList<T: AdjustToNextNeighbour> {
    vec: Vec<T>,
}

pub struct NeighbourListNode<'a, T> {
    pub prev: Option<&'a T>,
    pub val: &'a T,
    pub next: Option<&'a T>,
    pub index: usize,
}

impl<T: AdjustToNextNeighbour> NeighbourList<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn get(&self, index: usize) -> Option<NeighbourListNode<'_, T>> {
        if let Some(val) = self.vec.get(index) {
            return Some(NeighbourListNode {
                index,
                val,
                prev: self.vec.get(index - 1),
                next: self.vec.get(index + 1),
            });
        }
        None
    }

    pub fn push(mut self, val: T) -> Self {
        let len = self.vec.len();
        if let Some(prev) = self.vec.get_mut(len - 1) {
            prev.adjust_to_next(&val);
        }
        self.vec.push(val);
        self
    }

    pub fn push_raw(mut self, val: T) -> Self {
        self.vec.push(val);
        self
    }

    pub fn adjust_all(self) -> Self {
        todo!()
    }
}
