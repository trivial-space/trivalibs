pub trait Lerp {
	fn lerp(self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
	fn lerp(self, other: Self, t: f32) -> Self {
		self + (other - self) * t
	}
}
