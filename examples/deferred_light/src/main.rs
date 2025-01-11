use geom::{create_ball_geom, create_box_geom};
use trivalibs::{
	map,
	math::transform::Transform,
	painter::{
		effect::EffectProps,
		layer::{Layer, LayerProps},
		load_fragment_shader, load_vertex_shader,
		shade::{ShadeEffectProps, ShadeProps},
		sketch::SketchProps,
		texture::SamplerProps,
		uniform::UniformBuffer,
		wgpu::{self, TextureFormat, VertexFormat::*},
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

	ball_model_mat: UniformBuffer<Mat4>,
	ball_rot: UniformBuffer<Quat>,
	box_model_mat: UniformBuffer<Mat4>,
	box_rot: UniformBuffer<Quat>,
	vp_mat: UniformBuffer<Mat4>,
	scene_layer: Layer,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let uniform_type = p.uniform_type_buffered_vert();

		let scene_shade = p.shade_create(ShadeProps {
			vertex_format: &[Float32x3, Float32x3, Float32x3],
			uniform_types: &[uniform_type, uniform_type, uniform_type],
		});
		load_vertex_shader!(scene_shade, p, "../scene_shader/vertex.spv");
		load_fragment_shader!(scene_shade, p, "../scene_shader/fragment.spv");

		let ball_form = p.form_create(&create_ball_geom(), default());

		let vp_mat = uniform_type.create_mat4(p);

		let ball_model_mat = uniform_type.create_mat4(p);
		let ball_rot = uniform_type.create_quat(p);

		let ball_sketch = p.sketch_create(
			ball_form,
			scene_shade,
			SketchProps {
				uniforms: map! {
					0 => ball_model_mat.uniform,
					2 => ball_rot.uniform,
				},
				..default()
			},
		);

		let box_form = p.form_create(&create_box_geom(), default());

		let box_model_mat = uniform_type.create_mat4(p);
		let box_rot = uniform_type.create_quat(p);

		let box_sketch = p.sketch_create(
			box_form,
			scene_shade,
			SketchProps {
				uniforms: map! {
					0 => box_model_mat.uniform,
					2 => box_rot.uniform,
				},
				..default()
			},
		);

		let scene_sampler = p.sampler_create(SamplerProps {
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			..default()
		});

		let scene_layer = p.layer_create(LayerProps {
			clear_color: Some(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			}),
			sketches: vec![ball_sketch, box_sketch],
			uniforms: map! {
				1 => vp_mat.uniform,
			},
			formats: vec![
				TextureFormat::Rgba8UnormSrgb,
				TextureFormat::Rgba16Float,
				TextureFormat::Rgba16Float,
			],
			depth_test: true,
			multisampled: true,
			sampler: scene_sampler,
			..default()
		});

		let tex_type = p.uniform_type_tex_2d_frag();

		let canvas_shade = p.shade_create_effect(ShadeEffectProps {
			uniform_types: &[tex_type, tex_type, tex_type],
		});
		load_fragment_shader!(canvas_shade, p, "../light_shader/fragment.spv");

		let color_target = scene_layer.get_target_uniform(p, 0);
		let normal_target = scene_layer.get_target_uniform(p, 1);
		let position_target = scene_layer.get_target_uniform(p, 2);

		let canvas_effect = p.effect_create(
			canvas_shade,
			EffectProps {
				uniforms: map! {
					0 => color_target,
					1 => normal_target,
					2 => position_target,
				},
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			effects: vec![canvas_effect],
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

			ball_model_mat,
			ball_rot,
			box_model_mat,
			box_rot,
			vp_mat,
			scene_layer,
			canvas,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
		self.vp_mat.update(p, self.cam.view_proj_mat());
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.ball_transform.rotate_y(tpf * 0.25);
		self.box_transform.rotate_y(tpf * 0.25);
		self.box_transform.rotate_x(tpf * 0.3);

		self.ball_model_mat
			.update(p, self.ball_transform.model_mat());
		self.ball_rot.update(p, self.ball_transform.rotation);
		self.box_model_mat.update(p, self.box_transform.model_mat());
		self.box_rot.update(p, self.box_transform.rotation);

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		p.paint(self.scene_layer)?;
		p.paint(self.canvas)?;
		p.show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create().config(AppConfig { show_fps: true }).start();
}
