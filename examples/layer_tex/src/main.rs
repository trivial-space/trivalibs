use trivalibs::{
	bmap,
	painter::{
		create_canvas_app,
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		shade::ShadeProps,
		sketch::SketchProps,
		wgpu::{SurfaceError, VertexFormat::*},
		CanvasApp, Event, Painter, UniformType,
	},
	prelude::*,
	rendering::{camera::PerspectiveCamera, transform::Transform},
};

#[apply(gpu_data)]
struct Vertex {
	pos: Vec3,
	uv: Vec2,
}

const TRIANGLE: &[Vertex] = &[
	Vertex {
		pos: vec3(-1.0, -1.0, 0.0),
		uv: vec2(0.0, 0.0),
	},
	Vertex {
		pos: vec3(1.0, -1.0, 0.0),
		uv: vec2(1.0, 0.0),
	},
	Vertex {
		pos: vec3(0.0, 1.0, 0.0),
		uv: vec2(0.5, 1.0),
	},
];

const QUAD: &[Vertex] = &[
	Vertex {
		pos: vec3(-1.0, -1.0, 0.0),
		uv: vec2(0.0, 0.0),
	},
	Vertex {
		pos: vec3(1.0, -1.0, 0.0),
		uv: vec2(1.0, 0.0),
	},
	Vertex {
		pos: vec3(-1.0, 1.0, 0.0),
		uv: vec2(0.0, 1.0),
	},
	Vertex {
		pos: vec3(-1.0, 1.0, 0.0),
		uv: vec2(0.0, 1.0),
	},
	Vertex {
		pos: vec3(1.0, -1.0, 0.0),
		uv: vec2(1.0, 0.0),
	},
	Vertex {
		pos: vec3(1.0, 1.0, 0.0),
		uv: vec2(1.0, 1.0),
	},
];

struct ViewState {
	red_triangle: Layer,
	blue_quad: Layer,
	canvas: Layer,
}

#[derive(Default)]
struct App {
	cam: PerspectiveCamera,
	triangle_transform: Transform,
	quad_transform: Transform,
}

impl CanvasApp<ViewState, ()> for App {
	fn init(&self, p: &mut Painter) -> ViewState {
		let u_fs_type = p.uniform_type_buffered_frag();
		let u_vs_type = p.uniform_type_buffered_vert();
		let tex_type = p.uniform_type_tex_2d_frag();

		let color_shade = p.shade_create(ShadeProps {
			uniform_types: &[&u_vs_type, &u_fs_type],
			vertex_format: &[Float32x3, Float32x2],
		});
		load_vertex_shader!(color_shade, p, "../color_shader/vs_main.spv");
		load_fragment_shader!(color_shade, p, "../color_shader/fs_main.spv");

		let tex_shader = p.shade_create(ShadeProps {
			uniform_types: &[&u_vs_type, &tex_type],
			vertex_format: &[Float32x3, Float32x2],
		});
		load_vertex_shader!(tex_shader, p, "../tex_shader/vs_main.spv");
		load_fragment_shader!(tex_shader, p, "../tex_shader/fs_main.spv");

		let quad_form = p.form_create(QUAD, default());
		let triangle_form = p.form_create(TRIANGLE, default());

		let quad_mvp = u_vs_type.create_mat4(p);
		let triangle_mvp = u_vs_type.create_mat4(p);

		let quad_color = vec3(0.0, 0.0, 1.0);
		let triangle_color = vec3(1.0, 0.0, 0.0);

		let color_quad_sketch = p.sketch_create(
			quad_form,
			color_shade,
			&SketchProps {
				uniforms: bmap! {
					0 => quad_mvp.uniform,
					1 => u_fs_type.const_vec3(p, quad_color),
				},
				..default()
			},
		);

		let color_triangle_sketch = p.sketch_create(
			triangle_form,
			color_shade,
			&SketchProps {
				uniforms: bmap! {
					0 => triangle_mvp.uniform,
					1 => u_fs_type.const_vec3(p, triangle_color),
				},
				..default()
			},
		);

		let canvas = p.layer_create(&LayerProps {
			effects: vec![effect],
			..default()
		});

		ViewState { canvas, time, size }
	}

	fn resize(&mut self, p: &mut Painter, rs: &mut ViewState) {
		let size = p.canvas_size();
		rs.size.update(p, uvec2(size.width, size.height));
	}

	fn update(&mut self, p: &mut Painter, rs: &mut ViewState, tpf: f32) {
		self.time += tpf;
		rs.time.update(p, self.time);

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter, state: &ViewState) -> Result<(), SurfaceError> {
		p.paint(&state.canvas)?;
		p.show(&state.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
