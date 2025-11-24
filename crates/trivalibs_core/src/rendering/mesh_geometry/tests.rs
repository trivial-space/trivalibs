use crate::rendering::mesh_geometry::face_props;

use super::{DEFAULT_MESH_SECTION, MeshGeometry, PositionFaceRef, face_section};
use bytemuck::{Pod, Zeroable};
use glam::{Vec3, vec3};

use super::Position3D;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
struct Vert {
	pos: Vec3,
}
impl Position3D for Vert {
	fn position(&self) -> Vec3 {
		self.pos
	}
}
fn vert(x: f32, y: f32, z: f32) -> Vert {
	Vert { pos: vec3(x, y, z) }
}

fn position_face_refs(geom: &MeshGeometry<Vert>, v: Vert) -> Vec<PositionFaceRef> {
	geom.get_position_faces(v.position())
		.map(|faces| faces.to_vec())
		.unwrap_or_default()
}

fn position_face_indices(geom: &MeshGeometry<Vert>, v: Vert) -> Vec<usize> {
	let mut indices = position_face_refs(geom, v)
		.into_iter()
		.map(|r| r.face_index)
		.collect::<Vec<_>>();
	indices.sort();
	indices
}

fn position_face_vertices(geom: &MeshGeometry<Vert>, v: Vert) -> Vec<Vert> {
	let mut refs = position_face_refs(geom, v)
		.into_iter()
		.map(|face_ref| {
			let vertex = *geom.face(face_ref.face_index).vertex(face_ref.vertex_slot);
			(face_ref, vertex)
		})
		.collect::<Vec<_>>();
	refs.sort_by_key(|(face_ref, _)| (face_ref.face_index, face_ref.vertex_slot));
	refs.iter().map(|(_, v)| *v).collect()
}

#[test]
fn generate_geometry() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3]);

	assert_eq!(geom.face_count(), 1);
	assert_eq!(geom.position_count(), 3);

	let face = geom.face(0);
	assert_eq!(face.vertices(), &[v1, v2, v3]);

	assert_eq!(position_face_vertices(&geom, v1), vec![v1]);
	assert_eq!(position_face_vertices(&geom, v2), vec![v2]);
	assert_eq!(position_face_vertices(&geom, v3), vec![v3]);

	assert!(geom.get_position_faces(vec3(9.0, 9.0, 9.0)).is_none());
}

