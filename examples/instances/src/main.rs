use trivalibs::{
	map,
	math::transform::Transform,
	painter::{
		load_fragment_shader, load_vertex_shader,
		shade::{ShadeData, ShadeProps},
		shape::{Shape, ShapeProps},
		uniform::{UniformBuffer, UniformType},
		wgpu::{self, VertexFormat},
		AppConfig, CanvasApp, Event, Painter,
	},
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

	sketch: Shape,
	model_mats: Vec<UniformBuffer<Mat4>>,
	vp_mat: UniformBuffer<Mat4>,
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

		let u_type = p.uniform_type_buffered();

		let shade = p.shade_create(ShadeProps {
			attributes: vec![VertexFormat::Float32x3],
			uniforms: &[u_type.vert(), u_type.vert(), u_type.frag()],
			layers: &[],
		});
		load_vertex_shader!(shade, p, "../shader/vertex.spv");
		load_fragment_shader!(shade, p, "../shader/fragment.spv");

		let form = p.form_create(VERTICES, default());

		let model_mats = (0..triangles.len())
			.map(|_| u_type.create_mat4(p))
			.collect::<Vec<_>>();

		let cam = u_type.create_mat4(p);

		let instances = model_mats
			.iter()
			.map(|model| ShadeData {
				uniforms: map! {
					1 => model.uniform(),
					2 => u_type.const_vec4(p, rand_vec4())
				},
				layers: Vec::with_capacity(0),
			})
			.collect();

		let shape = p.shape_create(
			form,
			shade,
			ShapeProps {
				data: Some(ShadeData {
					uniforms: vec![(0, cam.uniform())],
					layers: Vec::with_capacity(0),
				}),
				instances,
				cull_mode: None,
				blend_state: wgpu::BlendState::ALPHA_BLENDING,
				..default()
			},
		);

		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: Some(vec3(0.0, 0.0, 80.0)),
				..default()
			}),
			triangles,

			sketch: shape,
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

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		p.request_next_frame();
		p.draw(self.sketch)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create().config(AppConfig { show_fps: true }).start();
}
