use glam::{Vec2, Vec3};

pub mod grid;
pub mod neighbour_list;
pub mod vertex_index;

pub trait Overridable {
	fn override_with(&self, other: &Self) -> Self;
}

pub trait NotOverridable: Copy + Clone {}

impl<T> Overridable for T
where
	T: NotOverridable,
{
	fn override_with(&self, _other: &Self) -> Self {
		self.clone()
	}
}

pub trait Position3D {
	fn position(&self) -> Vec3;
}

pub trait Position2D {
	fn position(&self) -> Vec2;
}
