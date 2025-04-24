use glam::{vec3, Vec3};

pub struct Cuboid {
	pub center: Vec3,
	pub size: Vec3,
}

impl Cuboid {
	pub fn box_at(center: Vec3, width: f32, height: f32, depth: f32) -> Self {
		Self {
			center,
			size: vec3(width, height, depth),
		}
	}

	pub fn unit_cube() -> Self {
		Self {
			center: Vec3::ZERO,
			size: Vec3::ONE,
		}
	}

	pub fn front_face_ccw(&self) -> [Vec3; 4] {
		let Vec3 { x, y, z } = self.center;
		let Vec3 { x: w, y: h, z: d } = self.size * 0.5;

		[
			vec3(x - w, y + h, z + d),
			vec3(x - w, y - h, z + d),
			vec3(x + w, y - h, z + d),
			vec3(x + w, y + h, z + d),
		]
	}

	pub fn front_face_cw(&self) -> [Vec3; 4] {
		let mut ret = self.front_face_ccw();
		ret.reverse();
		ret
	}

	pub fn back_face(&self) -> [Vec3; 4] {
		let Vec3 { x, y, z } = self.center;
		let Vec3 { x: w, y: h, z: d } = self.size * 0.5;

		[
			vec3(x - w, y + h, z - d),
			vec3(x + w, y + h, z - d),
			vec3(x + w, y - h, z - d),
			vec3(x - w, y - h, z - d),
		]
	}

	pub fn left_face(&self) -> [Vec3; 4] {
		let Vec3 { x, y, z } = self.center;
		let Vec3 { x: w, y: h, z: d } = self.size * 0.5;

		[
			vec3(x - w, y - h, z - d),
			vec3(x - w, y - h, z + d),
			vec3(x - w, y + h, z + d),
			vec3(x - w, y + h, z - d),
		]
	}

	pub fn right_face(&self) -> [Vec3; 4] {
		let Vec3 { x, y, z } = self.center;
		let Vec3 { x: w, y: h, z: d } = self.size * 0.5;

		[
			vec3(x + w, y + h, z - d),
			vec3(x + w, y + h, z + d),
			vec3(x + w, y - h, z + d),
			vec3(x + w, y - h, z - d),
		]
	}

	pub fn top_face(&self) -> [Vec3; 4] {
		let Vec3 { x, y, z } = self.center;
		let Vec3 { x: w, y: h, z: d } = self.size * 0.5;

		[
			vec3(x - w, y + h, z + d),
			vec3(x + w, y + h, z + d),
			vec3(x + w, y + h, z - d),
			vec3(x - w, y + h, z - d),
		]
	}

	pub fn bottom_face(&self) -> [Vec3; 4] {
		let Vec3 { x, y, z } = self.center;
		let Vec3 { x: w, y: h, z: d } = self.size * 0.5;

		[
			vec3(x - w, y - h, z - d),
			vec3(x + w, y - h, z - d),
			vec3(x + w, y - h, z + d),
			vec3(x - w, y - h, z + d),
		]
	}
}
