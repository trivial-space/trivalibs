use spirv_std::glam::{vec2, vec3, vec4, Vec2, Vec3, Vec4};

pub fn fit1101(x: f32) -> f32 {
	x * 0.5 + 0.5
}

pub fn fit0111(x: f32) -> f32 {
	x * 2.0 - 1.0
}

pub trait Fit {
	fn fit0111(self) -> Self;
	fn fit1101(self) -> Self;
}

impl Fit for f32 {
	fn fit0111(self) -> Self {
		fit0111(self)
	}

	fn fit1101(self) -> Self {
		fit1101(self)
	}
}

impl Fit for Vec2 {
	fn fit0111(self) -> Self {
		vec2(self.x.fit0111(), self.y.fit0111())
	}

	fn fit1101(self) -> Self {
		vec2(self.x.fit1101(), self.y.fit1101())
	}
}

impl Fit for Vec3 {
	fn fit0111(self) -> Self {
		vec3(self.x.fit0111(), self.y.fit0111(), self.z.fit0111())
	}

	fn fit1101(self) -> Self {
		vec3(self.x.fit1101(), self.y.fit1101(), self.z.fit1101())
	}
}

impl Fit for Vec4 {
	fn fit0111(self) -> Self {
		vec4(
			self.x.fit0111(),
			self.y.fit0111(),
			self.z.fit0111(),
			self.w.fit0111(),
		)
	}

	fn fit1101(self) -> Self {
		vec4(
			self.x.fit1101(),
			self.y.fit1101(),
			self.z.fit1101(),
			self.w.fit1101(),
		)
	}
}
