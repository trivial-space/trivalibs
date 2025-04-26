use crate::data::Position3D;
use glam::{vec2, Vec2, Vec3};
use lerp::Lerp;

pub struct Quad3D<P>
where
	P: Position3D,
{
	pub top_left: P,
	pub bottom_left: P,
	pub bottom_right: P,
	pub top_right: P,

	pub normal: Vec3,
}

impl<P> Quad3D<P>
where
	P: Position3D,
{
	pub fn from_dimensions_at_pos_f<F: Fn(Vec3, Vec2) -> P>(
		width: f32,
		height: f32,
		normal: Vec3,
		pos: Vec3,
		uv: Vec2,
		f: F,
	) -> Self {
		let normal = normal.normalize();
		let up = if normal.y.abs() > 0.999 {
			-Vec3::Z
		} else {
			Vec3::Y
		};
		let u_dir = up.cross(normal);
		let u_vec = u_dir * width;
		let v_vec = -normal.cross(u_dir) * height;

		let top_left = pos + -u_vec * uv.x - v_vec * uv.y;
		let top_right = top_left + u_vec;
		let bottom_left = top_left + v_vec;
		let bottom_right = bottom_left + u_vec;

		Self {
			top_left: f(top_left, vec2(0.0, 0.0)),
			bottom_left: f(bottom_left, vec2(0.0, 1.0)),
			bottom_right: f(bottom_right, vec2(1.0, 1.0)),
			top_right: f(top_right, vec2(1.0, 0.0)),

			normal,
		}
	}

	pub fn from_dimensions_center_f<F: Fn(Vec3, Vec2) -> P>(
		width: f32,
		height: f32,
		normal: Vec3,
		center: Vec3,
		f: F,
	) -> Self {
		Self::from_dimensions_at_pos_f(width, height, normal, center, vec2(0.5, 0.5), f)
	}

	pub fn from_dimensions_tl_f<F: Fn(Vec3, Vec2) -> P>(
		width: f32,
		height: f32,
		normal: Vec3,
		top_left: Vec3,
		f: F,
	) -> Self {
		Self::from_dimensions_at_pos_f(width, height, normal, top_left, vec2(0.0, 0.0), f)
	}

	pub fn from_verts_f<F: Fn(Vec3, Vec2) -> P>(
		top_left: Vec3,
		bottom_left: Vec3,
		bottom_right: Vec3,
		top_right: Vec3,
		f: F,
	) -> Self {
		let u_vec = top_right - top_left;
		let v_vec = bottom_left - top_left;

		assert!(
			top_left + u_vec + v_vec == bottom_right,
			"bottom_right is not equal to top_left + u_vec + v_vec"
		);

		let width = u_vec.length();
		let height = v_vec.length();

		assert!(width > 0.0, "width is zero");
		assert!(height > 0.0, "height is zero");

		let normal = u_vec.cross(v_vec).normalize();

		Self {
			top_left: f(top_left, vec2(0.0, 0.0)),
			bottom_left: f(bottom_left, vec2(0.0, 1.0)),
			bottom_right: f(bottom_right, vec2(1.0, 1.0)),
			top_right: f(top_right, vec2(1.0, 0.0)),

			normal,
		}
	}

	pub fn from_tl_bl_tr_f<F: Fn(Vec3, Vec2) -> P>(
		top_left: Vec3,
		bottom_left: Vec3,
		top_right: Vec3,
		f: F,
	) -> Self {
		let u_vec = top_right - top_left;
		let bottom_right = bottom_left + u_vec;

		Self::from_verts_f(top_left, bottom_left, bottom_right, top_right, f)
	}

	pub fn from_tl_bl_br_f<F: Fn(Vec3, Vec2) -> P>(
		top_left: Vec3,
		bottom_left: Vec3,
		bottom_right: Vec3,
		f: F,
	) -> Self {
		let u_vec = bottom_right - bottom_left;
		let top_right = top_left + u_vec;

		Self::from_verts_f(top_left, bottom_left, bottom_right, top_right, f)
	}

	pub fn from_tl_br_tr_f<F: Fn(Vec3, Vec2) -> P>(
		top_left: Vec3,
		bottom_right: Vec3,
		top_right: Vec3,
		f: F,
	) -> Self {
		let v_vec = bottom_right - top_right;
		let bottom_left = top_left + v_vec;

		Self::from_verts_f(top_left, bottom_left, bottom_right, top_right, f)
	}

	pub fn from_br_bl_tr_f<F: Fn(Vec3, Vec2) -> P>(
		bottom_right: Vec3,
		bottom_left: Vec3,
		top_right: Vec3,
		f: F,
	) -> Self {
		let u_vec = bottom_right - bottom_left;
		let top_left = top_right - u_vec;

		Self::from_verts_f(top_left, bottom_left, bottom_right, top_right, f)
	}

	pub fn from_verts(top_left: P, bottom_left: P, bottom_right: P, top_right: P) -> Self {
		let u_vec = top_right.position() - top_left.position();
		let v_vec = bottom_left.position() - top_left.position();

		assert!(
			top_left.position() + u_vec + v_vec == bottom_right.position(),
			"bottom_right is not equal to top_left + u_vec + v_vec"
		);

		let width = u_vec.length();
		let height = v_vec.length();

		assert!(width > 0.0, "width is zero");
		assert!(height > 0.0, "height is zero");

		let normal = u_vec.cross(v_vec).normalize();

		Self {
			top_left,
			bottom_left,
			bottom_right,
			top_right,

			normal,
		}
	}
}

