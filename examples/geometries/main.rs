use trivalibs::{
	common_utils::camera_controls::BasicFirstPersonCameraController,
	map,
	painter::{prelude::*, utils::input_state::InputState},
	prelude::*,
	rendering::{
		BufferedGeometry,
		camera::{CamProps, PerspectiveCamera},
		mesh_geometry::{
			MeshBufferType, MeshGeometry, face_normal, face_props,
			utils::{Vert3dUv, vert_pos_uv},
		},
		shapes::{cuboid::Cuboid, quad::Quad3D},
	},
};

pub fn create_plane(width: f32, height: f32, normal: Vec3, center: Vec3) -> BufferedGeometry {
	let plane: Quad3D<Vert3dUv> =
		Quad3D::from_dimensions_center_f(width, height, normal, center, vert_pos_uv).into();

	let mut geom = MeshGeometry::new();
	geom.add_face_data(&plane.to_ccw_verts(), face_normal(plane.normal));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithVertexNormals)
}

pub fn create_box(center: Vec3, size: Vec3) -> BufferedGeometry {
	let bbox = Cuboid::box_at(center, size.x, size.y, size.z);

	let mut geom = MeshGeometry::new();

	let front = bbox.front_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.y)));
	geom.add_face_data(&front.to_ccw_verts(), face_props(front.normal, 0));

	let back = bbox.back_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.x, uvw.y)));
	geom.add_face_data(&back.to_ccw_verts(), face_props(back.normal, 1));

	let left = bbox.left_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.z, uvw.y)));
	geom.add_face_data(&left.to_ccw_verts(), face_props(left.normal, 2));

	let right = bbox.right_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.z, uvw.y)));
	geom.add_face_data(&right.to_ccw_verts(), face_props(right.normal, 3));

	let top = bbox.top_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, 1.0 - uvw.z)));
	geom.add_face_data(&top.to_ccw_verts(), face_props(top.normal, 4));

	let bottom = bbox.bottom_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.z)));
	geom.add_face_data(&bottom.to_ccw_verts(), face_props(bottom.normal, 5));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormals)
}

struct App {
	cam: PerspectiveCamera,
	vp_mat: BindingBuffer<Mat4>,
	canvas: Layer,

	input: InputState,
	cam_controller: BasicFirstPersonCameraController,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p
			.shade(&[Float32x3, Float32x3, Float32x2])
			.with_bindings(&[BINDING_BUFFER_VERT])
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

		let x_axis_form = p
			.form(&create_box(vec3(2.5, 0.5, 0.0), vec3(5.0, 0.5, 0.5)))
			.create();
		let y_axis_form = p
			.form(&create_box(vec3(0.0, 3.0, 0.0), vec3(0.5, 5.0, 0.5)))
			.create();
		let z_axis_form = p
			.form(&create_box(vec3(0.0, 0.5, 2.5), vec3(0.5, 0.5, 5.0)))
			.create();

		let ground_shape = p.shape(ground_form, shade).create();
		let wall_shape = p.shape(wall_form, shade).create();
		let roof_shape = p.shape(roof_form, shade).create();
		let x_axis_shape = p.shape(x_axis_form, shade).with_cull_mode(None).create();
		let y_axis_shape = p.shape(y_axis_form, shade).with_cull_mode(None).create();
		let z_axis_shape = p.shape(z_axis_form, shade).with_cull_mode(None).create();

		let vp_mat = p.bind_mat4();

		let canvas = p
			.layer()
			.with_shapes(vec![
				ground_shape,
				wall_shape,
				roof_shape,
				x_axis_shape,
				y_axis_shape,
				z_axis_shape,
			])
			.with_clear_color(wgpu::Color {
				r: 0.5,
				g: 0.6,
				b: 0.7,
				a: 1.0,
			})
			.with_bindings(map! {
				0 => vp_mat.binding(),
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

	fn render(&self, p: &mut Painter) {
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
			remember_window_dimensions: true,
			..default()
		})
		.start();
}
