use std::f32::consts::PI;
use trivalibs::{
	data::grid::{make_grid_with_coord_ops, CIRCLE_COLS_COORD_OPS},
	prelude::*,
	rendering::mesh_geometry::{face_data, MeshBufferType, MeshGeometry},
	rendering::RenderableBuffer,
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

fn vert(pos: Vec3, uv: Vec2, color: Vec3) -> Vertex {
	Vertex { pos, color, uv }
}

pub fn create_ball_geom() -> RenderableBuffer {
	let mut grid = make_grid_with_coord_ops(CIRCLE_COLS_COORD_OPS);
	let mut col1 = vec![];
	let mut y = -5.0;
	while y <= 5.0 {
		let x = f32::sqrt(25.0 - y * y);
		col1.push((vec3(x, y, 0.0), vec2(0.0, y / 10.0 + 0.5)));
		y += 0.5;
	}
	grid.add_col(col1.clone());

	let stops = 20;
	let angle = (PI * 2.0) / stops as f32;
	for i in 1..stops {
		let q = Quat::from_rotation_y(angle * i as f32);
		let col = col1
			.iter()
			.map(|(pos, uv)| {
				let v = q.mul_vec3(*pos);
				(vec3(v.x, pos.y, v.z), vec2(i as f32 / stops as f32, uv.y))
			})
			.collect();
		grid.add_col(col)
	}

	let mut geom = MeshGeometry::new();
	for quad in grid.to_ccw_quads() {
		let r: f32 = random();
		let g: f32 = random();
		let b: f32 = random();

		let color = vec3(r, g, b);

		let v0 = vert(quad[0].0, quad[0].1, color);
		let v1 = vert(quad[1].0, quad[1].1, color);
		let v2 = vert(quad[2].0, quad[2].1, color);
		let v3 = vert(quad[3].0, quad[3].1, color);

		if v0.pos.y == -5.0 {
			// v0 == v1
			geom.add_face3_data(v0, v2, v3, face_data(v0));
		} else if v2.pos.y == 5.0 {
			// v2 == v3
			geom.add_face3_data(v0, v1, v2, face_data(v0));
		} else {
			geom.add_face4_data(v0, v1, v2, v3, face_data(v0));
		}
	}

	geom.to_renderable_buffer_by_type(MeshBufferType::FaceNormals)
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