impl<P> Quad3D<P>
where
	P: Position3D + Clone,
{
	pub fn to_ccw_verts(&self) -> [P; 4] {
		[
			self.top_left.clone(),
			self.bottom_left.clone(),
			self.bottom_right.clone(),
			self.top_right.clone(),
		]
	}

	pub fn to_cw_verts(&self) -> [P; 4] {
		[
			self.top_left.clone(),
			self.top_right.clone(),
			self.bottom_right.clone(),
			self.bottom_left.clone(),
		]
	}
}

impl<P> Quad3D<P>
where
	P: Position3D + Clone + Lerp<f32>,
{
	pub fn subdivide_at_v(&self, v: f32) -> (Quad3D<P>, Quad3D<P>) {
		let mid_left = self.top_left.clone().lerp(self.bottom_left.clone(), v);
		let mid_right = self.top_right.clone().lerp(self.bottom_right.clone(), v);

		let quad_top = Quad3D::from_verts(
			self.top_left.clone(),
			mid_left.clone(),
			mid_right.clone(),
			self.top_right.clone(),
		);

		let quad_bottom = Quad3D::from_verts(
			mid_left,
			self.bottom_left.clone(),
			self.bottom_right.clone(),
			mid_right,
		);

		(quad_top, quad_bottom)
	}

	pub fn subdivide_at_u(&self, u: f32) -> (Quad3D<P>, Quad3D<P>) {
		let mid_top = self.top_left.clone().lerp(self.top_right.clone(), u);
		let mid_bottom = self.bottom_left.clone().lerp(self.bottom_right.clone(), u);

		let quad_left = Quad3D::from_verts(
			self.top_left.clone(),
			self.bottom_left.clone(),
			mid_bottom.clone(),
			mid_top.clone(),
		);

		let quad_right = Quad3D::from_verts(
			mid_top,
			mid_bottom,
			self.bottom_right.clone(),
			self.top_right.clone(),
		);

		(quad_left, quad_right)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::rendering::mesh_geometry::utils::vert_pos_uv;
	use glam::{vec2, vec3};

	#[test]
	fn test_quad() {
		let quad =
			Quad3D::from_dimensions_center_f(4.0, 2.0, Vec3::Z, vec3(0.0, 3.0, 0.0), vert_pos_uv);

		assert_eq!(quad.normal, vec3(0.0, 0.0, 1.0));

		assert_eq!(quad.top_left.pos, vec3(-2.0, 4.0, 0.0));
		assert_eq!(quad.bottom_right.pos, vec3(2.0, 2.0, 0.0));
		assert_eq!(quad.top_right.pos, vec3(2.0, 4.0, 0.0));
		assert_eq!(quad.bottom_left.pos, vec3(-2.0, 2.0, 0.0));

		assert_eq!(quad.top_left.uv, vec2(0.0, 0.0,));
		assert_eq!(quad.bottom_right.uv, vec2(1.0, 1.0,));
		assert_eq!(quad.top_right.uv, vec2(1.0, 0.0,));
		assert_eq!(quad.bottom_left.uv, vec2(0.0, 1.0,));

		let quad = Quad3D::from_dimensions_at_pos_f(
			1.0,
			1.0,
			vec3(0.0, 3.3, 0.0),
			vec3(0.0, 0.0, 0.0),
			vec2(1.0, 1.0),
			vert_pos_uv,
		);

		assert_eq!(quad.normal, vec3(0.0, 1.0, 0.0));

		assert_eq!(quad.top_left.pos, vec3(-1.0, 0.0, -1.0));
		assert_eq!(quad.bottom_right.pos, vec3(0.0, 0.0, 0.0));
		assert_eq!(quad.top_right.pos, vec3(0.0, 0.0, -1.0));
		assert_eq!(quad.bottom_left.pos, vec3(-1.0, 0.0, 0.0));

		assert_eq!(quad.top_left.uv, vec2(0.0, 0.0,));
		assert_eq!(quad.bottom_right.uv, vec2(1.0, 1.0,));
		assert_eq!(quad.top_right.uv, vec2(1.0, 0.0,));
		assert_eq!(quad.bottom_left.uv, vec2(0.0, 1.0,));
	}
}
