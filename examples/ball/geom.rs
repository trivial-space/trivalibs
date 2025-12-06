use std::f32::consts::PI;
use trivalibs::{
	math::coords::angles_to_cartesian,
	prelude::*,
	rendering::{
		BufferedGeometry, mesh_geometry::MeshBufferType, shapes::sphere::create_sphere_mesh,
	},
};

#[apply(gpu_data)]
struct Vertex {
	pos: Vec3,
	uv: Vec2,
	color: Vec3,
}
impl Position3D for Vertex {
	fn position(&self) -> Vec3 {
		self.pos
	}
}

fn vert(pos: Vec3, uv: Vec2, color: Vec3) -> Vertex {
	Vertex { pos, uv, color }
}

fn pos_vert(pos: Vec3, uv: Vec2) -> Vertex {
	Vertex {
		pos,
		color: Vec3::ZERO,
		uv,
	}
}

const VERTICAL_SEGMENTS: u32 = 50;
const HORIZONTAL_SEGMENTS: u32 = 50;

pub fn create_ball_geom() -> BufferedGeometry {
	let geom = create_sphere_mesh(
		VERTICAL_SEGMENTS,
		HORIZONTAL_SEGMENTS,
		|horiz_angle, vert_angle| {
			let pos = angles_to_cartesian(horiz_angle, vert_angle);
			let uv = vec2(horiz_angle / (PI * 2.0), vert_angle / PI + 0.5);

			pos_vert(pos * 5.0, uv)
		},
	);

	let mut geom = geom.map(|face| {
		let vertices = face.vertices();

		let uv = vertices.iter().map(|v| v.uv).sum::<Vec2>() / vertices.len() as f32;

		let use_horiz_gradient = uv.x * HORIZONTAL_SEGMENTS as f32 % 2.0 < 1.0;
		let gradient = if use_horiz_gradient { uv.x } else { uv.y };
		let color = vec3(random(), random(), random()) * 0.2 + gradient * 0.8;

		vertices.iter().map(|v| vert(v.pos, v.uv, color)).collect()
	});

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceVerticesWithVertexNormal)
}

#[test]
fn ball1() {
	let res = create_ball_geom();
	// println!("{:?}", res);
	assert!(res.vertex_buffer.len() > 0);
	println!("buffer len: {}", res.vertex_buffer.len());
	println!(
		"index buffer len: {}",
		res.index_buffer.map(|i| i.len()).unwrap_or(0)
	);
}
