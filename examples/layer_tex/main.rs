use trivalibs::{
	map,
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
		let color_shade = p
			.shade(&[Float32x3, Float32x2])
			.with_uniforms(&[UNIFORM_BUFFER_VERT, UNIFORM_BUFFER_FRAG])
			.create();
		load_vertex_shader!(color_shade, p, "./shader/color_vs.spv");
		load_fragment_shader!(color_shade, p, "./shader/color_fs.spv");

		let tex_shader = p
			.shade(&[Float32x3, Float32x2])
			.with_uniforms(&[
				UNIFORM_BUFFER_VERT,
				UNIFORM_TEX2D_FRAG,
				UNIFORM_SAMPLER_FRAG,
			])
			.create();
		load_vertex_shader!(tex_shader, p, "./shader/texture_vs.spv");
		load_fragment_shader!(tex_shader, p, "./shader/texture_fs.spv");

		let quad_form = p.form(QUAD).create();
		let triangle_form = p.form(TRIANGLE).create();

		let color_quad_mvp = p.uniform_mat4();
		let color_triangle_mvp = p.uniform_mat4();

		let quad_color = p.uniform_const_vec3(vec3(0.0, 0.0, 1.0));
		let triangle_color = p.uniform_const_vec3(vec3(1.0, 0.0, 0.0));

		let color_quad_shape = p
			.shape(quad_form, color_shade)
			.with_uniforms(map! {
				0 => color_quad_mvp.uniform(),
				1 => quad_color,
			})
			.with_cull_mode(None)
			.create();

		let color_triangle_shape = p
			.shape(triangle_form, color_shade)
			.with_uniforms(map! {
				0 => color_triangle_mvp.uniform(),
				1 => triangle_color,
			})
			.with_cull_mode(None)
			.create();

		let color_triangle_layer = p
			.layer()
			.with_shape(color_triangle_shape)
			.with_size(COLOR_TEX_SIZE_BIG.0, COLOR_TEX_SIZE_BIG.1)
			.with_clear_color(YELLOW)
			.create();

		let sl = p.sampler_linear();
		let sn = p.sampler_nearest();

		let color_quad_layer = p
			.layer()
			.with_shape(color_quad_shape)
			.with_size(COLOR_TEX_SIZE_BIG.0, COLOR_TEX_SIZE_BIG.1)
			.with_clear_color(GREEN)
			.with_multisampling()
			.create();

		let tex_triangle_mvp = p.uniform_mat4();
		let tex_quad_mvp = p.uniform_mat4();

		let tex = color_triangle_layer.uniform(p);
		let tex_quad_shape = p
			.shape(quad_form, tex_shader)
			.with_cull_mode(None)
			.with_uniforms(map! {
				0 => tex_quad_mvp.uniform(),
				1 => tex,
				2 => sn.uniform(),
			})
			.create();

		let tex = color_quad_layer.uniform(p);
		let tex_triangle_shape = p
			.shape(triangle_form, tex_shader)
			.with_uniforms(map! {
				0 => tex_triangle_mvp.uniform(),
				1 => tex,
				2 => sl.uniform(),
			})
			.with_cull_mode(None)
			.create();

		let canvas = p
			.layer()
			.with_shapes(vec![tex_quad_shape, tex_triangle_shape])
			.with_clear_color(wgpu::Color::BLACK)
			.with_depth_test()
			.with_multisampling()
			.create();

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
		keep_window_dimensions: true,
		..default()
	});

	let handle = app.get_handle();

	std::thread::spawn(move || loop {
		std::thread::sleep(std::time::Duration::from_secs(2));
		let _ = handle.send_event(ResizeEvent);
	});

	app.start();
}
