use trivalibs::{
	math::transform::Transform,
	painter::prelude::*,
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
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
		uv: vec2(0.0, 1.0),
	},
	Vertex {
		pos: vec3(1.0, -1.0, 0.0),
		uv: vec2(1.0, 1.0),
	},
	Vertex {
		pos: vec3(0.0, 1.0, 0.0),
		uv: vec2(0.5, 0.0),
	},
];

const QUAD: &[Vertex] = &[
	Vertex {
		pos: vec3(-1.0, -1.0, 0.0),
		uv: vec2(0.0, 1.0),
	},
	Vertex {
		pos: vec3(1.0, -1.0, 0.0),
		uv: vec2(1.0, 1.0),
	},
	Vertex {
		pos: vec3(-1.0, 1.0, 0.0),
		uv: vec2(0.0, 0.0),
	},
	Vertex {
		pos: vec3(-1.0, 1.0, 0.0),
		uv: vec2(0.0, 0.0),
	},
	Vertex {
		pos: vec3(1.0, -1.0, 0.0),
		uv: vec2(1.0, 1.0),
	},
	Vertex {
		pos: vec3(1.0, 1.0, 0.0),
		uv: vec2(1.0, 0.0),
	},
];

const COLOR_TEX_SIZE_BIG: (u32, u32) = (800, 800);
const COLOR_TEX_SIZE_SMALL: (u32, u32) = (100, 100);

struct App {
	color_cam: PerspectiveCamera,
	tex_cam: PerspectiveCamera,
	triangle_transform: Transform,
	quad_transform: Transform,

	color_quad_mvp: UniformBuffer<Mat4>,
	color_triangle_mvp: UniformBuffer<Mat4>,
	tex_quad_mvp: UniformBuffer<Mat4>,
	tex_triangle_mvp: UniformBuffer<Mat4>,

	color_triangle_layer: Layer,
	color_quad_layer: Layer,
	canvas: Layer,

	is_big_tex: bool,
}

const YELLOW: wgpu::Color = wgpu::Color {
	r: 1.0,
	g: 1.0,
	b: 0.0,
	a: 1.0,
};

const GREEN: wgpu::Color = wgpu::Color {
	r: 0.0,
	g: 1.0,
	b: 0.0,
	a: 1.0,
};

#[derive(Debug, Clone, Copy)]
struct ResizeEvent;

