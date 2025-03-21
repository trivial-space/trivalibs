use spirv_std::glam::{vec2, vec3, Vec2, Vec3, Vec4};

pub fn smoothen(t: f32) -> f32 {
	let t = t.clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}

pub trait Smoothen {
	fn smoothen(self) -> Self;
}

impl Smoothen for f32 {
	fn smoothen(self) -> f32 {
		smoothen(self)
	}
}

impl Smoothen for Vec2 {
	fn smoothen(self) -> Vec2 {
		vec2(self.x.smoothen(), self.y.smoothen())
	}
}

impl Smoothen for Vec3 {
	fn smoothen(self) -> Vec3 {
		vec3(self.x.smoothen(), self.y.smoothen(), self.z.smoothen())
	}
}

impl Smoothen for Vec4 {
	fn smoothen(self) -> Vec4 {
		Vec4::new(
			self.x.smoothen(),
			self.y.smoothen(),
			self.z.smoothen(),
			self.w.smoothen(),
		)
	}
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = (x - edge0) / (edge1 - edge0);
	smoothen(t)
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
		vec2(
			self.x.smoothstep(edge1.x, x.x),
			self.y.smoothstep(edge1.y, x.y),
		)
	}
}

impl Smoothstep for Vec3 {
	fn smoothstep(self, edge1: Self, x: Self) -> Self {
		vec3(
			self.x.smoothstep(edge1.x, x.x),
			self.y.smoothstep(edge1.y, x.y),
			self.z.smoothstep(edge1.z, x.z),
		)
	}
}
