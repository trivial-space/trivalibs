use trivalibs::{
	bmap,
	painter::{
		create_canvas_app,
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		shade::ShadeProps,
		sketch::SketchProps,
		uniform::UniformBuffer,
		wgpu::{self, SurfaceError, VertexFormat::*},
		CanvasApp, Event, Painter, UniformType,
	},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
		transform::Transform,
	},
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

const COLOR_TEX_SIZE: (u32, u32) = (800, 800);

struct ViewState {
	quad_mvp: UniformBuffer<Mat4>,
	triangle_mvp: UniformBuffer<Mat4>,
	color_triangle_layer: Layer,
	color_quad_layer: Layer,
	canvas: Layer,
}

struct App {
	color_cam: PerspectiveCamera,
	tex_cam: PerspectiveCamera,
	triangle_transform: Transform,
	quad_transform: Transform,
}

impl Default for App {
	fn default() -> Self {
		Self {
			color_cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: vec3(0.0, 0.0, 5.0).into(),
				..default()
			}),
			tex_cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: vec3(0.0, 0.0, 2.0).into(),
				..default()
			}),
			triangle_transform: Transform::default(),
			quad_transform: Transform::default(),
		}
	}
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

		let quad_color = u_fs_type.const_vec3(p, vec3(0.0, 0.0, 1.0));
		let triangle_color = u_fs_type.const_vec3(p, vec3(1.0, 0.0, 0.0));

		let color_quad_sketch = p.sketch_create(
			quad_form,
			color_shade,
			&SketchProps {
				cull_mode: None,
				uniforms: bmap! {
					0 => quad_mvp.uniform,
					1 => quad_color
				},
				..default()
			},
		);

		let color_triangle_sketch = p.sketch_create(
			triangle_form,
			color_shade,
			&SketchProps {
				cull_mode: None,
				uniforms: bmap! {
					0 => triangle_mvp.uniform,
					1 => triangle_color,
				},
				..default()
			},
		);

		let color_triangle_layer = p.layer_create(&LayerProps {
			sketches: vec![color_triangle_sketch],
			width: COLOR_TEX_SIZE.0,
			height: COLOR_TEX_SIZE.1,
			clear_color: Some(wgpu::Color {
				r: 1.0,
				g: 1.0,
				b: 0.0,
				a: 1.0,
			}),
			..default()
		});

		let color_quad_layer = p.layer_create(&LayerProps {
			sketches: vec![color_quad_sketch],
			width: COLOR_TEX_SIZE.0,
			height: COLOR_TEX_SIZE.1,
			clear_color: Some(wgpu::Color {
				r: 0.0,
				g: 1.0,
				b: 0.0,
				a: 1.0,
			}),
			..default()
		});

		let canvas = p.layer_create(&LayerProps {
			sketches: vec![],
			..default()
		});

		ViewState {
			quad_mvp,
			triangle_mvp,

			canvas,
			color_triangle_layer,
			color_quad_layer,
		}
	}

	fn resize(&mut self, p: &mut Painter, _v: &mut ViewState) {
		let size = p.canvas_size();
		self.tex_cam
			.set_aspect_ratio(size.width as f32 / size.height as f32);
	}

	fn update(&mut self, p: &mut Painter, v: &mut ViewState, tpf: f32) {
		self.triangle_transform.rotate_y(0.25 * tpf);
		self.quad_transform.rotate_y(0.3 * tpf);

		v.triangle_mvp.update(
			p,
			self.triangle_transform.model_view_proj_mat(&self.color_cam),
		);

		v.quad_mvp
			.update(p, self.quad_transform.model_view_proj_mat(&self.color_cam));

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter, v: &ViewState) -> Result<(), SurfaceError> {
		p.paint(&v.canvas)?;
		p.paint(&v.color_triangle_layer)?;
		p.paint(&v.color_quad_layer)?;
		p.show(&v.color_quad_layer)
	}

	fn event(&mut self, _e: Event<()>, _p: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
