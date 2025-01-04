use trivalibs::{
	math::coords::angles_to_cartesian,
	prelude::*,
	rendering::{
		mesh_geometry::{face_data, MeshBufferType, MeshGeometry},
		shapes::{r#box::BoxGeom, sphere::create_sphere_mesh},
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
	let box_shape = BoxGeom::unit_cube();
	let mut geom = MeshGeometry::new();
	let front = box_shape.front_face();
	let back = box_shape.back_face();
	let left = box_shape.left_face();
	let right = box_shape.right_face();
	let top = box_shape.top_face();
	let bottom = box_shape.bottom_face();

	geom.add_face4_data(
		vert(front[0]),
		vert(front[1]),
		vert(front[2]),
		vert(front[3]),
		face_data(color_vert(vec3(1.0, 0.0, 0.0))),
	);
	geom.add_face4_data(
		vert(back[0]),
		vert(back[1]),
		vert(back[2]),
		vert(back[3]),
		face_data(color_vert(vec3(0.0, 1.0, 0.0))),
	);
	geom.add_face4_data(
		vert(left[0]),
		vert(left[1]),
		vert(left[2]),
		vert(left[3]),
		face_data(color_vert(vec3(0.0, 0.0, 1.0))),
	);
	geom.add_face4_data(
		vert(right[0]),
		vert(right[1]),
		vert(right[2]),
		vert(right[3]),
		face_data(color_vert(vec3(1.0, 1.0, 0.0))),
	);
	geom.add_face4_data(
		vert(top[0]),
		vert(top[1]),
		vert(top[2]),
		vert(top[3]),
		face_data(color_vert(vec3(0.0, 1.0, 1.0))),
	);
	geom.add_face4_data(
		vert(bottom[0]),
		vert(bottom[1]),
		vert(bottom[2]),
		vert(bottom[3]),
		face_data(color_vert(vec3(1.0, 0.0, 1.0))),
	);

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}
