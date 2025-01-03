use std::f32::consts::{PI, TAU};

use crate::{
	data::{Overridable, Position3D},
	rendering::mesh_geometry::MeshGeometry,
};

pub fn create_sphere_mesh<V: Overridable + Position3D + Clone>(
	vertical_segments: u32,
	horizontal_segments: u32,
	f: impl Fn(f32, f32) -> V,
) -> MeshGeometry<V> {
	let mut geom = MeshGeometry::new();

	let mut last_col = vec![];
	let mut first_col = vec![];

	let top_vert = f(0.0, PI / 2.0);
	let bottom_vert = f(0.0, -PI / 2.0);

	let add_cols = |geom: &mut MeshGeometry<V>, col1: Vec<V>, col2: Vec<V>| {
		for i in 0..(col1.len() - 1) {
			let v1 = col1[i].clone();
			let v2 = col1[i + 1].clone();
			let v3 = col2[i + 1].clone();
			let v4 = col2[i].clone();

			geom.add_face4(v1, v2, v4, v3);
		}

		geom.add_face3(
			col1[col1.len() - 1].clone(),
			col2[col2.len() - 1].clone(),
			top_vert.clone(),
		);
		geom.add_face3(col1[0].clone(), col2[0].clone(), bottom_vert.clone());
	};

	for j in 0..horizontal_segments {
		let mut col = vec![];

		let u = TAU * (j as f32 / horizontal_segments as f32);

		for i in 1..(vertical_segments - 1) {
			let v = PI * (i as f32 / vertical_segments as f32) - PI * 0.5;

			col.push(f(u, v));
		}

		if last_col.is_empty() {
			first_col = col.clone();
			last_col = col;
			continue;
		}

		add_cols(&mut geom, last_col, col.clone());

		last_col = col;
	}

	add_cols(&mut geom, last_col, first_col);

	geom
}
