use std::f32::consts::PI;
use trivalibs::{
	prelude::*,
	rendering::{
		mesh_geometry::MeshBufferType, shapes::sphere::create_sphere_mesh, BufferedGeometry,
	},
};

#[apply(gpu_data)]
struct Vertex {
	pos: Vec3,
	uv: Vec2,
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

fn pos_vert(pos: Vec3, uv: Vec2) -> Vertex {
	Vertex {
		pos,
		color: Vec3::ZERO,
		uv,
	}
}

fn color_vert(color: Vec3) -> Vertex {
	Vertex {
		pos: Vec3::ZERO,
		color,
		uv: Vec2::ZERO,
	}
}

pub fn create_ball_geom() -> BufferedGeometry {
	let mut geom = create_sphere_mesh(20, 20, |horiz_angle, vert_angle| {
		let x = vert_angle.cos() * horiz_angle.cos();
		let y = vert_angle.cos() * horiz_angle.sin();
		let z = vert_angle.sin();
		pos_vert(
			vec3(x, y, z) * 5.0,
			vec2(horiz_angle / (PI * 2.0), vert_angle / PI + 0.5),
		)
	});

	for i in 0..geom.face_count() {
		let face = geom.face_mut(i);
		face.data = Some(color_vert(vec3(random(), random(), random())));
	}

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}

#[test]
fn test_ball1() {
	let res = create_ball_geom();
	// println!("{:?}", res);
	assert!(res.vertex_buffer.len() > 0);
	println!("buffer len: {}", res.vertex_buffer.len());
	println!(
		"index buffer len: {}",
		res.index_buffer.map(|i| i.len()).unwrap_or(0)
	);
}
