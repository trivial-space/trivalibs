use geom::{create_ball_geom, create_box_geom};
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

mod geom;

struct App {
	cam: PerspectiveCamera,
	ball_transform: Transform,
	box_transform: Transform,

	u_ball_model_mat: UniformBuffer<Mat4>,
	u_ball_rot: UniformBuffer<Quat>,
	u_box_model_mat: UniformBuffer<Mat4>,
	u_box_rot: UniformBuffer<Quat>,
	u_vp_mat: UniformBuffer<Mat4>,
	scene_layer: Layer,
	canvas: Layer,
}

const LIGHTS_COUNT: usize = 10;

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let scene_shade = p
			.shade(&[Float32x3, Float32x3, Float32x3])
			.with_uniforms(&[
				UNIFORM_BUFFER_VERT,
				UNIFORM_BUFFER_VERT,
				UNIFORM_BUFFER_VERT,
			])
			.create();
		load_vertex_shader!(scene_shade, p, "./shader/scene_vs.spv");
		load_fragment_shader!(scene_shade, p, "./shader/scene_fs.spv");

		let ball_form = p.form(&create_ball_geom()).create();

		let u_vp_mat = p.uniform_mat4();

		let u_ball_model_mat = p.uniform_mat4();
		let u_ball_rot = p.uniform_quat();

		let ball_shape = p
			.shape(ball_form, scene_shade)
			.with_uniforms(map! {
				0 => u_ball_model_mat.uniform(),
				2 => u_ball_rot.uniform(),
			})
			.create();

		let box_form = p.form(&create_box_geom()).create();

		let u_box_model_mat = p.uniform_mat4();
		let u_box_rot = p.uniform_quat();

		let box_shape = p
			.shape(box_form, scene_shade)
			.with_uniforms(map! {
				0 => u_box_model_mat.uniform(),
				2 => u_box_rot.uniform(),
			})
			.create();

		let scene_layer = p
			.layer()
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_shapes(vec![ball_shape, box_shape])
			.with_formats(vec![Rgba8UnormSrgb, Rgba16Float, Rgba16Float])
			.with_uniforms(map! {
				1 => u_vp_mat.uniform(),
			})
			.with_multisampling()
			.with_depth_test()
			.create();

		let canvas_shade = p
			.shade_effect()
			.with_uniforms(&[
				UNIFORM_TEX2D_FRAG,
				UNIFORM_TEX2D_FRAG,
				UNIFORM_TEX2D_FRAG,
				UNIFORM_SAMPLER_FRAG,
				UNIFORM_BUFFER_FRAG,
				UNIFORM_BUFFER_FRAG,
				UNIFORM_BUFFER_FRAG,
			])
			.create();
		load_fragment_shader!(canvas_shade, p, "./shader/light_fs.spv");

		let color_target = scene_layer.uniform_at(p, 0);
		let normal_target = scene_layer.uniform_at(p, 1);
		let position_target = scene_layer.uniform_at(p, 2);

		let lights = (0..LIGHTS_COUNT)
			.map(|_| {
				let light_pos = rand_vec3_unit() * rand_range(10.0, 30.0);
				let light_pos_u = p.uniform_const_vec3(light_pos);

				let light_color_u = p.uniform_const_vec3(rand_vec3());

				InstanceUniforms {
					uniforms: map! {
						5 => light_pos_u,
						6 => light_color_u,
					},
					..default()
				}
			})
			.collect::<Vec<_>>();

		let cam_pos = vec3(0.0, 5.0, 0.0);
		let cam_pos_u = p.uniform_const_vec3(cam_pos);

		let s = p.sampler_nearest().uniform();

		let canvas_effect = p
			.effect(canvas_shade)
			.with_uniforms(map! {
				0 => color_target,
				1 => normal_target,
				2 => position_target,
				3 => s,
				4 => cam_pos_u,
			})
			.with_instances(lights)
			.with_blend_state(wgpu::BlendState {
				color: wgpu::BlendComponent {
					src_factor: wgpu::BlendFactor::One,
					dst_factor: wgpu::BlendFactor::One,
					operation: wgpu::BlendOperation::Add,
				},
				alpha: wgpu::BlendComponent {
					src_factor: wgpu::BlendFactor::One,
					dst_factor: wgpu::BlendFactor::One,
					operation: wgpu::BlendOperation::Add,
				},
			})
			.create();

		let canvas = p
			.layer()
			.with_effect(canvas_effect)
			.with_clear_color(wgpu::Color {
				r: 0.0,
				g: 0.0,
				b: 0.0,
				a: 1.0,
			})
			.create();

		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.65),
				translation: Some(cam_pos),
				rot_vertical: Some(-0.26),
				..default()
			}),
			ball_transform: Transform::from_translation(vec3(5.0, 0.0, -20.0))
				.with_scale(Vec3::ONE * 5.0),
			box_transform: Transform::from_translation(vec3(-5.0, 0.0, -20.0))
				.with_scale(Vec3::ONE * 7.5),

			u_ball_model_mat,
			u_ball_rot,
			u_box_model_mat,
			u_box_rot,
			u_vp_mat,
			scene_layer,
			canvas,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
		self.u_vp_mat.update(p, self.cam.view_proj_mat());
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.ball_transform.rotate_y(tpf * 0.25);
		self.box_transform.rotate_y(tpf * 0.25);
		self.box_transform.rotate_x(tpf * 0.3);

		self.u_ball_model_mat
			.update(p, self.ball_transform.model_mat());
		self.u_ball_rot.update(p, self.ball_transform.rotation);
		self.u_box_model_mat
			.update(p, self.box_transform.model_mat());
		self.u_box_rot.update(p, self.box_transform.rotation);

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint(self.scene_layer)?;
		p.paint(self.canvas)?;
		p.show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: false,
			keep_window_dimensions: true,
			..default()
		})
		.start();
}
