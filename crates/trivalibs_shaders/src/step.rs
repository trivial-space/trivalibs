use spirv_std::glam::{vec2, Vec2};

pub fn step(edge: f32, x: f32) -> f32 {
	if x < edge {
		0.0
	} else {
		1.0
	}
}

pub trait Step {
	fn step(self, x: Self) -> Self;
}

impl Step for f32 {
	fn step(self, x: f32) -> f32 {
		step(self, x)
	}
}

impl Step for Vec2 {
	fn step(self, x: Vec2) -> Vec2 {
		vec2(self.x.step(x.x), self.y.step(x.y))
	}
}