#[test]
fn remove_face() {
	let mut geom = MeshGeometry::new();
	let center = vert(1.0, 1.0, 0.0);
	let bottom_right = vert(2.0, 0.0, 0.0);
	let bottom_left = vert(0.0, 0.0, 0.0);
	let top_left = vert(0.0, 2.0, 0.0);
	let top_right = vert(2.0, 2.0, 0.0);

	geom.add_face(&[center, bottom_right, bottom_left]);
	geom.add_face(&[center, top_right, bottom_right]);
	geom.add_face(&[center, top_left, top_right]);
	geom.add_face(&[center, bottom_left, top_left]);

	assert_eq!(geom.face_count(), 4);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(geom.face(1).vertices(), &[center, top_right, bottom_right]);
	assert_eq!(geom.face(2).vertices(), &[center, top_left, top_right]);
	assert_eq!(geom.face(3).vertices(), &[center, bottom_left, top_left]);

	assert_eq!(position_face_indices(&geom, center), vec![0, 1, 2, 3]);
	assert_eq!(position_face_indices(&geom, bottom_right), vec![0, 1]);
	assert_eq!(position_face_indices(&geom, bottom_left), vec![0, 3]);
	assert_eq!(position_face_indices(&geom, top_right), vec![1, 2]);
	assert_eq!(position_face_indices(&geom, top_left), vec![2, 3]);

	geom.remove_face(1);

	assert_eq!(geom.face_count(), 3);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(geom.face(1).vertices(), &[center, bottom_left, top_left]);

	assert_eq!(position_face_indices(&geom, center), vec![0, 1, 2]);
	assert_eq!(position_face_indices(&geom, bottom_right), vec![0]);
	assert_eq!(position_face_indices(&geom, bottom_left), vec![0, 1]);
	assert_eq!(position_face_indices(&geom, top_right), vec![2]);
	assert_eq!(position_face_indices(&geom, top_left), vec![1, 2]);

	geom.remove_face(0);

	assert_eq!(geom.face_count(), 2);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(geom.face(0).vertices(), &[center, top_left, top_right]);

	assert_eq!(position_face_indices(&geom, center), vec![0, 1]);
	assert_eq!(position_face_indices(&geom, bottom_right), vec![]);
	assert_eq!(position_face_indices(&geom, bottom_left), vec![1]);
	assert_eq!(position_face_indices(&geom, top_right), vec![0]);
	assert_eq!(position_face_indices(&geom, top_left), vec![0, 1]);

	geom.remove_face(1);

	assert_eq!(geom.face_count(), 1);

	assert_eq!(position_face_indices(&geom, center), vec![0]);
	assert_eq!(position_face_indices(&geom, bottom_right), vec![]);
	assert_eq!(position_face_indices(&geom, bottom_left), vec![]);
	assert_eq!(position_face_indices(&geom, top_right), vec![0]);
	assert_eq!(position_face_indices(&geom, top_left), vec![0]);

	geom.remove_face(0);

	assert_eq!(geom.face_count(), 0);

	assert!(position_face_refs(&geom, center).is_empty());
	assert!(position_face_refs(&geom, bottom_right).is_empty());
	assert!(position_face_refs(&geom, bottom_left).is_empty());
	assert!(position_face_refs(&geom, top_right).is_empty());
	assert!(position_face_refs(&geom, top_left).is_empty());
}

#[test]
fn new_from_section_resets_sections() {
	let mut geom = MeshGeometry::new();
	let base = [
		vert(0.0, 0.0, 0.0),
		vert(1.0, 0.0, 0.0),
		vert(0.0, 1.0, 0.0),
	];
	let other = [
		vert(0.0, 0.0, 1.0),
		vert(1.0, 0.0, 1.0),
		vert(0.0, 1.0, 1.0),
	];

	geom.add_face_data(&base, face_section(2));
	geom.add_face_data(&other, face_section(3));

	let section = geom.new_from_section(3);

	assert_eq!(section.face_count(), 1);
	assert_eq!(section.position_count(), 3);
	assert_eq!(section.face(0).section, DEFAULT_MESH_SECTION);
	assert_eq!(section.face(0).vertex_count, 3);
}

#[test]
fn split_by_sections_produces_separate_meshes() {
	let mut geom = MeshGeometry::new();
	let base = [
		vert(0.0, 0.0, 0.0),
		vert(1.0, 0.0, 0.0),
		vert(0.0, 1.0, 0.0),
	];
	let shifted = [
		vert(0.0, 0.0, 1.0),
		vert(1.0, 0.0, 1.0),
		vert(0.0, 1.0, 1.0),
	];
	let quad = [
		vert(0.0, 0.0, 2.0),
		vert(1.0, 0.0, 2.0),
		vert(1.0, 1.0, 2.0),
		vert(0.0, 1.0, 2.0),
	];

	geom.add_face_data(&base, face_section(5));
	geom.add_face_data(&shifted, face_section(5));
	geom.add_face_data(&quad, face_section(8));

	let sections = geom.split_by_sections();

	assert_eq!(sections.len(), 2);

	let sec5 = sections.get(&5).unwrap();
	assert_eq!(sec5.face_count(), 2);
	assert!(
		sec5.faces
			.iter()
			.all(|face| face.section == DEFAULT_MESH_SECTION)
	);

	let sec8 = sections.get(&8).unwrap();
	assert_eq!(sec8.face_count(), 1);
	assert_eq!(sec8.face(0).vertex_count, 4);
	assert!(
		sec8.faces
			.iter()
			.all(|face| face.section == DEFAULT_MESH_SECTION)
	);
}

