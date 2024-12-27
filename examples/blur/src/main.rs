use trivalibs::{
	glam::{vec2, Vec2},
	gpu_data,
	macros::apply,
	painter::{layer::Layer, wgpu, CanvasApp, Event, Painter},
};

#[apply(gpu_data)]
struct Vertex {
	pos: Vec2,
	uv: Vec2,
}

const TRIANGLE: &[Vertex] = &[
	Vertex {
		pos: vec2(-0.7, -0.7),
		uv: vec2(0.0, 1.0),
	},
	Vertex {
		pos: vec2(0.7, -0.7),
		uv: vec2(1.0, 1.0),
	},
	Vertex {
		pos: vec2(0.0, 0.7),
		uv: vec2(0.5, 0.0),
	},
];

struct App {
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		todo!()
	}

	fn resize(&mut self, p: &mut Painter) {
		todo!()
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		todo!()
	}

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		todo!()
	}

	fn event(&mut self, event: Event<()>, p: &mut Painter) {
		todo!()
	}
}

pub fn main() {
	App::create().start();
}
