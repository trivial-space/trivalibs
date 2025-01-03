use crate::data::neighbour_list::traits::NeighbourFlatMapTransform;
use crate::prelude::*;
use glam::Vec2;
use lerp::Lerp;
use std::cell::Cell;
use std::slice::Iter;

#[derive(Clone, Copy)]
pub struct EmptyData {}
impl Default for EmptyData {
	fn default() -> Self {
		EmptyData {}
	}
}
impl<F> Lerp<F> for EmptyData {
	fn lerp(self, _: Self, _: F) -> Self {
		EmptyData {}
	}
}

#[derive(Clone, Copy)]
pub struct LineVertexData<T>
where
	T: Default + Copy + Clone + Lerp<f32> + Sized,
{
	pub pos: Vec2,
	pub width: f32,
	pub len: f32,
	pub dir: Vec2,
	pub data: T,
}

pub type LineVertex = LineVertexData<EmptyData>;

impl<T> Default for LineVertexData<T>
where
	T: Default + Copy + Clone + Lerp<f32>,
{
	fn default() -> Self {
		Self {
			pos: Vec2::ZERO,
			width: 1.0,
			len: 0.0,
			dir: Vec2::ZERO,
			data: T::default(),
		}
	}
}
impl<T> Lerp<f32> for LineVertexData<T>
where
	T: Default + Copy + Clone + Lerp<f32>,
{
	fn lerp(self, other: Self, t: f32) -> Self {
		line_vert_w_d(
			self.pos.lerp(other.pos, t),
			Lerp::lerp(self.width, other.width, t),
			self.data.lerp(other.data, t),
		)
	}
}

impl<T> LineVertexData<T>
where
	T: Default + Copy + Clone + Lerp<f32>,
{
	fn new(pos: Vec2) -> Self {
		LineVertexData { pos, ..default() }
	}

	pub fn point_to(&mut self, point: &Vec2) {
		let mut vec = *point - self.pos;
		let len = vec.length();
		self.len = len;
		vec /= len;
		self.dir = vec;
	}

	pub fn smouth_edge_threshold(
		&self,
		prev: &Self,
		next: &Self,
		ratio: f32,
		angle_threshold: f32,
	) -> Vec<Self> {
		let d = 1. - self.dir.dot(prev.dir);
		if d > angle_threshold {
			let v1 = prev.lerp(*self, 1.0 - ratio);
			let v2 = self.lerp(*next, ratio);

			vec![v1, v2]
		} else {
			vec![*self]
		}
	}

	pub fn smouth_edge(&self, prev: &Self, next: &Self, ratio: f32) -> Vec<Self> {
		self.smouth_edge_threshold(prev, next, ratio, 0.0)
	}
}

pub fn line_vert<T: Default + Copy + Clone + Lerp<f32>>(pos: Vec2) -> LineVertexData<T> {
	LineVertexData::new(pos)
}

pub fn line_vert_w<T: Default + Copy + Clone + Lerp<f32>>(
	pos: Vec2,
	width: f32,
) -> LineVertexData<T> {
	LineVertexData {
		pos,
		width,
		..default()
	}
}
pub fn line_vert_w_d<T: Default + Copy + Clone + Lerp<f32>>(
	pos: Vec2,
	width: f32,
	data: T,
) -> LineVertexData<T> {
	LineVertexData {
		pos,
		width,
		data,
		..default()
	}
}

#[derive(Clone)]
pub struct LineData<T>
where
	T: Default + Copy + Clone + Lerp<f32>,
{
	list: Vec<LineVertexData<T>>,
	len: f32,
	default_width: f32,
	len_offset: f32,
}

pub type Line = LineData<EmptyData>;