#[test]
fn face_calculate_normal_triangle() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3]);
	let face = geom.face(0);

	let normal = face.calculate_normal();

	// Counter-clockwise triangle should have normal pointing up (0, 0, 1)
	assert!((normal - vec3(0.0, 0.0, 1.0)).length() < 0.0001);
}

#[test]
fn face_calculate_normal_quad() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(1.0, 1.0, 0.0);
	let v4 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3, v4]);
	let face = geom.face(0);

	let normal = face.calculate_normal();

	// CCW quad in XY plane should have normal pointing up (0, 0, 1)
	assert!((normal - vec3(0.0, 0.0, 1.0)).length() < 0.0001);
}

#[test]
fn face_calculate_and_store_normal() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3]);

	// Initially no normal stored
	assert_eq!(geom.faces[0].face_normal, None);

	geom.faces[0].calculate_and_store_normal();

	// Now normal should be stored
	let stored_normal = geom.faces[0].face_normal.unwrap();
	assert!((stored_normal - vec3(0.0, 0.0, 1.0)).length() < 0.0001);
}

#[test]
fn map_scale_vertices() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3]);

	// Scale all vertices by 2
	let scaled = geom.map(|face| {
		face.vertices()
			.iter()
			.map(|v| vert(v.pos.x * 2.0, v.pos.y * 2.0, v.pos.z * 2.0))
			.collect()
	});

	assert_eq!(scaled.face_count(), 1);
	assert_eq!(scaled.position_count(), 3);

	let face = scaled.face(0);
	let verts = (face).vertices();
	assert_eq!(verts[0], vert(0.0, 0.0, 0.0));
	assert_eq!(verts[1], vert(2.0, 0.0, 0.0));
	assert_eq!(verts[2], vert(0.0, 2.0, 0.0));
}

#[test]
fn map_preserves_face_properties() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	let normal = vec3(0.0, 0.0, 1.0);
	geom.add_face_data(&[v1, v2, v3], face_props(normal, 5));

	let mapped = geom.map(|face| face.vertices().to_vec());

	assert_eq!(mapped.face_count(), 1);
	let face = mapped.face(0);
	assert_eq!(face.section, 5);
	assert_eq!(face.face_normal, Some(normal));
}

#[test]
fn map_multiple_faces() {
	let mut geom = MeshGeometry::new();
	geom.add_face(&[
		vert(0.0, 0.0, 0.0),
		vert(1.0, 0.0, 0.0),
		vert(0.0, 1.0, 0.0),
	]);
	geom.add_face(&[
		vert(1.0, 0.0, 0.0),
		vert(1.0, 1.0, 0.0),
		vert(0.0, 1.0, 0.0),
	]);

	// Translate all faces by (10, 0, 0)
	let translated = geom.map(|face| {
		face.vertices()
			.iter()
			.map(|v| vert(v.pos.x + 10.0, v.pos.y, v.pos.z))
			.collect()
	});

	assert_eq!(translated.face_count(), 2);
	assert_eq!(translated.position_count(), 4);

	// Check first face
	let face0 = translated.face(0);
	let verts0 = (face0).vertices();
	assert_eq!(verts0[0], vert(10.0, 0.0, 0.0));
	assert_eq!(verts0[1], vert(11.0, 0.0, 0.0));
	assert_eq!(verts0[2], vert(10.0, 1.0, 0.0));
}

#[test]
fn map_data_modify_section_and_normal() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face_data(&[v1, v2, v3], super::face_section(1));

	let mapped = geom.map_data(|face| {
		let new_normal = vec3(1.0, 0.0, 0.0);
		let new_section = 3;
		(
			face.vertices().to_vec(),
			super::face_props(new_normal, new_section),
		)
	});

	assert_eq!(mapped.face_count(), 1);
	let face = mapped.face(0);
	assert_eq!(face.section, 3);
	assert_eq!(face.face_normal, Some(vec3(1.0, 0.0, 0.0)));
}

