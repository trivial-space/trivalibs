use super::{
	DEFAULT_MESH_SECTION, Face, MeshGeometry, PositionFaceRef, VertexPosition, face_section,
};
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

fn face_indices(face: &Face<Vert>) -> Vec<usize> {
	face.position_indices().to_vec()
}

fn vertex_face_indices(vertex: &VertexPosition) -> Vec<usize> {
	vertex.faces.iter().map(|f| f.face_index).collect()
}

fn vertex_face_refs(vertex: &VertexPosition) -> Vec<PositionFaceRef> {
	vertex.faces.clone()
}

#[test]
fn generate_geometry() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face(&[v1, v2, v3]);

	assert_eq!(geom.vertex(0).position, v1.position());
	assert_eq!(geom.vertex(1).position, v2.position());
	assert_eq!(geom.vertex(2).position, v3.position());

	let face = geom.face(0);
	assert_eq!(face.position_indices(), &[0, 1, 2]);
	assert_eq!(face.vertex_data()[0], v1);
	assert_eq!(face.vertex_data()[1], v2);
	assert_eq!(face.vertex_data()[2], v3);

	assert_eq!(geom.face_count(), 1);
	assert_eq!(geom.position_count(), 3);

	assert_eq!(geom.get_vertex_index(v2.position()), Some(1));
	assert_eq!(geom.get_vertex_index(v3.position()), Some(2));
}

#[test]
fn remove_face() {
	let mut geom = MeshGeometry::new();
	let v = vert(1.0, 1.0, 0.0);

	geom.add_face(&[v, vert(2.0, 0.0, 0.0), vert(0.0, 0.0, 0.0)]);
	geom.add_face(&[v, vert(2.0, 2.0, 0.0), vert(2.0, 0.0, 0.0)]);
	geom.add_face(&[v, vert(0.0, 2.0, 0.0), vert(2.0, 2.0, 0.0)]);
	geom.add_face(&[v, vert(0.0, 0.0, 0.0), vert(0.0, 2.0, 0.0)]);

	assert_eq!(geom.face_count(), 4);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(face_indices(geom.face(1)), vec![0, 3, 1]);
	assert_eq!(face_indices(geom.face(2)), vec![0, 4, 3]);
	assert_eq!(face_indices(geom.face(3)), vec![0, 2, 4]);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0, 1, 2, 3]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![0, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![0, 3]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![1, 2]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![2, 3]);

	geom.remove_face(1);

	assert_eq!(geom.face_count(), 3);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(face_indices(geom.face(1)), vec![0, 2, 4]);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0, 2, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![0, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![2]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![2, 1]);

	geom.remove_face(0);

	assert_eq!(geom.face_count(), 2);
	assert_eq!(geom.position_count(), 5);
	assert_eq!(face_indices(geom.face(0)), vec![0, 4, 3]);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![1]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![0, 1]);

	geom.remove_face(1);

	assert_eq!(geom.face_count(), 1);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![0]);

	geom.remove_face(0);

	assert_eq!(geom.face_count(), 0);

	assert_eq!(vertex_face_refs(geom.vertex(0)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(1)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(2)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(3)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(4)), vec![]);
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
