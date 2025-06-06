use trivalibs::{
	glam::{vec2, Vec2},
	gpu_data,
	macros::apply,
	map,
	painter::prelude::*,
	utils::default,
};

const BLUR_DIAMETER: f32 = 400.0;

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
	size: BindingBuffer<Vec2>,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let triangle_shade = p.shade(&[Float32x2, Float32x2]).create();
		load_vertex_shader!(triangle_shade, p, "./shader/triangle_vs.spv");
		load_fragment_shader!(triangle_shade, p, "./shader/triangle_fs.spv");

		let blur_shade = p
			.shade_effect()
			.with_uniforms(&[
				BINDING_BUFFER_FRAG,
				BINDING_BUFFER_FRAG,
				BINDING_BUFFER_FRAG,
				BINDING_SAMPLER_FRAG,
			])
			.with_layer()
			.create();
		load_fragment_shader!(blur_shade, p, "./shader/blur_fs.spv");

		let tri_form = p.form(TRIANGLE).create();

		let tri_shape = p.shape(tri_form, triangle_shade).create();

		let size = p.bind_vec2();
		let horiz = p.bind_const_vec2(vec2(1.0, 0.0));
		let vertical = p.bind_const_vec2(vec2(0.0, 1.0));
		let s = p.sampler_linear().binding();

		let mut effects = vec![];

		// ===  This does blur in multiple passes ===
		// It cuts the number of texture reads logarithmically, but increases the number of passes

		let mut counter = BLUR_DIAMETER / 9.0; // Fixed diameter in shader is 9.0
		while counter > 2.0 {
			let diameter = p.bind_const_f32(counter);
			effects.push(
				p.effect(blur_shade)
					.with_bindings(map! {
						0 => diameter,
						1 => size.binding(),
						2 => horiz,
						3 => s
					})
					.create(),
			);
			effects.push(
				p.effect(blur_shade)
					.with_bindings(map! {
						0 => diameter,
						1 => size.binding(),
						2 => vertical,
						3 => s
					})
					.create(),
			);
			counter /= 2.0;
		}

		// println!("effects: {:?}", effects.len());

		// === This does all blurs in one pass ===

		// let diameter = p.uniform_const_f32(BLUR_DIAMETER);
		// effects.push(
		// 	p.effect(blur_shade)
		// 		.with_uniforms(map! {
		// 			0 => diameter,
		// 			1 => size.uniform(),
		// 			2 => horiz,
		// 			3 => s
		// 		})
		// 		.create(),
		// );
		// effects.push(
		// 	p.effect(blur_shade)
		// 		.with_uniforms(map! {
		// 			0 => diameter,
		// 			1 => size.uniform(),
		// 			2 => vertical,
		// 			3 => s
		// 		})
		// 		.create(),
		// );

		let canvas = p
			.layer()
			.with_shape(tri_shape)
			.with_effects(effects)
			.with_clear_color(wgpu::Color::BLUE)
			.create();

		Self { canvas, size }
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.size.update(p, vec2(width as f32, height as f32));
	}

	fn update(&mut self, p: &mut Painter, _tpf: f32) {
		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint_and_show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: false,
			keep_window_dimensions: true,
			..default()
		})
		.start();
}
