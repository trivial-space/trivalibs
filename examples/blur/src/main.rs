use trivalibs::{
	bmap,
	glam::{vec2, Vec2},
	gpu_data,
	macros::apply,
	painter::{
		effect::EffectProps,
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		shade::{ShadeEffectProps, ShadeProps},
		sketch::SketchProps,
		uniform::UniformBuffer,
		wgpu::{self, VertexFormat::*},
		AppConfig, CanvasApp, Event, Painter,
	},
	utils::default,
};

const BLUR_RADIUS: f32 = 200.0;

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
	size: UniformBuffer<Vec2>,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let u_fs_type = p.uniform_type_buffered_frag();
		let tex_type = p.uniform_type_tex_2d_frag();

		let triangle_shade = p.shade_create(ShadeProps {
			uniform_types: &[],
			vertex_format: &[Float32x2, Float32x2],
		});
		load_vertex_shader!(triangle_shade, p, "../triangle_shader/vert.spv");
		load_fragment_shader!(triangle_shade, p, "../triangle_shader/frag.spv");

		let blur_shade = p.shade_create_effect(ShadeEffectProps {
			uniform_types: &[tex_type, u_fs_type, u_fs_type, u_fs_type],
		});
		load_fragment_shader!(blur_shade, p, "../blur_shader/frag.spv");

		let tri_form = p.form_create(TRIANGLE, default());

		let tri_sketch = p.sketch_create(tri_form, triangle_shade, SketchProps { ..default() });

		let size = u_fs_type.create_vec2(p);
		let horiz = u_fs_type.const_vec2(p, vec2(1.0, 0.0));
		let vertical = u_fs_type.const_vec2(p, vec2(0.0, 1.0));

		let mut effects = vec![];

		let mut counter = BLUR_RADIUS / 5.0;
		while counter > 1.0 {
			let radius = u_fs_type.const_f32(p, counter);
			effects.push(p.effect_create(
				blur_shade,
				EffectProps {
					uniforms: bmap! {
						1 => radius,
						2 => size.uniform,
						3 => horiz
					},
					..default()
				},
			));
			effects.push(p.effect_create(
				blur_shade,
				EffectProps {
					uniforms: bmap! {
						1 => radius,
						2 => size.uniform,
						3 => vertical
					},
					..default()
				},
			));
			counter /= 2.0;
		}

		println!("effects: {:?}", effects.len());

		// let radius = u_fs_type.const_f32(p, BLUR_RADIUS);
		// effects.push(p.effect_create(
		// 	blur_shade,
		// 	EffectProps {
		// 		uniforms: bmap! {
		// 			1 => radius,
		// 			2 => size.uniform,
		// 			3 => horiz
		// 		},
		// 		..default()
		// 	},
		// ));
		// effects.push(p.effect_create(
		// 	blur_shade,
		// 	EffectProps {
		// 		uniforms: bmap! {
		// 			1 => radius,
		// 			2 => size.uniform,
		// 			3 => vertical
		// 		},
		// 		..default()
		// 	},
		// ));

		let canvas = p.layer_create(LayerProps {
			sketches: vec![tri_sketch],
			effects,
			clear_color: Some(wgpu::Color::BLUE),
			..default()
		});

		Self { canvas, size }
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.size.update(p, vec2(width as f32, height as f32));
	}

	fn update(&mut self, p: &mut Painter, _tpf: f32) {
		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		p.paint(self.canvas)?;
		p.show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create().config(AppConfig { show_fps: true }).start();
}
