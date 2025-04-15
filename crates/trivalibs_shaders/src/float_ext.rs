#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub fn fit1101(x: f32) -> f32 {
	x * 0.5 + 0.5
}

pub fn fit0111(x: f32) -> f32 {
	x * 2.0 - 1.0
}

pub fn step(edge: f32, x: f32) -> f32 {
	if x < edge {
		0.0
	} else {
		1.0
	}
}

/// Third order polynomial interpolation of values between 0 and 1.
/// Other values are clamped to 0 or 1.
pub fn smoothen(t: f32) -> f32 {
	let t = t.clamp(0.0, 1.0);
	t * t * (3.0 - 2.0 * t)
}

/// Fifth order polynomial interpolation of values between 0 and 1.
/// Other values are clamped to 0 or 1.
pub fn smoothen_more(t: f32) -> f32 {
	let t = t.clamp(0.0, 1.0);
	t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

pub trait FloatExt {
	fn fit0111(self) -> Self;
	fn fit1101(self) -> Self;

	fn lerp(self, other: Self, t: f32) -> Self;
	fn step(self, edge: Self) -> Self;

	fn smoothen(self) -> Self;
	fn smoothen_more(self) -> Self;
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self;
}

impl FloatExt for f32 {
	fn fit0111(self) -> Self {
		fit0111(self)
	}
	fn fit1101(self) -> Self {
		fit1101(self)
	}

	fn lerp(self, other: Self, t: f32) -> Self {
		self + (other - self) * t
	}
	fn step(self, edge: Self) -> Self {
		step(edge, self)
	}

	fn smoothen(self) -> Self {
		smoothen(self)
	}
	fn smoothen_more(self) -> Self {
		smoothen_more(self)
	}
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self {
		let t = (self - edge0) / (edge1 - edge0);
		t.smoothen()
	}
}
