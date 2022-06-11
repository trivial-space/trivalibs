use crate::{
    geometry::{
        mesh_geometry_3d::{Face, MeshGeometry, VertexPosition},
        vertex_index::{VertIdx3f, WithVertexIndex},
    },
    rendering::buffered_geometry::{BufferedVertexData, VertexType},
};
use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec3};

use super::MeshVertex;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Pod, Zeroable)]
struct Vert {
    pos: Vec3,
}

impl VertexPosition for Vert {
    fn position(&self) -> Vec3 {
        self.pos
    }
}
impl WithVertexIndex<VertIdx3f> for Vert {
    fn vertex_index(&self) -> VertIdx3f {
        VertIdx3f::from(self.pos)
    }
}
impl BufferedVertexData for Vert {
    fn vertex_layout() -> Vec<VertexType> {
        todo!()
    }
}
impl MeshVertex<VertIdx3f, Vert> for Vert {}

fn vert(x: f32, y: f32, z: f32) -> Vert {
    Vert { pos: vec3(x, y, z) }
}

#[test]
fn generate_geometry() {
    let mut geom = MeshGeometry::new();
    let v1 = vert(0.0, 0.0, 0.0);
    let v2 = vert(1.0, 0.0, 0.0);
    let v3 = vert(0.0, 1.0, 0.0);

    geom.add_face3(v1, v2, v3);

    assert_eq!(geom.vertex(0).vertex, v1);
    assert_eq!(geom.vertex(1).vertex, v2);
    assert_eq!(geom.vertex(2).vertex, v3);

    let Face { vertices, .. } = geom.face(0);
    assert_eq!(vertices, &[0, 1, 2]);

    assert_eq!(geom.faces.len(), 1);
    assert_eq!(geom.vertices.len(), 3);

    assert_eq!(geom.get_vertex_index(v2.vertex_index()), 1);
    assert_eq!(geom.get_vertex_index(v3.vertex_index()), 2);
}

#[test]
fn remove_face() {
    let mut geom = MeshGeometry::new();
    let v = vert(1.0, 1.0, 0.0);

    geom.add_face3(v, vert(2.0, 0.0, 0.0), vert(0.0, 0.0, 0.0));
    geom.add_face3(v, vert(2.0, 2.0, 0.0), vert(2.0, 0.0, 0.0));
    geom.add_face3(v, vert(0.0, 2.0, 0.0), vert(2.0, 2.0, 0.0));
    geom.add_face3(v, vert(0.0, 0.0, 0.0), vert(0.0, 2.0, 0.0));

    assert_eq!(geom.faces.len(), 4);
    assert_eq!(geom.next_index, 5);
    assert_eq!(geom.vertices.len(), 5);
    assert_eq!(geom.face(1).vertices, [0, 3, 1]);
    assert_eq!(geom.face(2).vertices, [0, 4, 3]);
    assert_eq!(geom.face(3).vertices, [0, 2, 4]);

    assert_eq!(geom.vertex(0).faces, [0, 1, 2, 3]);
    assert_eq!(geom.vertex(1).faces, [0, 1]);
    assert_eq!(geom.vertex(2).faces, [0, 3]);
    assert_eq!(geom.vertex(3).faces, [1, 2]);
    assert_eq!(geom.vertex(4).faces, [2, 3]);

    geom.remove_face(1);

    assert_eq!(geom.faces.len(), 3);
    assert_eq!(geom.vertices.len(), 5);
    assert_eq!(geom.face(1).vertices, [0, 2, 4]);

    assert_eq!(geom.vertex(0).faces, [0, 2, 1]);
    assert_eq!(geom.vertex(1).faces, [0]);
    assert_eq!(geom.vertex(2).faces, [0, 1]);
    assert_eq!(geom.vertex(3).faces, [2]);
    assert_eq!(geom.vertex(4).faces, [2, 1]);

    geom.remove_face(0);

    assert_eq!(geom.faces.len(), 2);
    assert_eq!(geom.vertices.len(), 5);
    assert_eq!(geom.face(0).vertices, [0, 4, 3]);

    assert_eq!(geom.vertex(0).faces, [0, 1]);
    assert_eq!(geom.vertex(1).faces, []);
    assert_eq!(geom.vertex(2).faces, [1]);
    assert_eq!(geom.vertex(3).faces, [0]);
    assert_eq!(geom.vertex(4).faces, [0, 1]);

    geom.remove_face(1);

    assert_eq!(geom.faces.len(), 1);

    assert_eq!(geom.vertex(0).faces, [0]);
    assert_eq!(geom.vertex(1).faces, []);
    assert_eq!(geom.vertex(2).faces, []);
    assert_eq!(geom.vertex(3).faces, [0]);
    assert_eq!(geom.vertex(4).faces, [0]);

    geom.remove_face(0);

    assert_eq!(geom.faces.len(), 0);

    assert_eq!(geom.vertex(0).faces, []);
    assert_eq!(geom.vertex(1).faces, []);
    assert_eq!(geom.vertex(2).faces, []);
    assert_eq!(geom.vertex(3).faces, []);
    assert_eq!(geom.vertex(4).faces, []);
}

#[test]
fn triangulate() {
    let mut geom = MeshGeometry::new();

    geom.add_face4(
        vert(0.0, 0.0, 0.0),
        vert(1.0, 0.0, 0.0),
        vert(1.0, 1.0, 0.0),
        vert(0.0, 1.0, 0.0),
    );
    geom.add_face4(
        vert(0.0, 0.0, 0.0),
        vert(0.0, 0.0, 1.0),
        vert(0.0, 1.0, 1.0),
        vert(0.0, 1.0, 0.0),
    );

    assert_eq!(geom.faces.len(), 2);
    assert_eq!(geom.face(0).vertices, [0, 1, 2, 3]);

    geom.triangulate();

    assert_eq!(geom.faces.len(), 4);
    for face in &geom.faces {
        assert_eq!(face.vertices.len(), 3);
    }
    assert!(geom
        .faces
        .iter()
        .find(|f| { f.vertices == [0, 1, 2] })
        .is_some());
    assert!(geom
        .faces
        .iter()
        .find(|f| { f.vertices == [0, 2, 3] })
        .is_some());
    assert!(geom
        .faces
        .iter()
        .find(|f| { f.vertices == [0, 4, 5] })
        .is_some());
    assert!(geom
        .faces
        .iter()
        .find(|f| { f.vertices == [0, 5, 3] })
        .is_some());
}
