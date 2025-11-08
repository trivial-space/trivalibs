#![allow(unexpected_cfgs)]

#[cfg(target_arch = "spirv")]
#[allow(unused_imports)]
use spirv_std::num_traits::Float;

pub fn fit1101(x: f32) -> f32 {
	x * 0.5 + 0.5
}

pub fn fit1101_f64(x: f64) -> f64 {
	x * 0.5 + 0.5
}

pub fn fit0111(x: f32) -> f32 {
	x * 2.0 - 1.0
}

pub fn fit0111_f64(x: f64) -> f64 {
	x * 2.0 - 1.0
}

pub fn step(edge: f32, x: f32) -> f32 {
	if x < edge { 0.0 } else { 1.0 }
}

pub fn step_f64(edge: f64, x: f64) -> f64 {
	if x < edge { 0.0 } else { 1.0 }
}

/// Third order polynomial interpolation of values between 0 and 1.
/// Make sure to clamp the input to [0, 1] before using this function.
pub fn smoothen(t: f32) -> f32 {
	t * t * (3.0 - 2.0 * t)
}

/// Third order polynomial interpolation of values between 0 and 1.
/// Make sure to clamp the input to [0, 1] before using this function.
pub fn smoothen_f64(t: f64) -> f64 {
	t * t * (3.0 - 2.0 * t)
}

/// Fifth order polynomial interpolation of values between 0 and 1.
/// Make sure to clamp the input to [0, 1] before using this function.
pub fn smoothen_more(t: f32) -> f32 {
	t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Fifth order polynomial interpolation of values between 0 and 1.
/// Make sure to clamp the input to [0, 1] before using this function.
pub fn smoothen_more_f64(t: f64) -> f64 {
	t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Fractional part of a number. It is defined as `x - floor(x)`.
/// In contrast, std implementation fract is defined as `x - trunc(x)`, which inverts direction when negative.
pub fn frct(x: f32) -> f32 {
	x - x.floor()
}

/// Fractional part of a number. It is defined as `x - floor(x)`.
/// In contrast, std implementation fract is defined as `x - trunc(x)`, which inverts direction when negative.
pub fn frct_f64(x: f64) -> f64 {
	x - x.floor()
}

pub trait NumExt
where
	Self: Sized,
{
	fn fit0111(self) -> Self;
	fn fit1101(self) -> Self;
	fn clamp01(self) -> Self;

	/// Fractional part of a number. It is defined as `x - floor(x)`.
	/// In contrast, std implementation fract is defined as `x - trunc(x)`, which inverts direction when negative.
	fn frct(self) -> Self;
	fn rem(self, other: Self) -> Self;

	fn lerp(self, other: Self, t: Self) -> Self;
	fn step(self, edge: Self) -> Self;
	fn gtf(self, edge: Self) -> Self;
	fn ltf(self, edge: Self) -> Self;
	fn gtef(self, edge: Self) -> Self;
	fn ltef(self, edge: Self) -> Self;

	/// Third order polynomial interpolation of values between 0 and 1.
	/// Make sure to clamp the input to [0, 1] before using this function.
	fn smoothen(self) -> Self;

	/// Fifth order polynomial interpolation of values between 0 and 1.
	/// Make sure to clamp the input to [0, 1] before using this function.
	fn smoothen_more(self) -> Self;

	fn smoothstep(self, edge0: Self, edge1: Self) -> Self;
	fn step_fn<F: Fn(Self) -> Self>(self, edge0: Self, edge1: Self, f: F) -> Self;
}

impl NumExt for f32 {
	fn fit0111(self) -> Self {
		fit0111(self)
	}
	fn fit1101(self) -> Self {
		fit1101(self)
	}
	fn clamp01(self) -> Self {
		self.clamp(0., 1.)
	}
	fn frct(self) -> Self {
		frct(self)
	}
	fn rem(self, other: Self) -> Self {
		let r = self % other;
		if r < 0.0 { r + other.abs() } else { r }
	}

	fn lerp(self, other: Self, t: f32) -> Self {
		self + (other - self) * t
	}
	fn step(self, edge: Self) -> Self {
		step(edge, self)
	}
	fn gtf(self, edge: Self) -> Self {
		if self > edge { 1.0 } else { 0.0 }
	}
	fn ltf(self, edge: Self) -> Self {
		if self < edge { 1.0 } else { 0.0 }
	}
	fn gtef(self, edge: Self) -> Self {
		if self >= edge { 1.0 } else { 0.0 }
	}
	fn ltef(self, edge: Self) -> Self {
		if self <= edge { 1.0 } else { 0.0 }
	}
	fn step_fn<F: Fn(Self) -> Self>(self, edge0: Self, edge1: Self, f: F) -> Self {
		let t = (self - edge0) / (edge1 - edge0);
		f(t.clamp01())
	}

	fn smoothen(self) -> Self {
		smoothen(self)
	}
	fn smoothen_more(self) -> Self {
		smoothen_more(self)
	}
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self {
		self.step_fn(edge0, edge1, |t| t.smoothen())
	}
}

impl NumExt for f64 {
	fn fit0111(self) -> Self {
		fit0111_f64(self)
	}
	fn fit1101(self) -> Self {
		fit1101_f64(self)
	}
	fn clamp01(self) -> Self {
		self.clamp(0., 1.)
	}
	fn frct(self) -> Self {
		frct_f64(self)
	}
	fn rem(self, other: Self) -> Self {
		let r = self % other;
		if r < 0.0 { r + other.abs() } else { r }
	}

	fn lerp(self, other: Self, t: f64) -> Self {
		self + (other - self) * t
	}
	fn step(self, edge: Self) -> Self {
		step_f64(edge, self)
	}
	fn gtf(self, edge: Self) -> Self {
		if self > edge { 1.0 } else { 0.0 }
	}
	fn ltf(self, edge: Self) -> Self {
		if self < edge { 1.0 } else { 0.0 }
	}
	fn gtef(self, edge: Self) -> Self {
		if self >= edge { 1.0 } else { 0.0 }
	}
	fn ltef(self, edge: Self) -> Self {
		if self <= edge { 1.0 } else { 0.0 }
	}
	fn step_fn<F: Fn(Self) -> Self>(self, edge0: Self, edge1: Self, f: F) -> Self {
		let t = (self - edge0) / (edge1 - edge0);
		f(t.clamp01())
	}

	fn smoothen(self) -> Self {
		smoothen_f64(self)
	}
	fn smoothen_more(self) -> Self {
		smoothen_more_f64(self)
	}
	fn smoothstep(self, edge0: Self, edge1: Self) -> Self {
		self.step_fn(edge0, edge1, |t| t.smoothen())
	}
}
