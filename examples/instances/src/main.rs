use trivalibs::{
	bmap,
	painter::{
		create_canvas_app, load_fragment_shader, load_vertex_shader,
		shade::ShadeProps,
		sketch::{Sketch, SketchProps},
		uniform::UniformBuffer,
		wgpu::{self, VertexFormat},
		CanvasApp, Event, Painter, UniformType,
	},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
		transform::Transform,
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
}

const TRIANGLE_COUNT: usize = 1100;
impl Default for App {
	fn default() -> Self {
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

		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: Some(vec3(0.0, 0.0, 80.0)),
				..default()
			}),
			triangles,
		}
	}
}

struct ViewState {
	sketch: Sketch,
	model_mats: Vec<UniformBuffer<Mat4>>,
	vp_mat: UniformBuffer<Mat4>,
}

impl CanvasApp<ViewState, ()> for App {
	fn init(&self, p: &mut Painter) -> ViewState {
		let vert_u_type = p.uniform_type_buffered_vert();
		let frag_u_type = p.uniform_type_buffered_frag();

		let shade = p.shade_create(ShadeProps {
			vertex_format: vec![VertexFormat::Float32x3],
			uniform_types: &[&vert_u_type, &vert_u_type, &frag_u_type],
		});
		load_vertex_shader!(shade, p, "../shader/vertex.spv");
		load_fragment_shader!(shade, p, "../shader/fragment.spv");

		let form = p.form_create(VERTICES, default());

		let model_mats = (0..self.triangles.len())
			.map(|_| vert_u_type.create_mat4(p))
			.collect::<Vec<_>>();

		let cam = vert_u_type.create_mat4(p);

		let instances = model_mats
			.iter()
			.map(|model| {
				bmap! {
					1 => model.uniform,
					2 => frag_u_type.const_vec4(p, rand_vec4()),
				}
			})
			.collect();

		let sketch = p.sketch_create(
			form,
			shade,
			&SketchProps {
				uniforms: bmap! {
					0 => cam.uniform,
				},
				instances,
				cull_mode: None,
				blend_state: wgpu::BlendState::ALPHA_BLENDING,
				..default()
			},
		);

		ViewState {
			sketch,
			model_mats,
			vp_mat: cam,
		}
	}

	fn resize(&mut self, p: &mut Painter, v: &mut ViewState) {
		let size = p.canvas_size();
		self.cam
			.set_aspect_ratio(size.width as f32 / size.height as f32);

		v.vp_mat.update(p, self.cam.view_proj_mat());
	}

	fn update(&mut self, p: &mut Painter, v: &mut ViewState, tpf: f32) {
		for (tri, model) in self.triangles.iter_mut().zip(v.model_mats.iter_mut()) {
			tri.transform.rotate_y(tpf * tri.speed);

			model.update(p, tri.transform.model_mat());
		}
	}

	fn render(&self, p: &mut Painter, v: &ViewState) -> Result<(), wgpu::SurfaceError> {
		p.request_next_frame();
		p.draw(&v.sketch)
	}

	fn event(&mut self, _e: Event<()>, _p: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
