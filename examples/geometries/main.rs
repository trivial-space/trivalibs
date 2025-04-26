use trivalibs::{
	common_utils::camera_controls::BasicFirstPersonCameraController,
	map,
	painter::{prelude::*, utils::input_state::InputState},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		mesh_geometry::{
			face_normal,
			utils::{vert_pos_uv, Vert3dUv},
			MeshBufferType, MeshGeometry,
		},
		shapes::quad::Quad3D,
		BufferedGeometry,
	},
};

pub fn create_plane(width: f32, height: f32, normal: Vec3, center: Vec3) -> BufferedGeometry {
	let plane: Quad3D<Vert3dUv> =
		Quad3D::from_dimensions_center_f(width, height, normal, center, vert_pos_uv).into();

	let mut geom = MeshGeometry::new();
	geom.add_face4_data(plane.to_ccw_verts(), face_normal(plane.normal));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}

struct App {
	cam: PerspectiveCamera,
	vp_mat: UniformBuffer<Mat4>,
	canvas: Layer,

	input: InputState,
	cam_controller: BasicFirstPersonCameraController,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p
			.shade(&[Float32x3, Float32x3, Float32x2])
			.with_uniforms(&[UNIFORM_BUFFER_VERT])
			.create();
		load_vertex_shader!(shade, p, "./shader/ground_vert.spv");
		load_fragment_shader!(shade, p, "./shader/ground_frag.spv");

		let cam = PerspectiveCamera::create(CamProps {
			fov: Some(0.6),
			translation: Some(vec3(0.0, 3.0, 15.0)),
			..default()
		});

		let ground_form = p
			.form(&create_plane(100.0, 100.0, Vec3::Y, Vec3::ZERO))
			.create();
		let roof_form = p
			.form(&create_plane(100.0, 100.0, -Vec3::Y, vec3(0.0, 10.0, 0.0)))
			.create();
		let wall_form = p
			.form(&create_plane(20.5, 5.0, Vec3::Z, vec3(15.0, 3.0, 0.0)))
			.create();

		let ground_shape = p.shape(ground_form, shade).create();
		let wall_shape = p.shape(wall_form, shade).create();
		let roof_shape = p.shape(roof_form, shade).create();

		let vp_mat = p.uniform_mat4();

		let canvas = p
			.layer()
			.with_shapes(vec![ground_shape, wall_shape, roof_shape])
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_uniforms(map! {
				0 => vp_mat.uniform(),
			})
			.with_multisampling()
			.with_depth_test()
			.create();

		Self {
			cam,
			canvas,
			vp_mat,
			input: default(),
			cam_controller: BasicFirstPersonCameraController::new(1.0, 3.0),
		}
	}

	fn resize(&mut self, _p: &mut Painter, width: u32, height: u32) {
		self.cam.set_aspect_ratio(width as f32 / height as f32);
		self.cam_controller.set_screen_size(width, height);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.cam_controller
			.update_camera(&mut self.cam, &self.input, tpf);

		self.vp_mat.update(p, self.cam.view_proj_mat());

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> std::result::Result<(), wgpu::SurfaceError> {
		p.paint_and_show(self.canvas)
	}

	fn event(&mut self, e: Event<()>, _p: &mut Painter) {
		self.input.process_event(e);
	}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: true,
			keep_window_dimensions: true,
			..default()
		})
		.start();
}