#[test]
fn map_data_calculate_normal_on_fly() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3]);

	// Scale vertices and recalculate normal
	let scaled = geom.map_data(|face| {
		let new_verts: Vec<_> = face
			.vertices()
			.iter()
			.map(|v| vert(v.pos.x * 2.0, v.pos.y * 2.0, v.pos.z * 2.0))
			.collect();

		// Create a temporary face to calculate new normal
		let mut temp_geom = MeshGeometry::new();
		temp_geom.add_face(&new_verts);
		let new_normal = temp_geom.face(0).calculate_normal();

		(new_verts, super::face_normal(new_normal))
	});

	assert_eq!(scaled.face_count(), 1);
	let face = scaled.face(0);
	// Normal should still point in same direction even after scaling
	assert!((face.face_normal.unwrap() - vec3(0.0, 0.0, 1.0)).length() < 0.0001);
}

#[test]
fn flat_map_filter_out_faces() {
	let mut geom = MeshGeometry::new();
	geom.add_face_data(
		&[
			vert(0.0, 0.0, 0.0),
			vert(1.0, 0.0, 0.0),
			vert(0.0, 1.0, 0.0),
		],
		super::face_section(1),
	);
	geom.add_face_data(
		&[
			vert(1.0, 0.0, 0.0),
			vert(1.0, 1.0, 0.0),
			vert(0.0, 1.0, 0.0),
		],
		super::face_section(2),
	);
	geom.add_face_data(
		&[
			vert(2.0, 0.0, 0.0),
			vert(3.0, 0.0, 0.0),
			vert(2.0, 1.0, 0.0),
		],
		super::face_section(1),
	);

	// Filter: keep only faces in section 1
	let filtered = geom.flat_map(|face| {
		if face.section == 1 {
			vec![face.vertices().to_vec()]
		} else {
			vec![]
		}
	});

	assert_eq!(filtered.face_count(), 2);
	assert_eq!(filtered.face(0).section, 1);
	assert_eq!(filtered.face(1).section, 1);
}

#[test]
fn flat_map_identity_map() {
	let mut geom = MeshGeometry::new();
	geom.add_face(&[
		vert(0.0, 0.0, 0.0),
		vert(1.0, 0.0, 0.0),
		vert(0.0, 1.0, 0.0),
	]);
	geom.add_face(&[
		vert(1.0, 0.0, 0.0),
		vert(1.0, 1.0, 0.0),
		vert(0.0, 1.0, 0.0),
	]);

	// Identity mapping: each face maps to exactly one face
	let mapped = geom.flat_map(|face| vec![face.vertices().to_vec()]);

	assert_eq!(mapped.face_count(), 2);
	assert_eq!((mapped.face(0)).vertices(), (geom.face(0).vertices()));
	assert_eq!((mapped.face(1).vertices()), (geom.face(1).vertices()));
}

#[test]
fn flat_map_split_quad_into_triangles() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(1.0, 1.0, 0.0);
	let v4 = vert(0.0, 1.0, 0.0);

	geom.add_face_data(&[v1, v2, v3, v4], super::face_section(5));

	// Split each quad into two triangles
	let triangulated = geom.flat_map(|face| {
		let verts = face.vertices();
		if verts.len() == 4 {
			vec![
				vec![verts[0], verts[1], verts[2]],
				vec![verts[0], verts[2], verts[3]],
			]
		} else {
			vec![verts.to_vec()]
		}
	});

	assert_eq!(triangulated.face_count(), 2);
	assert_eq!(triangulated.face(0).vertex_count, 3);
	assert_eq!(triangulated.face(1).vertex_count, 3);
	// Both triangles should preserve the section
	assert_eq!(triangulated.face(0).section, 5);
	assert_eq!(triangulated.face(1).section, 5);
}