impl CanvasApp<ResizeEvent> for App {
	fn init(p: &mut Painter) -> Self {
		let color_shade = p.shade_create(ShadeProps {
			attributes: &[Float32x3, Float32x2],
			uniforms: &[UNIFORM_BUFFER_VERT, UNIFORM_BUFFER_FRAG],
			layers: &[],
		});
		load_vertex_shader!(color_shade, p, "./color_shader/vs_main.spv");
		load_fragment_shader!(color_shade, p, "./color_shader/fs_main.spv");

		let tex_shader = p.shade_create(ShadeProps {
			attributes: &[Float32x3, Float32x2],
			uniforms: &[UNIFORM_BUFFER_VERT],
			layers: &[UNIFORM_LAYER_FRAG],
		});
		load_vertex_shader!(tex_shader, p, "./tex_shader/vs_main.spv");
		load_fragment_shader!(tex_shader, p, "./tex_shader/fs_main.spv");

		let quad_form = p.form_create(QUAD, default());
		let triangle_form = p.form_create(TRIANGLE, default());

		let color_quad_mvp = p.uniform_mat4();
		let color_triangle_mvp = p.uniform_mat4();

		let quad_color = p.uniform_const_vec3(vec3(0.0, 0.0, 1.0));
		let triangle_color = p.uniform_const_vec3(vec3(1.0, 0.0, 0.0));

		let color_quad_shape = p.shape_create(
			quad_form,
			color_shade,
			ShapeProps {
				cull_mode: None,
				uniforms: vec![(0, color_quad_mvp.uniform()), (1, quad_color)],
				..default()
			},
		);

		let color_triangle_shape = p.shape_create(
			triangle_form,
			color_shade,
			ShapeProps {
				cull_mode: None,
				uniforms: vec![(0, color_triangle_mvp.uniform()), (1, triangle_color)],
				..default()
			},
		);

		let color_triangle_layer = p.layer_create(LayerProps {
			shapes: vec![color_triangle_shape],
			width: COLOR_TEX_SIZE_BIG.0,
			height: COLOR_TEX_SIZE_BIG.1,
			clear_color: Some(YELLOW),
			multisampled: false,
			..default()
		});

		let color_quad_layer = p.layer_create(LayerProps {
			shapes: vec![color_quad_shape],
			sampler: p.sampler_linear(),
			width: COLOR_TEX_SIZE_BIG.0,
			height: COLOR_TEX_SIZE_BIG.1,
			clear_color: Some(GREEN),
			multisampled: true,
			..default()
		});

		let tex_triangle_mvp = p.uniform_mat4();
		let tex_quad_mvp = p.uniform_mat4();

		let tex_quad_shape = p.shape_create(
			quad_form,
			tex_shader,
			ShapeProps {
				cull_mode: None,
				uniforms: vec![(0, tex_quad_mvp.uniform())],
				layer_uniforms: vec![(0, color_triangle_layer)],
				..default()
			},
		);

		let tex_triangle_shape = p.shape_create(
			triangle_form,
			tex_shader,
			ShapeProps {
				cull_mode: None,
				uniforms: vec![(0, tex_triangle_mvp.uniform())],
				layer_uniforms: vec![(0, color_quad_layer)],
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			shapes: vec![tex_quad_shape, tex_triangle_shape],
			clear_color: Some(wgpu::Color::BLACK),
			depth_test: true,
			multisampled: true,
			..default()
		});

		Self {
			color_cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: vec3(0.0, 0.0, 5.0).into(),
				..default()
			}),
			tex_cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: vec3(0.0, 0.0, 5.0).into(),
				..default()
			}),
			triangle_transform: Transform::default(),
			quad_transform: Transform::default(),

			color_quad_mvp,
			color_triangle_mvp,
			tex_quad_mvp,
			tex_triangle_mvp,

			canvas,
			color_triangle_layer,
			color_quad_layer,

			is_big_tex: true,
		}
	}

	fn resize(&mut self, _p: &mut Painter, width: u32, height: u32) {
		self.tex_cam.set_aspect_ratio(width as f32 / height as f32);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.triangle_transform.rotate_y(0.25 * tpf);
		self.quad_transform.rotate_y(0.3 * tpf);

		self.color_triangle_mvp.update(
			p,
			self.triangle_transform.model_view_proj_mat(&self.color_cam),
		);
		self.tex_triangle_mvp.update(
			p,
			self.triangle_transform.model_view_proj_mat(&self.tex_cam),
		);

		self.color_quad_mvp
			.update(p, self.quad_transform.model_view_proj_mat(&self.color_cam));
		self.tex_quad_mvp
			.update(p, self.quad_transform.model_view_proj_mat(&self.tex_cam));

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint(self.color_triangle_layer)?;
		p.paint(self.color_quad_layer)?;
		p.paint(self.canvas)?;
		p.show(self.canvas)
	}

	fn event(&mut self, e: Event<ResizeEvent>, p: &mut Painter) {
		match e {
			Event::UserEvent(ResizeEvent) => {
				let size = if self.is_big_tex {
					COLOR_TEX_SIZE_SMALL
				} else {
					COLOR_TEX_SIZE_BIG
				};

				self.color_triangle_layer.resize(p, size.0, size.1);
				self.color_quad_layer.resize(p, size.0, size.1);

				if self.is_big_tex {
					self.color_triangle_layer.set_clear_color(p, Some(GREEN));
					self.color_quad_layer.set_clear_color(p, Some(YELLOW));
				} else {
					self.color_triangle_layer.set_clear_color(p, Some(YELLOW));
					self.color_quad_layer.set_clear_color(p, Some(GREEN));
				}

				self.is_big_tex = !self.is_big_tex;
			}
			_ => {}
		}
	}
}

pub fn main() {
	let app = App::create().config(AppConfig {
		show_fps: true,
		use_vsync: false,
	});

	let handle = app.get_handle();

	std::thread::spawn(move || loop {
		std::thread::sleep(std::time::Duration::from_secs(2));
		let _ = handle.send_event(ResizeEvent);
	});

	app.start();
}
