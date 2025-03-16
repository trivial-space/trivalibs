use spirv_std::glam::{Vec2, Vec3};

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = (x - edge0) / (edge1 - edge0);
	let t = t.clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}

pub trait Smoothstep {
	fn smoothstep(self, edge1: Self, x: Self) -> Self;
}

impl Smoothstep for f32 {
	fn smoothstep(self, edge1: f32, x: f32) -> f32 {
		smoothstep(self, edge1, x)
	}
}

impl Smoothstep for Vec2 {
	fn smoothstep(self, edge1: Vec2, x: Vec2) -> Vec2 {
		Vec2::new(
			self.x.smoothstep(edge1.x, x.x),
			self.y.smoothstep(edge1.y, x.y),
		)
	}
}

impl Smoothstep for Vec3 {
	fn smoothstep(self, edge1: Self, x: Self) -> Self {
		Vec3::new(
			self.x.smoothstep(edge1.x, x.x),
			self.y.smoothstep(edge1.y, x.y),
			self.z.smoothstep(edge1.z, x.z),
		)
	}
}
