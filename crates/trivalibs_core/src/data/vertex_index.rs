use std::hash::Hash;

use glam::Vec3;

pub trait VertexIndex: Eq + Hash + Clone + Copy {}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct VertIdx2U(u32, u32);
impl VertexIndex for VertIdx2U {}
impl Hash for VertIdx2U {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let hash_val: u64 = self.0 as u64 + self.1 as u64 * 100_000_000;
		hash_val.hash(state)
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct VertIdx2Usize(pub usize, pub usize);
impl VertexIndex for VertIdx2Usize {}
impl Hash for VertIdx2Usize {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let hash_val: u64 = self.0 as u64 + self.1 as u64 * 100_000_000;
		hash_val.hash(state)
	}
}

#[derive(Clone, Copy, Debug)]
pub struct VertIdx3f(pub f32, pub f32, pub f32);
impl VertexIndex for VertIdx3f {}

impl From<Vec3> for VertIdx3f {
	fn from(v: Vec3) -> Self {
		Self(v.x, v.y, v.z)
	}
}

impl PartialEq for VertIdx3f {
	fn eq(&self, other: &Self) -> bool {
		const SCALE: f32 = 10_000.0;
		((self.0 * SCALE).round() as i32) == ((other.0 * SCALE).round() as i32)
			&& ((self.1 * SCALE).round() as i32) == ((other.1 * SCALE).round() as i32)
			&& ((self.2 * SCALE).round() as i32) == ((other.2 * SCALE).round() as i32)
	}
}

impl Eq for VertIdx3f {}

impl Hash for VertIdx3f {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		const SCALE: f32 = 10_000.0;
		let x = (self.0 * SCALE).round() as i32;
		let y = (self.1 * SCALE).round() as i32;
		let z = (self.2 * SCALE).round() as i32;
		x.hash(state);
		y.hash(state);
		z.hash(state);
	}
}
