pub mod traits;

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
	pub idx: usize,
	prev: Option<usize>,
	next: Option<usize>,
}

pub struct NeighbourListIter<'a, T: AdjustToNextNeighbour> {
	list: &'a NeighbourList<T>,
	next: Option<usize>,
	next_back: Option<usize>,
}

impl<'a, T: AdjustToNextNeighbour> NeighbourListIter<'a, T> {
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
	fn next_back(&mut self) -> Option<Self::Item> {
		if let Some(idx) = self.next_back {
			let node = &self.list.nodes[idx];
			self.next_back = node.prev;
			return Some(node);
		}
		None
	}
}

pub struct NeighbourListValsIter<'a, T: AdjustToNextNeighbour> {
	list: &'a NeighbourList<T>,
	next: Option<usize>,
	next_back: Option<usize>,
}

impl<'a, T: AdjustToNextNeighbour> NeighbourListValsIter<'a, T> {
	pub fn new(list: &'a NeighbourList<T>) -> Self {
		Self {
			list,
			next: list.first,
			next_back: list.last,
		}
	}
}

impl<'a, T: AdjustToNextNeighbour> Iterator for NeighbourListValsIter<'a, T> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(idx) = self.next {
			let node = &self.list.nodes[idx];
			self.next = node.next;
			return Some(&node.val);
		}
		None
	}
}

impl<'a, T: AdjustToNextNeighbour> DoubleEndedIterator for NeighbourListValsIter<'a, T> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if let Some(idx) = self.next_back {
			let node = &self.list.nodes[idx];
			self.next_back = node.prev;
			return Some(&node.val);
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

impl<T: AdjustToNextNeighbour> NeighbourList<T> {
	pub fn new() -> Self {
		Self {
			nodes: Vec::new(),
			first: None,
			last: None,
		}
	}

	pub fn append(&mut self, val: T) -> &Self {
		let idx = self.nodes.len();
		if let Some(last_idx) = self.last {
			let new_node = NeighbourListNode {
				prev: Some(last_idx),
				next: None,
				val,
				idx,
			};

			let last_node = &mut self.nodes[last_idx];
			last_node.val.adjust_to_next(&new_node.val);
			last_node.next = Some(idx);

			self.nodes.push(new_node);
		} else {
			self.first = Some(idx);
			self.nodes.push(NeighbourListNode {
				prev: None,
				next: None,
				val,
				idx,
			});
		}
		self.last = Some(idx);
		self
	}

	pub fn append_at(&mut self, curr_idx: usize, val: T) -> &Self {
		let node = &self.nodes[curr_idx];
		if let Some(next_idx) = node.next {
			let prev_idx = node.idx;
			let idx = self.nodes.len();

			let mut new_node = NeighbourListNode {
				val,
				idx,
				next: Some(next_idx),
				prev: Some(prev_idx),
			};

			let next_node = &mut self.nodes[next_idx];
			next_node.prev = Some(idx);

			new_node.val.adjust_to_next(&next_node.val);

			let prev_node = &mut self.nodes[prev_idx];
			prev_node.next = Some(idx);

			prev_node.val.adjust_to_next(&new_node.val);

			self.nodes.push(new_node);
		} else {
			return self.append(val);
		}
		self
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn iter(&self) -> NeighbourListIter<'_, T> {
		NeighbourListIter::new(self)
	}

	pub fn vals(&self) -> NeighbourListValsIter<'_, T> {
		NeighbourListValsIter::new(self)
	}

	pub fn iter_mut(&mut self) -> NeighbourListIterMut<'_, T> {
		NeighbourListIterMut::new(self)
	}

	pub fn first(&self) -> Option<&NeighbourListNode<T>> {
		self.first.and_then(|idx| self.nodes.get(idx))
	}

	pub fn last(&self) -> Option<&NeighbourListNode<T>> {
		self.last.and_then(|idx| self.nodes.get(idx))
	}

	pub fn next(&self, curr_idx: usize) -> Option<&NeighbourListNode<T>> {
		self.nodes[curr_idx]
			.next
			.and_then(|idx| self.nodes.get(idx))
	}

	pub fn prev(&self, curr_idx: usize) -> Option<&NeighbourListNode<T>> {
		self.nodes[curr_idx]
			.prev
			.and_then(|idx| self.nodes.get(idx))
	}

	pub fn first_mut(&mut self) -> Option<&mut NeighbourListNode<T>> {
		self.first.map(|idx| &mut self.nodes[idx])
	}

	pub fn last_mut(&mut self) -> Option<&mut NeighbourListNode<T>> {
		self.last.map(|idx| &mut self.nodes[idx])
	}

	pub fn next_mut(&mut self, curr_idx: usize) -> Option<&mut NeighbourListNode<T>> {
		self.nodes[curr_idx].next.map(|idx| &mut self.nodes[idx])
	}

	pub fn prev_mut(&mut self, curr_idx: usize) -> Option<&mut NeighbourListNode<T>> {
		self.nodes[curr_idx].prev.map(|idx| &mut self.nodes[idx])
	}

	pub fn adjust_all(self) -> Self {
		todo!()
	}
}

impl<T: AdjustToNextNeighbour> PartialEq for NeighbourListNode<T> {
	fn eq(&self, other: &Self) -> bool {
		self.idx == other.idx
	}
}

impl<'a, T> IntoIterator for &'a NeighbourList<T>
where
	T: AdjustToNextNeighbour,
{
	type Item = &'a T;
	type IntoIter = NeighbourListValsIter<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.vals()
	}
}

impl<T: AdjustToNextNeighbour> FromIterator<T> for NeighbourList<T> {
	fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
		let mut list = Self::new();
		for item in iter {
			list.append(item);
		}
		list
	}
}

#[cfg(test)]
mod tests;
