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

	let add_cols =
		|geom: &mut MeshGeometry<V>, col1: Vec<V>, col2: Vec<V>, first_vert: V, last_vert: V| {
			for i in 0..(col1.len() - 1) {
				let v1 = col2[i].clone();
				let v2 = col2[i + 1].clone();
				let v3 = col1[i].clone();
				let v4 = col1[i + 1].clone();

				geom.add_face4(v1, v2, v4, v3);
			}

			geom.add_face3(first_vert, col2[0].clone(), col1[0].clone());

			geom.add_face3(
				col2[col2.len() - 1].clone(),
				last_vert,
				col1[col1.len() - 1].clone(),
			);
		};

	for j in 0..=horizontal_segments {
		let mut col = vec![];

		let u = TAU * (j as f32 / horizontal_segments as f32);

		let first_vert = f(u, -PI * 0.5);
		let last_vert = f(u, PI * 0.5);

		for i in 1..(vertical_segments - 1) {
			let v = PI * (i as f32 / vertical_segments as f32) - PI * 0.5;

			col.push(f(u, v));
		}

		if last_col.is_empty() {
			last_col = col;
			continue;
		}

		add_cols(&mut geom, last_col, col.clone(), first_vert, last_vert);

		last_col = col;
	}

	geom
}
