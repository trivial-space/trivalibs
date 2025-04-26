use trivalibs::{
	math::coords::angles_to_cartesian,
	prelude::*,
	rendering::{
		mesh_geometry::{face_data, MeshBufferType, MeshGeometry},
		shapes::{cuboid::Cuboid, sphere::create_sphere_mesh},
		BufferedGeometry,
	},
};

#[apply(gpu_data)]
struct Vertex {
	pos: Vec3,
	color: Vec3,
}
impl Overridable for Vertex {
	fn override_with(&self, attribs: &Self) -> Self {
		Vertex {
			color: attribs.color,
			..*self
		}
	}
}
impl Position3D for Vertex {
	fn position(&self) -> Vec3 {
		self.pos
	}
}

fn vert(pos: Vec3) -> Vertex {
	Vertex {
		pos,
		color: Vec3::ZERO,
	}
}

fn color_vert(color: Vec3) -> Vertex {
	Vertex {
		pos: Vec3::ZERO,
		color,
	}
}

const VERTICAL_SEGMENTS: u32 = 25;
const HORIZONTAL_SEGMENTS: u32 = 50;

pub fn create_ball_geom() -> BufferedGeometry {
	let mut geom = create_sphere_mesh(
		VERTICAL_SEGMENTS,
		HORIZONTAL_SEGMENTS,
		|horiz_angle, vert_angle| vert(angles_to_cartesian(horiz_angle, vert_angle)),
	);

	for i in 0..geom.face_count() {
		let color = vec3(random(), random(), random());
		let face = geom.face_mut(i);
		face.data = Some(color_vert(color));
	}

	geom.to_buffered_geometry_by_type(MeshBufferType::VertexNormalFaceData)
}

pub fn create_box_geom() -> BufferedGeometry {
	let box_shape = Cuboid::unit_cube();
	let mut geom = MeshGeometry::new();

	let v = |pos, _| vert(pos);

	geom.add_face4_data(
		box_shape.front_face_f(v).to_ccw_verts(),
		face_data(color_vert(vec3(1.0, 0.0, 0.0))),
	);
	geom.add_face4_data(
		box_shape.back_face_f(v).to_ccw_verts(),
		face_data(color_vert(vec3(0.0, 1.0, 0.0))),
	);
	geom.add_face4_data(
		box_shape.left_face_f(v).to_ccw_verts(),
		face_data(color_vert(vec3(0.0, 0.0, 1.0))),
	);
	geom.add_face4_data(
		box_shape.right_face_f(v).to_ccw_verts(),
		face_data(color_vert(vec3(1.0, 1.0, 0.0))),
	);
	geom.add_face4_data(
		box_shape.top_face_f(v).to_ccw_verts(),
		face_data(color_vert(vec3(0.0, 1.0, 1.0))),
	);
	geom.add_face4_data(
		box_shape.bottom_face_f(v).to_ccw_verts(),
		face_data(color_vert(vec3(1.0, 0.0, 1.0))),
	);

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}
