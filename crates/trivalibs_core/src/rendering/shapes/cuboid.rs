use glam::{vec3, Vec3};

use crate::data::Position3D;

use super::quad::Quad3D;

pub struct Cuboid {
	pub center: Vec3,
	pub size: Vec3,

	pub front_top_left: Vec3,
	pub front_top_right: Vec3,
	pub front_bottom_left: Vec3,
	pub front_bottom_right: Vec3,

	pub back_top_left: Vec3,
	pub back_top_right: Vec3,
	pub back_bottom_left: Vec3,
	pub back_bottom_right: Vec3,
}

impl Cuboid {
	pub fn box_at(center: Vec3, width: f32, height: f32, depth: f32) -> Self {
		let Vec3 { x, y, z } = center;
		let hw = width * 0.5;
		let hh = height * 0.5;
		let hd = depth * 0.5;

		Self {
			center,
			size: vec3(width, height, depth),

			front_top_left: vec3(x - hw, y + hh, z + hd),
			front_top_right: vec3(x + hw, y + hh, z + hd),
			front_bottom_left: vec3(x - hw, y - hh, z + hd),
			front_bottom_right: vec3(x + hw, y - hh, z + hd),

			back_top_left: vec3(x - hw, y + hh, z - hd),
			back_top_right: vec3(x + hw, y + hh, z - hd),
			back_bottom_left: vec3(x - hw, y - hh, z - hd),
			back_bottom_right: vec3(x + hw, y - hh, z - hd),
		}
	}

	pub fn unit_cube() -> Self {
		Self::box_at(Vec3::ZERO, 1.0, 1.0, 1.0)
	}

	pub fn front_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Quad3D<P> {
		Quad3D {
			top_left: f(self.front_top_left, vec3(0.0, 0.0, 0.0)),
			bottom_left: f(self.front_bottom_left, vec3(0.0, 1.0, 0.0)),
			bottom_right: f(self.front_bottom_right, vec3(1.0, 1.0, 0.0)),
			top_right: f(self.front_top_right, vec3(1.0, 0.0, 0.0)),

			normal: Vec3::Z,
		}
	}

	pub fn front_face(&self) -> Quad3D<Vec3> {
		self.front_face_f(|pos, _| pos)
	}

	pub fn back_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Quad3D<P> {
		Quad3D {
			top_left: f(self.back_top_right, vec3(1.0, 0.0, 1.0)),
			bottom_left: f(self.back_bottom_right, vec3(1.0, 1.0, 1.0)),
			bottom_right: f(self.back_bottom_left, vec3(0.0, 1.0, 1.0)),
			top_right: f(self.back_top_left, vec3(0.0, 0.0, 1.0)),

			normal: -Vec3::Z,
		}
	}

	pub fn back_face(&self) -> Quad3D<Vec3> {
		self.back_face_f(|pos, _| pos)
	}

	pub fn left_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Quad3D<P> {
		Quad3D {
			top_left: f(self.back_top_left, vec3(0.0, 0.0, 1.0)),
			bottom_left: f(self.back_bottom_left, vec3(0.0, 1.0, 1.0)),
			bottom_right: f(self.front_bottom_left, vec3(0.0, 1.0, 0.0)),
			top_right: f(self.front_top_left, vec3(0.0, 0.0, 0.0)),

			normal: -Vec3::X,
		}
	}

	pub fn left_face(&self) -> Quad3D<Vec3> {
		self.left_face_f(|pos, _| pos)
	}

	pub fn right_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Quad3D<P> {
		Quad3D {
			top_left: f(self.front_top_right, vec3(1.0, 0.0, 0.0)),
			bottom_left: f(self.front_bottom_right, vec3(1.0, 1.0, 0.0)),
			bottom_right: f(self.back_bottom_right, vec3(1.0, 1.0, 1.0)),
			top_right: f(self.back_top_right, vec3(1.0, 0.0, 1.0)),

			normal: Vec3::X,
		}
	}

	pub fn right_face(&self) -> Quad3D<Vec3> {
		self.right_face_f(|pos, _| pos)
	}

	pub fn top_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Quad3D<P> {
		Quad3D {
			top_left: f(self.back_top_left, vec3(0.0, 0.0, 1.0)),
			bottom_left: f(self.front_top_left, vec3(0.0, 0.0, 0.0)),
			bottom_right: f(self.front_top_right, vec3(1.0, 0.0, 0.0)),
			top_right: f(self.back_top_right, vec3(1.0, 0.0, 1.0)),

			normal: Vec3::Y,
		}
	}

	pub fn top_face(&self) -> Quad3D<Vec3> {
		self.top_face_f(|pos, _| pos)
	}

	pub fn bottom_face_f<P: Position3D, F: Fn(Vec3, Vec3) -> P>(&self, f: F) -> Quad3D<P> {
		Quad3D {
			top_left: f(self.front_bottom_left, vec3(0.0, 1.0, 0.0)),
			bottom_left: f(self.back_bottom_left, vec3(0.0, 1.0, 1.0)),
			bottom_right: f(self.back_bottom_right, vec3(1.0, 1.0, 1.0)),
			top_right: f(self.front_bottom_right, vec3(1.0, 1.0, 0.0)),

			normal: -Vec3::Y,
		}
	}

	pub fn bottom_face(&self) -> Quad3D<Vec3> {
		self.bottom_face_f(|pos, _| pos)
	}
}
