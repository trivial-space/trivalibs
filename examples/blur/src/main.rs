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
			uniform_types: &[tex_type, u_fs_type, u_fs_type],
		});
		load_fragment_shader!(blur_shade, p, "../blur_shader/frag.spv");

		let tri_form = p.form_create(TRIANGLE, default());

		let tri_sketch = p.sketch_create(tri_form, triangle_shade, SketchProps { ..default() });

		let size = u_fs_type.create_vec2(p);

		let horiz = u_fs_type.const_vec2(p, vec2(1.0, 0.0));
		let vertical = u_fs_type.const_vec2(p, vec2(0.0, 1.0));

		let blur_horiz = p.effect_create(
			blur_shade,
			EffectProps {
				uniforms: bmap! {
					1 => size.uniform,
					2 => horiz
				},
				..default()
			},
		);

		let blur_vert = p.effect_create(
			blur_shade,
			EffectProps {
				uniforms: bmap! {
					1 => size.uniform,
					2 => vertical
				},
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			sketches: vec![tri_sketch],
			effects: vec![blur_horiz, blur_vert],
			clear_color: Some(wgpu::Color::BLUE),
			..default()
		});

		Self { canvas, size }
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.size.update(p, vec2(width as f32, height as f32));
	}

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		p.paint(self.canvas)?;
		p.show(self.canvas)
	}

	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}
	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create().start();
}
