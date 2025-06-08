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

const VERTICES: &[Vec3] = &[vec3(0.0, 5.0, 0.0), vec3(-2.5, 0., 0.0), vec3(2.5, 0., 0.0)];

struct Triangle {
	transform: Transform,
	speed: f32,
}

struct App {
	cam: PerspectiveCamera,
	triangles: Vec<Triangle>,

	canvas: Layer,
	model_mats: Vec<BindingBuffer<Mat4>>,
	vp_mat: BindingBuffer<Mat4>,
}

const TRIANGLE_COUNT: usize = 1100;

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let mut triangles = Vec::with_capacity(TRIANGLE_COUNT);

		for _ in 0..TRIANGLE_COUNT {
			let scale = rand_vec3_range(1., 2.0);
			let mut t = Transform::from_translation(rand_vec3_range(-30.0, 30.0)).with_scale(scale);
			t.look_at(rand_vec3_range(-30.0, 30.0), Vec3::Y);
			triangles.push(Triangle {
				transform: t,
				speed: rand_range(0.1, 1.0) * rand_sign(),
			});
		}

		triangles.sort_by(|a, b| {
			a.transform
				.translation
				.z
				.partial_cmp(&b.transform.translation.z)
				.unwrap()
		});

		let shade = p
			.shade(&[Float32x3])
			.with_bindings(&[
				BINDING_BUFFER_VERT,
				BINDING_BUFFER_VERT,
				BINDING_BUFFER_FRAG,
			])
			.create();
		load_vertex_shader!(shade, p, "./shader/vertex.spv");
		load_fragment_shader!(shade, p, "./shader/fragment.spv");

		let form = p.form(VERTICES).create();

		let model_mats = (0..triangles.len())
			.map(|_| p.bind_mat4())
			.collect::<Vec<_>>();

		let cam = p.bind_mat4();

		let instances = model_mats
			.iter()
			.map(|model| InstanceBinding {
				bindings: map! {
					1 => model.binding(),
					2 => p.bind_const_vec4(rand_vec4())
				},
				..default()
			})
			.collect();

		let shape = p
			.shape(form, shade)
			.with_bindings(map! {
				0 => cam.binding()
			})
			.with_instances(instances)
			.with_cull_mode(None)
			.with_blend_state(wgpu::BlendState::ALPHA_BLENDING)
			.create();

		let canvas = p
			.layer()
			.with_shape(shape)
			.with_clear_color(wgpu::Color::BLACK)
			.with_multisampling()
			.create();

		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: Some(vec3(0.0, 0.0, 80.0)),
				..default()
			}),
			triangles,

			canvas,
			model_mats,
			vp_mat: cam,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);

		self.vp_mat.update(p, self.cam.view_proj_mat());
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		for (tri, model) in self.triangles.iter_mut().zip(self.model_mats.iter_mut()) {
			tri.transform.rotate_y(tpf * tri.speed);

			model.update(p, tri.transform.model_mat());
		}
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.request_next_frame();
		p.paint_and_show(self.canvas)
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