#[test]
fn flat_map_duplicate_faces() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3]);

	// Duplicate each face 3 times
	let duplicated = geom.flat_map(|face| {
		vec![
			face.vertices().to_vec(),
			face.vertices().to_vec(),
			face.vertices().to_vec(),
		]
	});

	assert_eq!(duplicated.face_count(), 3);
	assert_eq!(duplicated.face(0).vertices(), &[v1, v2, v3]);
	assert_eq!(duplicated.face(1).vertices(), &[v1, v2, v3]);
	assert_eq!(duplicated.face(2).vertices(), &[v1, v2, v3]);
}

#[test]
fn flat_map_data_split_with_different_sections() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(1.0, 1.0, 0.0);
	let v4 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3, v4]);

	// Split quad into two triangles with different sections
	let split = geom.flat_map_data(|face| {
		let verts = face.vertices();
		if verts.len() == 4 {
			vec![
				(vec![verts[0], verts[1], verts[2]], super::face_section(10)),
				(vec![verts[0], verts[2], verts[3]], super::face_section(20)),
			]
		} else {
			vec![(verts.to_vec(), super::face_section(0))]
		}
	});

	assert_eq!(split.face_count(), 2);
	assert_eq!(split.face(0).section, 10);
	assert_eq!(split.face(1).section, 20);
}

#[test]
fn flat_map_data_recalculate_normals_per_split() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(1.0, 1.0, 0.0);
	let v4 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3, v4]);

	// Split and recalculate normals for each part
	let split = geom.flat_map_data(|face| {
		let verts = face.vertices();
		if verts.len() == 4 {
			// Create two triangles and calculate their normals
			let tri1 = vec![verts[0], verts[1], verts[2]];
			let tri2 = vec![verts[0], verts[2], verts[3]];

			let mut temp_geom1 = MeshGeometry::new();
			temp_geom1.add_face(&tri1);
			let normal1 = temp_geom1.face(0).calculate_normal();

			let mut temp_geom2 = MeshGeometry::new();
			temp_geom2.add_face(&tri2);
			let normal2 = temp_geom2.face(0).calculate_normal();

			vec![
				(tri1, super::face_normal(normal1)),
				(tri2, super::face_normal(normal2)),
			]
		} else {
			vec![(verts.to_vec(), Default::default())]
		}
	});

	assert_eq!(split.face_count(), 2);
	// Both triangles should have the same normal since they're coplanar
	let normal1 = split.face(0).face_normal.unwrap();
	let normal2 = split.face(1).face_normal.unwrap();
	assert!((normal1 - vec3(0.0, 0.0, 1.0)).length() < 0.0001);
	assert!((normal2 - vec3(0.0, 0.0, 1.0)).length() < 0.0001);
}

#[test]
fn flat_map_data_filter_and_modify() {
	let mut geom = MeshGeometry::new();
	geom.add_face_data(
		&[
			vert(0.0, 0.0, 0.0),
			vert(1.0, 0.0, 0.0),
			vert(0.0, 1.0, 0.0),
		],
		super::face_section(1),
	);
	geom.add_face_data(
		&[
			vert(1.0, 0.0, 0.0),
			vert(1.0, 1.0, 0.0),
			vert(0.0, 1.0, 0.0),
		],
		super::face_section(2),
	);
	geom.add_face_data(
		&[
			vert(2.0, 0.0, 0.0),
			vert(3.0, 0.0, 0.0),
			vert(2.0, 1.0, 0.0),
		],
		super::face_section(1),
	);

	// Keep only section 1 faces and move them to section 10
	let filtered = geom.flat_map_data(|face| {
		if face.section == 1 {
			vec![(face.vertices().to_vec(), super::face_section(10))]
		} else {
			vec![]
		}
	});

	assert_eq!(filtered.face_count(), 2);
	assert_eq!(filtered.face(0).section, 10);
	assert_eq!(filtered.face(1).section, 10);
}
