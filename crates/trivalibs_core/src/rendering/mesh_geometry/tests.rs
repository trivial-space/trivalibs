use super::{
	DEFAULT_MESH_SECTION, Face, MeshGeometry, PositionFaceRef, VertexPosition, section_index,
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
	face.vertices
		.iter()
		.map(|vertex| vertex.position_index)
		.collect()
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

	let Face { vertices, .. } = geom.face(0);
	let indices = vertices
		.iter()
		.map(|fv| fv.position_index)
		.collect::<Vec<_>>();
	assert_eq!(indices, vec![0, 1, 2]);
	assert_eq!(vertices[0].data, v1);
	assert_eq!(vertices[1].data, v2);
	assert_eq!(vertices[2].data, v3);

	assert_eq!(geom.face_count(), 1);
	assert_eq!(geom.vertex_count(), 3);

	assert_eq!(
		geom.get_vertex_index(v2.position()),
		vec![section_index(DEFAULT_MESH_SECTION, 1)]
	);
	assert_eq!(
		geom.get_vertex_index(v3.position()),
		vec![section_index(DEFAULT_MESH_SECTION, 2)]
	);
}

#[test]
fn remove_face() {
	let mut geom = MeshGeometry::new();
	let v = vert(1.0, 1.0, 0.0);

	geom.add_face(&[v, vert(2.0, 0.0, 0.0), vert(0.0, 0.0, 0.0)]);
	geom.add_face(&[v, vert(2.0, 2.0, 0.0), vert(2.0, 0.0, 0.0)]);
	geom.add_face(&[v, vert(0.0, 2.0, 0.0), vert(2.0, 2.0, 0.0)]);
	geom.add_face(&[v, vert(0.0, 0.0, 0.0), vert(0.0, 2.0, 0.0)]);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 4);
	assert_eq!(section.positions.len(), 5);
	assert_eq!(face_indices(geom.face(1)), vec![0, 3, 1]);
	assert_eq!(face_indices(geom.face(2)), vec![0, 4, 3]);
	assert_eq!(face_indices(geom.face(3)), vec![0, 2, 4]);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0, 1, 2, 3]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![0, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![0, 3]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![1, 2]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![2, 3]);

	geom.remove_face(1);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 3);
	assert_eq!(section.positions.len(), 5);
	assert_eq!(face_indices(geom.face(1)), vec![0, 2, 4]);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0, 2, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![0, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![2]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![2, 1]);

	geom.remove_face(0);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 2);
	assert_eq!(section.positions.len(), 5);
	assert_eq!(face_indices(geom.face(0)), vec![0, 4, 3]);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0, 1]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![1]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![0, 1]);

	geom.remove_face(1);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 1);

	assert_eq!(vertex_face_indices(geom.vertex(0)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(1)), vec![]);
	assert_eq!(vertex_face_indices(geom.vertex(2)), vec![]);
	assert_eq!(vertex_face_indices(geom.vertex(3)), vec![0]);
	assert_eq!(vertex_face_indices(geom.vertex(4)), vec![0]);

	geom.remove_face(0);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 0);

	assert_eq!(vertex_face_refs(geom.vertex(0)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(1)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(2)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(3)), vec![]);
	assert_eq!(vertex_face_refs(geom.vertex(4)), vec![]);
}
