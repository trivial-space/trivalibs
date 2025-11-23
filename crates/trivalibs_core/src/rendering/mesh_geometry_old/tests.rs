use crate::{
	data::NotOverridable,
	rendering::mesh_geometry_old::{DEFAULT_MESH_SECTION, section_index},
};

use super::{Face, MeshGeometry};
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
impl NotOverridable for Vert {}

fn vert(x: f32, y: f32, z: f32) -> Vert {
	Vert { pos: vec3(x, y, z) }
}

#[test]
fn generate_geometry() {
	let mut geom = MeshGeometry::new();
	let v1 = vert(0.0, 0.0, 0.0);
	let v2 = vert(1.0, 0.0, 0.0);
	let v3 = vert(0.0, 1.0, 0.0);

	geom.add_face3([v1, v2, v3]);

	assert_eq!(geom.vertex(0).data, v1);
	assert_eq!(geom.vertex(1).data, v2);
	assert_eq!(geom.vertex(2).data, v3);

	let Face { vertices, .. } = geom.face(0);
	assert_eq!(vertices, &[0, 1, 2]);

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

	geom.add_face3([v, vert(2.0, 0.0, 0.0), vert(0.0, 0.0, 0.0)]);
	geom.add_face3([v, vert(2.0, 2.0, 0.0), vert(2.0, 0.0, 0.0)]);
	geom.add_face3([v, vert(0.0, 2.0, 0.0), vert(2.0, 2.0, 0.0)]);
	geom.add_face3([v, vert(0.0, 0.0, 0.0), vert(0.0, 2.0, 0.0)]);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 4);
	assert_eq!(section.next_index, 5);
	assert_eq!(section.vertices.len(), 5);
	assert_eq!(geom.face(1).vertices, [0, 3, 1]);
	assert_eq!(geom.face(2).vertices, [0, 4, 3]);
	assert_eq!(geom.face(3).vertices, [0, 2, 4]);

	assert_eq!(geom.vertex(0).faces, [0, 1, 2, 3]);
	assert_eq!(geom.vertex(1).faces, [0, 1]);
	assert_eq!(geom.vertex(2).faces, [0, 3]);
	assert_eq!(geom.vertex(3).faces, [1, 2]);
	assert_eq!(geom.vertex(4).faces, [2, 3]);

	geom.remove_face(1);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 3);
	assert_eq!(section.vertices.len(), 5);
	assert_eq!(geom.face(1).vertices, [0, 2, 4]);

	assert_eq!(geom.vertex(0).faces, [0, 2, 1]);
	assert_eq!(geom.vertex(1).faces, [0]);
	assert_eq!(geom.vertex(2).faces, [0, 1]);
	assert_eq!(geom.vertex(3).faces, [2]);
	assert_eq!(geom.vertex(4).faces, [2, 1]);

	geom.remove_face(0);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 2);
	assert_eq!(section.vertices.len(), 5);
	assert_eq!(geom.face(0).vertices, [0, 4, 3]);

	assert_eq!(geom.vertex(0).faces, [0, 1]);
	assert_eq!(geom.vertex(1).faces, []);
	assert_eq!(geom.vertex(2).faces, [1]);
	assert_eq!(geom.vertex(3).faces, [0]);
	assert_eq!(geom.vertex(4).faces, [0, 1]);

	geom.remove_face(1);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 1);

	assert_eq!(geom.vertex(0).faces, [0]);
	assert_eq!(geom.vertex(1).faces, []);
	assert_eq!(geom.vertex(2).faces, []);
	assert_eq!(geom.vertex(3).faces, [0]);
	assert_eq!(geom.vertex(4).faces, [0]);

	geom.remove_face(0);

	let section = geom.sections.get(&DEFAULT_MESH_SECTION).unwrap();

	assert_eq!(section.faces.len(), 0);

	assert_eq!(geom.vertex(0).faces, []);
	assert_eq!(geom.vertex(1).faces, []);
	assert_eq!(geom.vertex(2).faces, []);
	assert_eq!(geom.vertex(3).faces, []);
	assert_eq!(geom.vertex(4).faces, []);
}
