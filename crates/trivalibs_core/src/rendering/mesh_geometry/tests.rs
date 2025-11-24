use super::{DEFAULT_MESH_SECTION, Face, MeshGeometry, PositionFaceRef, face_section};
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

fn face_vertices(face: &Face<Vert>) -> Vec<Vert> {
	face.vertices().to_vec()
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
	assert_eq!(face_vertices(face), vec![v1, v2, v3]);

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
	assert_eq!(
		face_vertices(geom.face(1)),
		vec![center, top_right, bottom_right]
	);
	assert_eq!(
		face_vertices(geom.face(2)),
		vec![center, top_left, top_right]
	);
	assert_eq!(
		face_vertices(geom.face(3)),
		vec![center, bottom_left, top_left]
	);

	assert_eq!(position_face_indices(&geom, center), vec![0, 1, 2, 3]);
	assert_eq!(position_face_indices(&geom, bottom_right), vec![0, 1]);
	assert_eq!(position_face_indices(&geom, bottom_left), vec![0, 3]);
	assert_eq!(position_face_indices(&geom, top_right), vec![1, 2]);
	assert_eq!(position_face_indices(&geom, top_left), vec![2, 3]);

	geom.remove_face(1);

	assert_eq!(geom.face_count(), 3);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(
		face_vertices(geom.face(1)),
		vec![center, bottom_left, top_left]
	);

	assert_eq!(position_face_indices(&geom, center), vec![0, 1, 2]);
	assert_eq!(position_face_indices(&geom, bottom_right), vec![0]);
	assert_eq!(position_face_indices(&geom, bottom_left), vec![0, 1]);
	assert_eq!(position_face_indices(&geom, top_right), vec![2]);
	assert_eq!(position_face_indices(&geom, top_left), vec![1, 2]);

	geom.remove_face(0);

	assert_eq!(geom.face_count(), 2);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(
		face_vertices(geom.face(0)),
		vec![center, top_left, top_right]
	);

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
