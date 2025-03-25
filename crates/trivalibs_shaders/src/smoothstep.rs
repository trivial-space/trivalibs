use spirv_std::glam::{vec2, vec3, Vec2, Vec3, Vec4};

/// Third order polynomial interpolation of values between 0 and 1.
/// Other values are clamped to 0 or 1.
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

/// Fifth order polynomial interpolation of values between 0 and 1.
/// Other values are clamped to 0 or 1.
pub fn smoothen_more(t: f32) -> f32 {
	let t = t.clamp(0.0, 1.0);
	t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub trait SmoothenMore {
	fn smoothen_more(self) -> Self;
}

impl SmoothenMore for f32 {
	fn smoothen_more(self) -> f32 {
		smoothen_more(self)
	}
}

impl SmoothenMore for Vec2 {
	fn smoothen_more(self) -> Vec2 {
		vec2(self.x.smoothen_more(), self.y.smoothen_more())
	}
}

impl SmoothenMore for Vec3 {
	fn smoothen_more(self) -> Vec3 {
		vec3(
			self.x.smoothen_more(),
			self.y.smoothen_more(),
			self.z.smoothen_more(),
		)
	}
}

impl SmoothenMore for Vec4 {
	fn smoothen_more(self) -> Vec4 {
		Vec4::new(
			self.x.smoothen_more(),
			self.y.smoothen_more(),
			self.z.smoothen_more(),
			self.w.smoothen_more(),
		)
	}
}

// === Smoothstep ===

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = (x - edge0) / (edge1 - edge0);
	smoothen(t)
}

pub trait Smoothstep {
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self;
}

impl Smoothstep for f32 {
	fn smoothstep(self, edge0: f32, edge1: f32) -> f32 {
		smoothstep(edge0, edge1, self)
	}
}

impl Smoothstep for Vec2 {
	fn smoothstep(self, edge0: Vec2, edge1: Vec2) -> Vec2 {
		vec2(
			self.x.smoothstep(edge0.x, edge1.x),
			self.y.smoothstep(edge0.y, edge1.y),
		)
	}
}

impl Smoothstep for Vec3 {
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self {
		vec3(
			self.x.smoothstep(edge0.x, edge1.x),
			self.y.smoothstep(edge0.y, edge1.y),
			self.z.smoothstep(edge0.z, edge1.z),
		)
	}
}

impl Smoothstep for Vec4 {
	fn smoothstep(self, edge0: Self, egde1: Self) -> Self {
		Vec4::new(
			self.x.smoothstep(edge0.x, egde1.x),
			self.y.smoothstep(edge0.y, egde1.y),
			self.z.smoothstep(edge0.z, egde1.z),
			self.w.smoothstep(edge0.w, egde1.w),
		)
	}
}

// === Smootherstep ===

pub fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let t = (x - edge0) / (edge1 - edge0);
	smoothen_more(t)
}

pub trait Smootherstep {
	fn smootherstep(self, edge0: Self, edge1: Self) -> Self;
}

impl Smootherstep for f32 {
	fn smootherstep(self, edge0: f32, edge1: f32) -> f32 {
		smootherstep(edge0, edge1, self)
	}
}

impl Smootherstep for Vec2 {
	fn smootherstep(self, edge0: Vec2, edge1: Vec2) -> Vec2 {
		vec2(
			self.x.smootherstep(edge0.x, edge1.x),
			self.y.smootherstep(edge0.y, edge1.y),
		)
	}
}

impl Smootherstep for Vec3 {
	fn smootherstep(self, edge0: Self, edge1: Self) -> Self {
		vec3(
			self.x.smootherstep(edge0.x, edge1.x),
			self.y.smootherstep(edge0.y, edge1.y),
			self.z.smootherstep(edge0.z, edge1.z),
		)
	}
}

impl Smootherstep for Vec4 {
	fn smootherstep(self, edge0: Self, edge1: Self) -> Self {
		Vec4::new(
			self.x.smootherstep(edge0.x, edge1.x),
			self.y.smootherstep(edge0.y, edge1.y),
			self.z.smootherstep(edge0.z, edge1.z),
			self.w.smootherstep(edge0.w, edge1.w),
		)
	}
}
