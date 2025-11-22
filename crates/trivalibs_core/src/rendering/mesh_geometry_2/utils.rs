use crate::{
	data::{NotOverridable, Position3D},
	macros::gpu_data,
};
use glam::{Vec2, Vec3};
use lerp::Lerp;
use macro_rules_attribute::apply;

#[apply(gpu_data)]
pub struct Vert3d {
	pub pos: Vec3,
}
impl NotOverridable for Vert3d {}
impl Position3D for Vert3d {
	fn position(&self) -> Vec3 {
		self.pos
	}
}
impl Lerp<f32> for Vert3d {
	fn lerp(self, other: Self, t: f32) -> Self {
		Vert3d {
			pos: self.pos.lerp(other.pos, t),
		}
	}
}

pub fn vert_pos(pos: Vec3) -> Vert3d {
	Vert3d { pos }
}

#[apply(gpu_data)]
pub struct Vert3dUv {
	pub pos: Vec3,
	pub uv: Vec2,
}
impl NotOverridable for Vert3dUv {}
impl Position3D for Vert3dUv {
	fn position(&self) -> Vec3 {
		self.pos
	}
}
impl Lerp<f32> for Vert3dUv {
	fn lerp(self, other: Self, t: f32) -> Self {
		Vert3dUv {
			pos: self.pos.lerp(other.pos, t),
			uv: self.uv.lerp(other.uv, t),
		}
	}
}

pub fn vert_pos_uv(pos: Vec3, uv: Vec2) -> Vert3dUv {
	Vert3dUv { pos, uv }
}
