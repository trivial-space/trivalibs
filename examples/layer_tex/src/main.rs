use trivalibs::{
	bmap,
	painter::{
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		shade::ShadeProps,
		sketch::SketchProps,
		texture::SamplerProps,
		uniform::UniformBuffer,
		wgpu::{self, SurfaceError, VertexFormat::*},
		CanvasApp, Event, Painter,
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
		let u_fs_type = p.uniform_type_buffered_frag();
		let u_vs_type = p.uniform_type_buffered_vert();
		let tex_type = p.uniform_type_tex_2d_frag();

		let color_shade = p.shade_create(ShadeProps {
			uniform_types: &[u_vs_type, u_fs_type],
			vertex_format: &[Float32x3, Float32x2],
		});
		load_vertex_shader!(color_shade, p, "../color_shader/vs_main.spv");
		load_fragment_shader!(color_shade, p, "../color_shader/fs_main.spv");

		let tex_shader = p.shade_create(ShadeProps {
			uniform_types: &[u_vs_type, tex_type],
			vertex_format: &[Float32x3, Float32x2],
		});
		load_vertex_shader!(tex_shader, p, "../tex_shader/vs_main.spv");
		load_fragment_shader!(tex_shader, p, "../tex_shader/fs_main.spv");

		let quad_form = p.form_create(QUAD, default());
		let triangle_form = p.form_create(TRIANGLE, default());

		let color_quad_mvp = u_vs_type.create_mat4(p);
		let color_triangle_mvp = u_vs_type.create_mat4(p);

		let quad_color = u_fs_type.const_vec3(p, vec3(0.0, 0.0, 1.0));
		let triangle_color = u_fs_type.const_vec3(p, vec3(1.0, 0.0, 0.0));

		let color_quad_sketch = p.sketch_create(
			quad_form,
			color_shade,
			&SketchProps {
				cull_mode: None,
				uniforms: bmap! {
					0 => color_quad_mvp.uniform,
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
					0 => color_triangle_mvp.uniform,
					1 => triangle_color,
				},
				..default()
			},
		);

		let color_triangle_layer = p.layer_create(&LayerProps {
			sketches: vec![color_triangle_sketch],
			width: COLOR_TEX_SIZE_BIG.0,
			height: COLOR_TEX_SIZE_BIG.1,
			clear_color: Some(YELLOW),
			multisampled: false,
			..default()
		});

		let color_quad_layer = p.layer_create(&LayerProps {
			sketches: vec![color_quad_sketch],
			width: COLOR_TEX_SIZE_BIG.0,
			height: COLOR_TEX_SIZE_BIG.1,
			clear_color: Some(GREEN),
			multisampled: true,
			..default()
		});

		let sampler = p.sampler_create(&SamplerProps {
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			..default()
		});
		let tri_tex = color_triangle_layer.get_uniform(p, sampler);
		let quad_tex = color_quad_layer.get_uniform(p, p.sampler_default());
		let tex_triangle_mvp = u_vs_type.create_mat4(p);
		let tex_quad_mvp = u_vs_type.create_mat4(p);

		let tex_quad_sketch = p.sketch_create(
			quad_form,
			tex_shader,
			&SketchProps {
				cull_mode: None,
				uniforms: bmap! {
					0 => tex_quad_mvp.uniform,
					1 => tri_tex.uniform,
				},
				..default()
			},
		);

		let tex_triangle_sketch = p.sketch_create(
			triangle_form,
			tex_shader,
			&SketchProps {
				cull_mode: None,
				uniforms: bmap! {
					0 => tex_triangle_mvp.uniform,
					1 => quad_tex.uniform,
				},
				..default()
			},
		);

		let canvas = p.layer_create(&LayerProps {
			sketches: vec![tex_quad_sketch, tex_triangle_sketch],
			depth_test: true,
			clear_color: Some(wgpu::Color::BLACK),
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

	fn resize(&mut self, p: &mut Painter) {
		let size = p.canvas_size();
		self.tex_cam
			.set_aspect_ratio(size.width as f32 / size.height as f32);
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
		p.paint(&self.color_triangle_layer)?;
		p.paint(&self.color_quad_layer)?;
		p.paint(&self.canvas)?;
		p.show(&self.canvas)
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
	let app = App::create();
	let handle = app.get_handle();

	std::thread::spawn(move || loop {
		std::thread::sleep(std::time::Duration::from_secs(2));
		let _ = handle.send_event(ResizeEvent);
	});

	app.start();
}
