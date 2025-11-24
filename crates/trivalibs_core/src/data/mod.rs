use glam::{Vec2, Vec3};

pub mod grid;
pub mod neighbour_list;
pub mod vertex_index;

pub trait Position3D {
	fn position(&self) -> Vec3;
}

impl Position3D for Vec3 {
	fn position(&self) -> Vec3 {
		*self
	}
}

pub trait Position2D {
	fn position(&self) -> Vec2;
}

impl Position2D for Vec2 {
	fn position(&self) -> Vec2 {
		*self
	}
}
