use trivalibs::{
	math::coords::angles_to_cartesian,
	prelude::*,
	rendering::{
		BufferedGeometry,
		mesh_geometry::{MeshBufferType, MeshGeometry},
		shapes::{cuboid::Cuboid, quad::Quad3D, sphere::create_sphere_mesh},
	},
};

#[apply(gpu_data)]
struct Vertex {
	pos: Vec3,
	color: Vec3,
}
impl Position3D for Vertex {
	fn position(&self) -> Vec3 {
		self.pos
	}
}

fn vert_pos(pos: Vec3) -> Vertex {
	Vertex {
		pos,
		color: Vec3::ZERO,
	}
}

fn vert(pos: Vec3, color: Vec3) -> Vertex {
	Vertex { pos, color }
}

const VERTICAL_SEGMENTS: u32 = 25;
const HORIZONTAL_SEGMENTS: u32 = 50;

pub fn create_ball_geom() -> BufferedGeometry {
	let geom = create_sphere_mesh(
		VERTICAL_SEGMENTS,
		HORIZONTAL_SEGMENTS,
		|horiz_angle, vert_angle| vert_pos(angles_to_cartesian(horiz_angle, vert_angle)),
	);

	let mut geom = geom.map(|face| {
		let color = vec3(random(), random(), random());

		face.vertices().iter().map(|v| vert(v.pos, color)).collect()
	});

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithVertexNormals)
}

pub fn create_box_geom() -> BufferedGeometry {
	let box_shape = Cuboid::unit_cube();
	let mut geom = MeshGeometry::new();

	let add = |geom: &mut MeshGeometry<_>, quad: Quad3D<Vec3>, color: Vec3| {
		geom.add_face(
			&quad
				.to_ccw_verts()
				.iter()
				.map(|&pos| vert(pos, color))
				.collect::<Vec<_>>(),
		);
	};

	add(&mut geom, box_shape.front_face(), vec3(1.0, 0.0, 0.0));
	add(&mut geom, box_shape.back_face(), vec3(0.0, 1.0, 0.0));
	add(&mut geom, box_shape.left_face(), vec3(0.0, 0.0, 1.0));
	add(&mut geom, box_shape.right_face(), vec3(1.0, 1.0, 0.0));
	add(&mut geom, box_shape.top_face(), vec3(0.0, 1.0, 1.0));
	add(&mut geom, box_shape.bottom_face(), vec3(1.0, 0.0, 1.0));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithFaceNormals)
}
