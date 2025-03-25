use spirv_std::glam::{vec2, Vec2};

pub fn step(edge: f32, x: f32) -> f32 {
	if x < edge {
		0.0
	} else {
		1.0
	}
}

pub trait Step {
	fn step(self, edge: Self) -> Self;
	fn step_f32(self, edge: f32) -> Self;
}

impl Step for f32 {
	fn step(self, edge: f32) -> f32 {
		step(edge, self)
	}
	fn step_f32(self, edge: f32) -> Self {
		step(edge, self)
	}
}

impl Step for Vec2 {
	fn step(self, edge: Vec2) -> Vec2 {
		vec2(self.x.step(edge.x), self.y.step(edge.y))
	}
	fn step_f32(self, edge: f32) -> Vec2 {
		vec2(self.x.step(edge), self.y.step(edge))
	}
}
