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
		CanvasApp, Event, Painter,
	},
	utils::default,
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
	triangle: Layer,
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
			uniform_types: &[tex_type, u_fs_type],
		});
		load_fragment_shader!(blur_shade, p, "../blur_shader/frag.spv");

		let tri_form = p.form_create(TRIANGLE, default());

		let tri_sketch = p.sketch_create(tri_form, triangle_shade, SketchProps { ..default() });

		let triangle = p.layer_create(LayerProps {
			sketches: vec![tri_sketch],
			clear_color: Some(wgpu::Color::BLUE),
			..default()
		});

		let tex = triangle.get_uniform(p, p.sampler_default());
		let size = u_fs_type.create_vec2(p);

		let effect = p.effect_create(
			blur_shade,
			EffectProps {
				uniforms: bmap! {
					0 => tex.uniform,
					1 => size.uniform,
				},
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			effects: vec![effect],
			..default()
		});

		Self {
			canvas,
			triangle,
			size,
		}
	}

	fn resize(&mut self, p: &mut Painter) {
		let size = p.canvas_size();
		self.size
			.update(p, vec2(size.width as f32, size.height as f32));
	}

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		p.paint(&self.canvas)?;
		p.paint(&self.triangle)?;
		p.show(&self.canvas)
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}
	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create().start();
}