impl<T> LineData<T>
where
	T: Default + Copy + Clone + Lerp<f32>,
{
	pub fn new_offset(width: f32, offset: f32) -> Self {
		LineData::<T> {
			list: Vec::new(),
			len: 0.0,
			default_width: width,
			len_offset: offset,
		}
	}

	pub fn new(width: f32) -> Self {
		Self::new_offset(width, 0.0)
	}

	pub fn from_vecs<I: IntoIterator<Item = Vec2>>(line_width: f32, iter: I) -> Self {
		let mut line = LineData::<T>::new(line_width);
		for vert in iter {
			line.add(vert);
		}
		line
	}

	pub fn line_length(&self) -> f32 {
		self.len
	}

	pub fn vert_count(&self) -> usize {
		self.list.len()
	}

	pub fn add(&mut self, pos: Vec2) {
		self.add_vert(line_vert_w(pos, self.default_width));
	}

	pub fn add_width(&mut self, pos: Vec2, width: f32) {
		self.add_vert(line_vert_w(pos, width));
	}

	pub fn add_width_data(&mut self, pos: Vec2, width: f32, data: T) {
		self.add_vert(line_vert_w_d(pos, width, data));
	}

	pub fn add_vert(&mut self, mut vert: LineVertexData<T>) {
		let curr_len = self.list.len();

		if curr_len > 0 {
			let idx = curr_len - 1;
			let prev = &mut self.list[idx];
			prev.point_to(&vert.pos);

			self.len += prev.len;
			vert.dir = prev.dir;
		}

		self.list.push(vert);
	}

	pub fn add_vert_raw(&mut self, vert: LineVertexData<T>) {
		let curr_len = self.list.len();

		if curr_len > 0 {
			let idx = curr_len - 1;
			let prev = &self.list[idx];
			self.len += prev.len;
		}

		self.list.push(vert);
	}

	pub fn iter(&self) -> Iter<'_, LineVertexData<T>> {
		self.list.iter()
	}

	pub fn get(&self, i: usize) -> &LineVertexData<T> {
		&self.list[i]
	}

	pub fn get_opt(&self, i: usize) -> Option<&LineVertexData<T>> {
		self.list.get(i)
	}

	pub fn set_raw(&mut self, i: usize, vert: LineVertexData<T>) {
		self.list[i] = vert;
	}

	pub fn first(&self) -> &LineVertexData<T> {
		&self.list[0]
	}

	pub fn last(&self) -> &LineVertexData<T> {
		&self.list[self.list.len() - 1]
	}

	pub fn split_at_angle(&self, angle_threshold: f32) -> Vec<Self> {
		let mut lines = vec![];
		let mut line = LineData::<T>::new(self.default_width);
		let mut prev: Option<&LineVertexData<T>> = None;
		let cos_threshold = f32::cos(angle_threshold);
		let mut len_offset = 0.0;

		for v in self {
			line.add_vert_raw(*v);

			if let Some(prev) = prev {
				let dot = v.dir.dot(prev.dir);

				if dot <= cos_threshold {
					len_offset += line.len;
					let mut last = line.last().clone();
					last.dir = prev.dir;
					line.set_raw(line.list.len() - 1, last);
					lines.push(line);
					line = LineData::new(self.default_width);
					line.len_offset = len_offset;
					line.add_vert_raw(*v);
				}
			}
			prev = Some(v);
		}

		lines.push(line);
		lines
	}

	pub fn flat_map_with_prev_next<
		F: Fn(
			&LineVertexData<T>,
			Option<&LineVertexData<T>>,
			Option<&LineVertexData<T>>,
		) -> Vec<LineVertexData<T>>,
	>(
		&self,
		f: F,
	) -> Self {
		let new_vertices = self.iter().flat_map_with_prev_next(f);
		LineData::from_iter(new_vertices)
	}

	pub fn smouth_edges_threshold(&self, ratio: f32, min_dist: f32, angle_threshold: f32) -> Self {
		self.flat_map_with_prev_next(|curr, prev, next| {
			if prev.is_none() || next.is_none() {
				return vec![*curr];
			}

			let prev = prev.unwrap();
			let next = next.unwrap();

			if prev.len < min_dist || curr.len < min_dist {
				return vec![*curr];
			}

			return curr.smouth_edge_threshold(prev, next, ratio, angle_threshold);
		})
	}

	pub fn smouth_edges(&self, ratio: f32, min_dist: f32) -> Self {
		self.smouth_edges_threshold(ratio, min_dist, 0.0)
	}

	pub fn cleanup_vertices(
		&self,
		min_len_wid_ratio: f32,
		width_threshold: f32,
		angle_threshold: f32,
	) -> Self {
		let travelled_min_length_cell = Cell::new(0.0_f32);

		self.flat_map_with_prev_next(|curr, prev, next| {
			if prev.is_none() || next.is_none() {
				return vec![curr.clone()];
			}

			let prev = prev.unwrap();
			let next = next.unwrap();
			let travelled_min_length = travelled_min_length_cell.get();
			let len = prev.len + curr.len + travelled_min_length;
			let avg_width = (prev.width + curr.width * 2.0 + next.width) / 4.0;

			// handle min length, and skip vertices in between

			let min_len = f32::max(avg_width * min_len_wid_ratio, 1.0);

			if len < min_len {
				travelled_min_length_cell.set(travelled_min_length + prev.len);
				return vec![];
			}

			// TODO: Check if this is right!
			if prev.len + travelled_min_length < min_len {
				let dist = curr.len - (len - min_len);
				let ratio = dist / curr.len;
				travelled_min_length_cell.set(-dist);
				return vec![curr.lerp(*next, ratio)];
			}

			travelled_min_length_cell.set(0.0);

			// handle unneeded vertices when similar width
			// and similar direction as prev and next

			let is_same_width_prev =
				prev.width == curr.width || (1.0 - prev.width / curr.width).abs() < width_threshold;
			let is_same_width_next =
				curr.width == next.width || (1.0 - next.width / curr.width).abs() < width_threshold;
			let is_same_direction = 1.0 - prev.dir.dot(curr.dir) < angle_threshold;

			if is_same_width_next && is_same_width_prev && is_same_direction {
				return vec![];
			}

			return vec![curr.clone()];
		})
	}
}

impl<'a, T> IntoIterator for &'a LineData<T>
where
	T: Default + Copy + Clone + Lerp<f32>,
{
	type Item = &'a LineVertexData<T>;
	type IntoIter = Iter<'a, LineVertexData<T>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<T> FromIterator<LineVertexData<T>> for LineData<T>
where
	T: Default + Copy + Clone + Lerp<f32>,
{
	fn from_iter<I: IntoIterator<Item = LineVertexData<T>>>(iter: I) -> Self {
		let mut line = LineData::new(1.0);
		for vert in iter {
			line.add_vert(vert);
		}
		line
	}
}

pub mod buffered_geometry;
#[cfg(test)]
mod tests;
