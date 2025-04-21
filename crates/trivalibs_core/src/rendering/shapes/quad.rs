use crate::{
	data::Position3D,
	rendering::mesh_geometry::utils::{vert_pos_uv, Vert3dUv},
};
use glam::{vec3, Vec2, Vec3};
use lerp::Lerp;

pub struct Quad<P>
where
	P: Position3D,
{
	pub top_left: P,
	pub bottom_right: P,
}

pub trait QuadVertices<P>
where
	P: Position3D,
{
	fn to_cw_verts(&self) -> [P; 4];
	fn to_ccw_verts(&self) -> [P; 4];
}

impl<P> Quad<P>
where
	P: Position3D + Lerp<f32> + Clone,
{
	pub fn new(top_left: P, bottom_right: P) -> Self {
		Self {
			top_left,
			bottom_right,
		}
	}

	pub fn to_vec3(&self) -> Quad<Vec3> {
		Quad {
			top_left: self.top_left.position(),
			bottom_right: self.bottom_right.position(),
		}
	}
}

impl Quad<Vec3> {
	pub fn from_dimensions(center: Vec3, size: Vec2, normal: Vec3) -> Self {
		let half_size = size / 2.0;
		let mut top = Vec3::Y;
		if normal.y > 0.999 {
			top = -Vec3::Z;
		}
		let top_left = center + normal.cross(top) * half_size.x + top * half_size.y;
		let bottom_right = center - normal.cross(top) * half_size.x - top * half_size.y;

		Quad::new(top_left, bottom_right)
	}
}

impl QuadVertices<Vec3> for Quad<Vec3> {
	fn to_cw_verts(&self) -> [Vec3; 4] {
		[
			self.top_left,
			vec3(self.bottom_right.x, self.top_left.y, self.top_left.z),
			self.bottom_right,
			vec3(self.top_left.x, self.bottom_right.y, self.bottom_right.z),
		]
	}

	fn to_ccw_verts(&self) -> [Vec3; 4] {
		[
			self.top_left,
			vec3(self.top_left.x, self.bottom_right.y, self.bottom_right.z),
			self.bottom_right,
			vec3(self.bottom_right.x, self.top_left.y, self.top_left.z),
		]
	}
}

impl From<Quad<Vec3>> for Quad<Vert3dUv> {
	fn from(quad: Quad<Vec3>) -> Self {
		Self {
			top_left: vert_pos_uv(quad.top_left, Vec2::new(0.0, 0.0)),
			bottom_right: vert_pos_uv(quad.bottom_right, Vec2::new(1.0, 1.0)),
		}
	}
}

impl QuadVertices<Vert3dUv> for Quad<Vert3dUv> {
	fn to_cw_verts(&self) -> [Vert3dUv; 4] {
		let verts = self.to_vec3().to_cw_verts();
		[
			vert_pos_uv(verts[0], self.top_left.uv),
			vert_pos_uv(
				verts[1],
				Vec2::new(self.bottom_right.uv.x, self.top_left.uv.y),
			),
			vert_pos_uv(verts[2], self.bottom_right.uv),
			vert_pos_uv(
				verts[3],
				Vec2::new(self.top_left.uv.x, self.bottom_right.uv.y),
			),
		]
	}

	fn to_ccw_verts(&self) -> [Vert3dUv; 4] {
		let verts = self.to_vec3().to_ccw_verts();
		[
			vert_pos_uv(verts[0], self.top_left.uv),
			vert_pos_uv(
				verts[1],
				Vec2::new(self.top_left.uv.x, self.bottom_right.uv.y),
			),
			vert_pos_uv(verts[2], self.bottom_right.uv),
			vert_pos_uv(
				verts[3],
				Vec2::new(self.bottom_right.uv.x, self.top_left.uv.y),
			),
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use glam::{vec2, vec3};

	#[test]
	fn test_quad_vertices() {
		let quad = Quad::from_dimensions(Vec3::ONE, vec2(4.0, 5.0), Vec3::Z);

		assert_eq!(quad.top_left, vec3(-1.0, 3.5, 1.0));
		assert_eq!(quad.bottom_right, vec3(3.0, -1.5, 1.0));

		let verts = quad.to_ccw_verts();
		assert_eq!(verts[0], vec3(-1.0, 3.5, 1.0));
		assert_eq!(verts[1], vec3(-1.0, -1.5, 1.0));
		assert_eq!(verts[2], vec3(3.0, -1.5, 1.0));
		assert_eq!(verts[3], vec3(3.0, 3.5, 1.0));

		let quad = Quad::from_dimensions(Vec3::ONE, vec2(2.0, 3.0), Vec3::Y);

		assert_eq!(quad.top_left, vec3(0.0, 1.0, -0.5));
		assert_eq!(quad.bottom_right, vec3(2.0, 1.0, 2.5));

		let verts = quad.to_cw_verts();
		assert_eq!(verts[0], vec3(0.0, 1.0, -0.5));
		assert_eq!(verts[1], vec3(2.0, 1.0, -0.5));
		assert_eq!(verts[2], vec3(2.0, 1.0, 2.5));
		assert_eq!(verts[3], vec3(0.0, 1.0, 2.5));
	}
}
