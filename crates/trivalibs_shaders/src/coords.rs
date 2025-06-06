#![allow(unexpected_cfgs)]

#[cfg(not(target_arch = "spirv"))]
use glam::{vec2, Vec2};
#[cfg(target_arch = "spirv")]
use spirv_std::glam::{vec2, Vec2};
#[cfg(target_arch = "spirv")]
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub struct PolarCoord {
	pub radius: f32,
	pub angle: f32,
}

impl PolarCoord {
	pub fn to_2d(&self) -> Vec2 {
		vec2(
			self.radius * self.angle.cos(),
			self.radius * self.angle.sin(),
		)
	}

	pub fn as_vec(&self) -> Vec2 {
		vec2(self.radius, self.angle)
	}

	pub fn from_2d(v: Vec2) -> Self {
		Self {
			radius: v.length(),
			angle: v.y.atan2(v.x),
		}
	}

	pub fn from_vec(v: Vec2) -> Self {
		Self {
			radius: v.x,
			angle: v.y,
		}
	}

	pub fn from_2d_with_center(v: Vec2, center: Vec2) -> Self {
		let diff = v - center;
		Self::from_2d(diff)
	}
}

impl Default for PolarCoord {
	fn default() -> Self {
		Self {
			radius: 0.0,
			angle: 0.0,
		}
	}
}

impl From<Vec2> for PolarCoord {
	fn from(v: Vec2) -> Self {
		Self::from_vec(v)
	}
}

impl From<PolarCoord> for Vec2 {
	fn from(p: PolarCoord) -> Self {
		p.as_vec()
	}
}
