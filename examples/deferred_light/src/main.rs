use geom::{create_ball_geom, create_box_geom};
use trivalibs::{
	map,
	math::transform::Transform,
	painter::{
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		shade::ShadeProps,
		sketch::SketchProps,
		uniform::{Mat3U, UniformBuffer},
		wgpu::{self, VertexFormat::*},
		AppConfig, CanvasApp, Event, Painter,
	},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
	},
};

mod geom;

struct App {
	cam: PerspectiveCamera,
	ball_transform: Transform,
	box_transform: Transform,

	ball_mvp: UniformBuffer<Mat4>,
	ball_norm: UniformBuffer<Mat3U>,
	box_mvp: UniformBuffer<Mat4>,
	box_norm: UniformBuffer<Mat3U>,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let uniform_type = p.uniform_type_buffered_vert();

		let scene_shade = p.shade_create(ShadeProps {
			vertex_format: &[Float32x3, Float32x3, Float32x3],
			uniform_types: &[uniform_type, uniform_type],
		});
		load_vertex_shader!(scene_shade, p, "../scene_shader/vertex.spv");
		load_fragment_shader!(scene_shade, p, "../scene_shader/fragment.spv");

		let ball_form = p.form_create(&create_ball_geom(), default());

		let ball_mvp = uniform_type.create_mat4(p);
		let ball_norm = uniform_type.create_mat3(p);

		let ball_sketch = p.sketch_create(
			ball_form,
			scene_shade,
			SketchProps {
				uniforms: map! {
					0 => ball_mvp.uniform,
					1 => ball_norm.uniform,
				},
				..default()
			},
		);

		let box_form = p.form_create(&create_box_geom(), default());

		let box_mvp = uniform_type.create_mat4(p);
		let box_norm = uniform_type.create_mat3(p);

		let box_sketch = p.sketch_create(
			box_form,
			scene_shade,
			SketchProps {
				uniforms: map! {
					0 => box_mvp.uniform,
					1 => box_norm.uniform,
				},
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			clear_color: Some(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			}),
			sketches: vec![ball_sketch, box_sketch],
			depth_test: true,
			multisampled: true,
			..default()
		});

		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.65),
				translation: Some(vec3(0.0, 5.0, 0.0)),
				rot_vertical: Some(-0.26),
				..default()
			}),
			ball_transform: Transform::from_translation(vec3(5.0, 0.0, -20.0))
				.with_scale(Vec3::ONE * 5.0),
			box_transform: Transform::from_translation(vec3(-5.0, 0.0, -20.0))
				.with_scale(Vec3::ONE * 7.5),

			ball_mvp,
			ball_norm,
			box_mvp,
			box_norm,
			canvas,
		}
	}

	fn resize(&mut self, _p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.ball_transform.rotate_y(tpf * 0.25);
		self.box_transform.rotate_y(tpf * 0.25);
		self.box_transform.rotate_x(tpf * 0.3);

		self.ball_mvp
			.update(p, self.ball_transform.model_view_proj_mat(&self.cam));
		self.ball_norm
			.update_mat3(p, self.ball_transform.view_normal_mat(&self.cam));
		self.box_mvp
			.update(p, self.box_transform.model_view_proj_mat(&self.cam));
		self.box_norm
			.update_mat3(p, self.box_transform.view_normal_mat(&self.cam));

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
